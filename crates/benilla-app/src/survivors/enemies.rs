//! The horde: spawning on a ring outside the view, the chase, contact damage, death, and the
//! experience gems the dead leave.

use std::collections::HashMap;

use benilla_world::rig_anim::GlobalSeqDrive;
use bevy::prelude::*;

use super::abilities::Hit;
use super::data::{self, EnemyDef};
use super::hero::{self, Hero, HERO_RADIUS};
use super::units::{self, flat, yaw_toward, Fx, Marks, UnitSpec, COLOR_HURT};
use super::Floor;
use super::{Phase, Rng, Run, Selection};
use crate::creature_anim::{move_flags, MovementState};
use crate::net::{FieldChanged, Guid, ObjectStore};

#[derive(Component)]
pub(super) struct Enemy {
    pub def: &'static EnemyDef,
    pub hp: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub damage: f32,
    pub radius: f32,
    pub xp: u32,
    pub boss: bool,
    pub elite: bool,
    pub attack_cd: f32,
    pub slow: f32,
    pub slow_t: f32,
    pub root_t: f32,
    pub frozen_t: f32,
    pub winters_chill: survivors_core::talents::WintersChill,
    pub shadow_weaving: survivors_core::talents::ShadowWeaving,
    pub stun_t: f32,
    pub fear_t: f32,
    pub weaken: f32,
    pub weaken_t: f32,
    pub dots: Vec<Dot>,
    pub dead: bool,
}

pub(super) struct Dot {
    pub dps: f32,
    pub left: f32,
    pub tick: f32,
    pub source: &'static str,
}

impl Dot {
    /// Count each due one-second tick, including the last at expiry, across long frames.
    pub(super) fn advance(&mut self, dt: f32) -> u32 {
        let elapsed = dt.max(0.0).min(self.left.max(0.0));
        self.left = (self.left - elapsed).max(0.0);
        self.tick -= elapsed;
        if self.tick > 0.00001 || elapsed == 0.0 {
            return 0;
        }
        let ticks = (-self.tick + 0.00001).floor() as u32 + 1;
        self.tick += ticks as f32;
        ticks
    }
}

/// A corpse playing its death before it fades.
#[derive(Component)]
pub(super) struct Dying(f32);

#[derive(Resource, Default)]
pub(super) struct Spawner {
    timer: f32,
    chest_t: f32,
}

/// An enemy died here, worth this much.
#[derive(Message, Clone, Copy)]
pub(super) struct Loot {
    pub pos: Vec3,
    pub xp: u32,
    pub boss: bool,
    pub elite: bool,
}

#[derive(Clone, Copy)]
pub(super) enum GemKind {
    Xp(u32),
    Heal(f32),
    Gold(u32),
}

/// A chest standing in the world. Walking into it pays gold and opens a trade of cards.
#[derive(Component)]
pub(super) struct Chest {
    gold: u32,
    tier: super::campaign::ChestTier,
}

#[derive(Component)]
pub(super) struct Gem {
    pub(super) kind: GemKind,
    pub(super) pulled: bool,
    speed: f32,
}

/// The `GameObjectDisplayInfo` row and scale a pickup wears: a wisp per experience tier, the
/// bigger tiers rarer, and a turkey leg for health.
fn look_for(kind: GemKind) -> (u32, f32) {
    match kind {
        // `TurkeyLeg.mdx`.
        GemKind::Heal(_) => (114, 2.5),
        // The green wisp, smaller than an experience gem, so a coin pile reads apart from XP.
        GemKind::Gold(_) => (1267, 0.45),
        // `BFD_WispSmallPurple.mdx`.
        GemKind::Xp(xp) if xp >= 20 => (1307, 1.0),
        // `BFD_WispSmallGreen.mdx`.
        GemKind::Xp(xp) if xp >= 5 => (1267, 1.0),
        // `BFD_WispSmall.mdx`.
        GemKind::Xp(_) => (426, 1.0),
    }
}

/// Body radius: the big beasts need more room.
fn radius_for(def: &EnemyDef) -> f32 {
    (if def.big { 1.3 } else { 0.6 }) * def.scale
}

