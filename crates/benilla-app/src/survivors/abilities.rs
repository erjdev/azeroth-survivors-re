//! Auto-cast: every owned ability fires on its own cooldown at the targets its kind picks, with
//! the spell's own visuals; damage lands when its missile would.

use bevy::prelude::*;

use super::data::{AbilityDef, Effect, Kind};
use super::enemies::{self, Enemy, Loot};
use super::hero::{self, Hero};
use super::units::{self, flat, Fx, Marks, UnitSpec, COLOR_HEAL, COLOR_MELEE, COLOR_SPELL};
use super::{Rng, Run, RunEntity};
use crate::net::{FieldChanged, Guid, ObjectStore};

/// One blow landing on one enemy.
#[derive(Message, Clone)]
pub(super) struct Hit {
    pub target: Entity,
    pub damage: f32,
    pub dot: f32,
    pub dot_dur: f32,
    pub effect: Effect,
    pub mark: u32,
    pub mark_dur: f32,
    /// Also hits everything this close to the target, for less.
    pub splash: f32,
    pub crit: bool,
    pub color: u32,
    /// Ability key, or `"Thorns"` for contact reflect. The meter groups on this.
    pub source: &'static str,
}

impl Hit {
    pub fn direct(target: Entity, damage: f32, crit: bool) -> Self {
        Self {
            target,
            damage,
            dot: 0.0,
            dot_dur: 0.0,
            effect: Effect::None,
            mark: 0,
            mark_dur: 0.0,
            splash: 0.0,
            crit,
            color: COLOR_MELEE,
            source: "Thorns",
        }
    }
}

/// Hits in flight, by the run time they land.
#[derive(Resource, Default)]
pub(super) struct PendingHits(pub Vec<(f32, Hit)>);

/// A ground spell's area, ticking on whatever stands in it.
#[derive(Component)]
pub(super) struct GroundArea {
    damage: f32,
    dot: f32,
    dot_dur: f32,
    radius: f32,
    left: f32,
    every: f32,
    next_tick: f32,
    effect: Effect,
    mark: u32,
    /// The spell whose impact plays on each enemy a tick hits, 0 for none.
    impact: u32,
    source: &'static str,
}

#[derive(Component)]
pub(super) struct Totem {
    cast: u32,
    damage: f32,
    range: f32,
    radius: f32,
    left: f32,
    tick: f32,
    pulse: bool,
    source: &'static str,
}

#[derive(Clone, Copy)]
struct Foe {
    e: Entity,
    pos: Vec3,
    hp: f32,
    dist: f32,
    radius: f32,
}

/// The numbers one cast uses, from the ability's level and the hero's stats.
pub(super) struct Params {
    pub dmg: f32,
    pub dot: f32,
    pub count: usize,
    pub radius: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub crit_damage: f32,
}

impl Params {
    pub fn new(
        o: &super::Owned,
        stats: &super::Stats,
        milestones: survivors_core::talents::Milestones,
    ) -> Self {
        let scoped = stats.for_spell(o.def.key);
        let stats = &scoped;
        let effect = milestones.cast(o.def.key);
        let mult = o.damage_mult() * stats.might * effect.damage;
        Self {
            dmg: o.def.damage * mult,
            dot: o.def.dot * mult,
            count: o.count(stats) as usize + effect.extra_targets,
            radius: o.radius(stats) * effect.radius,
            duration: o.duration() * effect.duration,
            cooldown: o.cooldown(stats) * effect.cooldown,
            crit: (stats.crit + effect.crit).min(1.0),
            crit_damage: stats.crit_damage,
        }
    }
}

fn mark_secs(def: &AbilityDef, duration: f32) -> f32 {
    match def.effect {
        Effect::Slow(_, t) | Effect::Weaken(_, t) => t,
        Effect::Root(t) | Effect::Stun(t) | Effect::Fear(t) => t,
        _ if duration > 0.0 => duration,
        _ => 3.0,
    }
}

fn hit_for(def: &AbilityDef, p: &Params, target: Entity, damage: f32, crit: bool) -> Hit {
    Hit {
        target,
        damage,
        dot: p.dot,
        dot_dur: p.duration,
        effect: def.effect,
        mark: def.mark,
        mark_dur: mark_secs(def, p.duration),
        splash: if matches!(def.kind, Kind::Bolt | Kind::Strike) {
            p.radius
        } else {
            0.0
        },
        crit,
        color: if def.kind == Kind::Melee {
            COLOR_MELEE
        } else {
            COLOR_SPELL
        },
        source: def.key,
    }
}

fn schedule(
    pending: &mut PendingHits,
    hits: &mut MessageWriter<Hit>,
    now: f32,
    delay: f32,
    hit: Hit,
) {
    if delay <= 0.0 {
        hits.write(hit);
    } else {
        pending.0.push((now + delay, hit));
    }
}

fn roll(rng: &mut Rng, crit: f32, crit_damage: f32, dmg: f32) -> (f32, bool) {
    if rng.f32() < crit {
        (dmg * crit_damage.max(1.0), true)
    } else {
        (dmg, false)
    }
}

fn nearest(foes: &[Foe], range: f32, n: usize) -> Vec<Foe> {
    foes.iter()
        .filter(|f| f.dist <= range + f.radius)
        .take(n)
        .copied()
        .collect()
}

fn toughest(foes: &[Foe], range: f32, n: usize) -> Vec<Foe> {
    let mut v: Vec<Foe> = foes
        .iter()
        .filter(|f| f.dist <= range + f.radius)
        .copied()
        .collect();
    v.sort_by(|a, b| b.hp.total_cmp(&a.hp));
    v.truncate(n);
    v
}

