//! Companions fetch coins and experience wisps. They run a trip, sweep what is under their
//! paws, then rest. They are not a second magnet, and they leave chests where they stand.

use bevy::prelude::*;

use super::account::PetRanks;
use super::enemies::{self, Gem, GemKind};
use super::hero::Hero;
use super::units::{self, flat, yaw_toward};
use super::{navigation::Navigation, Floor, Selection};

const BODY_RADIUS: f32 = 0.35;
use super::{Phase, Run};
use crate::creature_anim::{move_flags, MovementState};

/// Append only: save files persist these indices.
pub(super) struct Look {
    pub name: &'static str,
    pub display: u32,
    pub icon: &'static str,
    pub achievement: Option<u16>,
    pub locked_label: &'static str,
}

pub(super) static LOOKS: &[Look] = &[
    Look {
        name: "Snowshoe Rabbit",
        display: 328,
        icon: "INV_Box_PetCarrier_01",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Squirrel",
        display: 134,
        icon: "INV_Box_PetCarrier_01",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Westfall Chicken",
        display: 304,
        icon: "INV_Box_PetCarrier_01",
        achievement: Some(65),
        locked_label: "Veteran · 100 runs",
    },
    Look {
        name: "Black Kingsnake",
        display: 1206,
        icon: "Ability_Hunter_Pet_WindSerpent",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Black Whelpling",
        display: 6288,
        icon: "INV_Misc_Head_Dragon_Black",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Crimson Macaw",
        display: 9554,
        icon: "Ability_Hunter_Pet_Owl",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Great Horned Owl",
        display: 4615,
        icon: "Ability_Hunter_Pet_Owl",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Tree Frog",
        display: 6295,
        icon: "Ability_Hunter_Pet_Turtle",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Sprite Darter",
        display: 6294,
        icon: "Ability_Hunter_Pet_WindSerpent",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Zergling",
        display: 10993,
        icon: "Ability_Hunter_Pet_Scorpid",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Undercity Cockroach",
        display: 2177,
        icon: "Ability_Hunter_Pet_Crab",
        achievement: None,
        locked_label: "",
    },
    Look {
        name: "Panda Cub",
        display: 10990,
        icon: "Ability_Hunter_Pet_Bear",
        achievement: Some(super::rewards::DEATHLESS_HOUR),
        locked_label: "Deathless Hour · criteria pending",
    },
    Look {
        name: "Sapphiron",
        display: 16033,
        icon: "INV_Misc_Head_Dragon_Blue",
        achievement: Some(super::rewards::NO_U),
        locked_label: "no u · criteria pending",
    },
];

const BASE_LEASH: f32 = 25.0;
type PickupQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform, &'static mut Gem),
    (Without<Hero>, Without<Fetcher>),
>;

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