fn spawn_enemy(
    commands: &mut Commands,
    run: &mut Run,
    def: &'static EnemyDef,
    pos: Vec3,
    hero: Vec3,
    boss: bool,
    elite: bool,
) {
    let scale = data::hp_scale(run.elapsed);
    let serial = run.guid();
    let e = units::spawn_unit(
        commands,
        &UnitSpec {
            serial,
            display: def.display,
            scale: def.scale,
            level: def.level,
            faction: units::FACTION_MONSTER,
            bytes0: 1 << 8,
            weapons: &[],
            sheath: 0,
        },
        pos,
        yaw_toward(hero - pos),
    );
    let hp = def.hp * scale;
    commands.entity(e).insert((
        Enemy {
            def,
            hp,
            max_hp: hp,
            speed: def.speed,
            damage: def.damage * data::damage_scale(run.elapsed),
            radius: radius_for(def),
            xp: def.xp,
            boss,
            elite: elite && !boss,
            attack_cd: 0.5,
            slow: 1.0,
            slow_t: 0.0,
            root_t: 0.0,
            frozen_t: 0.0,
            winters_chill: Default::default(),
            shadow_weaving: Default::default(),
            stun_t: 0.0,
            fear_t: 0.0,
            weaken: 1.0,
            weaken_t: 0.0,
            dots: Vec::new(),
            dead: false,
        },
        Marks::debuffs(),
    ));
}

fn ground(world: &Floor, mut p: Vec3, fallback: f32) -> Vec3 {
    p.y = world.height(p).unwrap_or(fallback);
    p
}

/// Recycling has the same geometry and arena constraints as a new spawn. Keep the
/// old position while streamed geometry has no safe replacement.
fn recycle(world: &Floor, map: &data::MapDef, hero: Vec3, at: &mut Vec3, radius: f32) {
    let direction = flat(hero - *at);
    if let Some(safe) = super::spawn::point(
        world,
        map,
        hero,
        direction.z.atan2(direction.x),
        38.0,
        radius,
    ) {
        *at = safe;
    }
}

/// Keep the horde topped up on a ring outside the view, with the bosses and the encircling waves
/// on the clock.
pub(super) fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    mut spawner: ResMut<Spawner>,
    selection: Res<Selection>,
    hero: Query<&Transform, With<Hero>>,
    enemies: Query<&Enemy>,
    chests: Query<&Chest>,
    world: Floor,
) {
    let Ok(h) = hero.single() else { return };
    if !run.alive() {
        return;
    }
    let hpos = h.translation;
    let elapsed = run.elapsed;
    let roster = data::roster(selection.map);
    let pool = data::wave_pool(elapsed, roster.enemies.len());

    let map = selection.map_def();
    while let Some(def) = super::encounters::next(selection.map, run.bosses, elapsed) {
        let a = rng.range(0.0, std::f32::consts::TAU);
        let Some(pos) = super::spawn::point(&world, map, hpos, a, 30.0, radius_for(def)) else {
            // The same scheduled boss remains pending until a supported, clear position exists.
            break;
        };
        spawn_enemy(&mut commands, &mut run, def, pos, hpos, true, false);
        run.bosses += 1;
        run.banner = Some((format!("{} approaches!", def.name), 4.0));
    }

    // Plan a complete ring before spawning any of it, so a retry cannot duplicate half a wave.
    let due_rings = ((elapsed + data::BOSS_EVERY_SECS / 2.0) / data::BOSS_EVERY_SECS) as u32;
    if run.rings < due_rings {
        let def = &roster.enemies[pool[0]];
        let n = 28 + (data::progress(elapsed) * 60.0) as usize;
        let positions: Option<Vec<_>> = (0..n)
            .map(|i| {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                super::spawn::point(&world, map, hpos, a, 26.0, radius_for(def))
            })
            .collect();
        if let Some(positions) = positions {
            for pos in positions {
                let elite = super::campaign::is_named_elite(map.name, def.name);
                spawn_enemy(&mut commands, &mut run, def, pos, hpos, false, elite);
            }
            run.rings = due_rings;
            run.banner = Some(("They're surrounding you!".into(), 3.0));
        }
    }

    if !run.stocked {
        let positions: Option<Vec<_>> = (0..3)
            .map(|i| {
                let a = i as f32 / 3.0 * std::f32::consts::TAU;
                super::spawn::point(&world, map, hpos, a, 14.0, 1.0)
            })
            .collect();
        if let Some(positions) = positions {
            for pos in positions {
                spawn_chest(&mut commands, &mut run, pos, super::campaign::ChestTier::A);
            }
            run.stocked = true;
        }
    }

    spawner.chest_t -= time.delta_secs();
    if spawner.chest_t <= 0.0 {
        let woods = chests
            .iter()
            .filter(|c| c.tier == super::campaign::ChestTier::C)
            .count();
        if woods >= 3 {
            spawner.chest_t = 45.0;
        } else {
            let a = rng.range(0.0, std::f32::consts::TAU);
            let distance = rng.range(16.0, 24.0);
            if let Some(pos) = super::spawn::point(&world, map, hpos, a, distance, 1.0) {
                spawn_chest(&mut commands, &mut run, pos, super::campaign::ChestTier::C);
                spawner.chest_t = 45.0;
            } else {
                spawner.chest_t = 0.5;
            }
        }
    }

    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }
    spawner.timer = 0.12;
    let alive = enemies.iter().filter(|e| !e.dead).count();
    let target = data::target_alive(elapsed);
    if alive >= target {
        return;
    }
    let n = (((target - alive) as f32) / 6.0).ceil().min(4.0) as usize;
    for _ in 0..n {
        let def = &roster.enemies[pool[rng.index(pool.len())]];
        let a = rng.range(0.0, std::f32::consts::TAU);
        let r = rng.range(34.0, 42.0);
        let Some(pos) = super::spawn::point(&world, map, hpos, a, r, radius_for(def)) else {
            continue;
        };
        let elite = super::campaign::is_named_elite(selection.map_def().name, def.name);
        spawn_enemy(&mut commands, &mut run, def, pos, hpos, false, elite);
    }
}