fn around(foes: &[Foe], center: Vec3, r: f32) -> Vec<Foe> {
    foes.iter()
        .filter(|f| flat(f.pos - center).length() <= r + f.radius * 0.5)
        .copied()
        .collect()
}

/// The point among the nearest enemies in range with the most others within `r` of it.
fn densest(foes: &[Foe], range: f32, r: f32) -> Option<Vec3> {
    let near: Vec<&Foe> = foes.iter().filter(|f| f.dist <= range).take(40).collect();
    near.iter()
        .map(|f| {
            (
                f.pos,
                near.iter()
                    .filter(|o| flat(o.pos - f.pos).length() <= r)
                    .count(),
            )
        })
        .max_by_key(|&(_, n)| n)
        .map(|(p, _)| p)
}

fn entities(foes: &[Foe]) -> Vec<Entity> {
    foes.iter().map(|f| f.e).collect()
}

pub(super) fn cast_abilities(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    spells: Option<Res<crate::ui_action::Spells>>,
    hero: Query<(Entity, &Transform), With<Hero>>,
    enemies: Query<(Entity, &Transform, &Enemy)>,
    mut fx: Fx,
    mut pending: ResMut<PendingHits>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    if run.dead_t.is_some() || dt <= 0.0 {
        return;
    }
    let Ok((me, htf)) = hero.single() else { return };
    let hpos = htf.translation;
    let stats = run.stats();
    let now = run.elapsed;
    let mut foes: Vec<Foe> = enemies
        .iter()
        .filter(|(_, _, e)| !e.dead)
        .map(|(e, tf, en)| Foe {
            e,
            pos: tf.translation,
            hp: en.hp,
            dist: hero::reach(hpos, tf.translation),
            radius: en.radius,
        })
        .collect();
    foes.sort_by(|a, b| a.dist.total_cmp(&b.dist));
    let speed_of = |spell: u32| {
        spells
            .as_deref()
            .and_then(|s| s.catalog.get(spell))
            .map_or(0.0, |d| d.speed)
    };

    for i in 0..run.abilities.len() {
        run.abilities[i].cd -= dt;
        if run.abilities[i].cd > 0.0 {
            continue;
        }
        let o = &run.abilities[i];
        let def = o.def;
        let milestone = run.milestones.cast(def.key);
        let p = Params::new(o, &stats, run.milestones);
        let crit_chance = p.crit;
        let speed = speed_of(def.spell);
        let fired = match def.kind {
            Kind::Bolt | Kind::Strike | Kind::Melee => {
                let reach = if def.kind == Kind::Melee {
                    def.range + 0.5
                } else {
                    def.range
                };
                let targets = if matches!(
                    def.key,
                    "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
                ) {
                    toughest(&foes, reach, p.count)
                } else {
                    nearest(&foes, reach, p.count)
                };
                if targets.is_empty() {
                    false
                } else {
                    if def.kind == Kind::Melee {
                        fx.swing(me, targets[0].e, p.dmg);
                        for f in &targets {
                            fx.impact(f.e, def.spell);
                        }
                    } else {
                        fx.cast(me, def.spell, &entities(&targets), None);
                    }
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, crit_chance, p.crit_damage, p.dmg);
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            }
            Kind::Nova | Kind::Cone => {
                let mut targets = around(&foes, hpos, p.radius);
                if def.kind == Kind::Cone {
                    if let Some(first) = targets.first().copied() {
                        let aim = flat(first.pos - hpos).normalize_or_zero();
                        targets.retain(|f| {
                            let d = flat(f.pos - hpos);
                            d.length() < 2.0 || d.normalize_or_zero().dot(aim) >= 0.8
                        });
                    }
                }
                if targets.is_empty() {
                    false
                } else {
                    fx.cast(me, def.spell, &entities(&targets), None);
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, crit_chance, p.crit_damage, p.dmg);
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            }
            Kind::Ground | Kind::GroundSelf => {
                let center = if def.kind == Kind::GroundSelf {
                    foes.first()
                        .filter(|f| f.dist <= p.radius + 4.0)
                        .map(|_| hpos)
                } else {
                    densest(&foes, def.range, p.radius)
                };
                match center {
                    None => false,
                    Some(center) => {
                        fx.cast(me, def.spell, &[], Some(center));
                        let area = if def.overhead {
                            commands
                                .spawn((Transform::from_translation(center), RunEntity))
                                .id()
                        } else {
                            let serial = run.guid();
                            units::spawn_area_object(
                                &mut commands,
                                serial,
                                def.spell,
                                p.radius,
                                center,
                            )
                        };
                        let burst = def.duration <= 0.0;
                        commands.entity(area).insert(GroundArea {
                            damage: p.dmg,
                            dot: p.dot,
                            dot_dur: if burst { 5.0 } else { 0.0 },
                            radius: p.radius,
                            left: if burst { 1.5 } else { p.duration },
                            every: if burst { f32::INFINITY } else { 1.0 },
                            next_tick: if burst { 0.0 } else { 0.4 },
                            effect: def.effect,
                            mark: def.mark,
                            impact: if def.overhead { def.spell } else { 0 },
                            source: def.key,
                        });
                        true
                    }
                }
            }
            Kind::Chain => match nearest(&foes, def.range, 1).first().copied() {
                None => false,
                Some(first) => {
                    let mut chain = vec![first];
                    while chain.len() < p.count + 1 {
                        let last = chain[chain.len() - 1].pos;
                        let next = foes
                            .iter()
                            .filter(|f| !chain.iter().any(|c| c.e == f.e))
                            .filter(|f| flat(f.pos - last).length() <= p.radius)
                            .min_by(|a, b| {
                                flat(a.pos - last)
                                    .length()
                                    .total_cmp(&flat(b.pos - last).length())
                            });
                        match next {
                            Some(f) => chain.push(*f),
                            None => break,
                        }
                    }
                    fx.cast(me, def.spell, &entities(&chain), None);
                    for (k, f) in chain.iter().enumerate() {
                        let (dmg, crit) = roll(
                            &mut rng,
                            crit_chance,
                            p.crit_damage,
                            p.dmg * 0.85f32.powi(k as i32),
                        );
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            0.08 * k as f32,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            },
            Kind::Dot => {
                let mut pool: Vec<Foe> = foes
                    .iter()
                    .filter(|f| f.dist <= def.range)
                    .take(p.count * 3)
                    .copied()
                    .collect();
                if pool.is_empty() {
                    false
                } else {
                    let mut targets = Vec::with_capacity(p.count);
                    while targets.len() < p.count && !pool.is_empty() {
                        targets.push(pool.swap_remove(rng.index(pool.len())));
                    }
                    fx.cast(me, def.spell, &entities(&targets), None);
                    for f in &targets {
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, 0.0, false),
                        );
                    }
                    true
                }
            }
            Kind::Aura => {
                let targets = around(&foes, hpos, p.radius);
                if targets.is_empty() {
                    false
                } else {
                    for f in targets.iter().take(6) {
                        fx.impact(f.e, def.spell);
                    }
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, crit_chance, p.crit_damage, p.dmg);
                        hits.write(hit_for(def, &p, f.e, dmg, crit));
                    }
                    true
                }
            }
            Kind::Orbit => {
                let targets = nearest(&foes, p.radius, p.count);
                if targets.is_empty() {
                    false
                } else {
                    for f in &targets {
                        fx.impact(f.e, 403);
                        let (dmg, crit) = roll(&mut rng, crit_chance, p.crit_damage, p.dmg);
                        hits.write(hit_for(def, &p, f.e, dmg, crit));
                    }
                    true
                }
            }
            Kind::Shield => {
                run.shield = p.dmg;
                run.shield_t = p.duration;
                fx.cast(me, def.spell, &[me], None);
                true
            }
            Kind::Immune => {
                if foes.first().is_some_and(|f| f.dist <= 5.0) {
                    run.immune_t = p.duration;
                    fx.cast(me, def.spell, &[me], None);
                    true
                } else {
                    false
                }
            }
            Kind::Thorns => {
                if foes.first().is_some_and(|f| f.dist <= 8.0) {
                    run.thorns = p.dmg;
                    run.thorns_t = p.duration;
                    run.thorns_spell = def.spell;
                    fx.cast(me, def.spell, &[me], None);
                    true
                } else {
                    false
                }
            }
            Kind::Heal => {
                if run.hp < stats.max_hp * 0.85 {
                    run.heal_rate = p.dmg / p.duration.max(1.0);
                    run.heal_t = p.duration;
                    fx.cast(me, def.spell, &[me], None);
                    fx.number(me, p.dmg, false, COLOR_HEAL);
                    true
                } else {
                    false
                }
            }
            Kind::Totem => {
                if foes
                    .first()
                    .is_some_and(|f| f.dist <= def.range.max(p.radius) + 6.0)
                {
                    fx.cast(me, def.spell, &[], None);
                    let a = rng.range(0.0, std::f32::consts::TAU);
                    let pos = hpos + Vec3::new(a.cos(), 0.0, a.sin()) * 2.0;
                    let serial = run.guid();
                    let totem = units::spawn_unit(
                        &mut commands,
                        &UnitSpec {
                            serial,
                            display: def.totem.0,
                            scale: 1.0,
                            level: 60,
                            faction: units::FACTION_FRIENDLY,
                            bytes0: 0,
                            weapons: &[],
                            sheath: 0,
                        },
                        pos,
                        0.0,
                    );
                    commands.entity(totem).insert(Totem {
                        cast: def.totem.1,
                        damage: p.dmg,
                        range: def.range,
                        radius: p.radius,
                        left: p.duration,
                        tick: 0.3,
                        pulse: def.key == "magma_totem",
                        source: def.key,
                    });
                    true
                } else {
                    false
                }
            }
        };
        run.abilities[i].cd = if fired { p.cooldown } else { 0.2 };
        if fired {
            let healed = finish_milestone_cast(&mut run, milestone, stats.max_hp);
            if healed > 0.0 {
                fx.number(me, healed, false, COLOR_HEAL);
            }
            if milestone.storm_fraction > 0.0 {
                // One hero-centered area per Judgement cast, independent of +target.
                for hit in storm_hits(&foes, hpos, stats.area, p.dmg * milestone.storm_fraction) {
                    fx.impact(hit.target, 2812);
                    hits.write(hit);
                }
            }
        }
    }
}

