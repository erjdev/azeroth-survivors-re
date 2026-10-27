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
    frozen: bool,
    winters_chill: survivors_core::talents::WintersChill,
    dist: f32,
    radius: f32,
}

/// The numbers one cast uses, from the ability's level and the hero's stats.
pub(super) struct Params {
    pub dmg: f32,
    pub dot: f32,
    pub count: usize,
    pub radius: f32,
    pub range: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub crit: f32,
    pub crit_damage: f32,
    pub shield_fraction: f32,
    pub shield_seconds: f32,
    pub immune_seconds: f32,
    pub heal_fraction: f32,
    pub blackout_chance: f32,
    pub vampiric_fraction: f32,
    pub shadow_weaving_chance: f32,
    pub effect: Effect,
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
            range: o.def.range * effect.range + effect.range_yards,
            duration: o.duration() * effect.duration,
            effect: milestones.control(
                o.def.key,
                o.def
                    .effect
                    .scaled_duration((1.0 + 0.1 * f32::from(o.level - 1)) * effect.duration),
            ),
            cooldown: o.cooldown(stats) * effect.cooldown,
            crit: (stats.crit + effect.crit).min(1.0),
            crit_damage: stats.crit_damage + effect.crit_damage,
            shield_fraction: effect.shield_fraction,
            shield_seconds: effect.shield_seconds,
            immune_seconds: effect.immune_seconds,
            heal_fraction: effect.heal_fraction,
            blackout_chance: milestones.blackout_chance(o.def.key, 1.0),
            vampiric_fraction: milestones.vampiric_healing(o.def.key, 1.0),
            shadow_weaving_chance: milestones.shadow_weaving_chance(o.def.key, 1.0),
        }
    }
}

fn mark_secs(p: &Params) -> f32 {
    let control = p.effect.duration();
    if control > 0.0 {
        control
    } else if p.duration > 0.0 {
        p.duration
    } else {
        3.0
    }
}