fn spawn_chest(
    commands: &mut Commands,
    run: &mut Run,
    pos: Vec3,
    tier: super::campaign::ChestTier,
) {
    let serial = run.guid();
    let e = units::spawn_object(
        commands,
        serial,
        super::campaign::chest_display(tier),
        1.15,
        pos,
    );
    commands.entity(e).insert(Chest {
        gold: super::campaign::chest_gold(tier, run.level),
        tier,
    });
}

/// The chase: straight at the hero, pushed apart from each other, stopped by crowd control, and
/// swinging when in reach.
#[allow(clippy::type_complexity)]
pub(super) fn move_enemies(
    time: Res<Time>,
    mut run: ResMut<Run>,
    world: Floor,
    selection: Res<Selection>,
    hero: Query<(Entity, &Transform), (With<Hero>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy, &mut MovementState), Without<Hero>>,
    mut navigation: Local<super::navigation::Navigation>,
    mut fx: Fx,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let Ok((hero_e, htf)) = hero.single() else {
        return;
    };
    let hpos = htf.translation;
    let hero_down = !run.alive();

    // Separation over a coarse grid: only neighbours in the same or adjacent cells push.
    const CELL: f32 = 3.0;
    let bodies: Vec<(Entity, Vec3, f32)> = enemies
        .iter()
        .filter(|(_, _, e, _)| !e.dead)
        .map(|(ent, tf, e, _)| (ent, tf.translation, e.radius))
        .collect();
    navigation.advance(dt, selection.map, &bodies, |from, to, radius| {
        let map = selection.map_def();
        if flat(to - hero::arena(map)).length() + radius > map.leash {
            return None;
        }
        let end = world.travel(from, to, radius);
        (flat(end - to).length() < 0.1 && world.can_spawn(end, radius)).then_some(end)
    });
    let key = |p: Vec3| ((p.x / CELL).floor() as i32, (p.z / CELL).floor() as i32);
    let mut grid: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    for (i, b) in bodies.iter().enumerate() {
        grid.entry(key(b.1)).or_default().push(i);
    }
    let mut push: HashMap<Entity, Vec3> = HashMap::with_capacity(bodies.len());
    for (i, &(ent, pos, r)) in bodies.iter().enumerate() {
        let (cx, cz) = key(pos);
        let mut acc = Vec3::ZERO;
        for dx in -1..=1 {
            for dz in -1..=1 {
                let Some(cell) = grid.get(&(cx + dx, cz + dz)) else {
                    continue;
                };
                for &j in cell {
                    if j == i {
                        continue;
                    }
                    let (_, other, or) = bodies[j];
                    let d = flat(pos - other);
                    let len = d.length();
                    let min = r + or;
                    if len < min && len > 1e-4 {
                        acc += d / len * (min - len);
                    } else if len <= 1e-4 {
                        acc += Vec3::new((i as f32).sin(), 0.0, (i as f32).cos()) * 0.1;
                    }
                }
            }
        }
        push.insert(ent, acc);
    }

    for (ent, mut tf, mut e, mut mv) in &mut enemies {
        let e = &mut *e;
        if e.dead {
            if mv.speed != 0.0 || mv.flags != 0 {
                mv.speed = 0.0;
                mv.flags = 0;
            }
            continue;
        }
        e.attack_cd -= dt;
        e.winters_chill.advance(dt);
        e.shadow_weaving.advance(dt);
        for t in [
            &mut e.slow_t,
            &mut e.root_t,
            &mut e.frozen_t,
            &mut e.stun_t,
            &mut e.fear_t,
            &mut e.weaken_t,
        ] {
            *t = (*t - dt).max(0.0);
        }
        if e.slow_t <= 0.0 {
            e.slow = 1.0;
        }
        if e.weaken_t <= 0.0 {
            e.weaken = 1.0;
        }
        let pos = tf.translation;
        let to_hero = flat(hpos - pos);
        let dist = to_hero.length();

        // Strays far behind are brought back in front, as the reference genre does.
        if dist > 75.0 {
            navigation.forget(ent);
            recycle(
                &world,
                selection.map_def(),
                hpos,
                &mut tf.translation,
                e.radius,
            );
            mv.speed = 0.0;
            mv.flags = 0;
            continue;
        }

        // Under Fade the horde has lost the hero: it mills where it stands.
        let lost = run.mobile.vanish_t > 0.0 && !e.boss;
        let held = e.stun_t > 0.0 || e.root_t > 0.0 || lost;
        let contact = dist <= HERO_RADIUS + e.radius + 0.35 && world.line_clear(pos, hpos);
        let mut step = Vec3::ZERO;
        let mut speed = 0.0;
        if held || hero_down || e.fear_t > 0.0 {
            navigation.forget(ent);
        }
        let waypoint = navigation.waypoint(ent, pos, hpos);
        if !held && !hero_down {
            let dir = if e.fear_t > 0.0 {
                -to_hero.normalize_or_zero()
            } else {
                flat(waypoint.unwrap_or(hpos) - pos).normalize_or_zero()
            };
            if !contact || e.fear_t > 0.0 {
                speed = e.speed * e.slow;
                step = dir * speed;
                if let Some(point) = waypoint {
                    step = step.clamp_length_max(flat(point - pos).length() / dt);
                }
            }
            if dir != Vec3::ZERO {
                let target = Quat::from_rotation_y(yaw_toward(dir));
                tf.rotation = tf.rotation.slerp(target, (dt * 10.0).min(1.0));
            }
        }
        let shove = push.get(&ent).copied().unwrap_or_default();
        let mut next = pos + step * dt + shove * (dt * 8.0).min(0.5);
        next = if waypoint.is_some() {
            world.travel(pos, next, e.radius)
        } else {
            world.slide(pos, next, e.radius)
        };
        next = ground(&world, next, pos.y);
        tf.translation = next;
        if !held && !hero_down && e.fear_t <= 0.0 && !contact {
            let expected = speed * dt;
            let progress = flat(next - pos).dot(step.normalize_or_zero());
            navigation.observe(
                ent,
                next,
                hpos,
                e.radius,
                expected > 0.001 && progress < expected * 0.2,
                dt,
            );
        }
        if speed > 0.0 {
            mv.speed = speed;
            mv.flags = move_flags::FORWARD;
        } else if mv.speed != 0.0 || mv.flags != 0 {
            mv.speed = 0.0;
            mv.flags = 0;
        }

        if contact
            && !lost
            && e.stun_t <= 0.0
            && e.fear_t <= 0.0
            && e.attack_cd <= 0.0
            && !hero_down
        {
            e.attack_cd = if e.boss { 1.0 } else { 1.3 };
            let raw = e.damage * e.weaken;
            let taken = hero::hurt_hero(&mut run, raw);
            fx.swing(ent, hero_e, taken);
            if taken > 0.0 {
                fx.number(hero_e, taken, false, COLOR_HURT);
            }
            if run.thorns_t > 0.0 {
                hits.write(Hit::direct(ent, run.thorns * run.stats().might, false));
            }
        }
    }
}