fn storm_hits(foes: &[Foe], center: Vec3, area: f32, damage: f32) -> Vec<Hit> {
    around(foes, center, 6.0 * area)
        .into_iter()
        .map(|foe| {
            let mut hit = Hit::direct(foe.e, damage, false);
            hit.source = "divine_storm";
            hit.color = COLOR_SPELL;
            hit
        })
        .collect()
}

/// Once per successful cast, not once per hit or extra target.
fn finish_milestone_cast(
    run: &mut Run,
    effect: survivors_core::talents::CastEffect,
    max_hp: f32,
) -> f32 {
    let healed = (max_hp * effect.heal_fraction).min((max_hp - run.hp).max(0.0));
    run.hp += healed;
    let absorb = max_hp * effect.shield_fraction;
    if absorb > 0.0 && (run.shield_t <= 0.0 || absorb >= run.shield) {
        // Refresh this small absorb without stacking or extending a stronger existing shield.
        run.shield = absorb;
        run.shield_t = effect.shield_seconds;
    }
    if effect.wrath_advance > 0.0 {
        for owned in &mut run.abilities {
            if owned.def.key == "hammer_of_wrath" {
                owned.cd = (owned.cd - effect.wrath_advance).max(0.0);
            }
        }
    }
    healed
}

/// Ground areas tick, and totems fight, until they run out.
#[allow(clippy::type_complexity)]
pub(super) fn tick_areas(
    mut commands: Commands,
    time: Res<Time>,
    run: Res<Run>,
    mut rng: ResMut<Rng>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut areas: Query<(Entity, &Transform, &mut GroundArea)>,
    mut totems: Query<(Entity, &Transform, &mut Totem), Without<GroundArea>>,
    enemies: Query<(Entity, &Transform, &Enemy), (Without<GroundArea>, Without<Totem>)>,
    mut fx: Fx,
    mut pending: ResMut<PendingHits>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let stats = run.stats();
    let now = run.elapsed;
    let live: Vec<Foe> = enemies
        .iter()
        .filter(|(_, _, e)| !e.dead)
        .map(|(e, tf, en)| Foe {
            e,
            pos: tf.translation,
            hp: en.hp,
            dist: 0.0,
            radius: en.radius,
        })
        .collect();

    for (e, tf, mut area) in &mut areas {
        let stats = stats.for_spell(area.source);
        area.left -= dt;
        area.next_tick -= dt;
        if area.next_tick <= 0.0 {
            area.next_tick += area.every;
            for (i, f) in around(&live, tf.translation, area.radius)
                .into_iter()
                .enumerate()
            {
                if area.impact != 0 && i < 12 {
                    fx.impact(f.e, area.impact);
                }
                let (dmg, crit) = roll(&mut rng, stats.crit, stats.crit_damage, area.damage);
                hits.write(Hit {
                    target: f.e,
                    damage: dmg,
                    dot: area.dot,
                    dot_dur: area.dot_dur,
                    effect: area.effect,
                    mark: area.mark,
                    mark_dur: 1.5,
                    splash: 0.0,
                    crit,
                    color: COLOR_SPELL,
                    source: area.source,
                });
            }
        }
        if area.left <= 0.0 {
            commands.entity(e).despawn();
        }
    }

    for (e, tf, mut totem) in &mut totems {
        let stats = stats.for_spell(totem.source);
        totem.left -= dt;
        if totem.left <= 0.0 {
            commands
                .entity(e)
                .remove::<Totem>()
                .insert(benilla_world::model_fade::DespawnFade::default());
            continue;
        }
        totem.tick -= dt;
        if totem.tick > 0.0 {
            continue;
        }
        let pos = tf.translation;
        if totem.pulse {
            let targets = around(&live, pos, totem.radius);
            totem.tick = 2.0;
            if targets.is_empty() {
                continue;
            }
            fx.cast(e, totem.cast, &[], None);
            for f in &targets {
                let (dmg, crit) = roll(&mut rng, stats.crit, stats.crit_damage, totem.damage);
                hits.write(Hit {
                    color: COLOR_SPELL,
                    source: totem.source,
                    ..Hit::direct(f.e, dmg, crit)
                });
            }
        } else {
            totem.tick = 1.0;
            let target = live
                .iter()
                .map(|f| (flat(f.pos - pos).length(), f))
                .filter(|(d, _)| *d <= totem.range)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            let Some((dist, f)) = target else { continue };
            fx.cast(e, totem.cast, &[f.e], None);
            let speed = spells
                .as_deref()
                .and_then(|s| s.catalog.get(totem.cast))
                .map_or(0.0, |d| d.speed);
            let delay = if speed > 0.0 { dist / speed } else { 0.0 };
            let (dmg, crit) = roll(&mut rng, stats.crit, stats.crit_damage, totem.damage);
            schedule(
                &mut pending,
                &mut hits,
                now,
                delay,
                Hit {
                    color: COLOR_SPELL,
                    source: totem.source,
                    ..Hit::direct(f.e, dmg, crit)
                },
            );
        }
    }
}

