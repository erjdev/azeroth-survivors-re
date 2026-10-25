//! Companions fetch coins and experience wisps. They run a trip, sweep what is under their
//! paws, then rest. They are not a second magnet, and they leave chests where they stand.

use bevy::prelude::*;

use super::account::PetRanks;
use super::enemies::{self, Gem, GemKind};
use super::hero::Hero;
use super::units::{self, flat, yaw_toward};
use super::Floor;
use super::{Phase, Run};
use crate::creature_anim::{move_flags, MovementState};

/// Critter looks. Display ids are 1.12 creature displays whose models ship with the client.
pub(super) static LOOKS: &[(&str, u32)] = &[
    ("Snowshoe Rabbit", 328),
    ("Squirrel", 134),
    ("Westfall Chicken", 304),
    ("Black Kingsnake", 1206),
    ("Black Whelpling", 629),
];

const BASE_LEASH: f32 = 25.0;
type PickupQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform, &'static Gem), (Without<Hero>, Without<Fetcher>)>;

const MAX_LEASH: f32 = 55.0;
const BASE_SPEED: f32 = 9.0;
const BASE_REST: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Drop {
    Gold,
    Xp,
    Heal,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Rune collection rules precede rune pickup spawning"
        )
    )]
    Rune,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Trip {
    /// How far past the hero this companion will run, in yards.
    pub reach: f32,
    /// Experience wisps, when this companion herds them.
    pub wisp_reach: f32,
    /// The radius under its paws. A quarter of the hero's pickup.
    pub grasp: f32,
    pub speed: f32,
    pub rest: f32,
    /// Multiplier on coins this companion itself picks up.
    pub gold_bonus: f32,
    pub instinct: u8,
}

/// `instinct`: 0 none, 1 gold-sniffer, 2 wisp-herder, 3 rune-hound.
pub(super) fn trip(magnet: f32, leash_rank: u8, paws_rank: u8, instinct: u8) -> Trip {
    let leash = leash_rank.min(3);
    let paws = paws_rank.min(3);
    let extra = (MAX_LEASH - BASE_LEASH) * f32::from(leash) / 3.0;
    let reach = magnet + BASE_LEASH + extra;
    Trip {
        reach,
        wisp_reach: if instinct == 2 { reach * 2.0 } else { reach },
        grasp: (magnet * 0.25).max(1.5),
        speed: BASE_SPEED * (1.0 + 0.2 * f32::from(paws)),
        rest: BASE_REST - f32::from(paws) / 3.0,
        gold_bonus: if instinct == 1 { 1.1 } else { 1.0 },
        instinct,
    }
}

pub(super) fn in_reach(kind: Drop, dist_from_hero: f32, trip: &Trip) -> bool {
    let limit = match kind {
        Drop::Heal => return false,
        Drop::Rune => {
            if trip.instinct != 3 {
                return false;
            }
            trip.reach
        }
        Drop::Xp if trip.instinct == 2 => trip.wisp_reach,
        Drop::Gold | Drop::Xp => trip.reach,
    };
    dist_from_hero <= limit
}

/// Lower sorts first. A sniffer takes coins before wisps; a herder does the opposite.
fn preference(kind: Drop, instinct: u8) -> u8 {
    match instinct {
        1 => u8::from(kind != Drop::Gold),
        2 => u8::from(kind != Drop::Xp),
        _ => 0,
    }
}

/// The gem this trip runs to. `rows` are `(index, kind, yards from the hero, yards from the pet)`.
pub(super) fn choose(rows: &[(usize, Drop, f32, f32, Vec3)], trip: &Trip) -> Option<usize> {
    let mut best: Option<(u8, f32, usize)> = None;
    for &(index, kind, from_hero, from_pet, _) in rows {
        if !in_reach(kind, from_hero, trip) {
            continue;
        }
        let pref = preference(kind, trip.instinct);
        let better = match best {
            None => true,
            Some((p, dist, i)) => {
                pref < p || (pref == p && (from_pet < dist || (from_pet == dist && index < i)))
            }
        };
        if better {
            best = Some((pref, from_pet, index));
        }
    }
    best.map(|(_, _, index)| index)
}