/// Mark timers run down; the state kits reap with them.
pub(super) fn tick_marks(
    time: Res<Time>,
    mut units: Query<(Entity, &Guid, &mut ObjectStore, &mut Marks), With<Enemy>>,
    mut fields: MessageWriter<FieldChanged>,
) {
    let dt = time.delta_secs();
    for (entity, guid, mut store, mut marks) in &mut units {
        marks.tick(dt);
        marks.flush(entity, guid.0, &mut store, &mut fields);
    }
}

/// Corpses fade once their death has played.
pub(super) fn reap_dead(
    mut commands: Commands,
    time: Res<Time>,
    mut dying: Query<(Entity, &mut Dying)>,
) {
    for (e, mut d) in &mut dying {
        d.0 += time.delta_secs();
        if d.0 > 2.5 {
            commands
                .entity(e)
                .remove::<Dying>()
                .insert(benilla_world::model_fade::DespawnFade::default());
        }
    }
}

pub(super) fn mark_dying(commands: &mut Commands, e: Entity) {
    commands.entity(e).insert(Dying(0.0));
}

/// Most gems on the ground at once; past it, a kill's worth joins an existing gem.
const MAX_GEMS: usize = 350;

pub(super) fn drop_loot(
    mut commands: Commands,
    mut drops: MessageReader<Loot>,
    mut rng: ResMut<Rng>,
    mut run: ResMut<Run>,
    mut gems: Query<&mut Gem>,
) {
    let mut count = gems.iter().count();
    let mut gold = 0u32;
    let mut gold_at = Vec3::ZERO;
    for d in drops.read() {
        gold += if d.boss { 10 + run.level / 4 } else { 1 };
        gold_at = d.pos;
        if d.boss {
            spawn_chest(
                &mut commands,
                &mut run,
                d.pos,
                super::campaign::ChestTier::A,
            );
        } else if d.elite {
            spawn_chest(
                &mut commands,
                &mut run,
                d.pos,
                super::campaign::ChestTier::B,
            );
        }
        if d.boss || rng.f32() < 0.015 {
            let heal = GemKind::Heal(if d.boss { 1000.0 } else { 30.0 });
            spawn_gem(&mut commands, run.guid(), d.pos + Vec3::X * 0.8, heal);
        }
        if count >= MAX_GEMS && !d.boss {
            if let Some(mut g) = gems.iter_mut().find(|g| matches!(g.kind, GemKind::Xp(_))) {
                if let GemKind::Xp(xp) = &mut g.kind {
                    *xp += d.xp;
                }
                continue;
            }
        }
        spawn_gem(&mut commands, run.guid(), d.pos, GemKind::Xp(d.xp));
        count += 1;
    }
    if gold > 0 {
        if let Some(mut g) = gems.iter_mut().find(|g| matches!(g.kind, GemKind::Gold(_))) {
            if let GemKind::Gold(n) = &mut g.kind {
                *n += gold;
            }
        } else {
            spawn_gem(&mut commands, run.guid(), gold_at, GemKind::Gold(gold));
        }
    }
}