/// Land every hit due: damage and its number, the over-time part, crowd control, the marks, and
/// death with its loot.
fn refresh_deep_wounds(dots: &mut Vec<enemies::Dot>, total: f32) {
    if total <= 0.0 {
        return;
    }
    if let Some(dot) = dots.iter_mut().find(|dot| dot.source == "Deep Wounds") {
        dot.dps = dot.dps.max(total / 3.0);
        dot.left = 3.0;
        // Keep the next tick: repeated criticals must not postpone all bleed damage.
    } else {
        if dots.len() >= 6 {
            dots.remove(0);
        }
        dots.push(enemies::Dot {
            dps: total / 3.0,
            left: 3.0,
            tick: 1.0,
            source: "Deep Wounds",
        });
    }
}

pub(super) fn apply_hits(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    mut account: ResMut<super::account::Account>,
    selection: Res<super::Selection>,
    mut pending: ResMut<PendingHits>,
    mut incoming: MessageReader<Hit>,
    mut enemies: Query<(
        Entity,
        &Guid,
        &Transform,
        &mut Enemy,
        &mut ObjectStore,
        &mut Marks,
    )>,
    mut fx: Fx,
    mut loot: MessageWriter<Loot>,
    mut fields: MessageWriter<FieldChanged>,
) {
    let dt = time.delta_secs();
    let now = run.elapsed;
    let mut due: Vec<Hit> = incoming.read().cloned().collect();
    pending.0.retain(|(t, h)| {
        if *t <= now {
            due.push(h.clone());
            false
        } else {
            true
        }
    });

    // Splash reaches the target's neighbours at seventy percent.
    let mut all = Vec::with_capacity(due.len());
    for h in due {
        if h.splash > 0.0 {
            if let Ok((_, _, center, ..)) = enemies.get(h.target) {
                let c = center.translation;
                for (e, _, tf, en, ..) in enemies.iter() {
                    if e != h.target && !en.dead && flat(tf.translation - c).length() <= h.splash {
                        all.push(Hit {
                            target: e,
                            damage: h.damage * 0.7,
                            splash: 0.0,
                            mark: 0,
                            ..h.clone()
                        });
                    }
                }
            }
        }
        all.push(Hit { splash: 0.0, ..h });
    }

    let mut kill = |commands: &mut Commands,
                    run: &mut Run,
                    account: &mut super::account::Account,
                    rng: &mut Rng,
                    e: Entity,
                    pos: Vec3,
                    en: &mut Enemy| {
        en.dead = true;
        en.hp = 0.0;
        en.dots.clear();
        enemies::mark_dying(commands, e);
        loot.write(Loot {
            pos,
            xp: en.xp,
            boss: en.boss,
            elite: en.elite,
        });
        run.kills += 1;
        if en.boss {
            run.bosses_defeated = run.bosses_defeated.saturating_add(1);
        }
        let mut dropped = false;
        if let Some(class) = selection.class {
            if let Some(zone) = super::campaign::zone_index(selection.map_def().name) {
                if let Some(row) = super::campaign::drop_for(zone, en.def.name) {
                    let chance = super::campaign::drop_chance(row.chance, en.boss, run.keen_eye);
                    if rng.f32() < chance {
                        if let Some(item) = account.collect_drop(class, zone, en.def.name) {
                            run.banner = Some((format!("{item} collected"), 4.0));
                            dropped = true;
                        }
                    }
                }
            }
        }
        if en.boss && !dropped {
            run.banner = Some((format!("{} is slain!", en.def.name), 3.0));
        }
    };

    for h in all {
        let Ok((e, guid, tf, mut en, mut store, mut marks)) = enemies.get_mut(h.target) else {
            continue;
        };
        if en.dead {
            continue;
        }
        let stats = run.stats();
        let mut dmg = h.damage;
        if let Effect::Execute(threshold) = h.effect {
            if !en.boss && en.hp / en.max_hp <= threshold {
                dmg = dmg.max(en.hp);
            }
        }
        if dmg > 0.0 {
            let echoes = super::sheet::strike_count(stats.multistrike, rng.f32());
            dmg *= echoes as f32;
            en.hp -= dmg;
            run.add_damage(h.source, dmg);
            fx.number(e, dmg, h.crit, h.color);
            let heal = if let Effect::Drain(frac) = h.effect {
                dmg * frac
            } else {
                0.0
            } + dmg * stats.leech;
            if heal > 0.0 {
                run.hp = (run.hp + heal).min(stats.max_hp);
            }
        }
        let bleed = run.milestones.critical_bleed(h.source, h.damage, h.crit);
        refresh_deep_wounds(&mut en.dots, bleed);
        if h.dot > 0.0 && h.dot_dur > 0.0 {
            if en.dots.len() >= 6 {
                en.dots.remove(0);
            }
            en.dots.push(enemies::Dot {
                dps: h.dot,
                left: h.dot_dur,
                tick: 1.0,
                source: h.source,
            });
        }
        let boss = en.boss;
        en.stun_t = en.stun_t.max(run.milestones.secondary_stun(h.source, boss));
        if let Some((speed, duration)) = run.milestones.secondary_slow(h.source, boss) {
            en.slow = if en.slow_t > 0.0 {
                en.slow.min(speed)
            } else {
                speed
            };
            en.slow_t = en.slow_t.max(duration);
        }
        match h.effect {
            Effect::Slow(m, t) => {
                let m = if boss { m.max(0.75) } else { m };
                en.slow = if en.slow_t > 0.0 { en.slow.min(m) } else { m };
                en.slow_t = en.slow_t.max(t);
            }
            Effect::Root(t) => en.root_t = en.root_t.max(if boss { t * 0.3 } else { t }),
            Effect::Stun(t) => en.stun_t = en.stun_t.max(if boss { t * 0.3 } else { t }),
            Effect::Fear(t) if !boss => en.fear_t = en.fear_t.max(t),
            Effect::Weaken(m, t) => {
                en.weaken = m;
                en.weaken_t = t;
            }
            _ => {}
        }
        if h.mark != 0 {
            marks.add(h.mark, h.mark_dur);
        }
        if en.hp <= 0.0 {
            kill(
                &mut commands,
                &mut run,
                &mut account,
                &mut rng,
                e,
                tf.translation,
                &mut en,
            );
            units::kill_unit(&mut store);
            marks.clear();
        }
        marks.flush(e, guid.0, &mut store, &mut fields);
    }

    // The over-time ticks, once a second each.
    for (e, guid, tf, mut en, mut store, mut marks) in &mut enemies {
        if en.dead || en.dots.is_empty() {
            continue;
        }
        let stats = run.stats();
        let mut dealt = 0.0;
        for d in &mut en.dots {
            for _ in 0..d.advance(dt) {
                let echoes = super::sheet::strike_count(stats.multistrike, rng.f32());
                let tick = d.dps * echoes as f32;
                dealt += tick;
                run.add_damage(d.source, tick);
            }
        }
        en.dots.retain(|d| d.left > 0.0);
        if dealt > 0.0 {
            en.hp -= dealt;
            if stats.leech > 0.0 {
                run.hp = (run.hp + dealt * stats.leech).min(stats.max_hp);
            }
            fx.number(e, dealt, false, COLOR_SPELL);
            if en.hp <= 0.0 {
                kill(
                    &mut commands,
                    &mut run,
                    &mut account,
                    &mut rng,
                    e,
                    tf.translation,
                    &mut en,
                );
                units::kill_unit(&mut store);
                marks.clear();
                marks.flush(e, guid.0, &mut store, &mut fields);
            }
        }
    }
}