/// Only instincts backed by spawned pickups may charge achievement points.
pub(super) fn instinct_available(kind: u8) -> bool {
    matches!(kind, 1 | 2)
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
    selection: Res<Selection>,
    mut navigation: Local<Navigation>,
    hero: Query<&Transform, (With<Hero>, Without<Fetcher>)>,
    mut pets: Query<(Entity, &mut Transform, &mut MovementState, &mut Fetcher), Without<Hero>>,
    mut gems: PickupQuery<'_, '_>,
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
        if !world.can_spawn(hero_tf.translation, BODY_RADIUS) {
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
    let bodies: Vec<_> = pets
        .iter()
        .map(|(entity, tf, ..)| (entity, tf.translation, BODY_RADIUS))
        .collect();
    navigation.advance(dt, selection.map, &bodies, |from, to, radius| {
        let end = world.travel(from, to, radius);
        (flat(end - to).length() < 0.1 && world.can_spawn(end, radius)).then_some(end)
    });
    for (entity, mut tf, mut mv, mut pet) in &mut pets {
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
                entity,
                &mut navigation,
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
                entity,
                &mut navigation,
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
        if gem_pos.distance(tf.translation) <= plan.grasp
            && world.line_clear(tf.translation, gem_pos)
        {
            let mut taken = Vec::new();
            for (e, gtf, g) in &gems {
                if g.pulled {
                    continue;
                }
                if gtf.translation.distance(tf.translation) > plan.grasp
                    || !world.line_clear(tf.translation, gtf.translation)
                {
                    continue;
                }
                let kind = drop_of(g.kind);
                let from_hero = gtf.translation.distance(hero_pos);
                if in_reach(kind, from_hero, &plan) {
                    taken.push((e, g.kind));
                }
            }
            for (e, kind) in taken {
                // Despawn is deferred; reserve now so a second companion cannot bank it again.
                if let Ok((_, _, mut gem)) = gems.get_mut(e) {
                    gem.pulled = true;
                }
                enemies::bank_gem(&mut run, &mut stash, &mut next, kind, plan.gold_bonus);
                commands.entity(e).despawn();
            }
            navigation.forget(entity);
            pet.rest = plan.rest;
            mv.speed = 0.0;
            mv.flags = 0;
        } else {
            step_toward(
                entity,
                &mut navigation,
                &mut tf,
                &mut mv,
                gem_pos,
                plan.speed,
                dt,
                &world,
            );
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
                tf.translation.distance(hero),
                tf.translation.distance(pet),
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

#[allow(clippy::too_many_arguments)]
fn step_toward(
    entity: Entity,
    navigation: &mut Navigation,
    tf: &mut Transform,
    mv: &mut MovementState,
    goal: Vec3,
    speed: f32,
    dt: f32,
    world: &Floor,
) {
    mv.speed = 0.0;
    mv.flags = 0;
    let at = tf.translation;
    // Pickups hover half a yard above their support; route to feet, not art height.
    let goal = world
        .height(goal)
        .map_or(goal, |y| Vec3::new(goal.x, y, goal.z));
    if at.distance(goal) <= 0.8 && world.line_clear(at, goal) {
        navigation.forget(entity);
        return;
    }
    let waypoint = navigation.waypoint(entity, at, goal).unwrap_or(goal);
    let delta = flat(waypoint - at);
    let step = (speed * dt).min(delta.length());
    let desired = at + delta.normalize_or_zero() * step;
    let end = world.travel(at, desired, BODY_RADIUS);
    navigation.observe(
        entity,
        at,
        goal,
        BODY_RADIUS,
        flat(end - desired).length() > 0.05,
        dt,
    );
    tf.translation = end;
    let traveled = flat(end - at);
    if traveled.length_squared() > 0.0001 {
        tf.rotation = tf.rotation.slerp(
            Quat::from_rotation_y(yaw_toward(traveled)),
            (dt * 12.0).min(1.0),
        );
        mv.speed = speed;
        mv.flags = move_flags::FORWARD;
    }
}

fn spawn_fetcher(commands: &mut Commands, run: &mut Run, pos: Vec3, slot: u8, ranks: PetRanks) {
    let look = ranks.look[slot as usize] as usize;
    let display = LOOKS.get(look).map(|look| look.display).unwrap_or(328);
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
    fn fetcher_collection_requires_local_distance_and_clear_sight() {
        use avian3d::prelude::{Collider, RigidBody};
        use bevy::ecs::system::RunSystemOnce;
        for (wall, height, collected) in
            [(false, 0.5, true), (true, 0.5, false), (false, 4.5, false)]
        {
            let mut app = super::super::floor::tests::fixture(&[0.]);
            app.init_resource::<Selection>();
            let mut account = super::super::account::empty();
            account.pets.owned = 1;
            app.insert_resource(account);
            app.insert_resource(Run::default());
            app.insert_resource(super::super::stash::Stash::blank());
            app.init_resource::<NextState<Phase>>();
            app.world_mut()
                .spawn((Hero { vy: 0., air: 0. }, Transform::default()));
            app.world_mut().spawn((
                Fetcher { slot: 0, rest: 0. },
                Transform::default(),
                MovementState::default(),
            ));
            let coin = app
                .world_mut()
                .spawn((
                    Gem::new(GemKind::Gold(10)),
                    Transform::from_xyz(1., height, 0.),
                ))
                .id();
            if wall {
                app.world_mut().spawn((
                    RigidBody::Static,
                    Collider::cuboid(0.1, 4., 6.),
                    Transform::from_xyz(0.5, 2., 0.),
                ));
            }
            app.update();
            app.world_mut().run_system_once(herd).unwrap();
            assert_eq!(
                app.world().get_entity(coin).is_err(),
                collected,
                "wall={wall}, height={height}"
            );
            assert_eq!(
                app.world().resource::<Run>().gold_found,
                if collected { 10 } else { 0 }
            );
        }
    }

    #[test]
    fn fetcher_routes_around_fence_and_returns_after_banking_once() {
        use avian3d::prelude::{Collider, RigidBody};
        use bevy::ecs::system::RunSystemOnce;
        let mut app = super::super::floor::tests::fixture(&[0.]);
        app.init_resource::<Selection>();
        let mut account = super::super::account::empty();
        account.pets.owned = 1;
        account.pets.instinct[0] = 1;
        app.insert_resource(account);
        app.insert_resource(Run::default());
        app.insert_resource(super::super::stash::Stash::blank());
        app.init_resource::<NextState<Phase>>();
        let start = Vec3::X * -4.;
        let goal = Vec3::X * 4.;
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(0.4, 0.8, 6.),
            Transform::from_xyz(0., 0.4, 0.),
        ));
        app.world_mut()
            .spawn((Hero { vy: 0., air: 0. }, Transform::from_translation(start)));
        let pet = app
            .world_mut()
            .spawn((
                Fetcher { slot: 0, rest: 0. },
                Transform::from_translation(start),
                MovementState::default(),
            ))
            .id();
        let coin = app
            .world_mut()
            .spawn((
                Gem::new(GemKind::Gold(10)),
                Transform::from_translation(goal + Vec3::Y * 0.5),
            ))
            .id();
        app.update();
        app.world_mut()
            .run_system_once(move |floor: Floor| {
                assert!(
                    floor.line_clear(start, goal),
                    "sight alone does not prove body clearance"
                );
                assert!(floor.travel(start, goal, BODY_RADIUS).distance(goal) > 1.);
            })
            .unwrap();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.05));
        app.insert_resource(time);
        let system = app.world_mut().register_system(herd);
        let mut detoured = false;
        let mut banked = false;
        let mut returned = false;
        for _ in 0..1000 {
            app.world_mut().run_system(system).unwrap();
            let at = app.world().get::<Transform>(pet).unwrap().translation;
            assert!(at.y.abs() < 0.05, "never climb the fence: {at:?}");
            detoured |= at.z.abs() > 3.;
            if app.world().get_entity(coin).is_err() {
                assert!(detoured, "must walk around before banking");
                banked = true;
                assert_eq!(app.world().resource::<Run>().gold_found, 11);
                if at.distance(follow_at(start, 0)) <= 0.9 {
                    returned = true;
                    break;
                }
            }
        }
        assert!(banked, "fetcher must reach the coin beyond the fence");
        assert!(returned, "return trip must also route around the fence");
        assert_eq!(
            app.world().resource::<super::super::stash::Stash>().gold,
            11
        );
    }

    #[test]
    fn two_companions_bank_each_shared_pickup_once_before_deferred_despawn() {
        use bevy::ecs::system::RunSystemOnce;
        for slots in [[0, 1], [1, 0]] {
            let mut app = super::super::floor::tests::fixture(&[]);
            let mut account = super::super::account::empty();
            account.pets.owned = 2;
            account.pets.instinct = [1, 1];
            app.insert_resource(account);
            app.init_resource::<Selection>();
            app.insert_resource(Run::default());
            app.insert_resource(super::super::stash::Stash::blank());
            app.init_resource::<NextState<Phase>>();
            app.world_mut()
                .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
            let at = Vec3::X * 10.0;
            for slot in slots {
                app.world_mut().spawn((
                    Fetcher { slot, rest: 0.0 },
                    Transform::from_translation(at),
                    MovementState::default(),
                ));
            }
            let coin = app
                .world_mut()
                .spawn((Gem::new(GemKind::Gold(10)), Transform::from_translation(at)))
                .id();
            let xp = app
                .world_mut()
                .spawn((Gem::new(GemKind::Xp(1)), Transform::from_translation(at)))
                .id();
            let heal = app
                .world_mut()
                .spawn((
                    Gem::new(GemKind::Heal(10.0)),
                    Transform::from_translation(at),
                ))
                .id();
            app.world_mut().run_system_once(herd).unwrap();
            assert!(app.world().get_entity(coin).is_err());
            assert!(app.world().get_entity(xp).is_err());
            assert!(app.world().get_entity(heal).is_ok());
            assert_eq!(app.world().resource::<Run>().gold_found, 11);
            assert_eq!(
                app.world().resource::<super::super::stash::Stash>().gold,
                11
            );
            assert_eq!(
                app.world().resource::<Run>().xp,
                super::super::data::XP_RATE
            );
            app.world_mut().run_system_once(herd).unwrap();
            assert_eq!(app.world().resource::<Run>().gold_found, 11);
        }
    }

    #[test]
    fn chained_hero_and_companion_collection_preserves_bonus_ownership_and_death_gate() {
        for alive in [false, true] {
            for pulled in [false, true] {
                let mut app = super::super::floor::tests::fixture(&[]);
                let mut account = super::super::account::empty();
                account.pets.owned = 2;
                account.pets.instinct = [1, 1];
                app.insert_resource(account);
                app.init_resource::<Selection>();
                let mut run = Run::default();
                if !alive {
                    run.hp = 0.0;
                }
                app.insert_resource(run);
                app.insert_resource(super::super::stash::Stash::blank());
                app.init_resource::<NextState<Phase>>();
                app.world_mut()
                    .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
                let at = Vec3::Y;
                for slot in 0..2 {
                    app.world_mut().spawn((
                        Fetcher { slot, rest: 0.0 },
                        Transform::from_translation(at),
                        MovementState::default(),
                    ));
                }
                let mut gem = Gem::new(GemKind::Gold(10));
                gem.pulled = pulled;
                let coin = app
                    .world_mut()
                    .spawn((gem, Transform::from_translation(at)))
                    .id();
                let mut schedule = Schedule::default();
                schedule.add_systems((herd, enemies::collect_gems).chain());
                schedule.run(app.world_mut());
                let expected = if !alive {
                    0
                } else if pulled {
                    10
                } else {
                    11
                };
                assert_eq!(app.world().resource::<Run>().gold_found, expected);
                assert_eq!(
                    app.world().resource::<super::super::stash::Stash>().gold,
                    expected
                );
                assert_eq!(app.world().get_entity(coin).is_err(), alive);
                schedule.run(app.world_mut());
                assert_eq!(app.world().resource::<Run>().gold_found, expected);
            }
        }
    }

    #[test]
    fn companion_displays_models_and_icons_exist_in_the_patch_chain() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_creature_catalog(&mut chain).unwrap();
        for look in LOOKS {
            let model = catalog.model(look.display).expect(look.name);
            let url = benilla_assets::m2_url(&model.model_path);
            assert!(!chain
                .read(url.strip_prefix("mpq://").unwrap())
                .expect(look.name)
                .is_empty());
            assert!(!chain
                .read(&format!("Interface\\Icons\\{}.blp", look.icon))
                .expect(look.name)
                .is_empty());
        }
        assert_eq!(
            LOOKS[..5]
                .iter()
                .map(|look| look.display)
                .collect::<Vec<_>>(),
            [328, 134, 304, 1206, 6288]
        );
    }

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