fn spawn_gem(commands: &mut Commands, serial: u64, pos: Vec3, kind: GemKind) {
    let (display, scale) = look_for(kind);
    let e = units::spawn_object(commands, serial, display, scale, pos + Vec3::Y * 0.5);
    commands.entity(e).insert(Gem {
        kind,
        pulled: false,
        speed: 6.0,
    });
}

/// Gems in the magnet's reach fly to the hero; levels bank as they land.
pub(super) fn collect_gems(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut stash: ResMut<super::stash::Stash>,
    mut next: ResMut<NextState<Phase>>,
    hero: Query<&Transform, (With<Hero>, Without<Gem>)>,
    mut gems: Query<(Entity, &mut Transform, &mut Gem)>,
) {
    let Ok(h) = hero.single() else { return };
    if !run.alive() {
        return;
    }
    let dt = time.delta_secs();
    let stats = run.stats();
    let target = h.translation + Vec3::Y * 1.0;
    for (e, mut tf, mut g) in &mut gems {
        let d = target - tf.translation;
        let dist = d.length();
        if !g.pulled && flat(d).length() <= stats.magnet {
            g.pulled = true;
        }
        if !g.pulled {
            continue;
        }
        if dist < 0.9 {
            bank_gem(&mut run, &mut stash, &mut next, g.kind, 1.0);
            commands.entity(e).despawn();
            continue;
        }
        g.speed += 30.0 * dt;
        tf.translation += d / dist * (g.speed * dt).min(dist);
    }
    settle_levels(&mut run, &mut next);
}