#[cfg(test)]
mod milestone_tests {
    use super::*;
    use crate::survivors::{campaign, data, Owned, Stats};
    use survivors_core::talents::Milestones;

    #[test]
    fn protection_healing_and_shields_apply_once_and_preserve_stronger_defenses() {
        let mut run = Run::default();
        run.milestones = Milestones {
            class: 0,
            points: [0, 0, 30],
            ..Default::default()
        };
        let last_stand = run.milestones.cast("retaliation");
        run.hp = 50.0;
        assert_eq!(finish_milestone_cast(&mut run, last_stand, 100.0), 10.0);
        run.hp = 99.0;
        assert_eq!(finish_milestone_cast(&mut run, last_stand, 100.0), 1.0);
        assert_eq!(finish_milestone_cast(&mut run, last_stand, 100.0), 0.0);

        let shield_slam = run.milestones.cast("heroic_strike");
        let mut stats = run.stats();
        stats.targets = 10;
        let owned = Owned {
            def: data::ability("heroic_strike"),
            level: 1,
            cd: 0.0,
        };
        assert!(Params::new(&owned, &stats, run.milestones).count > 1);
        finish_milestone_cast(&mut run, shield_slam, 100.0);
        assert_eq!(run.shield, 3.0);
        assert_eq!(run.shield_t, 3.0);
        finish_milestone_cast(&mut run, shield_slam, 100.0);
        assert_eq!(run.shield, 3.0, "no stacking");
        let expected = super::super::sheet::mitigate(10.0, stats.armor, stats.reduction) - 3.0;
        assert!((hero::hurt_hero(&mut run, 10.0) - expected).abs() < 0.00001);
        assert_eq!(run.shield, 0.0);
        run.shield = 20.0;
        run.shield_t = 1.0;
        finish_milestone_cast(&mut run, shield_slam, 100.0);
        assert_eq!((run.shield, run.shield_t), (20.0, 1.0));
        run.shield_t = 0.0;
        finish_milestone_cast(&mut run, shield_slam, 100.0);
        assert_eq!((run.shield, run.shield_t), (3.0, 3.0));
    }

