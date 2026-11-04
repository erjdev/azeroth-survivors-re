//! Per-victim category bonuses in actual hit and periodic settlement.
use super::poison_tests::{advance, fixture};
use super::*;
use crate::survivors::data;
use survivors_core::{creature::CreatureType, talents::Milestones};

fn definition(kind: CreatureType) -> &'static data::EnemyDef {
    (0..data::MAPS.len())
        .flat_map(|index| {
            data::roster(index)
                .enemies
                .iter()
                .chain(&data::roster(index).bosses)
        })
        .find(|def| def.creature_type == kind)
        .unwrap()
}

fn train(app: &mut App, class: usize) {
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[class];
    run.milestones = Milestones {
        class,
        ..run.milestones
    }
    .with_ranks(&[(0, 274, 2), (2, 1623, 3), (2, 1301, 3)]);
}

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.0002, "{actual} != {expected}");
}

#[test]
fn direct_critical_and_pet_hits_settle_against_each_category() {
    for (class, source, origin) in [
        (2, "arcane_shot", HitOrigin::Hero),
        (3, "sinister_strike", HitOrigin::Hero),
        (2, super::super::hunter_pet::SOURCE, HitOrigin::CombatPet),
        (2, "arcane_shot", HitOrigin::CombatPet),
        (2, "Renamed companion", HitOrigin::CombatPet),
        (2, super::super::hunter_pet::SOURCE, HitOrigin::Hero),
    ] {
        for kind in [
            CreatureType::Beast,
            CreatureType::Humanoid,
            CreatureType::Undead,
            CreatureType::Mechanical,
        ] {
            for critical in [false, true] {
                let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
                train(&mut app, class);
                app.world_mut().resource_mut::<Run>().abilities.clear();
                app.world_mut().get_mut::<Enemy>(target).unwrap().def = definition(kind);
                let input = if critical { 200.0 } else { 100.0 };
                app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
                    source,
                    origin,
                    ..Hit::direct(target, input, critical)
                });
                advance(&mut app, &mut schedule, 0.0);
                let matches = matches!(kind, CreatureType::Beast | CreatureType::Humanoid);
                let factor = if !matches || origin == HitOrigin::CombatPet {
                    1.0
                } else if class == 3 {
                    1.02
                } else if critical {
                    1.03 * 1.03
                } else {
                    1.03
                };
                near(
                    app.world().get::<Enemy>(target).unwrap().hp,
                    1000.0 - input * factor,
                );
                near(
                    app.world().resource::<Run>().damage_done as f32,
                    input * factor,
                );
                near(
                    app.world().resource::<Run>().hp,
                    50.0 + input * factor * 0.1,
                );
            }
        }
    }
}

#[test]
fn splash_uses_its_own_victim_category_without_baking_in_the_primary_bonus() {
    let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
    train(&mut app, 2);
    app.world_mut().resource_mut::<Run>().abilities.clear();
    let (mut other, other_target, _) = fixture("instant_poison", 1000.0);
    let mut bundle = other
        .world_mut()
        .entity_mut(other_target)
        .take::<(Enemy, Guid, Transform, ObjectStore, Marks)>()
        .unwrap();
    bundle.0.def = definition(CreatureType::Undead);
    bundle.1 = Guid(124);
    bundle.2.translation = Vec3::X;
    let splash = app.world_mut().spawn(bundle).id();
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source: "arcane_shot",
        splash: 5.0,
        ..Hit::direct(target, 100.0, true)
    });
    advance(&mut app, &mut schedule, 0.0);
    near(
        app.world().get::<Enemy>(target).unwrap().hp,
        1000.0 - 106.09,
    );
    near(app.world().get::<Enemy>(splash).unwrap().hp, 930.0);
}

#[test]
fn murder_applies_once_to_instant_deadly_and_generic_periodic_damage() {
    for key in ["instant_poison", "deadly_poison"] {
        let (mut app, target, mut schedule) = fixture(key, 1000.0);
        train(&mut app, 3);
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "sinister_strike",
            ..Hit::direct(target, 10.0, false)
        });
        advance(&mut app, &mut schedule, 0.0);
        let first = if key == "instant_poison" {
            32.0 * 1.02
        } else {
            10.0 * 1.02
        };
        near(app.world().get::<Enemy>(target).unwrap().hp, 1000.0 - first);
        advance(&mut app, &mut schedule, 3.0);
        let poison = if key == "deadly_poison" {
            9.0 * 1.02
        } else {
            0.0
        };
        near(
            app.world().get::<Enemy>(target).unwrap().hp,
            1000.0 - first - poison,
        );
    }
    let (mut app, target, mut schedule) = fixture("instant_poison", 100.0);
    train(&mut app, 3);
    app.world_mut()
        .get_mut::<Enemy>(target)
        .unwrap()
        .dots
        .push(super::super::enemies::Dot {
            origin: HitOrigin::Hero,
            source: "rupture",
            dps: 10.0,
            left: 3.0,
            tick: 1.0,
        });
    advance(&mut app, &mut schedule, 3.0);
    near(app.world().get::<Enemy>(target).unwrap().hp, 69.4);
}

#[test]
fn generated_sword_strike_receives_murder_once_without_recursive_procs() {
    use survivors_core::gear_loadout::{MeleeKind, MeleeWeapon};
    let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.abilities.clear();
        run.milestones.weapon = Some(MeleeWeapon {
            kind: MeleeKind::Sword,
            two_handed: false,
        });
        run.milestones = run.milestones.with_ranks(&[(0, 274, 2), (1, 242, 5)]);
    }
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source: "sinister_strike",
        ..Hit::direct(target, 100.0, true)
    });
    advance(&mut app, &mut schedule, 0.0);
    near(app.world().get::<Enemy>(target).unwrap().hp, 796.0);
    let run = app.world().resource::<Run>();
    assert_eq!(run.damage_sources.len(), 2);
    for (_, damage) in &run.damage_sources {
        near(*damage as f32, 102.0);
    }
}

#[test]
fn category_boosted_poison_kills_keep_single_loot_and_effective_leech() {
    for key in ["instant_poison", "deadly_poison"] {
        let (mut app, target, mut schedule) = fixture(key, 25.0);
        train(&mut app, 3);
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "sinister_strike",
            ..Hit::direct(target, 10.0, false)
        });
        advance(&mut app, &mut schedule, 0.0);
        advance(&mut app, &mut schedule, 30.0);
        advance(&mut app, &mut schedule, 30.0);
        assert!(app.world().get::<Enemy>(target).unwrap().dead);
        let run = app.world().resource::<Run>();
        assert_eq!(run.kills, 1);
        near(run.damage_done as f32, 25.0);
        near(run.hp, 52.5);
        assert_eq!(app.world().resource::<Messages<Loot>>().len(), 1);
    }
}