#[derive(Component)]
pub(super) struct Fetcher {
    slot: u8,
    rest: f32,
}

pub(super) fn herd(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    account: Res<super::account::Account>,
    mut stash: ResMut<super::stash::Stash>,
    mut next: ResMut<NextState<Phase>>,
    world: Floor,
    hero: Query<&Transform, (With<Hero>, Without<Fetcher>)>,
    mut pets: Query<(Entity, &mut Transform, &mut MovementState, &mut Fetcher), Without<Hero>>,
    gems: PickupQuery<'_, '_>,
) {
    let Ok(hero_tf) = hero.single() else { return };
    if !run.alive() {
        return;
    }
    let ranks = account.pets;
    let want = usize::from(ranks.owned.min(2));
    let mut seen = [false; 2];
    let mut doomed = Vec::new();
    for (e, _, _, pet) in &pets {
        if usize::from(pet.slot) < want {
            seen[pet.slot as usize] = true;
        } else {
            doomed.push(e);
        }
    }
    for e in doomed {
        commands.entity(e).despawn();
    }
    for (slot, present) in seen.iter().enumerate().take(want) {
        if *present {
            continue;
        }
        spawn_fetcher(
            &mut commands,
            &mut run,
            hero_tf.translation,
            slot as u8,
            ranks,
        );
    }

    let dt = time.delta_secs();
    let magnet = run.stats().magnet;
    let hero_pos = hero_tf.translation;
    for (_, mut tf, mut mv, mut pet) in &mut pets {
        if usize::from(pet.slot) >= want {
            continue;
        }
        let plan = trip(
            magnet,
            ranks.leash,
            ranks.paws,
            ranks.instinct[pet.slot as usize],
        );
        if pet.rest > 0.0 {
            pet.rest = (pet.rest - dt).max(0.0);
            step_toward(
                &mut tf,
                &mut mv,
                follow_at(hero_pos, pet.slot),
                plan.speed,
                dt,
                &world,
            );
            continue;
        }
        let rows = gem_rows(&gems, hero_pos, tf.translation);
        let Some(pick) = choose(&rows, &plan) else {
            step_toward(
                &mut tf,
                &mut mv,
                follow_at(hero_pos, pet.slot),
                plan.speed,
                dt,
                &world,
            );
            continue;
        };
        let Some(gem_pos) = rows.iter().find(|row| row.0 == pick).map(|row| row.4) else {
            continue;
        };
        if flat(gem_pos - tf.translation).length() <= plan.grasp {
            let mut taken = Vec::new();
            for (e, gtf, g) in &gems {
                if g.pulled {
                    continue;
                }
                if flat(gtf.translation - tf.translation).length() > plan.grasp {
                    continue;
                }
                let kind = drop_of(g.kind);
                let from_hero = flat(gtf.translation - hero_pos).length();
                if in_reach(kind, from_hero, &plan) {
                    taken.push((e, g.kind));
                }
            }
            for (e, kind) in taken {
                enemies::bank_gem(&mut run, &mut stash, &mut next, kind, plan.gold_bonus);
                commands.entity(e).despawn();
            }
            pet.rest = plan.rest;
            mv.speed = 0.0;
            mv.flags = 0;
        } else {
            step_toward(&mut tf, &mut mv, gem_pos, plan.speed, dt, &world);
        }
    }
}

fn gem_rows(
    gems: &PickupQuery<'_, '_>,
    hero: Vec3,
    pet: Vec3,
) -> Vec<(usize, Drop, f32, f32, Vec3)> {
    gems.iter()
        .enumerate()
        .filter(|(_, (_, _, g))| !g.pulled)
        .map(|(i, (_, tf, g))| {
            (
                i,
                drop_of(g.kind),
                flat(tf.translation - hero).length(),
                flat(tf.translation - pet).length(),
                tf.translation,
            )
        })
        .collect()
}