    #[test]
    fn fury_casts_and_damage_tradeoff_reach_combat() {
        let mut run = Run::default();
        let base = Milestones {
            class: 0,
            ..Default::default()
        };
        run.milestones = Milestones {
            points: [0, 30, 0],
            ..base
        };
        let owned = Owned {
            def: data::ability("retaliation"),
            level: 1,
            cd: 0.0,
        };
        let original = Params::new(&owned, &run.stats(), base);
        let enhanced = Params::new(&owned, &run.stats(), run.milestones);
        assert!((enhanced.dmg - original.dmg * 1.3).abs() < 0.00001);
        assert_eq!(enhanced.duration, original.duration);
        let effect = run.milestones.cast("execute");
        run.hp = 50.0;
        assert_eq!(finish_milestone_cast(&mut run, effect, 100.0), 3.0);
        run.hp = 99.0;
        assert_eq!(finish_milestone_cast(&mut run, effect, 100.0), 1.0);
        assert_eq!(finish_milestone_cast(&mut run, effect, 100.0), 0.0);

        run.thorns_t = 5.0;
        run.thorns_spell = data::ability("retaliation").spell;
        run.shield_t = 5.0;
        run.shield = 10.0;
        let expected =
            super::super::sheet::mitigate(20.0 * 1.1, run.stats().armor, run.stats().reduction)
                - 10.0;
        assert!((hero::hurt_hero(&mut run, 20.0) - expected).abs() < 0.00001);
        run.immune_t = 1.0;
        assert_eq!(hero::hurt_hero(&mut run, 20.0), 0.0);
        run.immune_t = 0.0;
        run.thorns_t = 0.0;
        let plain = super::super::sheet::mitigate(20.0, run.stats().armor, run.stats().reduction);
        assert_eq!(hero::hurt_hero(&mut run, 20.0), plain);
        run.thorns_t = 5.0;
        run.thorns_spell = 0;
        assert_eq!(hero::hurt_hero(&mut run, 20.0), plain);
    }

    #[test]
    fn damage_over_time_delivers_all_ticks_across_frame_sizes() {
        for frames in [vec![3.5], vec![1.5, 1.5], vec![0.1; 31], vec![1.0; 4]] {
            let mut dot = enemies::Dot {
                dps: 4.0,
                left: 3.0,
                tick: 1.0,
                source: "test",
            };
            let ticks: u32 = frames.into_iter().map(|dt| dot.advance(dt)).sum();
            assert_eq!(ticks, 3);
            assert_eq!(dot.advance(10.0), 0);
        }
        let mut dot = enemies::Dot {
            dps: 4.0,
            left: 2.5,
            tick: 1.0,
            source: "test",
        };
        assert_eq!(dot.advance(5.0), 2, "no tick beyond the effect duration");
    }