/// A pickup landed. `gold_bonus` is 1 for the hero and 1.1 for a gold-sniffer companion.
pub(super) fn bank_gem(
    run: &mut Run,
    stash: &mut super::stash::Stash,
    next: &mut NextState<Phase>,
    kind: GemKind,
    gold_bonus: f32,
) {
    if !run.alive() {
        return;
    }
    let stats = run.stats();
    match kind {
        GemKind::Xp(xp) => run.xp += xp as f32 * data::XP_RATE * stats.growth,
        GemKind::Heal(hp) => {
            run.heal(hp, stats.max_hp);
        }
        GemKind::Gold(n) => {
            let n = super::sheet::gold_amount(n, stats.gold_find * gold_bonus);
            run.gold_found += n;
            stash.add_gold(n);
        }
    }
    settle_levels(run, next);
}

fn settle_levels(run: &mut Run, next: &mut NextState<Phase>) {
    while run.xp >= data::xp_to_next(run.level) {
        run.xp -= data::xp_to_next(run.level);
        run.level += 1;
        run.pending_levels += 1;
        if run.level == super::mobility::LEARN_LEVEL {
            if let Some(m) = super::mobility::for_class(run.class.id) {
                run.banner = Some((format!("You have learned {}! Press F.", m.name), 4.0));
            }
        }
    }
    if run.pending_levels > 0 {
        next.set(Phase::LevelUp);
    }
}

/// Pickups hang still: a wisp's own loop (its circling motes and their trails) stops on its first
/// frame, clip and global sequences alike, as soon as the model attaches.
pub(super) fn still_gems(
    mut players: Query<&mut AnimationPlayer, (With<Gem>, Added<AnimationPlayer>)>,
    mut drives: Query<&mut GlobalSeqDrive, (With<Gem>, Added<GlobalSeqDrive>)>,
) {
    for mut player in &mut players {
        for (_, active) in player.playing_animations_mut() {
            active.set_speed(0.0);
        }
    }
    for mut drive in &mut drives {
        drive.set_paused(true);
    }
}