fn hit_for(def: &AbilityDef, p: &Params, target: Entity, damage: f32, crit: bool) -> Hit {
    Hit {
        target,
        damage,
        dot: p.dot,
        dot_dur: p.duration,
        effect: p.effect,
        mark: def.mark,
        mark_dur: mark_secs(p),
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
    if !run.alive() || dt <= 0.0 {
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
            frozen: en.frozen_t > 0.0,
            winters_chill: en.winters_chill,
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
        let crit_chance = (p.crit + run.remorseless_crit(def.key)).min(1.0);
        let speed = speed_of(def.spell);
        let fired = match def.kind {
            Kind::Bolt | Kind::Strike | Kind::Melee => {
                let reach = if def.kind == Kind::Melee {
                    p.range + 0.5
                } else {
                    p.range
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
                        let (dmg, crit) = roll(
                            &mut rng,
                            (crit_chance
                                + run
                                    .milestones
                                    .target_crit(def.key, f.frozen, f.winters_chill))
                            .min(1.0),
                            p.crit_damage,
                            p.dmg,
                        );
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
                        let (dmg, crit) = roll(
                            &mut rng,
                            (crit_chance
                                + run
                                    .milestones
                                    .target_crit(def.key, f.frozen, f.winters_chill))
                            .min(1.0),
                            p.crit_damage,
                            p.dmg,
                        );
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
                    densest(&foes, p.range, p.radius)
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
                            effect: p.effect,
                            mark: def.mark,
                            impact: if def.overhead { def.spell } else { 0 },
                            source: def.key,
                        });
                        true
                    }
                }
            }
            Kind::Chain => match nearest(&foes, p.range, 1).first().copied() {
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
                            (crit_chance
                                + run
                                    .milestones
                                    .target_crit(def.key, f.frozen, f.winters_chill))
                            .min(1.0),
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
                    .filter(|f| f.dist <= p.range)
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
                        let (dmg, crit) = roll(
                            &mut rng,
                            (crit_chance
                                + run
                                    .milestones
                                    .target_crit(def.key, f.frozen, f.winters_chill))
                            .min(1.0),
                            p.crit_damage,
                            p.dmg,
                        );
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
                        let (dmg, crit) = roll(
                            &mut rng,
                            (crit_chance
                                + run
                                    .milestones
                                    .target_crit(def.key, f.frozen, f.winters_chill))
                            .min(1.0),
                            p.crit_damage,
                            p.dmg,
                        );
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
                    .is_some_and(|f| f.dist <= p.range.max(p.radius) + 6.0)
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
                        range: p.range,
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
            run.consume_remorseless(def.key);
            let healed = finish_milestone_cast(&mut run, milestone, stats.max_hp);
            run.record_totem_healing(def.key, healed);
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
    if !run.alive() {
        return 0.0;
    }
    run.immune_t = run.immune_t.max(effect.immune_seconds);
    let healed = run.heal(max_hp * effect.heal_fraction, max_hp);
    let absorb = max_hp * effect.shield_fraction;
    if absorb > 0.0 && (run.shield_t <= 0.0 || absorb >= run.shield) {
        // Refresh this small absorb without stacking or extending a stronger existing shield.
        run.shield = absorb;
        run.shield_t = effect.shield_seconds;
    }
    if let Some((key, seconds)) = effect.cooldown_advance {
        for owned in &mut run.abilities {
            if owned.def.key == key {
                owned.cd = (owned.cd - seconds).max(0.0);
            }
        }
    }
    healed
}

/// A tick can ready an owned spell, but never grant one or bank an instant-cast charge.
fn apply_nightfall(run: &mut Run, rng: &mut Rng, source: &str, removed: f32) {
    let chance = run.milestones.nightfall_chance(source, removed);
    if !run.alive() || chance <= 0.0 {
        return;
    }
    let Some(bolt) = run
        .abilities
        .iter_mut()
        .find(|owned| owned.def.key == "shadow_bolt")
    else {
        return;
    };
    if bolt.cd > 0.0 && rng.f32() < chance {
        bolt.cd = 0.0;
    }
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
            frozen: en.frozen_t > 0.0,
            winters_chill: en.winters_chill,
            dist: 0.0,
            radius: en.radius,
        })
        .collect();

    for (e, tf, mut area) in &mut areas {
        let stats = stats.for_spell(area.source);
        let training = run.milestones.cast(area.source);
        let crit_chance = (stats.crit + training.crit).min(1.0);
        let crit_damage = stats.crit_damage + training.crit_damage;
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
                let (dmg, crit) = roll(
                    &mut rng,
                    (crit_chance
                        + run
                            .milestones
                            .target_crit(area.source, f.frozen, f.winters_chill))
                    .min(1.0),
                    crit_damage,
                    area.damage,
                );
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

/// One burn per enemy; carry unpaid ticks forward without delaying the next tick.
fn refresh_ignite(dots: &mut Vec<enemies::Dot>, added: f32) {
    if !added.is_finite() || added <= 0.0 {
        return;
    }
    if let Some(dot) = dots.iter_mut().find(|dot| dot.source == "Ignite") {
        let ticks = (dot.left - dot.tick + 0.00001).floor().max(-1.0) + 1.0;
        let total = dot.dps * ticks + added;
        if total.is_finite() {
            dot.dps = total / 4.0;
            dot.left = 4.0;
        }
    } else {
        if dots.len() >= 6 {
            dots.remove(0);
        }
        dots.push(enemies::Dot {
            dps: added / 4.0,
            left: 4.0,
            tick: 1.0,
            source: "Ignite",
        });
    }
}

fn apply_freeze(en: &mut Enemy, duration: f32) {
    let duration = if en.boss { duration * 0.3 } else { duration };
    en.root_t = en.root_t.max(duration);
    en.frozen_t = en.frozen_t.max(duration);
}

fn apply_blackout(en: &mut Enemy) {
    en.stun_t = en.stun_t.max(if en.boss { 0.9 } else { 3.0 });
}

fn apply_impact(en: &mut Enemy) {
    en.stun_t = en.stun_t.max(if en.boss { 0.3 } else { 1.0 });
}

fn apply_frostbite(en: &mut enemies::Enemy) {
    apply_freeze(en, 5.0);
}

fn apply_secondary_control(
    en: &mut Enemy,
    milestones: survivors_core::talents::Milestones,
    source: &'static str,
) {
    let boss = en.boss;
    en.stun_t = en.stun_t.max(milestones.secondary_stun(source, boss));
    en.root_t = en.root_t.max(milestones.secondary_root(source, boss));
    if let Some((speed, duration)) = milestones.secondary_slow(source, boss) {
        en.slow = if en.slow_t > 0.0 {
            en.slow.min(speed)
        } else {
            speed
        };
        en.slow_t = en.slow_t.max(duration);
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
        run.record_remorseless_kill();
        run.record_spirit_tap_kill(rng);
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
                } else if let Some(slot) = survivors_core::supplemental_gear::poor_drop_slot(
                    zone,
                    en.boss,
                    en.elite,
                    rng.f32(),
                ) {
                    if let Some(item) = account.collect_poor_drop(class, slot) {
                        run.banner = Some((format!("{item} collected"), 4.0));
                        dropped = true;
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
        let mut dmg = h.damage
            * run
                .milestones
                .shadow_damage_multiplier(h.source, en.shadow_weaving);
        if let Effect::Execute(threshold) = h.effect {
            if !en.boss && en.hp / en.max_hp <= threshold {
                dmg = dmg.max(en.hp);
            }
        }
        if dmg > 0.0 {
            let echoes = super::sheet::strike_count(stats.multistrike, rng.f32());
            dmg *= echoes as f32;
            let removed = dmg.min(en.hp.max(0.0));
            en.hp -= dmg;
            run.add_damage(h.source, dmg);
            fx.number(e, dmg, h.crit, h.color);
            let heal = if let Effect::Drain(frac) = h.effect {
                dmg * frac
            } else {
                0.0
            } + dmg * stats.leech
                + run.milestones.vampiric_healing(h.source, removed);
            if heal > 0.0 {
                run.heal(heal, stats.max_hp);
            }
        }
        run.record_flurry_hit(h.source, h.damage, h.crit);
        run.record_vengeance_hit(h.source, dmg, h.crit);
        let bleed = run.milestones.critical_bleed(h.source, h.damage, h.crit);
        refresh_deep_wounds(&mut en.dots, bleed);
        refresh_ignite(
            &mut en.dots,
            run.milestones.ignite(h.source, h.damage, h.crit),
        );
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
        let weaving = run.milestones.shadow_weaving_chance(h.source, h.damage);
        if weaving > 0.0 && rng.f32() < weaving {
            en.shadow_weaving.apply();
        }
        let chill_chance = run.milestones.winters_chill_chance(h.source, h.damage);
        if chill_chance > 0.0 && rng.f32() < chill_chance {
            en.winters_chill.apply();
        }
        let blackout = run.milestones.blackout_chance(h.source, h.damage);
        if blackout > 0.0 && rng.f32() < blackout {
            apply_blackout(&mut en);
        }
        let impact_chance = run.milestones.impact_chance(h.source, h.damage);
        if impact_chance > 0.0 && rng.f32() < impact_chance {
            apply_impact(&mut en);
        }
        let freeze_chance = run.milestones.frostbite_chance(h.source, h.damage);
        if freeze_chance > 0.0 && rng.f32() < freeze_chance {
            apply_frostbite(&mut en);
        }
        let boss = en.boss;
        apply_secondary_control(&mut en, run.milestones, h.source);
        match h.effect {
            Effect::Slow(m, t) => {
                let m = if boss { m.max(0.75) } else { m };
                en.slow = if en.slow_t > 0.0 { en.slow.min(m) } else { m };
                en.slow_t = en.slow_t.max(t);
            }
            Effect::Root(t) if h.source == "frost_nova" => apply_freeze(&mut en, t),
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
        let mut blackout = false;
        let mut vampiric_heal = 0.0;
        let mut remaining_hp = en.hp.max(0.0);
        let mut weaving = en.shadow_weaving;
        for d in &mut en.dots {
            for _ in 0..d.advance(dt) {
                let echoes = super::sheet::strike_count(stats.multistrike, rng.f32());
                let tick = d.dps
                    * echoes as f32
                    * run.milestones.shadow_damage_multiplier(d.source, weaving);
                let removed = tick.max(0.0).min(remaining_hp);
                remaining_hp = (remaining_hp - removed).max(0.0);
                vampiric_heal += run.milestones.vampiric_healing(d.source, removed);
                apply_nightfall(&mut run, &mut rng, d.source, removed);
                dealt += tick;
                run.add_damage(d.source, tick);
                let chance = run.milestones.shadow_weaving_chance(d.source, tick);
                if chance > 0.0 && rng.f32() < chance {
                    weaving.apply();
                }
                let chance = run.milestones.blackout_chance(d.source, tick);
                if chance > 0.0 && rng.f32() < chance {
                    blackout = true;
                }
            }
        }
        en.shadow_weaving = weaving;
        if blackout {
            apply_blackout(&mut en);
        }
        en.dots.retain(|d| d.left > 0.0);
        if dealt > 0.0 {
            en.hp -= dealt;
            if stats.leech > 0.0 || vampiric_heal > 0.0 {
                run.heal(dealt * stats.leech + vampiric_heal, stats.max_hp);
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
    fn lethal_hits_cannot_be_undone_by_cast_healing_or_grant_defenses() {
        let effect = survivors_core::talents::CastEffect {
            heal_fraction: 1.,
            shield_fraction: 0.1,
            shield_seconds: 3.,
            immune_seconds: 0.5,
            ..Default::default()
        };
        for hp in [0., -1.] {
            let mut run = Run {
                hp,
                ..Default::default()
            };
            assert_eq!(finish_milestone_cast(&mut run, effect, 160.), 0.);
            assert_eq!(run.hp, hp);
            assert_eq!(run.shield, 0.);
            assert_eq!(run.immune_t, 0.);
        }
    }

    #[test]
    fn mage_nova_defenses_preserve_longer_buffs_and_absorb_real_damage() {
        let trained = Milestones {
            class: 6,
            points: [0, 0, 30],
            ..Default::default()
        }
        .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
        let effect = trained.cast("frost_nova");
        let mut run = Run {
            hp: 100.,
            ..Default::default()
        };
        for _ in 0..3 {
            finish_milestone_cast(&mut run, effect, 100.);
        }
        assert!((run.shield - 15.).abs() < 0.00001);
        assert_eq!((run.shield_t, run.immune_t), (5., 0.5));
        assert_eq!(hero::hurt_hero(&mut run, 10.), 0.);
        assert_eq!(run.hp, 100.);
        run.immune_t = 0.;
        assert_eq!(hero::hurt_hero(&mut run, 10.), 0.);
        assert!(run.shield < 15.);
        run.shield = 30.;
        run.shield_t = 1.;
        run.immune_t = 2.;
        finish_milestone_cast(&mut run, effect, 100.);
        assert_eq!((run.shield, run.shield_t, run.immune_t), (30., 1., 2.));
    }

    #[test]
    fn protection_healing_and_shields_apply_once_and_preserve_stronger_defenses() {
        let mut run = Run {
            milestones: Milestones {
                class: 0,
                points: [0, 0, 30],
                ..Default::default()
            },
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
    fn trained_protection_absorb_refreshes_without_stacking() {
        let mut run = Run::default();
        let effect = Milestones {
            class: 0,
            points: [0, 0, 30],
            ..Default::default()
        }
        .with_ranks(&[(2, 1601, 5), (2, 145, 3), (2, 148, 1)])
        .cast("heroic_strike");
        for _ in 0..3 {
            finish_milestone_cast(&mut run, effect, 100.);
            assert!((run.shield - 11.).abs() < 0.00001);
            assert_eq!(run.shield_t, 4.5);
        }
        run.shield = 20.;
        run.shield_t = 1.;
        finish_milestone_cast(&mut run, effect, 100.);
        assert_eq!((run.shield, run.shield_t), (20., 1.));
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
    fn trained_warrior_bleed_delivers_its_snapshot_over_three_ticks() {
        let talents = Milestones {
            class: 0,
            points: [10, 0, 0],
            ..Default::default()
        }
        .with_ranks(&[(0, 121, 3), (0, 127, 3)]);
        let mut dots = Vec::new();
        refresh_deep_wounds(
            &mut dots,
            talents.critical_bleed("heroic_strike", 100., true),
        );
        assert_eq!(dots.len(), 1);
        let ticks = dots[0].advance(3.);
        assert_eq!(ticks, 3);
        assert!((dots[0].dps * ticks as f32 - 31.2).abs() < 0.00001);
        assert_eq!(dots[0].advance(1.), 0);
        assert_eq!(
            talents.critical_bleed(dots[0].source, dots[0].dps, true),
            0.
        );
    }

    #[test]
    fn talent_control_preserves_longer_effects_and_other_state() {
        let trained = Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(1, 1353, 1), (2, 1304, 5), (2, 1325, 1)]);
        for boss in [false, true] {
            let mut enemy = Enemy {
                def: &data::roster(0).enemies[0],
                hp: 100.,
                max_hp: 100.,
                speed: 3.,
                damage: 5.,
                radius: 0.6,
                xp: 1,
                boss,
                elite: false,
                attack_cd: 0.,
                slow: 0.4,
                slow_t: 8.,
                root_t: 0.,
                frozen_t: 0.,
                winters_chill: Default::default(),
                shadow_weaving: Default::default(),
                stun_t: 0.,
                fear_t: 0.,
                weaken: 1.,
                weaken_t: 0.,
                dots: Vec::new(),
                dead: false,
            };
            apply_impact(&mut enemy);
            assert_eq!(enemy.stun_t, if boss { 0.3 } else { 1. });
            apply_frostbite(&mut enemy);
            assert_eq!(enemy.root_t, if boss { 1.5 } else { 5. });
            assert_eq!(enemy.frozen_t, enemy.root_t);
            enemy.root_t = 8.;
            apply_frostbite(&mut enemy);
            assert_eq!(enemy.root_t, 8.);
            assert_eq!(enemy.frozen_t, if boss { 1.5 } else { 5. });
            assert_eq!((enemy.slow, enemy.slow_t, enemy.hp), (0.4, 8., 100.));
            enemy.root_t = 0.;
            enemy.frozen_t = 0.;
            apply_secondary_control(&mut enemy, trained, "concussive_shot");
            assert_eq!(enemy.stun_t, if boss { 0.3 } else { 1. });
            apply_secondary_control(&mut enemy, trained, "serpent_sting");
            assert!((enemy.stun_t - if boss { 0.45 } else { 1.5 }).abs() < 0.00001);
            let survival = Milestones {
                points: [0, 0, 30],
                ..trained
            };
            apply_secondary_control(&mut enemy, survival, "serpent_sting");
            assert!((enemy.stun_t - if boss { 0.75 } else { 2.5 }).abs() < 0.00001);
            apply_secondary_control(&mut enemy, trained, "explosive_trap");
            assert!((enemy.root_t - if boss { 0.6 } else { 2. }).abs() < 0.00001);
            enemy.stun_t = 4.;
            apply_impact(&mut enemy);
            enemy.root_t = 5.;
            for source in [
                "concussive_shot",
                "serpent_sting",
                "explosive_trap",
                "volley",
            ] {
                apply_secondary_control(&mut enemy, trained, source);
            }
            assert_eq!((enemy.stun_t, enemy.root_t), (4., 5.));
            assert_eq!(enemy.frozen_t, 0.);
            assert_eq!((enemy.slow, enemy.slow_t), (0.4, 8.));
            assert_eq!(enemy.hp, 100.);
        }
    }

    #[test]
    fn hunter_milestones_reach_casts_without_expanding_areas() {
        let mut stats = Run::default().stats();
        stats.crit_damage = 2.2;
        stats.targets = 4;
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let trained = Milestones {
            points: [0, 30, 0],
            ..base
        }
        .with_ranks(&[(1, 1346, 5), (1, 1349, 5), (1, 1345, 1)]);
        for key in [
            "arcane_shot",
            "aimed_shot",
            "multi_shot",
            "volley",
            "explosive_trap",
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            let shot = matches!(key, "arcane_shot" | "aimed_shot" | "multi_shot");
            let cooldown = if key == "arcane_shot" {
                0.675
            } else if shot {
                0.9
            } else {
                1.
            };
            assert!(
                (after.cooldown - before.cooldown * cooldown).abs() < 0.0001,
                "{key}"
            );
            let damage = if key == "aimed_shot" { 1.5625 } else { 1. };
            assert!((after.dmg - before.dmg * damage).abs() < 0.0001, "{key}");
            assert!(
                (after.crit_damage - before.crit_damage - if shot { 0.5 } else { 0. }).abs()
                    < 0.0001
            );
            assert_eq!(
                after.count,
                before.count + usize::from(key == "multi_shot") * 2
            );
            assert_eq!(after.radius, before.radius);
            assert_eq!(after.duration, before.duration);
        }
    }

    #[test]
    fn hunter_survival_casts_apply_damage_and_nonstacking_defense() {
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let trained = Milestones {
            points: [0, 0, 30],
            ..base
        }
        .with_ranks(&[(1, 1348, 5), (2, 1306, 2), (2, 1325, 1)]);
        let stats = Run::default().stats();
        for level in [1, 5] {
            for (key, multiplier) in [
                ("serpent_sting", 1.32),
                ("explosive_trap", 1.56),
                ("concussive_shot", 1.),
                ("volley", 1.),
            ] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!(
                    (after.dmg - before.dmg * multiplier).abs() < 0.0001,
                    "{key}"
                );
                assert_eq!(
                    (after.count, after.radius, after.duration),
                    (before.count, before.radius, before.duration)
                );
                assert_eq!(after.cooldown, before.cooldown);
            }
        }
        let mut run = Run {
            hp: 100.,
            ..Default::default()
        };
        let effect = trained.cast("concussive_shot");
        for _ in 0..4 {
            finish_milestone_cast(&mut run, effect, 200.);
        }
        assert_eq!((run.shield, run.shield_t), (10., 3.));
        run.shield = 30.;
        run.shield_t = 2.;
        finish_milestone_cast(&mut run, effect, 200.);
        assert_eq!((run.shield, run.shield_t), (30., 2.));
        assert_eq!(run.hp, 100.);
    }

    #[test]
    fn hunter_shot_crit_bonuses_compose_with_gear_and_keep_one_volley_area() {
        let mut stats = Run::default().stats();
        stats.crit = 0.98;
        stats.crit_damage = 2.2;
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(1, 1344, 5), (1, 1349, 5), (1, 1347, 3), (1, 1362, 5)]);
        for (key, damage, chance, critical_damage) in [
            ("multi_shot", 1.176, 1., 2.5),
            ("serpent_sting", 1.05, 1., 2.5),
            ("volley", 1.12, 0.98, 2.2),
            ("explosive_trap", 1., 0.98, 2.2),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 3,
                cd: 0.,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            assert!((after.dmg - before.dmg * damage).abs() < 0.0001, "{key}");
            assert!((after.dot - before.dot * damage).abs() < 0.0001, "{key}");
            assert!((after.crit - chance).abs() < 0.00001, "{key}");
            assert!(
                (after.crit_damage - critical_damage).abs() < 0.00001,
                "{key}"
            );
            assert_eq!(after.count, before.count);
            assert_eq!(after.radius, before.radius);
            assert_eq!(after.duration, before.duration);
        }
    }

    #[test]
    fn ignite_rolls_unpaid_damage_without_stacking_or_delaying_ticks() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(1, 34, 5)]);
        let mut dots = Vec::new();
        refresh_ignite(&mut dots, mage.ignite("fireball", 100., true));
        assert_eq!(dots.len(), 1);
        assert_eq!(dots[0].advance(1.5), 1);
        let paid = dots[0].dps;
        assert!((paid - 10.).abs() < 0.0001);
        assert_eq!(dots[0].tick, 0.5);
        refresh_ignite(&mut dots, mage.ignite("fire_blast", 50., true));
        assert_eq!(dots.len(), 1);
        assert_eq!(dots[0].tick, 0.5);
        assert!((dots[0].dps - 12.5).abs() < 0.0001); // 30 unpaid + 20 new, over four ticks.
        assert_eq!(dots[0].advance(0.5), 1);
        let next_paid = dots[0].dps;
        assert_eq!(dots[0].advance(3.5), 3);
        assert!((paid + next_paid + dots[0].dps * 3. - 60.).abs() < 0.0001);
        assert_eq!(dots[0].advance(1.), 0);
        assert_eq!(mage.ignite(dots[0].source, dots[0].dps, true), 0.);
        refresh_ignite(&mut dots, f32::NAN);
        refresh_ignite(&mut dots, 0.);
        assert_eq!(dots[0].left, 0.);
    }

    #[test]
    fn mage_hits_apply_trained_critical_control_and_target_range() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for (source, low, high, expected, frozen, critical, chilled) in [
            ("flamestrike", 0.10, 0.20, 20.0, false, true, false),
            ("blizzard", 0.0, 0.04, 25.0, false, true, false),
            ("flamestrike", 0.30, 0.40, 20.0, true, true, false),
            ("flamestrike", 0.30, 0.40, 10.0, false, false, false),
            ("blizzard", 0.30, 0.40, 25.0, true, true, false),
            ("blizzard", 0.30, 0.40, 10.0, false, false, false),
            ("blizzard", 0.06, 0.09, 25.0, false, true, true),
            ("blizzard", 0.06, 0.09, 10.0, false, false, false),
        ] {
            let seed = (1..10000)
                .find(|seed| {
                    let value = Rng(*seed).f32();
                    value >= low && value < high
                })
                .unwrap();
            let mut app = App::new();
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            app.insert_resource(time)
                .insert_resource(Run {
                    milestones: Milestones {
                        class: 6,
                        ..Default::default()
                    }
                    .with_ranks(&[
                        (1, 31, 3),
                        (1, 33, 3),
                        (2, 73, 5),
                        (2, 67, 5),
                        (2, 65, 3),
                        (2, 63, 3),
                    ]),
                    ..Default::default()
                })
                .insert_resource(Rng(seed))
                .init_resource::<PlaySeq>()
                .init_resource::<PendingHits>()
                .add_message::<CastEvent>()
                .add_message::<SpellGoTargets>()
                .add_message::<SwingMessage>()
                .add_message::<crate::combat_text::CombatTextSpawn>()
                .add_message::<Hit>();
            let enemy = app
                .world_mut()
                .spawn((
                    Transform::default(),
                    Enemy {
                        def: &data::roster(0).enemies[0],
                        hp: 100.,
                        max_hp: 100.,
                        speed: 3.,
                        damage: 5.,
                        radius: 0.6,
                        xp: 1,
                        boss: false,
                        elite: false,
                        attack_cd: 0.,
                        slow: 1.,
                        slow_t: 0.,
                        root_t: 8.,
                        frozen_t: if frozen { 3. } else { 0. },
                        winters_chill: Default::default(),
                        shadow_weaving: Default::default(),
                        stun_t: 0.,
                        fear_t: 0.,
                        weaken: 1.,
                        weaken_t: 0.,
                        dots: Vec::new(),
                        dead: false,
                    },
                ))
                .id();
            if chilled {
                let mut en = app.world_mut().get_mut::<Enemy>(enemy).unwrap();
                for _ in 0..5 {
                    en.winters_chill.apply();
                }
            }
            let area_effect = {
                let run = app.world().resource::<Run>();
                Params::new(
                    &super::super::Owned {
                        def: data::ability(source),
                        level: 1,
                        cd: 0.,
                    },
                    &run.stats(),
                    run.milestones,
                )
                .effect
            };
            app.world_mut().spawn((
                Transform::default(),
                GroundArea {
                    damage: 10.,
                    dot: 0.,
                    dot_dur: 0.,
                    radius: 5.,
                    left: 3.,
                    every: 1.,
                    next_tick: 0.,
                    effect: area_effect,
                    mark: 0,
                    impact: 0,
                    source,
                },
            ));
            app.world_mut().run_system_once(tick_areas).unwrap();
            let hits: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<Hit>>()
                .drain()
                .collect();
            assert_eq!(hits.len(), 1);
            assert_eq!(hits[0].target, enemy);
            assert_eq!(hits[0].crit, critical, "{source}, frozen={frozen}");
            assert_eq!(hits[0].damage, expected, "{source}");
            assert_eq!(hits[0].splash, 0.);
            assert_eq!(hits[0].effect, area_effect);
            if source == "blizzard" {
                let Effect::Slow(speed, seconds) = hits[0].effect else {
                    panic!("missing Blizzard slow")
                };
                assert!((speed - 0.21).abs() < 0.00001);
                assert!((seconds - 4.5).abs() < 0.00001);
            }

            if low == 0.06 {
                app.world_mut()
                    .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
                app.world_mut()
                    .resource_mut::<Run>()
                    .abilities
                    .push(super::super::Owned {
                        def: data::ability("frostbolt"),
                        level: 1,
                        cd: 0.,
                    });
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let direct: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                assert_eq!(direct.len(), 1);
                assert_eq!(direct[0].crit, chilled);
                app.world_mut()
                    .get_mut::<Enemy>(enemy)
                    .unwrap()
                    .winters_chill
                    .advance(15.);
                app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let expired: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                assert_eq!(expired.len(), 1);
                assert!(!expired[0].crit);
            }

            if source == "blizzard" && low == 0.0 {
                verify_mage_reach(&mut app, enemy);
            }
            if low == 0.30 {
                app.world_mut()
                    .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
                app.world_mut()
                    .resource_mut::<Run>()
                    .abilities
                    .push(super::super::Owned {
                        def: data::ability("fireball"),
                        level: 1,
                        cd: 0.,
                    });
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let direct: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                assert_eq!(direct.len(), 1);
                assert_eq!(direct[0].crit, frozen);
                assert_eq!(direct[0].target, enemy);
                // Once the freeze expires, a longer ordinary root must not keep Shatter active.
                app.world_mut().get_mut::<Enemy>(enemy).unwrap().frozen_t = 0.;
                app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let expired: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                assert_eq!(expired.len(), 1);
                assert!(!expired[0].crit);
            }
        }
    }

    fn verify_mage_reach(app: &mut App, enemy: Entity) {
        use bevy::ecs::system::RunSystemOnce;
        app.world_mut()
            .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        for key in [
            "fireball",
            "flamestrike",
            "frostbolt",
            "blizzard",
            "frost_nova",
        ] {
            let def = data::ability(key);
            let distance = if def.kind == Kind::Nova {
                def.radius
            } else {
                def.range
            } + 1.;
            app.world_mut()
                .get_mut::<Transform>(enemy)
                .unwrap()
                .translation = Vec3::X * distance;
            for trained in [false, true] {
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.milestones = if trained {
                        base.with_ranks(&[(1, 28, 2), (2, 741, 2)])
                    } else {
                        base
                    };
                    run.abilities = vec![super::super::Owned {
                        def,
                        level: 1,
                        cd: 0.,
                    }];
                }
                let before = app
                    .world_mut()
                    .query::<&GroundArea>()
                    .iter(app.world())
                    .count();
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let hits: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                let areas: Vec<_> = app
                    .world_mut()
                    .query::<(&GroundArea, &Transform)>()
                    .iter(app.world())
                    .collect();
                if def.kind == Kind::Ground {
                    assert_eq!(
                        areas.len(),
                        before + usize::from(trained),
                        "{key}, trained={trained}"
                    );
                    if trained {
                        let matching: Vec<_> = areas
                            .iter()
                            .filter(|(area, tf)| area.source == key && tf.translation.x == distance)
                            .collect();
                        assert_eq!(matching.len(), 1, "one ground location");
                        assert_eq!(
                            matching[0].0.radius, def.radius,
                            "target range must not enlarge the area"
                        );
                    }
                    assert!(hits.is_empty());
                } else {
                    assert_eq!(hits.len(), usize::from(trained), "{key}, trained={trained}");
                    if trained {
                        assert_eq!(hits[0].target, enemy);
                    }
                }
            }
        }
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = Milestones {
                class: 6,
                points: [0, 0, 30],
                ..Default::default()
            }
            .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
            run.abilities = vec![super::super::Owned {
                def: data::ability("frost_nova"),
                level: 1,
                cd: 0.,
            }];
        }
        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::X * 30.;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert_eq!((run.shield, run.immune_t), (0., 0.));
        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::X * 2.;
        app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert!((run.shield - run.stats().max_hp * 0.15).abs() < 0.0001);
        assert_eq!((run.shield_t, run.immune_t), (5., 0.5));
    }

    #[test]
    fn frost_slow_training_scales_control_without_extending_ground_area_lifetime() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(2, 65, 3), (2, 63, 3)]);
        let stats = Run::default().stats();
        for (key, speed, seconds) in [
            ("frostbolt", 0.41, 5.75),
            ("cone_of_cold", 0.41, 6.3),
            ("blizzard", 0.21, 4.65),
        ] {
            let owned = super::super::Owned {
                def: data::ability(key),
                level: 2,
                cd: 0.,
            };
            let a = Params::new(&owned, &stats, base);
            let b = Params::new(&owned, &stats, trained);
            let Effect::Slow(actual_speed, actual_seconds) = b.effect else {
                panic!("slow lost")
            };
            assert!((actual_speed - speed).abs() < 0.00001, "{key}");
            assert!((actual_seconds - seconds).abs() < 0.00001, "{key}");
            assert_eq!(
                (a.dmg, a.duration, a.cooldown, a.radius),
                (b.dmg, b.duration, b.cooldown, b.radius)
            );
            assert_eq!(
                hit_for(owned.def, &b, Entity::PLACEHOLDER, b.dmg, false).effect,
                b.effect
            );
        }
    }

    #[test]
    fn mage_training_reaches_cast_parameters_and_preserves_area_geometry() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (0, 76, 5),
            (0, 80, 5),
            (0, 81, 3),
            (1, 26, 5),
            (1, 1141, 2),
            (1, 31, 3),
            (1, 33, 3),
            (1, 35, 5),
            (2, 37, 5),
            (2, 73, 5),
            (2, 61, 3),
            (2, 64, 3),
        ]);
        let mut stats = Run::default().stats();
        stats.targets = 10;
        for (key, damage, cooldown, crit, critical_damage) in [
            ("arcane_missiles", 1.375, 1., 0., 0.),
            ("arcane_explosion", 1.1, 1., 0.06, 0.),
            ("fireball", 1.1, 0.8, 0.06, 0.),
            ("fire_blast", 1.1, 1., 0.10, 0.),
            ("flamestrike", 1.1, 1., 0.21, 0.),
            ("frostbolt", 1.06, 0.8, 0., 0.5),
            ("blizzard", 1.06, 1., 0., 0.5),
            ("cone_of_cold", 1.378, 1., 0., 0.5),
        ] {
            for level in [1, 3] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.0001,
                    "{key}"
                );
                assert!((after.crit - before.crit - crit).abs() < 0.0001, "{key}");
                assert!(
                    (after.crit_damage - before.crit_damage - critical_damage).abs() < 0.0001,
                    "{key}"
                );
                assert_eq!(after.radius, before.radius);
                assert_eq!(after.count, before.count);
                assert_eq!(after.effect, before.effect);
            }
        }
    }

    #[test]
    fn mage_instability_and_precision_reach_all_spell_casts() {
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 421, 3), (2, 1649, 3)]);
        let stats = Run::default().stats();
        for key in data::CLASSES[6]
            .pool
            .iter()
            .copied()
            .chain(["holy_shock", "consecration"])
        {
            let mage = data::CLASSES[6].pool.contains(&key);
            let elemental = mage && !matches!(key, "arcane_missiles" | "arcane_explosion");
            let factor = if mage { 1.03 } else { 1.0 } * if elemental { 1.03 } else { 1.0 };
            for level in [1, 3, 8] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * factor).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * factor).abs() < 0.001, "{key}");
                assert!(
                    (after.crit - before.crit - if mage { 0.03 } else { 0.0 }).abs() < 0.0001,
                    "{key}"
                );
                assert_eq!(after.count, before.count);
                assert_eq!(after.radius, before.radius);
                assert_eq!(after.cooldown, before.cooldown);
                assert_eq!(after.effect, before.effect);
            }
        }
    }

    #[test]
    fn remorseless_bonus_reaches_actual_casts_and_survives_failed_target_selection() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for key in ["sinister_strike", "eviscerate", "gouge", "kidney_shot"] {
            let mut app = App::new();
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            let mut run = Run {
                class: &data::CLASSES[3],
                milestones: Milestones {
                    class: 3,
                    ..Default::default()
                }
                .with_ranks(&[(0, 272, 2)]),
                ..Default::default()
            };
            run.bonus.targets = 1;
            run.abilities.push(super::super::Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            });
            // Both target rolls lie above base crit and below the trained bonus.
            let seed = (1..100000)
                .find(|seed| {
                    let mut rng = Rng(*seed);
                    (0..2).all(|_| {
                        let roll = rng.f32();
                        roll > 0.10 && roll < 0.40
                    })
                })
                .unwrap();
            run.record_remorseless_kill();
            app.insert_resource(time)
                .insert_resource(run)
                .insert_resource(Rng(seed))
                .init_resource::<PlaySeq>()
                .init_resource::<PendingHits>()
                .add_message::<CastEvent>()
                .add_message::<SpellGoTargets>()
                .add_message::<SwingMessage>()
                .add_message::<crate::combat_text::CombatTextSpawn>()
                .add_message::<Hit>();
            app.world_mut()
                .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
            app.world_mut().run_system_once(cast_abilities).unwrap();
            assert_eq!(
                app.world().resource::<Run>().remorseless_until,
                20.,
                "{key}: no target"
            );
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            for x in [1., 2.] {
                app.world_mut().spawn((
                    Transform::from_xyz(x, 0., 0.),
                    Enemy {
                        def: &data::roster(0).enemies[0],
                        hp: 100.,
                        max_hp: 100.,
                        speed: 3.,
                        damage: 5.,
                        radius: 0.6,
                        xp: 1,
                        boss: false,
                        elite: false,
                        attack_cd: 0.,
                        slow: 1.,
                        slow_t: 0.,
                        root_t: 0.,
                        frozen_t: 0.,
                        winters_chill: Default::default(),
                        shadow_weaving: Default::default(),
                        stun_t: 0.,
                        fear_t: 0.,
                        weaken: 1.,
                        weaken_t: 0.,
                        dots: Vec::new(),
                        dead: false,
                    },
                ));
            }
            for expected_crit in [true, false] {
                app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let hits: Vec<_> = app
                    .world_mut()
                    .resource_mut::<Messages<Hit>>()
                    .drain()
                    .collect();
                assert_eq!(hits.len(), 2, "{key}: extra target");
                assert!(
                    hits.iter().all(|hit| hit.crit == expected_crit),
                    "{key}: bonus applies to entire cast exactly once"
                );
                assert_eq!(app.world().resource::<Run>().remorseless_until, 0.);
            }
        }
    }

    #[test]
    fn remorseless_kills_refresh_expire_and_consume_only_on_eligible_casts() {
        let mut run = Run {
            milestones: Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(0, 272, 2)]),
            ..Default::default()
        };
        assert_eq!(run.remorseless_crit("sinister_strike"), 0.0);
        run.record_remorseless_kill();
        assert_eq!(run.remorseless_crit("sinister_strike"), 0.4);
        run.elapsed = 19.9;
        assert_eq!(run.remorseless_crit("gouge"), 0.4);
        run.elapsed = 20.0;
        assert_eq!(run.remorseless_crit("gouge"), 0.0);
        run.record_remorseless_kill();
        run.elapsed = 21.0;
        run.record_remorseless_kill();
        assert_eq!(run.remorseless_until, 41.0);
        assert_eq!(run.remorseless_crit("eviscerate"), 0.4);
        for key in ["rupture", "blade_flurry", "fireball"] {
            assert_eq!(run.remorseless_crit(key), 0.0);
            run.consume_remorseless(key);
            assert_eq!(run.remorseless_crit("eviscerate"), 0.4);
        }
        run.consume_remorseless("kidney_shot");
        assert_eq!(run.remorseless_crit("sinister_strike"), 0.0);
        run.hp = 0.0;
        run.record_remorseless_kill();
        assert_eq!(run.remorseless_until, 0.0);
        run.hp = 100.0;
        run.record_remorseless_kill();
        run.milestones = run.milestones.with_ranks(&[]);
        assert_eq!(run.remorseless_crit("gouge"), 0.0);
    }

    #[test]
    fn rogue_subtlety_casts_advance_only_owned_spells_and_do_not_stack_shields() {
        let trained = Milestones {
            class: 3,
            points: [0, 0, 30],
            ..Default::default()
        };
        let mut run = Run::default();
        for key in ["gouge", "eviscerate", "kidney_shot", "hammer_of_wrath"] {
            run.abilities.push(Owned {
                def: data::ability(key),
                level: 1,
                cd: 3.0,
            });
        }
        let gouge = trained.cast("gouge");
        finish_milestone_cast(&mut run, gouge, 100.0);
        assert_eq!((run.shield, run.shield_t), (3.0, 3.0));
        assert_eq!(
            run.abilities.iter().map(|a| a.cd).collect::<Vec<_>>(),
            vec![3.0, 2.0, 3.0, 3.0]
        );
        finish_milestone_cast(&mut run, trained.cast("kidney_shot"), 100.0);
        assert_eq!(
            run.abilities.iter().map(|a| a.cd).collect::<Vec<_>>(),
            vec![1.0, 2.0, 3.0, 3.0]
        );
        for _ in 0..3 {
            finish_milestone_cast(&mut run, gouge, 100.0);
        }
        assert_eq!(run.abilities[1].cd, 0.0);
        assert_eq!(run.shield, 3.0);
        run.shield = 20.0;
        run.shield_t = 1.0;
        finish_milestone_cast(&mut run, gouge, 100.0);
        assert_eq!((run.shield, run.shield_t), (20.0, 1.0));
        run.hp = 0.0;
        finish_milestone_cast(&mut run, trained.cast("kidney_shot"), 100.0);
        assert_eq!(
            run.abilities[0].cd, 1.0,
            "dead heroes receive no cast triggers"
        );
        run.hp = 100.0;
        run.abilities.clear();
        finish_milestone_cast(&mut run, gouge, 100.0);
        assert!(
            run.abilities.is_empty(),
            "milestones do not grant unowned spells"
        );
    }

    #[test]
    fn rogue_milestones_guarantee_eviscerate_crits_and_keep_flurry_one_area() {
        let mut stats = Run::default().stats();
        stats.crit = 0.0;
        stats.crit_damage = 2.2;
        stats.targets = 10;
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let assassination = Milestones {
            points: [30, 0, 0],
            ..base
        }
        .with_ranks(&[(0, 276, 3), (0, 270, 5)]);
        let eviscerate = Owned {
            def: data::ability("eviscerate"),
            level: 3,
            cd: 0.0,
        };
        let p = Params::new(&eviscerate, &stats, assassination);
        assert_eq!(p.crit, 1.0);
        assert!((p.crit_damage - 2.4).abs() < 0.00001);
        for seed in [1, 17, 2999] {
            let (damage, crit) = roll(&mut Rng(seed), p.crit, p.crit_damage, p.dmg);
            assert!(crit);
            assert!((damage - p.dmg * 2.4).abs() < 0.001);
        }
        let combat = Milestones {
            points: [0, 30, 0],
            ..base
        }
        .with_ranks(&[(1, 201, 2)]);
        for key in ["sinister_strike", "blade_flurry", "gouge", "fireball"] {
            let owned = Owned {
                def: data::ability(key),
                level: 3,
                cd: 0.0,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, combat);
            let cooldown = match key {
                "sinister_strike" => 0.6885,
                "fireball" => 1.0,
                _ => 0.85,
            };
            assert!((after.cooldown - before.cooldown * cooldown).abs() < 0.0001);
            assert_eq!(after.count, before.count);
            let radius = if key == "blade_flurry" { 1.3 } else { 1.0 };
            assert!((after.radius - before.radius * radius).abs() < 0.0001);
            assert_eq!(after.effect, before.effect);
        }
    }

    #[test]
    fn siphon_life_heals_actual_damage_without_overkill_or_revival() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for (points, rank) in [(29, 0), (29, 1), (30, 0), (30, 1), (30, 255)] {
                for hp in [0.0, 50.0, 99.0] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(1.0));
                    let run = Run {
                        hp,
                        milestones: Milestones {
                            class: 7,
                            points: [points, 0, 0],
                            ..Default::default()
                        }
                        .with_ranks(&[(0, 1041, rank)]),
                        ..Default::default()
                    };
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 5.0,
                                max_hp: 100.0,
                                speed: 3.0,
                                damage: 1.0,
                                radius: 0.6,
                                xp: 1,
                                boss: false,
                                elite: false,
                                attack_cd: 0.0,
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
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.0,
                                        left: 3.0,
                                        tick: 1.0,
                                        source: "corruption",
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: false,
                            },
                        ))
                        .id();
                    if !periodic {
                        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                            source: "curse_of_agony",
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    let run = app.world().resource::<Run>();
                    let expected = if hp == 0.0 {
                        0.0
                    } else {
                        (hp + 5.0
                            * ((if points >= 30 { 0.05 } else { 0. })
                                + if rank > 0 { 0.05 } else { 0. }))
                        .min(run.stats().max_hp)
                    };
                    assert!((run.hp - expected).abs() < 0.00001);
                    assert!(app.world().get::<Enemy>(enemy).unwrap().dead);
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    assert!(
                        (app.world().resource::<Run>().hp - expected).abs() < 0.00001,
                        "dead targets cannot heal again"
                    );
                }
            }
        }
    }

    #[test]
    fn nightfall_obeys_rank_probability_ownership_and_does_not_bank_charges() {
        let seed = (1..1000)
            .find(|seed| {
                let roll = Rng(*seed).f32();
                (0.02..0.04).contains(&roll)
            })
            .unwrap();
        for rank in [1, 2] {
            let mut run = Run {
                milestones: Milestones {
                    class: 7,
                    ..Default::default()
                }
                .with_ranks(&[(0, 1002, rank)]),
                ..Default::default()
            };
            run.abilities.clear();
            let mut rng = Rng(seed);
            apply_nightfall(&mut run, &mut rng, "corruption", 1.);
            assert!(run.abilities.is_empty(), "does not grant Shadow Bolt");
            assert_eq!(rng.0, seed, "no roll without an owned spell");
            run.abilities.push(Owned {
                def: data::ability("shadow_bolt"),
                level: 1,
                cd: 0.,
            });
            apply_nightfall(&mut run, &mut rng, "corruption", 1.);
            assert_eq!(rng.0, seed, "no roll or charge for a ready spell");
            run.abilities[0].cd = 8.;
            apply_nightfall(&mut run, &mut rng, "corruption", 0.);
            assert_eq!(rng.0, seed, "no roll after all target health is removed");
            assert_eq!(run.abilities[0].cd, 8.);
            apply_nightfall(&mut run, &mut rng, "corruption", 1.);
            assert_eq!(run.abilities[0].cd, if rank == 2 { 0. } else { 8. });
            let mut expected_rng = Rng(seed);
            expected_rng.f32();
            assert_eq!(rng.0, expected_rng.0, "exactly one roll per eligible tick");
        }
    }

    #[test]
    fn nightfall_only_readies_owned_shadow_bolt_from_living_hero_corruption_ticks() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for (periodic, source) in [
            (false, "corruption"),
            (true, "corruption"),
            (true, "curse_of_agony"),
        ] {
            for rank in [0, 1, 2, 255] {
                for hp in [0.0, 50.0, 99.0] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(1.0));
                    let run = Run {
                        hp,
                        abilities: vec![
                            Owned {
                                def: data::ability("shadow_bolt"),
                                level: 1,
                                cd: 8.,
                            },
                            Owned {
                                def: data::ability("immolate"),
                                level: 1,
                                cd: 7.,
                            },
                        ],
                        milestones: Milestones {
                            class: 7,
                            points: [20, 0, 0],
                            ..Default::default()
                        }
                        .with_ranks(&[(0, 1002, rank)]),
                        ..Default::default()
                    };
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 5.0,
                                max_hp: 100.0,
                                speed: 3.0,
                                damage: 1.0,
                                radius: 0.6,
                                xp: 1,
                                boss: false,
                                elite: false,
                                attack_cd: 0.0,
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
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.0,
                                        left: 3.0,
                                        tick: 1.0,
                                        source,
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: false,
                            },
                        ))
                        .id();
                    if !periodic {
                        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                            source,
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    let run = app.world().resource::<Run>();
                    let expected = if periodic && source == "corruption" && rank > 0 && hp > 0. {
                        0.
                    } else {
                        8.
                    };
                    assert_eq!(run.abilities[0].cd, expected);
                    assert_eq!(run.abilities[1].cd, 7.);
                    // A later pass over the dead target cannot ready another cooldown.
                    app.world_mut().resource_mut::<Run>().abilities[0].cd = 6.;
                    assert!(app.world().get::<Enemy>(enemy).unwrap().dead);
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    assert_eq!(app.world().resource::<Run>().abilities[0].cd, 6.);
                }
            }
        }
    }

    #[test]
    fn flurry_damage_resolution_excludes_ticks_dead_units_and_foreign_sources() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        for (class, talent, source, eligible) in [
            (0, 156, "cleave", true),
            (0, 156, "lightning_bolt", false),
            (5, 602, "lightning_bolt", true),
            (5, 602, "flame_shock", true),
            (5, 602, "magma_totem", false),
            (5, 602, "cleave", false),
        ] {
            for rank in [0, 5] {
                for (critical, periodic, hero_dead, target_dead) in [
                    (true, false, false, false),
                    (false, false, false, false),
                    (true, true, false, false),
                    (true, false, true, false),
                    (true, false, false, true),
                ] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs(1));
                    app.insert_resource(time)
                        .insert_resource(Run {
                            class: &data::CLASSES[class],
                            hp: if hero_dead { 0. } else { 100. },
                            milestones: Milestones {
                                class,
                                ..Default::default()
                            }
                            .with_ranks(&[(1, talent, rank)]),
                            ..Default::default()
                        })
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 1000.,
                                max_hp: 1000.,
                                speed: 3.,
                                damage: 1.,
                                radius: 0.6,
                                xp: 1,
                                boss: false,
                                elite: false,
                                attack_cd: 0.,
                                slow: 1.,
                                slow_t: 0.,
                                root_t: 0.,
                                frozen_t: 0.,
                                winters_chill: Default::default(),
                                shadow_weaving: Default::default(),
                                stun_t: 0.,
                                fear_t: 0.,
                                weaken: 1.,
                                weaken_t: 0.,
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.,
                                        left: 3.,
                                        tick: 1.,
                                        source,
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: target_dead,
                            },
                        ))
                        .id();
                    if !periodic {
                        // Multiple critical hits in the same update refresh one timer.
                        for _ in 0..2 {
                            app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                                source,
                                ..Hit::direct(enemy, 10., critical)
                            });
                        }
                    }
                    let mut schedule = Schedule::default();
                    schedule.add_systems(apply_hits);
                    schedule.run(app.world_mut());
                    let triggered =
                        rank > 0 && eligible && critical && !periodic && !hero_dead && !target_dead;
                    assert_eq!(
                        app.world().resource::<Run>().flurry_t,
                        if triggered { 3. } else { 0. },
                        "{class} {source}"
                    );
                    let removed = if target_dead {
                        0.
                    } else if periodic {
                        10.
                    } else {
                        20.
                    };
                    assert_eq!(app.world().get::<Enemy>(enemy).unwrap().hp, 1000. - removed);
                    if triggered {
                        app.world_mut().resource_mut::<Run>().flurry_t = 1.;
                    }
                    schedule.run(app.world_mut());
                    assert_eq!(
                        app.world().resource::<Run>().flurry_t,
                        if triggered { 1. } else { 0. },
                        "old hits or ticks must not refresh"
                    );
                }
            }
        }
    }

    #[test]
    fn vampiric_embrace_heals_actual_direct_and_periodic_damage_without_reviving() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for rank in [0, 1, 2] {
                for hp in [0.0, 50.0, 99.0] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(1.0));
                    let run = Run {
                        hp,
                        milestones: Milestones {
                            class: 4,
                            ..Default::default()
                        }
                        .with_ranks(&[(2, 484, 1), (2, 1638, rank)]),
                        ..Default::default()
                    };
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 5.0,
                                max_hp: 100.0,
                                speed: 3.0,
                                damage: 1.0,
                                radius: 0.6,
                                xp: 1,
                                boss: false,
                                elite: false,
                                attack_cd: 0.0,
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
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.0,
                                        left: 3.0,
                                        tick: 1.0,
                                        source: "shadow_word_pain",
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: false,
                            },
                        ))
                        .id();
                    if !periodic {
                        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                            source: "mind_blast",
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    let run = app.world().resource::<Run>();
                    let expected = if hp == 0.0 {
                        0.0
                    } else {
                        (hp + 5.0 * (0.20 + 0.05 * f32::from(rank))).min(run.stats().max_hp)
                    };
                    assert!((run.hp - expected).abs() < 0.00001);
                    assert!(app.world().get::<Enemy>(enemy).unwrap().dead);
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    assert!(
                        (app.world().resource::<Run>().hp - expected).abs() < 0.00001,
                        "dead targets cannot heal again"
                    );
                }
            }
        }
    }

    #[test]
    fn shadow_weaving_affects_subsequent_direct_and_periodic_hits() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for rank in [0, 5] {
                for boss in [false, true] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(1.0));
                    let run = Run {
                        milestones: Milestones {
                            class: 4,
                            ..Default::default()
                        }
                        .with_ranks(&[(2, 461, rank)]),
                        ..Default::default()
                    };
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 100.0,
                                max_hp: 100.0,
                                speed: 3.0,
                                damage: 1.0,
                                radius: 0.6,
                                xp: 1,
                                boss,
                                elite: false,
                                attack_cd: 0.0,
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
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.0,
                                        left: 3.0,
                                        tick: 1.0,
                                        source: "shadow_word_pain",
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: false,
                            },
                        ))
                        .id();
                    if !periodic {
                        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                            source: "mind_blast",
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    assert_eq!(app.world().get::<Enemy>(enemy).unwrap().hp, 90.0);
                    if !periodic {
                        let mut hits = app.world_mut().resource_mut::<Messages<Hit>>();
                        hits.clear();
                        hits.write(Hit {
                            source: "mind_blast",
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    let expected = if rank == 0 { 80.0 } else { 79.7 };
                    assert!(
                        (app.world().get::<Enemy>(enemy).unwrap().hp - expected).abs() < 0.00001
                    );
                    let marks = app.world().get::<Enemy>(enemy).unwrap().shadow_weaving;
                    let multiplier = app
                        .world()
                        .resource::<Run>()
                        .milestones
                        .shadow_damage_multiplier("mind_blast", marks);
                    assert!((multiplier - if rank == 0 { 1.0 } else { 1.06 }).abs() < 0.00001);
                }
            }
        }
    }

    #[test]
    fn blackout_stuns_on_direct_and_periodic_shadow_damage_without_stacking() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for rank in [0, 5] {
                for boss in [false, true] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(1.0));
                    let run = Run {
                        milestones: Milestones {
                            class: 4,
                            ..Default::default()
                        }
                        .with_ranks(&[(2, 464, rank)]),
                        ..Default::default()
                    };
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(0))
                        .insert_resource(super::super::account::empty())
                        .init_resource::<super::super::Selection>()
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>()
                        .add_message::<Loot>()
                        .add_message::<FieldChanged>();
                    let enemy = app
                        .world_mut()
                        .spawn((
                            Guid(123),
                            Transform::default(),
                            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                            Marks::debuffs(),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 100.0,
                                max_hp: 100.0,
                                speed: 3.0,
                                damage: 1.0,
                                radius: 0.6,
                                xp: 1,
                                boss,
                                elite: false,
                                attack_cd: 0.0,
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
                                dots: if periodic {
                                    vec![enemies::Dot {
                                        dps: 10.0,
                                        left: 3.0,
                                        tick: 1.0,
                                        source: "shadow_word_pain",
                                    }]
                                } else {
                                    vec![]
                                },
                                dead: false,
                            },
                        ))
                        .id();
                    if !periodic {
                        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                            source: "mind_blast",
                            ..Hit::direct(enemy, 10.0, false)
                        });
                    }
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    let expected = if rank == 0 {
                        0.0
                    } else if boss {
                        0.9
                    } else {
                        3.0
                    };
                    assert_eq!(app.world().get::<Enemy>(enemy).unwrap().stun_t, expected);
                    assert!(!app.world().get::<Enemy>(enemy).unwrap().dead);
                    app.world_mut().get_mut::<Enemy>(enemy).unwrap().stun_t = 5.0;
                    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                        source: "mind_blast",
                        ..Hit::direct(enemy, 10.0, false)
                    });
                    app.world_mut().run_system_once(apply_hits).unwrap();
                    assert_eq!(app.world().get::<Enemy>(enemy).unwrap().stun_t, 5.0);
                }
            }
        }
    }

    #[test]
    fn poor_gear_is_collected_from_direct_and_periodic_ordinary_kills() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for (boss, elite) in [(false, false), (true, false), (false, true)] {
                let mut app = App::new();
                let mut time = Time::<()>::default();
                time.advance_by(std::time::Duration::from_secs_f32(1.0));
                app.insert_resource(time)
                    .insert_resource(Run::default())
                    .insert_resource(Rng(0))
                    .insert_resource(super::super::account::empty())
                    .insert_resource(super::super::Selection {
                        class: Some(0),
                        map: data::MAPS
                            .iter()
                            .position(|map| map.name == "Elwynn Forest")
                            .unwrap(),
                        ..Default::default()
                    })
                    .init_resource::<PlaySeq>()
                    .init_resource::<PendingHits>()
                    .add_message::<CastEvent>()
                    .add_message::<SpellGoTargets>()
                    .add_message::<SwingMessage>()
                    .add_message::<crate::combat_text::CombatTextSpawn>()
                    .add_message::<Hit>()
                    .add_message::<Loot>()
                    .add_message::<FieldChanged>();
                let enemy = app
                    .world_mut()
                    .spawn((
                        Guid(123),
                        Transform::default(),
                        ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                        Marks::debuffs(),
                        Enemy {
                            def: &data::roster(0).enemies[0],
                            hp: 5.0,
                            max_hp: 100.0,
                            speed: 3.0,
                            damage: 1.0,
                            radius: 0.6,
                            xp: 1,
                            boss,
                            elite,
                            attack_cd: 0.0,
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
                            dots: if periodic {
                                vec![enemies::Dot {
                                    dps: 10.0,
                                    left: 3.0,
                                    tick: 1.0,
                                    source: "shadow_word_pain",
                                }]
                            } else {
                                vec![]
                            },
                            dead: false,
                        },
                    ))
                    .id();
                if !periodic {
                    app.world_mut()
                        .resource_mut::<Messages<Hit>>()
                        .write(Hit::direct(enemy, 10.0, false));
                }
                app.world_mut().run_system_once(apply_hits).unwrap();
                let count = usize::from(!boss && !elite);
                assert_eq!(
                    app.world()
                        .resource::<super::super::account::Account>()
                        .supplemental
                        .keys()
                        .count(),
                    count
                );
                assert_eq!(app.world().resource::<Run>().kills, 1);
                assert!(app.world().get::<Enemy>(enemy).unwrap().dead);
                app.world_mut().run_system_once(apply_hits).unwrap();
                assert_eq!(
                    app.world()
                        .resource::<super::super::account::Account>()
                        .supplemental
                        .keys()
                        .count(),
                    count
                );
                assert_eq!(app.world().resource::<Run>().kills, 1);
            }
        }
    }

    #[test]
    fn spirit_tap_procs_from_direct_and_periodic_kills_only_once_per_enemy() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for periodic in [false, true] {
            for rank in [0, 5] {
                let mut app = App::new();
                let mut time = Time::<()>::default();
                time.advance_by(std::time::Duration::from_secs_f32(1.0));
                let run = Run {
                    milestones: Milestones {
                        class: 4,
                        ..Default::default()
                    }
                    .with_ranks(&[(2, 465, rank)]),
                    ..Default::default()
                };
                app.insert_resource(time)
                    .insert_resource(run)
                    .insert_resource(Rng(42))
                    .insert_resource(super::super::account::empty())
                    .init_resource::<super::super::Selection>()
                    .init_resource::<PlaySeq>()
                    .init_resource::<PendingHits>()
                    .add_message::<CastEvent>()
                    .add_message::<SpellGoTargets>()
                    .add_message::<SwingMessage>()
                    .add_message::<crate::combat_text::CombatTextSpawn>()
                    .add_message::<Hit>()
                    .add_message::<Loot>()
                    .add_message::<FieldChanged>();
                let enemy = app
                    .world_mut()
                    .spawn((
                        Guid(123),
                        Transform::default(),
                        ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                        Marks::debuffs(),
                        Enemy {
                            def: &data::roster(0).enemies[0],
                            hp: 5.0,
                            max_hp: 100.0,
                            speed: 3.0,
                            damage: 1.0,
                            radius: 0.6,
                            xp: 1,
                            boss: false,
                            elite: false,
                            attack_cd: 0.0,
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
                            dots: if periodic {
                                vec![enemies::Dot {
                                    dps: 10.0,
                                    left: 3.0,
                                    tick: 1.0,
                                    source: "shadow_word_pain",
                                }]
                            } else {
                                vec![]
                            },
                            dead: false,
                        },
                    ))
                    .id();
                if !periodic {
                    app.world_mut()
                        .resource_mut::<Messages<Hit>>()
                        .write(Hit::direct(enemy, 10.0, false));
                }
                app.world_mut().run_system_once(apply_hits).unwrap();
                let expected = if rank == 5 { 15.0 } else { 0.0 };
                assert_eq!(app.world().resource::<Run>().spirit_tap_t, expected);
                assert_eq!(app.world().resource::<Run>().kills, 1);
                assert!(app.world().get::<Enemy>(enemy).unwrap().dead);
                app.world_mut().resource_mut::<Run>().spirit_tap_t = 7.0;
                app.world_mut().run_system_once(apply_hits).unwrap();
                assert_eq!(
                    app.world().resource::<Run>().spirit_tap_t,
                    7.0,
                    "dead enemy cannot refresh"
                );
                assert_eq!(app.world().resource::<Run>().kills, 1);
            }
        }
    }

    #[test]
    fn priest_milestones_trigger_once_per_successful_cast_and_preserve_target_rules() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for trained_nova in [false, true] {
            for key in [
                "holy_fire",
                "holy_nova",
                "psychic_scream",
                "mind_blast",
                "power_word_shield",
            ] {
                for with_targets in [false, true] {
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(0.1));
                    let mut run = Run {
                        class: &data::CLASSES[4],
                        hp: 20.0,
                        milestones: Milestones {
                            class: 4,
                            points: [30; 3],
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    if trained_nova {
                        run.milestones = run.milestones.with_ranks(&[(1, 442, 1), (1, 404, 5)]);
                    }
                    let max_hp = run.stats().max_hp;
                    run.bonus.targets = 2;
                    run.abilities.push(Owned {
                        def: data::ability(key),
                        level: 1,
                        cd: 0.0,
                    });
                    let advanced = if key == "power_word_shield" {
                        "mind_blast"
                    } else {
                        "shadow_word_pain"
                    };
                    run.abilities.push(Owned {
                        def: data::ability(advanced),
                        level: 1,
                        cd: 5.0,
                    });
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(42))
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>();
                    app.world_mut()
                        .spawn((Transform::default(), Hero { vy: 0.0, air: 0.0 }));
                    if with_targets {
                        for x in [1.0, 2.0, 3.0] {
                            app.world_mut().spawn((
                                Transform::from_xyz(x, 0.0, 0.0),
                                Enemy {
                                    def: &data::roster(0).enemies[0],
                                    hp: 100.0,
                                    max_hp: 100.0,
                                    speed: 3.0,
                                    damage: 1.0,
                                    radius: 0.6,
                                    xp: 1,
                                    boss: false,
                                    elite: false,
                                    attack_cd: 0.0,
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
                            ));
                        }
                    }
                    app.world_mut().run_system_once(cast_abilities).unwrap();
                    let run = app.world().resource::<Run>();
                    let success = with_targets || key == "power_word_shield";
                    let healing = if success {
                        match key {
                            "holy_fire" => 0.03,
                            "holy_nova" => {
                                if trained_nova {
                                    0.11
                                } else {
                                    0.05
                                }
                            }
                            _ => 0.0,
                        }
                    } else {
                        0.0
                    };
                    assert!(
                        (run.hp - (20.0 + max_hp * healing)).abs() < 0.0001,
                        "{key}: heal once"
                    );
                    let advance = success && matches!(key, "mind_blast" | "power_word_shield");
                    assert!(
                        (run.abilities[1].cd - if advance { 2.9 } else { 4.9 }).abs() < 0.0001,
                        "{key}: advance once"
                    );
                    if key == "psychic_scream" {
                        assert!(
                            (run.shield - if success { max_hp * 0.05 } else { 0.0 }).abs() < 0.0001
                        );
                        assert_eq!(run.shield_t, if success { 3.0 } else { 0.0 });
                    }
                    if key == "power_word_shield" {
                        assert_eq!(run.immune_t, 1.0);
                        assert!(run.shield > 0.0);
                    }
                    if with_targets && matches!(key, "holy_nova" | "psychic_scream" | "mind_blast")
                    {
                        assert_eq!(
                            app.world().resource::<Messages<Hit>>().len(),
                            3,
                            "{key}: targets hit once"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn priest_training_reaches_ranked_damage_burns_shields_and_cooldowns() {
        let base = Milestones {
            class: 4,
            ..Default::default()
        };
        let ranks: Vec<_> = survivors_core::talents::definitions(4)
            .iter()
            .map(|node| (0, node.id, node.max_rank))
            .collect();
        let trained = base.with_ranks(&ranks);
        let mut stats = Run::default().stats();
        stats.haste = 0.25;
        stats.cooldown = 0.15;
        for (key, damage, cooldown, duration, crit) in [
            ("smite", 1.155, 0.8, 1.0, 0.10),
            ("holy_fire", 1.155, 0.8, 1.0, 0.10),
            ("holy_nova", 1.05, 0.9, 1.0, 0.10),
            ("shadow_word_pain", 1.155, 0.9, 1.3, 0.0),
            ("mind_blast", 1.155, 0.75, 1.0, 0.05),
            ("psychic_scream", 1.155, 0.72, 1.0, 0.05),
            ("power_word_shield", 1.15, 0.9, 1.0, 0.0),
            ("shadow_bolt", 1.0, 1.0, 1.0, 0.0),
        ] {
            for level in [1, 3, 8] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.0,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.0001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.0001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.0001,
                    "{key}"
                );
                assert!(
                    (after.duration - before.duration * duration).abs() < 0.0001,
                    "{key}"
                );
                assert!((after.crit - before.crit - crit).abs() < 0.0001, "{key}");
                assert_eq!(after.count, before.count);
                assert!(
                    (after.radius - before.radius * if key == "holy_nova" { 1.2 } else { 1.0 })
                        .abs()
                        < 0.0001
                );
                let reach = if matches!(
                    key,
                    "smite" | "holy_fire" | "shadow_word_pain" | "mind_blast"
                ) {
                    1.2
                } else {
                    1.0
                };
                assert!((after.range - before.range * reach).abs() < 0.0001);
                assert_eq!(after.effect, before.effect);
            }
        }
    }

    #[test]
    fn rogue_cooldown_training_reaches_ranked_cast_parameters() {
        let base = Milestones {
            class: 3,
            points: [30, 30, 30],
            ..Default::default()
        };
        let trained = base.with_ranks(&[(0, 277, 3), (2, 247, 2), (0, 382, 1)]);
        let mut stats = Run::default().stats();
        stats.haste = 0.25;
        stats.cooldown = 0.15;
        for (key, factor) in [
            ("blade_flurry", 0.85 * 0.9),
            ("gouge", 0.8 * 0.9),
            ("kidney_shot", 0.8 * 0.9),
            ("sinister_strike", 0.9),
            ("eviscerate", 0.9),
            ("rupture", 0.9),
            ("fireball", 1.0),
        ] {
            for level in [1, 3, 8] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.0,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!(
                    (after.cooldown - before.cooldown * factor).abs() < 0.0001,
                    "{key} rank {level}"
                );
                assert_eq!(after.dmg, before.dmg);
                assert_eq!(after.dot, before.dot);
                assert_eq!(after.effect, before.effect);
                assert_eq!(after.count, before.count);
                assert_eq!(after.radius, before.radius);
            }
        }
    }

    #[test]
    fn rogue_training_reaches_ranked_casts_and_critical_hits() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (0, 276, 3),
            (0, 270, 5),
            (0, 269, 5),
            (1, 201, 2),
            (1, 203, 3),
            (1, 1122, 3),
            (2, 1123, 3),
            (2, 1702, 5),
        ]);
        let mut stats = Run::default().stats();
        // Non-default sheet values exercise composition with equipment/progression.
        stats.might = 1.4;
        stats.crit = 0.98;
        stats.crit_damage = 2.2;
        stats.targets = 4;
        for (key, damage, critical_damage, cooldown) in [
            ("sinister_strike", 1.10 * 1.06, 2.5, 0.9),
            ("eviscerate", 1.10 * 1.15 * 1.06, 2.2, 1.0),
            ("gouge", 1.10, 2.5, 1.0),
            ("kidney_shot", 1.10, 2.2, 1.0),
            ("rupture", 1.10 * 1.30, 2.2, 1.0),
            ("blade_flurry", 1.10, 2.2, 1.0),
            ("fireball", 1.0, 2.2, 1.0),
        ] {
            for level in [1, 3, 8] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.0,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.0001,
                    "{key}"
                );
                assert!(
                    (after.crit_damage - critical_damage).abs() < 0.0001,
                    "{key}"
                );
                assert_eq!(after.crit, if key == "fireball" { 0.98 } else { 1.0 });
                assert_eq!(after.count, before.count);
                assert_eq!(after.radius, before.radius);
                let (damage, crit) = roll(&mut Rng(17), 1.0, after.crit_damage, after.dmg);
                let hit = hit_for(owned.def, &after, Entity::PLACEHOLDER, damage, crit);
                assert!(hit.crit);
                assert!(
                    (hit.damage - after.dmg * critical_damage).abs() < 0.001,
                    "{key}"
                );
                assert_eq!(
                    hit.dot, after.dot,
                    "critical hits do not multiply Rupture ticks"
                );
                assert_eq!(hit.source, key);
                let (normal, crit) = roll(&mut Rng(17), 0.0, after.crit_damage, after.dmg);
                assert_eq!(normal, after.dmg);
                assert!(!crit);
            }
        }
    }

    #[test]
    fn rogue_gouge_stun_and_marker_scale_together_without_changing_kidney_shot() {
        let base = Milestones {
            class: 3,
            ..Default::default()
        };
        let stats = Run::default().stats();
        for rank in 0..=3 {
            let trained = base.with_ranks(&[(1, 203, rank)]);
            for level in [1, 3, 8] {
                for (key, seconds) in [("gouge", 2.0), ("kidney_shot", 3.5)] {
                    let owned = Owned {
                        def: data::ability(key),
                        level,
                        cd: 0.0,
                    };
                    let after = Params::new(&owned, &stats, trained);
                    let factor = if key == "gouge" {
                        1.0 + 0.25 * f32::from(rank)
                    } else {
                        1.0
                    };
                    let expected = seconds * (1.0 + 0.1 * f32::from(level - 1)) * factor;
                    let hit = hit_for(owned.def, &after, Entity::PLACEHOLDER, after.dmg, false);
                    assert!(matches!(hit.effect, Effect::Stun(t) if (t - expected).abs() < 0.0001));
                    assert!((hit.mark_dur - expected).abs() < 0.0001);
                    assert_eq!(hit.splash, 0.0);
                    assert_eq!(hit.dot, 0.0);
                    let reset = Params::new(&owned, &stats, trained.with_ranks(&[]));
                    assert_eq!(reset.effect, Params::new(&owned, &stats, base).effect);
                }
            }
        }
    }

    #[test]
    fn hunter_training_reaches_direct_periodic_and_control_casts() {
        let base = Milestones {
            class: 2,
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (1, 1341, 5),
            (1, 1345, 1),
            (1, 1346, 5),
            (1, 1348, 5),
            (2, 1306, 2),
        ]);
        let stats = Run::default().stats();
        for (key, damage, cooldown) in [
            ("aimed_shot", 1.25, 1.),
            ("arcane_shot", 1., 0.75),
            ("serpent_sting", 1.1, 1.),
            ("explosive_trap", 1.3, 1.),
            ("concussive_shot", 1., 1.),
            ("multi_shot", 1., 1.),
            ("volley", 1., 1.),
        ] {
            for level in [1, 3] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.0001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.0001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.0001,
                    "{key}"
                );
                assert_eq!(after.count, before.count);
                assert_eq!(after.radius, before.radius);
                if key == "concussive_shot" {
                    let hit = hit_for(owned.def, &after, Entity::PLACEHOLDER, after.dmg, false);
                    let expected = 6. * (1. + 0.1 * f32::from(level - 1));
                    assert!(matches!(hit.effect, Effect::Slow(strength, duration)
                        if strength == 0.5 && (duration - expected).abs() < 0.0001));
                    assert!((hit.mark_dur - expected).abs() < 0.0001);
                }
            }
        }
    }

    #[test]
    fn control_duration_reaches_hits_and_markers_without_scaling_strength() {
        let base = Milestones {
            class: 0,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(1, 161, 5)]);
        let stats = Run::default().stats();
        let owned = Owned {
            def: data::ability("demoralizing_shout"),
            level: 3,
            cd: 0.0,
        };
        let p = Params::new(&owned, &stats, trained);
        assert!((p.effect.duration() - 9.0).abs() < 0.00001);
        let hit = hit_for(owned.def, &p, Entity::PLACEHOLDER, p.dmg, false);
        assert!(
            matches!(hit.effect, Effect::Weaken(m, t) if m == 0.5 && (t - 9.0).abs() < 0.00001)
        );
        assert!((hit.mark_dur - 9.0).abs() < 0.00001);
        assert_eq!(p.duration, 0.0, "no artificial area or DoT lifetime");
        let original = Params::new(&owned, &stats, base);
        assert_eq!(p.count, original.count);
        assert_eq!(p.radius, original.radius);
        for effect in [
            Effect::Slow(0.5, 2.0),
            Effect::Weaken(0.7, 2.0),
            Effect::Stun(2.0),
            Effect::Root(2.0),
            Effect::Fear(2.0),
        ] {
            assert_eq!(effect.scaled_duration(1.5).duration(), 3.0);
        }
        for effect in [Effect::None, Effect::Drain(0.2), Effect::Execute(0.3)] {
            assert_eq!(effect.scaled_duration(1.5), effect);
        }
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
    fn warrior_shout_and_retaliation_training_compose_without_more_targets() {
        let stats = Run::default().stats();
        let base = Milestones {
            class: 0,
            points: [0, 20, 0],
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (1, 165, 1),
            (2, 150, 2),
            (1, 161, 5),
            (2, 151, 3),
            (2, 143, 2),
        ]);
        for level in [1, 4, 8] {
            for key in ["retaliation", "demoralizing_shout"] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.0,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                if key == "retaliation" {
                    assert!((after.duration - before.duration * 1.3 * 1.25).abs() < 0.0001);
                    assert_eq!(after.cooldown, before.cooldown);
                    assert_eq!(trained.retaliation_risk(true), 1.1);
                    assert_eq!(trained.retaliation_risk(false), 1.0);
                } else {
                    assert!(
                        (after.effect.duration() - before.effect.duration() * 1.5 * 1.3).abs()
                            < 0.0001
                    );
                    assert!((after.cooldown - before.cooldown * 0.8).abs() < 0.0001);
                    assert_eq!(
                        trained.secondary_slow(key, false),
                        base.secondary_slow(key, false)
                    );
                }
                assert_eq!(after.dmg, before.dmg);
                assert_eq!(after.radius, before.radius);
                assert_eq!(after.count, before.count);
            }
        }
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
    fn warrior_strike_training_reaches_combat_and_caps_critical_chance() {
        let mut stats = Run::default().stats();
        stats.crit = 0.9;
        let base = Milestones::default();
        let trained = base.with_ranks(&[(0, 131, 2), (1, 168, 5)]);
        for (key, crit, cooldown) in [
            ("heroic_strike", 1.0, 1.0),
            ("mortal_strike", 0.9, 0.75),
            ("cleave", 0.9, 1.0),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 4,
                cd: 0.0,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            assert!((after.crit - crit).abs() < 0.00001);
            assert!((after.cooldown - before.cooldown * cooldown).abs() < 0.00001);
            assert_eq!(after.dmg, before.dmg);
            assert_eq!(after.count, before.count);
            assert_eq!(after.radius, before.radius);
        }
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
    fn reach_talents_extend_cast_selection_without_expanding_area_or_bounces() {
        let stats = Run::default().stats();
        for (class, id, rank, key, expected) in [
            (2, 1352, 3, "arcane_shot", 34.),
            (2, 1352, 3, "volley", 30.),
            (5, 1641, 2, "chain_lightning", 24.),
            (7, 1021, 2, "corruption", 24.),
            (7, 1021, 2, "death_coil", 26.4),
            (7, 964, 2, "rain_of_fire", 24.),
            (8, 764, 2, "entangling_roots", 19.2),
            (8, 764, 2, "hurricane", 24.),
        ] {
            let base = Milestones {
                class,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(0, id, rank)]);
            for level in [1, 4, 8] {
                let owned = Owned {
                    def: data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.range - expected).abs() < 0.0001, "{key}");
                assert_eq!(after.radius, before.radius);
                assert_eq!(after.count, before.count);
                assert_eq!(after.dmg, before.dmg);
                assert_eq!(after.dot, before.dot);
                assert_eq!(after.duration, before.duration);
                assert_eq!(after.cooldown, before.cooldown);
                let distance = (before.range + after.range) * 0.5;
                let foes = [Foe {
                    e: Entity::PLACEHOLDER,
                    pos: Vec3::X * distance,
                    radius: 0.6,
                    hp: 100.,
                    frozen: false,
                    winters_chill: Default::default(),
                    dist: distance,
                }];
                assert!(nearest(&foes, before.range, 1).is_empty());
                assert_eq!(nearest(&foes, after.range, 1).len(), 1);
            }
        }
    }

    #[test]
    fn divine_shield_legendary_recharges_only_its_spell_without_stacking() {
        let mut run = Run::default();
        let params = |key, rank, stats: &Stats| {
            Params::new(
                &Owned {
                    def: data::ability(key),
                    level: rank,
                    cd: 0.0,
                },
                stats,
                Milestones::default(),
            )
        };
        let base = run.stats();
        for _ in 0..2 {
            run.worn.equip(campaign::JewelKind::Legend("divine_shield"));
            let stats = run.stats();
            assert_eq!(stats.cooldown, base.cooldown);
            for rank in [1, 4, 8] {
                let before = params("divine_shield", rank, &base);
                let after = params("divine_shield", rank, &stats);
                assert!((after.cooldown - before.cooldown * 0.85).abs() < 0.001);
                assert_eq!(after.duration, before.duration);
                assert_eq!(after.count, before.count);
            }
            assert_eq!(
                params("fireball", 1, &stats).cooldown,
                params("fireball", 1, &base).cooldown
            );
        }
        run.worn.equip(campaign::JewelKind::Legend("fireball"));
        assert_eq!(
            params("divine_shield", 1, &run.stats()).cooldown,
            params("divine_shield", 1, &base).cooldown
        );
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
                frozen: false,
                winters_chill: Default::default(),
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
    fn druid_node_training_reaches_damage_retaliation_and_healing_parameters() {
        let base = Milestones {
            class: 8,
            ..Default::default()
        };
        let ranks: Vec<_> = survivors_core::talents::definitions(8)
            .iter()
            .map(|node| (0, node.id, node.max_rank))
            .collect();
        let trained = base.with_ranks(&ranks);
        let stats = Run::default().stats();
        for level in [1, 5] {
            for (key, damage, cooldown, crit, crit_damage) in [
                ("wrath", 1.1, 0.728, 0., 0.5),
                ("moonfire", 1.21, 0.91, 0.1, 0.5),
                ("starfire", 1.1, 0.7735, 0., 0.5),
                ("thorns", 1.3, 1., 0., 0.),
                ("rejuvenation", 1.3915, 0.5917275, 0., 0.),
                ("hurricane", 1., 1., 0., 0.),
                ("insect_swarm", 1., 1., 0., 0.),
                ("entangling_roots", 1., 1., 0., 0.),
                ("fireball", 1., 1., 0., 0.),
            ] {
                let owned = super::super::Owned {
                    def: super::super::data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                    "{key}"
                );
                assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
                assert!(
                    (after.crit_damage - before.crit_damage - crit_damage).abs() < 0.001,
                    "{key}"
                );
                let range_multiplier = if matches!(
                    key,
                    "wrath" | "moonfire" | "starfire" | "entangling_roots" | "hurricane"
                ) {
                    1.2
                } else {
                    1.0
                };
                assert!(
                    (after.range - before.range * range_multiplier).abs() < 0.001,
                    "{key}"
                );
                assert_eq!(
                    (after.count, after.radius, after.duration),
                    (before.count, before.radius, before.duration)
                );
                let reset = Params::new(&owned, &stats, trained.with_ranks(&[]));
                assert_eq!((reset.dmg, reset.cooldown), (before.dmg, before.cooldown));
            }
        }
    }

    #[test]
    fn spiritual_healing_reaches_unlocked_nova_heal_and_respects_health_caps() {
        for points in [0, 29, 30] {
            let trained = Milestones {
                class: 4,
                points: [0, points, 0],
                ..Default::default()
            }
            .with_ranks(&[(1, 404, 5)]);
            let effect = trained.cast("holy_nova");
            let expected = if points >= 30 { 5.5 } else { 0. };
            let mut run = Run {
                hp: 50.,
                ..Default::default()
            };
            assert!((finish_milestone_cast(&mut run, effect, 100.) - expected).abs() < 0.00001);
            assert!((run.hp - 50. - expected).abs() < 0.00001);
            run.hp = 99.;
            assert_eq!(
                finish_milestone_cast(&mut run, effect, 100.),
                expected.min(1.)
            );
            run.hp = 0.;
            assert_eq!(finish_milestone_cast(&mut run, effect, 100.), 0.);
            assert_eq!(run.hp, 0.);
        }
    }

    #[test]
    fn druid_individual_swiftmend_heals_once_without_consuming_regeneration() {
        let mut run = Run::default();
        for points in [0, 29, 30] {
            let base = Milestones {
                class: 8,
                points: [0, 0, points],
                ..Default::default()
            };
            let trained = base.with_ranks(&[(2, 831, 1), (2, 844, 1)]);
            let effect = trained.cast("rejuvenation");
            let expected = if points >= 30 { 10. } else { 5. };
            run.hp = 50.;
            run.heal_rate = 3.;
            run.heal_t = 10.;
            assert_eq!(finish_milestone_cast(&mut run, effect, 100.), expected);
            assert_eq!(run.hp, 50. + expected);
            assert_eq!((run.heal_rate, run.heal_t), (3., 10.));
            let healing = trained.with_ranks(&[(2, 844, 1), (2, 830, 3), (2, 828, 5), (2, 823, 5)]);
            run.hp = 50.;
            let restored = finish_milestone_cast(&mut run, healing.cast("rejuvenation"), 100.);
            assert!((restored - expected * 1.15 * 1.1 * 1.1).abs() < 0.00001);
            assert_eq!((run.heal_rate, run.heal_t), (3., 10.));
            run.hp = 99.;
            assert_eq!(finish_milestone_cast(&mut run, effect, 100.), 1.);
            run.hp = 100.;
            assert_eq!(finish_milestone_cast(&mut run, effect, 100.), 0.);
            run.hp = 0.;
            assert_eq!(finish_milestone_cast(&mut run, effect, 100.), 0.);
            assert_eq!(run.hp, 0.);
        }
    }

    #[test]
    fn druid_milestones_reach_casts_with_bounded_healing_and_defenses() {
        let base = Milestones {
            class: 8,
            ..Default::default()
        };
        let trained = Milestones {
            points: [30; 3],
            ..base
        };
        let mut run = Run::default();
        let stats = run.stats();
        for (key, damage, cooldown, crit) in [
            ("wrath", 1., 1., 0.05),
            ("moonfire", 1.25, 1., 0.05),
            ("starfire", 1., 1., 0.05),
            ("hurricane", 1., 1., 0.),
            ("insect_swarm", 1., 1., 0.),
            ("entangling_roots", 1., 0.85, 0.),
            ("thorns", 1., 0.85, 0.),
            ("rejuvenation", 1.2, 0.85, 0.),
            ("fireball", 1., 1., 0.),
        ] {
            let owned = super::super::Owned {
                def: super::super::data::ability(key),
                level: 3,
                cd: 0.,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
            assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
            assert!(
                (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                "{key}"
            );
            assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
            assert_eq!(
                (after.count, after.radius, after.duration),
                (before.count, before.radius, before.duration)
            );
        }
        run.hp = 50.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("thorns"), 100.),
            3.
        );
        assert_eq!((run.hp, run.shield, run.shield_t), (53., 5., 3.));
        finish_milestone_cast(&mut run, trained.cast("thorns"), 100.);
        assert_eq!(run.shield, 5.);
        run.shield = 20.;
        run.shield_t = 8.;
        finish_milestone_cast(&mut run, trained.cast("thorns"), 100.);
        assert_eq!((run.shield, run.shield_t), (20., 8.));
        run.heal_rate = 3.;
        run.heal_t = 10.;
        run.hp = 50.;
        let before_swiftmend = Milestones {
            points: [0, 0, 29],
            ..base
        };
        assert_eq!(
            finish_milestone_cast(&mut run, before_swiftmend.cast("rejuvenation"), 100.),
            0.
        );
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("rejuvenation"), 100.),
            5.
        );
        assert_eq!((run.heal_rate, run.heal_t), (3., 10.));
        run.hp = 99.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("rejuvenation"), 100.),
            1.
        );
        run.hp = 0.;
        run.shield = 0.;
        run.shield_t = 0.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("rejuvenation"), 100.),
            0.
        );
        finish_milestone_cast(&mut run, trained.cast("thorns"), 100.);
        assert_eq!((run.hp, run.shield, run.shield_t), (0., 0., 0.));
    }

    #[test]
    fn warlock_milestones_reach_casts_and_bound_healing_shields_and_cooldowns() {
        let base = Milestones {
            class: 7,
            ..Default::default()
        };
        let trained = Milestones {
            points: [30; 3],
            ..base
        };
        let mut run = Run::default();
        let stats = run.stats();
        for (key, damage, cd, crit_damage) in [
            ("shadow_bolt", 1.375, 1., 0.2),
            ("corruption", 1.1, 0.9, 0.),
            ("curse_of_agony", 1.1, 1., 0.),
            ("immolate", 1.1, 1., 0.2),
            ("rain_of_fire", 1.1, 1., 0.2),
            ("hellfire", 1.1, 1., 0.2),
            ("death_coil", 1.1, 1., 0.),
            ("shadowburn", 1.1, 1., 0.2),
        ] {
            let owned = super::super::Owned {
                def: super::super::data::ability(key),
                level: 3,
                cd: 0.,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
            assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
            assert!(
                (after.cooldown - before.cooldown * cd).abs() < 0.001,
                "{key}"
            );
            assert!(
                (after.crit_damage - before.crit_damage - crit_damage).abs() < 0.001,
                "{key}"
            );
            assert_eq!(
                (after.count, after.radius, after.duration),
                (before.count, before.radius, before.duration)
            );
        }
        run.hp = 99.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("death_coil"), 100.),
            1.
        );
        assert_eq!((run.hp, run.shield, run.shield_t), (100., 5., 3.));
        finish_milestone_cast(&mut run, trained.cast("death_coil"), 100.);
        assert_eq!(run.shield, 5.);
        run.shield = 20.;
        run.shield_t = 8.;
        finish_milestone_cast(&mut run, trained.cast("death_coil"), 100.);
        assert_eq!((run.shield, run.shield_t), (20., 8.));
        run.abilities.clear();
        for key in ["shadow_bolt", "shadowburn", "searing_pain"] {
            run.abilities.push(super::super::Owned {
                def: super::super::data::ability(key),
                level: 1,
                cd: 0.5,
            });
        }
        finish_milestone_cast(&mut run, trained.cast("corruption"), 100.);
        assert_eq!((run.abilities[0].cd, run.abilities[1].cd), (0., 0.5));
        finish_milestone_cast(&mut run, trained.cast("immolate"), 100.);
        assert_eq!((run.abilities[1].cd, run.abilities[2].cd), (0., 0.5));
        run.abilities.clear();
        finish_milestone_cast(&mut run, trained.cast("immolate"), 100.);
        assert!(run.abilities.is_empty());
    }

    #[test]
    fn warlock_node_training_reaches_direct_and_periodic_parameters() {
        let base = Milestones {
            class: 7,
            ..Default::default()
        };
        let ranks: Vec<_> = survivors_core::talents::definitions(7)
            .iter()
            .map(|node| (0, node.id, node.max_rank))
            .collect();
        let trained = base.with_ranks(&ranks);
        let stats = Run::default().stats();
        for level in [1, 5] {
            for (key, damage, cooldown, crit, crit_damage) in [
                ("shadow_bolt", 1.32, 0.72, 0.05, 0.5),
                ("corruption", 1.1, 0.8, 0., 0.),
                ("curse_of_agony", 1.3992, 1., 0., 0.),
                ("immolate", 1.375, 0.72, 0.05, 0.5),
                ("rain_of_fire", 1.1, 0.9, 0.05, 0.5),
                ("hellfire", 1.1, 0.9, 0.05, 0.5),
                ("searing_pain", 1.1, 0.9, 0.15, 0.5),
                ("shadowburn", 1.32, 0.9, 0.05, 0.5),
                ("death_coil", 1.1, 1., 0., 0.),
                ("fireball", 1., 1., 0., 0.),
            ] {
                let owned = super::super::Owned {
                    def: super::super::data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                    "{key}"
                );
                assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
                assert!(
                    (after.crit_damage - before.crit_damage - crit_damage).abs() < 0.001,
                    "{key}"
                );
                let range_multiplier = if matches!(
                    key,
                    "shadow_bolt"
                        | "corruption"
                        | "curse_of_agony"
                        | "immolate"
                        | "rain_of_fire"
                        | "searing_pain"
                        | "shadowburn"
                        | "death_coil"
                ) {
                    1.2
                } else {
                    1.0
                };
                assert!(
                    (after.range - before.range * range_multiplier).abs() < 0.001,
                    "{key}"
                );
                assert_eq!(
                    (after.count, after.radius, after.duration),
                    (before.count, before.radius, before.duration)
                );
                assert!(
                    (after.heal_fraction - if key == "death_coil" { 0.02 } else { 0.0 }).abs()
                        < 0.0001
                );
                let reset = Params::new(&owned, &stats, trained.with_ranks(&[]));
                assert_eq!((reset.dmg, reset.cooldown), (before.dmg, before.cooldown));
            }
        }
    }

    #[test]
    fn totem_deployment_heals_once_only_after_success_and_restoration_unlock() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for key in ["searing_totem", "magma_totem"] {
            for points in [0, 9, 10, 30] {
                for rank in [0, 1, 5, 255] {
                    // No enemy, successful cast, cooldown, dead hero, and healing cap.
                    for (enemy_count, initial_hp, cooldown) in [
                        (0, 20., 0.),
                        (1, 20., 0.),
                        (3, 20., 0.),
                        (3, 20., 5.),
                        (3, 0., 0.),
                        (3, 159., 0.),
                        (3, 110., 0.),
                    ] {
                        let mut run = Run {
                            class: &data::CLASSES[5],
                            hp: initial_hp,
                            milestones: Milestones {
                                class: 5,
                                points: [0, 0, points],
                                ..Default::default()
                            }
                            .with_ranks(&[
                                (2, 588, rank),
                                (2, 592, rank),
                                (2, 586, rank),
                                (2, 593, rank),
                                (2, 581, rank),
                            ]),
                            ..Default::default()
                        };
                        run.bonus.targets = 3;
                        let max_hp = run.stats().max_hp;
                        run.hp = initial_hp.min(max_hp);
                        let initial_hp = run.hp;
                        run.abilities.push(Owned {
                            def: data::ability(key),
                            level: 1,
                            cd: cooldown,
                        });
                        let r = f32::from(rank.min(5));
                        let expected_cd = run.abilities[0].cooldown(&run.stats())
                            * Milestones {
                                class: 5,
                                points: [0, 0, points],
                                ..Default::default()
                            }
                            .cast(key)
                            .cooldown
                            * (1. - 0.04 * r)
                            * (1. - 0.02 * r);
                        let mut time = Time::<()>::default();
                        time.advance_by(std::time::Duration::from_secs_f32(0.1));
                        let mut app = App::new();
                        app.insert_resource(time)
                            .insert_resource(run)
                            .insert_resource(Rng(42))
                            .init_resource::<PlaySeq>()
                            .init_resource::<PendingHits>()
                            .add_message::<CastEvent>()
                            .add_message::<SpellGoTargets>()
                            .add_message::<SwingMessage>()
                            .add_message::<crate::combat_text::CombatTextSpawn>()
                            .add_message::<Hit>();
                        app.world_mut()
                            .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
                        for index in 0..enemy_count {
                            app.world_mut().spawn((
                                Transform::from_xyz(1. + index as f32, 0., 0.),
                                Enemy {
                                    def: &data::roster(0).enemies[0],
                                    hp: 100.,
                                    max_hp: 100.,
                                    speed: 3.,
                                    damage: 1.,
                                    radius: 0.6,
                                    xp: 1,
                                    boss: false,
                                    elite: false,
                                    attack_cd: 0.,
                                    slow: 1.,
                                    slow_t: 0.,
                                    root_t: 0.,
                                    frozen_t: 0.,
                                    winters_chill: Default::default(),
                                    shadow_weaving: Default::default(),
                                    stun_t: 0.,
                                    fear_t: 0.,
                                    weaken: 1.,
                                    weaken_t: 0.,
                                    dots: Vec::new(),
                                    dead: false,
                                },
                            ));
                        }
                        app.world_mut().run_system_once(cast_abilities).unwrap();
                        let fired = enemy_count > 0 && initial_hp > 0. && cooldown == 0.;
                        if fired {
                            assert!(
                                (app.world().resource::<Run>().abilities[0].cd - expected_cd).abs()
                                    < 0.0001
                            );
                        }
                        let healing = if fired && points >= 10 {
                            max_hp * 0.03 * (1. + 0.07 * f32::from(rank.min(5)))
                        } else {
                            0.
                        };
                        let expected_hp = (initial_hp + healing).min(max_hp);
                        assert_eq!(
                            app.world().resource::<Run>().ancestral_healing_t,
                            if expected_hp > initial_hp && rank > 0 {
                                10.
                            } else {
                                0.
                            }
                        );
                        assert!((app.world().resource::<Run>().hp - expected_hp).abs() < 0.0001,
                            "{key}: points={points}, rank={rank}, enemies={enemy_count}, hp={initial_hp}, cd={cooldown}");
                        assert_eq!(
                            app.world_mut().query::<&Totem>().iter(app.world()).count(),
                            usize::from(fired)
                        );
                        // A later update and attacks must neither repeat healing nor refresh armor.
                        let remaining = if app.world().resource::<Run>().ancestral_healing_t > 0. {
                            4.
                        } else {
                            0.
                        };
                        app.world_mut().resource_mut::<Run>().ancestral_healing_t = remaining;
                        app.world_mut().run_system_once(cast_abilities).unwrap();
                        for _ in 0..25 {
                            app.world_mut().run_system_once(tick_areas).unwrap();
                        }
                        assert!((app.world().resource::<Run>().hp - expected_hp).abs() < 0.0001);
                        assert_eq!(app.world().resource::<Run>().ancestral_healing_t, remaining);
                        assert_eq!(
                            app.world_mut().query::<&Totem>().iter(app.world()).count(),
                            usize::from(fired)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn totemic_mastery_reaches_targets_only_inside_extended_distance() {
        use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
        use bevy::ecs::system::RunSystemOnce;
        for key in ["searing_totem", "magma_totem"] {
            for trained in [false, true] {
                for distance_scale in [0.9, 1.1, 1.3] {
                    let base = Milestones {
                        class: 5,
                        ..Default::default()
                    };
                    let mut run = Run {
                        class: &data::CLASSES[5],
                        milestones: if trained {
                            base.with_ranks(&[(2, 582, 1)])
                        } else {
                            base
                        },
                        ..Default::default()
                    };
                    run.bonus.targets = 3;
                    let owned = super::super::Owned {
                        def: data::ability(key),
                        level: 1,
                        cd: 0.,
                    };
                    let original = Params::new(&owned, &run.stats(), base);
                    let params = Params::new(&owned, &run.stats(), run.milestones);
                    run.abilities.push(owned);
                    let pulse = key == "magma_totem";
                    let distance = if pulse {
                        original.radius
                    } else {
                        original.range
                    };
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(0.1));
                    app.insert_resource(time)
                        .insert_resource(run)
                        .insert_resource(Rng(1))
                        .init_resource::<PlaySeq>()
                        .init_resource::<PendingHits>()
                        .add_message::<CastEvent>()
                        .add_message::<SpellGoTargets>()
                        .add_message::<SwingMessage>()
                        .add_message::<crate::combat_text::CombatTextSpawn>()
                        .add_message::<Hit>();
                    app.world_mut()
                        .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
                    for sign in [-1., 1.] {
                        app.world_mut().spawn((
                            Transform::from_xyz(sign * distance * distance_scale, 0., 0.),
                            Enemy {
                                def: &data::roster(0).enemies[0],
                                hp: 100.,
                                max_hp: 100.,
                                speed: 3.,
                                damage: 5.,
                                radius: 0.0,
                                xp: 1,
                                boss: false,
                                elite: false,
                                attack_cd: 0.,
                                slow: 1.,
                                slow_t: 0.,
                                root_t: 8.,
                                frozen_t: 0.,
                                winters_chill: Default::default(),
                                shadow_weaving: Default::default(),
                                stun_t: 0.,
                                fear_t: 0.,
                                weaken: 1.,
                                weaken_t: 0.,
                                dots: Vec::new(),
                                dead: false,
                            },
                        ));
                    }
                    app.world_mut().run_system_once(cast_abilities).unwrap();
                    let world = app.world_mut();
                    let mut deployed = world.query::<(&Transform, &Totem)>();
                    let (transform, totem) = deployed.single(world).unwrap();
                    assert!((transform.translation.length() - 2.).abs() < 0.0001);
                    assert_eq!(totem.source, key);
                    assert_eq!(totem.cast, data::ability(key).totem.1);
                    assert_eq!(totem.pulse, pulse);
                    assert_eq!(
                        (totem.damage, totem.range, totem.radius, totem.left),
                        (params.dmg, params.range, params.radius, params.duration)
                    );
                    let origin = transform.translation;
                    assert_eq!(world.resource::<Run>().abilities[0].cd, params.cooldown);
                    // Keep attack distances exact without replacing the deployed component.
                    for mut transform in world
                        .query_filtered::<&mut Transform, With<Enemy>>()
                        .iter_mut(world)
                    {
                        transform.translation += origin;
                    }
                    world
                        .resource_mut::<Time>()
                        .advance_by(std::time::Duration::from_secs_f32(0.4));
                    app.world_mut().run_system_once(tick_areas).unwrap();
                    let reachable = distance_scale < if trained { 1.2 } else { 1. };
                    assert_eq!(
                        app.world().resource::<Messages<Hit>>().len(),
                        if reachable {
                            if pulse {
                                2
                            } else {
                                1
                            }
                        } else {
                            0
                        },
                        "{key} trained={trained} distance={distance_scale}"
                    );
                    assert_eq!(
                        (params.dmg, params.duration, params.cooldown, params.count),
                        (
                            original.dmg,
                            original.duration,
                            original.cooldown,
                            original.count
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn cost_talents_compose_with_existing_cast_cooldowns_without_changing_geometry() {
        for (class, node, max, other, other_rank, keys, reduction) in [
            (
                5,
                586,
                5,
                595,
                5,
                &["searing_totem", "magma_totem"][..],
                0.20,
            ),
            (
                5,
                593,
                5,
                586,
                5,
                &["searing_totem", "magma_totem"][..],
                0.10,
            ),
            (
                2,
                1342,
                5,
                1346,
                5,
                &[
                    "arcane_shot",
                    "multi_shot",
                    "serpent_sting",
                    "concussive_shot",
                    "aimed_shot",
                ][..],
                0.10,
            ),
            (
                4,
                341,
                5,
                542,
                2,
                &[
                    "shadow_word_pain",
                    "holy_nova",
                    "psychic_scream",
                    "power_word_shield",
                ][..],
                0.10,
            ),
            (
                6,
                66,
                3,
                37,
                5,
                &["frostbolt", "frost_nova", "blizzard", "cone_of_cold"][..],
                0.15,
            ),
        ] {
            let base = Milestones {
                class,
                points: [30; 3],
                ..Default::default()
            }
            .with_ranks(&[(0, other, other_rank)]);
            let trained = base.with_ranks(&[(0, other, other_rank), (0, node, max)]);
            let mut run = Run::default();
            run.bonus.targets = 2;
            let stats = run.stats();
            for level in [1, 5] {
                for key in data::CLASSES[class].pool {
                    let owned = super::super::Owned {
                        def: data::ability(key),
                        level,
                        cd: 0.,
                    };
                    let before = Params::new(&owned, &stats, base);
                    let after = Params::new(&owned, &stats, trained);
                    let factor = if keys.contains(key) {
                        1. - reduction
                    } else {
                        1.
                    };
                    assert!(
                        (after.cooldown - before.cooldown * factor).abs() < 0.0001,
                        "{key}"
                    );
                    assert_eq!(
                        (
                            after.dmg,
                            after.dot,
                            after.count,
                            after.range,
                            after.radius,
                            after.duration
                        ),
                        (
                            before.dmg,
                            before.dot,
                            before.count,
                            before.range,
                            before.radius,
                            before.duration
                        )
                    );
                    assert_eq!(
                        (
                            after.heal_fraction,
                            after.shield_fraction,
                            after.shield_seconds,
                            after.immune_seconds
                        ),
                        (
                            before.heal_fraction,
                            before.shield_fraction,
                            before.shield_seconds,
                            before.immune_seconds
                        )
                    );
                }
            }
        }
    }

    #[test]
    fn shaman_node_training_reaches_direct_periodic_and_totem_parameters() {
        let base = Milestones {
            class: 5,
            ..Default::default()
        };
        let trained = base.with_ranks(&[
            (0, 564, 5),
            (0, 563, 5),
            (0, 561, 3),
            (0, 575, 5),
            (0, 562, 5),
            (0, 721, 5),
            (0, 565, 1),
            (1, 607, 3),
            (2, 595, 5),
            (2, 594, 5),
            (2, 591, 1),
        ]);
        let stats = Run::default().stats();
        for level in [1, 5] {
            for (key, damage, cooldown, crit, crit_damage) in [
                ("lightning_bolt", 1.1, 0.72, 0.10, 0.5),
                ("chain_lightning", 1.1, 0.72, 0.10, 0.5),
                ("earth_shock", 1.1, 0.65025, 0.05, 0.5),
                ("flame_shock", 1.1, 0.65025, 0.05, 0.5),
                ("frost_shock", 1.1, 0.65025, 0.05, 0.5),
                ("searing_totem", 1.15, 0.85, 0., 0.),
                ("magma_totem", 1.15, 0.85, 0., 0.),
                ("lightning_shield", 1.15, 1., 0., 0.),
                ("fireball", 1., 1., 0., 0.),
            ] {
                let owned = super::super::Owned {
                    def: super::super::data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!(
                    (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                    "{key}"
                );
                assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
                assert!(
                    (after.crit_damage - before.crit_damage - crit_damage).abs() < 0.001,
                    "{key}"
                );
                assert_eq!(
                    (after.count, after.radius, after.range, after.duration),
                    (before.count, before.radius, before.range, before.duration)
                );
                let reset = Params::new(&owned, &stats, trained.with_ranks(&[]));
                assert_eq!((reset.dmg, reset.cooldown), (before.dmg, before.cooldown));
            }
        }
    }

    #[test]
    fn shaman_milestones_reach_combat_and_deployment_benefits_are_bounded() {
        let mut run = Run::default();
        let stats = run.stats();
        let base = Milestones {
            class: 5,
            ..Default::default()
        };
        let elemental = Milestones {
            points: [30, 0, 0],
            ..base
        };
        let enhancement = Milestones {
            points: [0, 30, 0],
            ..base
        };
        let restoration = Milestones {
            points: [0, 0, 30],
            ..base
        };
        for (key, talents, damage, cooldown, extra) in [
            ("chain_lightning", elemental, 1., 0.9, 0),
            ("earth_shock", enhancement, 1.25, 1., 1),
            ("searing_totem", enhancement, 1.2, 1., 0),
            ("magma_totem", enhancement, 1.2, 1., 0),
            ("lightning_shield", enhancement, 1.2, 1., 0),
            ("flame_shock", restoration, 1., 0.85, 0),
        ] {
            let owned = super::super::Owned {
                def: super::super::data::ability(key),
                level: 3,
                cd: 0.,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, talents);
            assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
            assert!(
                (after.cooldown - before.cooldown * cooldown).abs() < 0.001,
                "{key}"
            );
            assert_eq!(after.count, before.count + extra);
            assert_eq!(after.radius, before.radius);
            if key == "chain_lightning" {
                assert_eq!(after.crit, 1.);
            }
        }
        run.hp = 99.;
        run.abilities.clear();
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("magma_totem"),
            level: 1,
            cd: 1.,
        });
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("earth_shock"),
            level: 1,
            cd: 4.,
        });
        assert_eq!(
            finish_milestone_cast(&mut run, restoration.cast("searing_totem"), 100.),
            1.
        );
        assert_eq!(run.hp, 100.);
        assert_eq!(run.abilities[0].cd, 0.);
        assert_eq!(run.abilities[1].cd, 4.);
        run.hp = 90.;
        assert_eq!(
            finish_milestone_cast(&mut run, restoration.cast("magma_totem"), 100.),
            3.
        );
        let trained = restoration.with_ranks(&[(2, 588, 5), (2, 592, 5)]);
        run.hp = 90.;
        let healed = finish_milestone_cast(&mut run, trained.cast("magma_totem"), 100.);
        assert!((healed - 4.05).abs() < 0.00001);
        assert!((run.hp - 94.05).abs() < 0.00001);
        run.hp = 99.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("searing_totem"), 100.),
            1.
        );
        run.hp = 0.;
        assert_eq!(
            finish_milestone_cast(&mut run, trained.cast("magma_totem"), 100.),
            0.
        );
        assert_eq!(run.hp, 0.);
        run.hp = 90.;
        assert_eq!(run.abilities.len(), 2); // Does not grant the unowned counterpart.
        finish_milestone_cast(&mut run, enhancement.cast("earth_shock"), 100.);
        assert_eq!((run.shield, run.shield_t), (3., 3.));
        finish_milestone_cast(&mut run, enhancement.cast("earth_shock"), 100.);
        assert_eq!((run.shield, run.shield_t), (3., 3.));
        run.shield = 20.;
        run.shield_t = 8.;
        finish_milestone_cast(&mut run, enhancement.cast("earth_shock"), 100.);
        assert_eq!((run.shield, run.shield_t), (20., 8.));
    }

    #[test]
    fn guardian_totems_refresh_absorption_without_stacking_or_extending_stronger_shields() {
        for points in [9, 10, 30] {
            for rank in [0, 1, 2, 255] {
                let milestones = Milestones {
                    class: 5,
                    points: [0, points, 0],
                    ..Default::default()
                }
                .with_ranks(&[(1, 609, rank), (1, 612, rank)]);
                let mut run = Run {
                    hp: 50.,
                    milestones,
                    ..Default::default()
                };
                let owned = Owned {
                    def: data::ability("earth_shock"),
                    level: 1,
                    cd: 0.,
                };
                let params = Params::new(&owned, &run.stats(), milestones);
                let expected = if points >= 10 {
                    3. * (1. + 0.1 * f32::from(rank.min(2)) + 0.05 * f32::from(rank.min(5)))
                } else {
                    0.
                };
                assert!((params.shield_fraction * 100. - expected).abs() < 0.00001);
                for _ in 0..3 {
                    finish_milestone_cast(&mut run, milestones.cast("earth_shock"), 100.);
                    assert!((run.shield - expected).abs() < 0.00001);
                    assert_eq!(run.shield_t, if points >= 10 { 3. } else { 0. });
                    assert_eq!(run.hp, 50.);
                }
                run.shield = 20.;
                run.shield_t = 8.;
                finish_milestone_cast(&mut run, milestones.cast("earth_shock"), 100.);
                assert_eq!((run.shield, run.shield_t), (20., 8.));
                run.shield = 0.;
                run.shield_t = 0.;
                run.hp = 0.;
                finish_milestone_cast(&mut run, milestones.cast("earth_shock"), 100.);
                assert_eq!((run.shield, run.shield_t), (0., 0.));
            }
        }
    }

    #[test]
    fn mage_arcane_fire_milestones_reach_cast_parameters_and_cooldown_finish() {
        let mut run = Run::default();
        let stats = run.stats();
        let base = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = Milestones {
            points: [30, 30, 0],
            ..base
        }
        .with_ranks(&[(0, 81, 3), (1, 33, 3), (1, 35, 5)]);
        for level in [1, 5] {
            for (key, damage, crit, critical_damage) in [
                ("arcane_missiles", 1.2, 0.05, 0.),
                ("arcane_explosion", 1.2, 0.11, 0.),
                ("fireball", 1.375, 0.16, 0.20),
                ("flamestrike", 1.1, 0.16, 0.20),
                ("frostbolt", 1., 0., 0.),
            ] {
                let owned = super::super::Owned {
                    def: super::super::data::ability(key),
                    level,
                    cd: 0.,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                assert!((after.dmg - before.dmg * damage).abs() < 0.001, "{key}");
                assert!((after.dot - before.dot * damage).abs() < 0.001, "{key}");
                assert!((after.crit - before.crit - crit).abs() < 0.001, "{key}");
                assert!(
                    (after.crit_damage - before.crit_damage - critical_damage).abs() < 0.001,
                    "{key}"
                );
                assert_eq!(after.count, before.count);
                assert_eq!(after.radius, before.radius);
            }
        }
        // Presence of Mind only advances an owned Explosion, once at cast completion.
        run.abilities.clear();
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("arcane_explosion"),
            level: 1,
            cd: 3.,
        });
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("fireball"),
            level: 1,
            cd: 3.,
        });
        finish_milestone_cast(&mut run, trained.cast("arcane_missiles"), stats.max_hp);
        assert_eq!(run.abilities[0].cd, 1.);
        assert_eq!(run.abilities[1].cd, 3.);
        finish_milestone_cast(&mut run, trained.cast("arcane_missiles"), stats.max_hp);
        assert_eq!(run.abilities[0].cd, 0.);
        run.abilities.remove(0);
        finish_milestone_cast(&mut run, trained.cast("arcane_missiles"), stats.max_hp);
        assert_eq!(run.abilities.len(), 1);
        assert_eq!(run.abilities[0].cd, 3.);
    }

    #[test]
    fn paladin_seal_training_reaches_ranked_judgement_parameters() {
        let stats = Run::default().stats();
        let base = Milestones {
            class: 1,
            points: [30; 3],
            ..Default::default()
        }
        .with_ranks(&[(2, 1631, 2), (2, 1409, 1)]);
        let trained = Milestones {
            class: 1,
            points: [30; 3],
            ..Default::default()
        }
        .with_ranks(&[(2, 1631, 2), (2, 1409, 1), (2, 1407, 5), (0, 1463, 5)]);
        for level in [1, 5] {
            for key in [
                "judgement",
                "holy_shock",
                "consecration",
                "divine_shield",
                "smite",
            ] {
                let owned = super::super::Owned {
                    def: super::super::data::ability(key),
                    level,
                    cd: 0.0,
                };
                let before = Params::new(&owned, &stats, base);
                let after = Params::new(&owned, &stats, trained);
                let (damage, cooldown) = if key == "judgement" {
                    (1.15, 0.85)
                } else {
                    (1.0, 1.0)
                };
                assert!((after.dmg - before.dmg * damage).abs() < 0.001);
                assert!((after.cooldown - before.cooldown * cooldown).abs() < 0.001);
                assert_eq!(
                    (
                        after.count,
                        after.radius,
                        after.duration,
                        after.heal_fraction
                    ),
                    (
                        before.count,
                        before.radius,
                        before.duration,
                        before.heal_fraction
                    )
                );
            }
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