    #[test]
    fn arms_cast_milestones_compose_with_individual_training() {
        let stats = Run::default().stats();
        let base = Milestones {
            class: 0,
            ..Default::default()
        };
        let trained = Milestones {
            points: [30, 0, 0],
            ..base
        }
        .with_ranks(&[(0, 133, 1), (0, 135, 1)]);
        let cast = |key, milestones| {
            Params::new(
                &Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.0,
                },
                &stats,
                milestones,
            )
        };
        let sweep = cast("sweeping_strikes", trained);
        let plain = cast("sweeping_strikes", base);
        assert!((sweep.dmg - plain.dmg * 1.2).abs() < 0.00001);
        assert_eq!(sweep.radius, plain.radius * 1.3);
        assert_eq!(sweep.count, plain.count);
        assert_eq!(
            cast("mortal_strike", trained).dmg,
            cast("mortal_strike", base).dmg * 1.5625
        );
    }

    #[test]
    fn deep_wounds_refreshes_without_stacking_or_postponing_ticks() {
        let mut dots = Vec::new();
        refresh_deep_wounds(&mut dots, 12.0);
        assert_eq!(dots[0].advance(0.5), 0);
        refresh_deep_wounds(&mut dots, 6.0);
        assert_eq!(dots.len(), 1);
        assert_eq!(dots[0].dps, 4.0);
        assert_eq!(dots[0].tick, 0.5);
        assert_eq!(dots[0].advance(0.5), 1);
        refresh_deep_wounds(&mut dots, 24.0);
        assert_eq!(dots[0].dps, 8.0);
        assert_eq!(dots[0].advance(4.0), 3);
        assert_eq!(dots[0].left, 0.0);
        assert_eq!(dots[0].advance(1.0), 0);
    }

    #[test]
    fn warrior_defensive_training_composes_with_milestones_and_heal_caps() {
        let base = Milestones {
            class: 0,
            points: [0, 30, 10],
            ..Default::default()
        };
        let trained = base.with_ranks(&[(2, 147, 3), (2, 150, 2), (2, 153, 1), (1, 167, 1)]);
        let mut run = Run::default();
        let stats = run.stats();
        let owned = Owned {
            def: data::ability("retaliation"),
            level: 4,
            cd: 0.0,
        };
        let before = Params::new(&owned, &stats, base);
        let after = Params::new(&owned, &stats, trained);
        assert!((after.dmg - before.dmg * 1.3).abs() < 0.0001);
        assert!((after.duration - before.duration * 1.3).abs() < 0.0001);
        assert_eq!(after.cooldown, before.cooldown);
        assert_eq!(after.count, before.count);
        run.hp = 40.0;
        let healed = finish_milestone_cast(&mut run, trained.cast("retaliation"), 100.0);
        assert!((healed - 15.0).abs() < 0.0001);
        let healed = finish_milestone_cast(&mut run, trained.cast("execute"), 100.0);
        assert!((healed - 5.0).abs() < 0.0001);
        run.hp = 99.0;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("execute"), 100.0),
            1.0
        );
        assert_eq!(run.hp, 100.0);
        assert_eq!(trained.with_ranks(&[]), base);
    }

    #[test]
    fn warrior_training_reaches_casts_without_extra_area_locations() {
        let stats = Run::default().stats();
        let base = Milestones {
            class: 0,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 124, 3), (1, 166, 3), (1, 158, 5)]);
        let cast = |key, milestones| {
            Params::new(
                &Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.0,
                },
                &stats,
                milestones,
            )
        };
        assert!(
            (cast("heroic_strike", trained).cooldown / cast("heroic_strike", base).cooldown - 0.85)
                .abs()
                < 0.00001
        );
        assert!((cast("cleave", trained).dmg / cast("cleave", base).dmg - 1.3).abs() < 0.00001);
        let shout = cast("demoralizing_shout", trained);
        let original = cast("demoralizing_shout", base);
        assert_eq!(shout.radius, original.radius * 1.5);
        assert_eq!(shout.count, original.count);
        assert_eq!(shout.dmg, original.dmg);
        assert_eq!(
            cast("holy_shock", trained).dmg,
            cast("holy_shock", base).dmg
        );
    }

    #[test]
    fn legendary_bonuses_are_cast_local_and_do_not_duplicate_areas() {
        let mut run = Run::default();
        run.tuning.damage = 2.0;
        let base = run.stats();
        let params = |key, stats: &Stats| {
            Params::new(
                &Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.0,
                },
                stats,
                Milestones::default(),
            )
        };
        let fire = params("fireball", &base);
        let area = params("consecration", &base);
        run.worn.equip(campaign::JewelKind::Legend("fireball"));
        let stats = run.stats();
        assert_eq!(stats.might, base.might);
        assert_eq!(stats.targets, base.targets);
        assert_eq!(stats.crit_damage, base.crit_damage);
        let enhanced = params("fireball", &stats);
        assert!((enhanced.dmg - fire.dmg * 1.05).abs() < 0.001);
        assert_eq!(enhanced.count, fire.count + 1);
        assert!((enhanced.crit_damage - fire.crit_damage - 0.15).abs() < 0.001);
        assert_eq!(params("consecration", &stats).dmg, area.dmg);
        run.worn.equip(campaign::JewelKind::Legend("consecration"));
        let stats = run.stats();
        assert_eq!(params("fireball", &stats).dmg, fire.dmg);
        let enhanced = params("consecration", &stats);
        assert!((enhanced.dmg - area.dmg * 1.05).abs() < 0.001);
        assert_eq!(enhanced.count, area.count);
        assert_eq!(enhanced.radius, area.radius);
        // Lingering effects resolve crit bonuses by their stored source.
        let scoped = stats.for_spell("consecration");
        assert!((scoped.crit_damage - base.crit_damage - 0.15).abs() < 0.001);
        assert_eq!(
            scoped.for_spell("consecration").crit_damage,
            scoped.crit_damage
        );
        assert_eq!(stats.for_spell("fireball").crit_damage, base.crit_damage);
    }

    #[test]
    fn divine_storm_hits_one_hero_centered_area_even_with_extra_targets() {
        let mut world = World::new();
        let foes: Vec<Foe> = [2.0, 5.0, 10.0]
            .into_iter()
            .map(|x| Foe {
                e: world.spawn_empty().id(),
                pos: Vec3::X * x,
                hp: 100.0,
                dist: x,
                radius: 0.5,
            })
            .collect();
        let run = Run::default();
        let mut stats = run.stats();
        stats.targets = 10;
        let owned = super::super::Owned {
            def: super::super::data::ability("judgement"),
            level: 1,
            cd: 0.0,
        };
        let talents = Milestones {
            class: 1,
            points: [0, 0, 30],
            ..Default::default()
        };
        let p = Params::new(&owned, &stats, talents);
        let hits = storm_hits(
            &foes,
            Vec3::ZERO,
            1.0,
            p.dmg * talents.cast("judgement").storm_fraction,
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].target, foes[0].e);
        assert_eq!(hits[1].target, foes[1].e);
        assert!(hits
            .iter()
            .all(|hit| hit.source == "divine_storm" && hit.splash == 0.0 && !hit.crit));
        assert_eq!(storm_hits(&foes, Vec3::ZERO, 2.0, p.dmg).len(), 3);
    }

    #[test]
    fn trained_healing_caps_once_per_cast_and_shield_duration_reaches_combat() {
        let mut run = Run::default();
        let mut stats = run.stats();
        stats.targets = 20;
        let base = Milestones {
            class: 1,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 1444, 3), (0, 1443, 2), (1, 1425, 2), (1, 1421, 5)]);
        run.hp = 50.0;
        assert!(
            (finish_milestone_cast(&mut run, trained.cast("holy_shock"), 100.0) - 3.0).abs()
                < 0.001
        );
        let with_daybreak = Milestones {
            points: [20, 0, 0],
            ..trained
        };
        assert!(
            (finish_milestone_cast(&mut run, with_daybreak.cast("holy_shock"), 100.0) - 6.0).abs()
                < 0.001
        );
        assert!((run.hp - 59.0).abs() < 0.001);
        assert!(
            (finish_milestone_cast(&mut run, trained.cast("divine_shield"), 100.0) - 10.0).abs()
                < 0.001
        );
        run.hp = 99.0;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("divine_shield"), 100.0),
            1.0
        );
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("divine_shield"), 100.0),
            0.0
        );
        let shield = super::super::Owned {
            def: super::super::data::ability("divine_shield"),
            level: 1,
            cd: 0.0,
        };
        let before = Params::new(&shield, &stats, base);
        let after = Params::new(&shield, &stats, trained);
        assert!((after.duration - before.duration * 1.3).abs() < 0.001);
        assert!((after.cooldown - before.cooldown * 0.85).abs() < 0.001);
        assert_eq!(after.dmg, before.dmg);
        assert_eq!(trained.cast("divine_shield").extra_targets, 0);
        assert_eq!(
            trained.cast("holy_shock").heal_fraction,
            base.with_ranks(&[(0, 1444, 3)])
                .cast("holy_shock")
                .heal_fraction
        );
    }

    #[test]
    fn individual_spell_training_reaches_shared_combat_parameters() {
        let run = Run::default();
        let stats = run.stats();
        let base = Milestones {
            class: 1,
            points: [30; 3],
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (2, 1631, 2),
            (1, 1521, 3),
            (0, 1627, 5),
            (0, 1435, 1),
            (0, 1502, 1),
        ]);
        for (key, damage, cooldown, crit) in [
            ("judgement", 1.0, 0.8, 0.0),
            ("hammer_of_justice", 1.0, 0.7, 0.0),
            ("consecration", 1.2, 1.0, 0.0),
            ("holy_shock", 1.25, 1.0, 0.1),
        ] {
            let owned = super::super::Owned {
                def: super::super::data::ability(key),
                level: 1,
                cd: 0.0,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
            assert!(
                (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                "{key}"
            );
            assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
            assert_eq!(after.count, before.count);
            assert_eq!(after.radius, before.radius);
        }
    }

    #[test]
    fn paladin_cast_numbers_and_once_per_cast_benefits_reach_combat() {
        let mut run = Run::default();
        let stats = run.stats();
        let holy = Milestones {
            class: 1,
            points: [30, 0, 0],
            ..Default::default()
        };
        let shock = super::super::Owned {
            def: super::super::data::ability("holy_shock"),
            level: 1,
            cd: 0.0,
        };
        let base = Params::new(&shock, &stats, Milestones::default());
        let trained = Params::new(&shock, &stats, holy);
        assert!((trained.crit - base.crit - 0.2).abs() < 0.001);
        assert!((trained.cooldown - base.cooldown * 0.8).abs() < 0.001);
        run.hp = 80.0;
        assert_eq!(
            finish_milestone_cast(&mut run, holy.cast("holy_shock"), 100.0),
            3.0
        );
        run.hp = 99.0;
        assert_eq!(
            finish_milestone_cast(&mut run, holy.cast("holy_shock"), 100.0),
            1.0
        );
        assert_eq!(run.hp, 100.0);
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("hammer_of_wrath"),
            level: 1,
            cd: 0.4,
        });
        let ret = Milestones {
            class: 1,
            points: [0, 0, 30],
            ..Default::default()
        };
        finish_milestone_cast(&mut run, ret.cast("judgement"), 100.0);
        assert_eq!(run.abilities[0].cd, 0.0);
    }

    #[test]
    fn ardent_defender_applies_after_absorption_and_preserves_immunity() {
        let mut run = Run::default();
        run.milestones = Milestones {
            class: 1,
            points: [0, 20, 0],
            ..Default::default()
        };
        run.hp = run.stats().max_hp * 0.2;
        run.shield_t = 5.0;
        run.shield = 10.0;
        let expected = run.milestones.incoming(
            super::super::sheet::mitigate(30.0, run.stats().armor, run.stats().reduction) - 10.0,
            run.hp,
            run.stats().max_hp,
        );
        assert!((hero::hurt_hero(&mut run, 30.0) - expected).abs() < 0.001);
        assert_eq!(run.shield, 0.0);
        run.immune_t = 1.0;
        assert_eq!(hero::hurt_hero(&mut run, 30.0), 0.0);
    }
}