/// A chest opens when the hero walks into it. It does not fly in on the magnet.
pub(super) fn collect_chests(
    mut commands: Commands,
    mut run: ResMut<Run>,
    mut stash: ResMut<super::stash::Stash>,
    mut next: ResMut<NextState<Phase>>,
    phase: Res<State<Phase>>,
    hero: Query<&Transform, (With<Hero>, Without<Chest>)>,
    chests: Query<(Entity, &Transform, &Chest)>,
) {
    let Ok(h) = hero.single() else { return };
    if !run.alive() {
        return;
    }
    for (e, tf, chest) in &chests {
        if flat(tf.translation - h.translation).length() > 2.2 {
            continue;
        }
        let gold = super::sheet::gold_amount(chest.gold, run.stats().gold_find);
        run.gold_found += gold;
        stash.add_gold(gold);
        if chest.tier == super::campaign::ChestTier::A {
            run.boss_trades += 1;
        }
        run.trades += 1;
        if *phase.get() == Phase::Playing && run.pending_levels == 0 {
            run.trading = true;
            next.set(Phase::LevelUp);
        }
        let kind = match chest.tier {
            super::campaign::ChestTier::C => "Wooden chest",
            super::campaign::ChestTier::B => "Elite chest",
            super::campaign::ChestTier::A => "Boss chest",
        };
        run.banner = Some((format!("{kind} — {gold} gold"), 3.5));
        commands.entity(e).despawn();
    }
}
#[cfg(test)]
mod spawn_tests {
    use super::*;
    use avian3d::prelude::*;
    use benilla_world::collision::MoverTraceExclusions;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn streaming_floor_defers_bosses_and_initial_chests_without_losing_or_duplicating_them() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin::default(),
            bevy::scene::ScenePlugin,
            PhysicsPlugins::new(PostUpdate),
        ));
        app.init_asset::<Mesh>();
        app.init_resource::<MoverTraceExclusions>();
        app.init_resource::<Run>();
        app.init_resource::<Rng>();
        app.init_resource::<Spawner>();
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Elwynn Forest")
            .unwrap();
        let center = super::super::hero::arena(&data::MAPS[map]);
        app.insert_resource(Selection {
            map,
            stage: map,
            ..default()
        });
        app.world_mut().resource_mut::<Run>().elapsed = 10_000.;
        app.world_mut().spawn((
            Hero { vy: 0., air: 0. },
            Transform::from_translation(center),
        ));
        app.finish();
        app.cleanup();
        app.update();
        let old = center - Vec3::X * 100.;
        let deferred = app
            .world_mut()
            .run_system_once(move |floor: Floor| {
                let mut at = old;
                recycle(&floor, &data::MAPS[map], center, &mut at, 1.0);
                at
            })
            .unwrap();
        assert_eq!(deferred, old, "missing streamed floor must defer recycling");
        app.world_mut().run_system_once(spawn_enemies).unwrap();
        let run = app.world().resource::<Run>();
        assert_eq!(run.bosses, 0);
        assert_eq!(run.rings, 0);
        assert!(!run.stocked);
        assert_eq!(
            app.world_mut().query::<&Enemy>().iter(app.world()).count(),
            0
        );
        assert_eq!(
            app.world_mut().query::<&Chest>().iter(app.world()).count(),
            0
        );

        app.world_mut().spawn((
            RigidBody::Static,
            Collider::trimesh(
                vec![
                    Vec3::new(-200., 0., -200.),
                    Vec3::new(200., 0., -200.),
                    Vec3::new(200., 0., 200.),
                    Vec3::new(-200., 0., 200.),
                ],
                vec![[0, 2, 1], [0, 3, 2]],
            ),
            Transform::from_translation(center),
        ));
        app.update();
        app.world_mut()
            .run_system_once(move |floor: Floor| {
                // An outward recycle near the leash must still fit the enemy's body.
                let map = &data::MAPS[map];
                let hero = center + Vec3::X * (map.leash - 1.);
                for radius in [0.6, 3.0] {
                    let mut at = hero - Vec3::X * 100.;
                    recycle(&floor, map, hero, &mut at, radius);
                    assert!(flat(at - center).length() + radius < map.leash);
                    assert!(flat(at - hero).length() >= 6.);
                    assert!((at.y - center.y).abs() < 0.01);
                    assert!(floor.can_spawn(at, radius));
                }
            })
            .unwrap();
        app.world_mut().run_system_once(spawn_enemies).unwrap();
        let bosses = app.world().resource::<Run>().bosses;
        assert!(bosses > 0);
        assert!(app.world().resource::<Run>().stocked);
        assert!(super::super::encounters::next(map, bosses, 10_000.).is_none());
        for _ in 0..3 {
            app.world_mut().run_system_once(spawn_enemies).unwrap();
        }
        assert_eq!(app.world().resource::<Run>().bosses, bosses);
        assert_eq!(
            app.world_mut()
                .query::<&Enemy>()
                .iter(app.world())
                .filter(|e| e.boss)
                .count(),
            bosses as usize
        );
        assert_eq!(
            app.world_mut()
                .query::<&Chest>()
                .iter(app.world())
                .filter(|c| c.tier == super::super::campaign::ChestTier::A)
                .count(),
            3
        );
    }
}

#[cfg(test)]
mod lethal_pickup_tests {
    use super::*;

    #[test]
    fn human_xp_and_gold_apply_to_hero_and_companion_pickups() {
        let mut run = Run {
            race: 1,
            ..Default::default()
        };
        let mut stash = super::super::stash::Stash::default();
        let mut next = NextState::Unchanged;
        bank_gem(&mut run, &mut stash, &mut next, GemKind::Xp(1), 1.0);
        assert!((run.xp - data::XP_RATE * 1.1).abs() < 0.001);
        bank_gem(&mut run, &mut stash, &mut next, GemKind::Gold(100), 1.0);
        assert_eq!(run.gold_found, 120);
        bank_gem(&mut run, &mut stash, &mut next, GemKind::Gold(100), 1.1);
        assert_eq!(run.gold_found, 252);
    }

    #[test]
    fn lethal_hit_blocks_healing_and_level_up_before_death_animation() {
        for kind in [GemKind::Heal(1000.), GemKind::Xp(10000), GemKind::Gold(100)] {
            let mut run = Run {
                hp: 0.,
                ..Default::default()
            };
            let mut stash = super::super::stash::Stash::default();
            let mut next = NextState::Unchanged;
            bank_gem(&mut run, &mut stash, &mut next, kind, 1.);
            assert_eq!(run.hp, 0.);
            assert_eq!(run.xp, 0.);
            assert_eq!(run.level, 1);
            assert_eq!(run.pending_levels, 0);
            assert_eq!(run.gold_found, 0);
            assert!(matches!(next, NextState::Unchanged));
        }
    }
}