fn drop_of(kind: GemKind) -> Drop {
    match kind {
        GemKind::Gold(_) => Drop::Gold,
        GemKind::Xp(_) => Drop::Xp,
        GemKind::Heal(_) => Drop::Heal,
    }
}

fn follow_at(hero: Vec3, slot: u8) -> Vec3 {
    let side = if slot == 0 { 1.6 } else { -1.6 };
    hero + Vec3::new(side, 0.0, 1.6)
}

fn step_toward(
    tf: &mut Transform,
    mv: &mut MovementState,
    goal: Vec3,
    speed: f32,
    dt: f32,
    world: &Floor,
) {
    let mut pos = tf.translation;
    let delta = flat(goal - pos);
    let dist = delta.length();
    if dist <= 0.8 {
        mv.speed = 0.0;
        mv.flags = 0;
        return;
    }
    let step = (speed * dt).min(dist);
    pos = world.slide(pos, pos + delta / dist * step, 0.35);
    if let Some(h) = world.height(pos) {
        pos.y = h;
    }
    tf.translation = pos;
    tf.rotation = tf.rotation.slerp(
        Quat::from_rotation_y(yaw_toward(delta)),
        (dt * 12.0).min(1.0),
    );
    mv.speed = speed;
    mv.flags = move_flags::FORWARD;
}

fn spawn_fetcher(commands: &mut Commands, run: &mut Run, pos: Vec3, slot: u8, ranks: PetRanks) {
    let look = ranks.look[slot as usize] as usize;
    let display = LOOKS.get(look).map(|(_, id)| *id).unwrap_or(328);
    let serial = run.guid();
    let e = units::spawn_unit(
        commands,
        &units::UnitSpec {
            serial,
            display,
            scale: 1.0,
            level: 1,
            faction: units::FACTION_FRIENDLY,
            bytes0: 1 << 8,
            weapons: &[],
            sheath: 0,
        },
        pos,
        0.0,
    );
    commands.entity(e).insert(Fetcher { slot, rest: 0.0 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_companion_reaches_past_the_hero_and_rests_between_trips() {
        let base = trip(4.5, 0, 0, 0);
        assert!((base.reach - 29.5).abs() < 0.01);
        assert!((base.grasp - 1.5).abs() < 0.01);
        assert!((base.rest - 2.0).abs() < 0.01);
        let leashed = trip(4.5, 3, 3, 0);
        assert!((leashed.reach - 59.5).abs() < 0.01);
        assert!((leashed.speed - BASE_SPEED * 1.6).abs() < 0.01);
        assert!((leashed.rest - 1.0).abs() < 0.01);
    }

    #[test]
    fn instincts_choose_and_never_open_a_heal_or_a_chest() {
        let sniffer = trip(4.0, 0, 0, 1);
        assert!((sniffer.gold_bonus - 1.1).abs() < 0.01);
        let rows = [
            (0, Drop::Xp, 10.0, 8.0, Vec3::ZERO),
            (1, Drop::Gold, 12.0, 11.0, Vec3::ZERO),
            (2, Drop::Heal, 3.0, 1.0, Vec3::ZERO),
        ];
        assert_eq!(choose(&rows, &sniffer), Some(1));
        let herder = trip(4.0, 0, 0, 2);
        assert!(in_reach(Drop::Xp, herder.reach + 5.0, &herder));
        assert!(!in_reach(Drop::Gold, herder.reach + 5.0, &herder));
        assert!(!in_reach(Drop::Heal, 1.0, &herder));
        assert!(!in_reach(Drop::Rune, 1.0, &herder));
        let hound = trip(4.0, 0, 0, 3);
        assert!(in_reach(Drop::Rune, 10.0, &hound));
    }
}
