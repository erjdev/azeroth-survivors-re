//! Actor ownership must survive shared ability labels, splash and periodic settlement.
use super::poison_tests::{advance, fixture};
use super::*;
use crate::survivors::Owned;
use survivors_core::talents::Milestones;

#[test]
fn pet_weapon_labels_cannot_trigger_hero_poison_or_seal_fate() {
    for origin in [HitOrigin::Hero, HitOrigin::CombatPet] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = run.milestones.with_ranks(&[(0, 283, 5)]);
            run.abilities.push(Owned {
                def: data::ability("eviscerate"),
                level: 1,
                cd: 10.0,
            });
        }
        app.world_mut().write_message(Hit {
            origin,
            source: "sinister_strike",
            ..Hit::direct(target, 10.0, true)
        });
        advance(&mut app, &mut schedule, 0.0);
        let run = app.world().resource::<Run>();
        let enemy = app.world().get::<Enemy>(target).unwrap();
        if origin == HitOrigin::CombatPet {
            assert_eq!(run.abilities[1].cd, 10.0);
            assert_eq!(enemy.hp, 990.0);
            assert_eq!(run.damage_sources, vec![("sinister_strike", 10.0)]);
        } else {
            assert_eq!(run.abilities[1].cd, 9.0);
            assert!(enemy.hp < 990.0);
            assert!(run
                .damage_sources
                .iter()
                .any(|(key, _)| *key == "instant_poison"));
        }
    }
}

#[test]
fn pet_fire_labels_neither_build_combustion_nor_ignite_or_scorch() {
    for origin in [HitOrigin::Hero, HitOrigin::CombatPet] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &data::CLASSES[6];
            run.abilities.clear();
            run.milestones = Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(1, 36, 1), (1, 34, 5), (1, 25, 3)]);
        }
        app.world_mut().write_message(Hit {
            origin,
            source: "scorch",
            ..Hit::direct(target, 10.0, true)
        });
        advance(&mut app, &mut schedule, 0.0);
        let run = app.world().resource::<Run>();
        let enemy = app.world().get::<Enemy>(target).unwrap();
        assert_eq!(enemy.dots.is_empty(), origin == HitOrigin::CombatPet);
        assert_eq!(
            run.combustion
                .progress(run.milestones, run.elapsed)
                .is_none(),
            origin == HitOrigin::CombatPet
        );
        assert_eq!(
            app.world()
                .get::<FireVulnerability>(target)
                .unwrap()
                .0
                .multiplier(true, 0.0),
            if origin == HitOrigin::Hero { 1.03 } else { 1.0 }
        );
    }
}

#[test]
fn pet_splash_and_periodic_damage_keep_origin_and_normal_settlement() {
    let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.abilities.clear();
        run.milestones = run.milestones.with_ranks(&[(0, 274, 2)]);
    }
    let (mut other, other_target, _) = fixture("instant_poison", 1000.0);
    let mut bundle = other
        .world_mut()
        .entity_mut(other_target)
        .take::<(Enemy, Guid, Transform, ObjectStore, Marks)>()
        .unwrap();
    bundle.1 = Guid(124);
    bundle.2.translation = Vec3::X;
    let neighbor = app.world_mut().spawn(bundle).id();
    app.world_mut().write_message(Hit {
        origin: HitOrigin::CombatPet,
        source: "sinister_strike",
        splash: 3.0,
        dot: 4.0,
        dot_dur: 2.0,
        ..Hit::direct(target, 10.0, false)
    });
    advance(&mut app, &mut schedule, 0.0);
    for e in [target, neighbor] {
        assert_eq!(
            app.world().get::<Enemy>(e).unwrap().dots[0].origin,
            HitOrigin::CombatPet
        );
    }
    advance(&mut app, &mut schedule, 2.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 982.0);
    assert_eq!(app.world().get::<Enemy>(neighbor).unwrap().hp, 985.0);
    assert_eq!(app.world().resource::<Run>().damage_done, 33.0);
    assert!((app.world().resource::<Run>().hp - 53.3).abs() < 0.001);

    let (mut app, target, mut schedule) = fixture("instant_poison", 10.0);
    for _ in 0..2 {
        app.world_mut().write_message(Hit {
            origin: HitOrigin::CombatPet,
            source: "Renamed pet",
            ..Hit::direct(target, 10.0, false)
        });
    }
    advance(&mut app, &mut schedule, 0.0);
    let run = app.world().resource::<Run>();
    assert_eq!(run.kills, 1);
    assert_eq!(run.damage_done, 10.0);
    assert_eq!(run.hp, 51.0);
    assert_eq!(app.world().resource::<Messages<Loot>>().len(), 1);
}

#[test]
fn pet_corruption_ticks_do_not_ready_the_owners_shadow_bolt() {
    for origin in [HitOrigin::Hero, HitOrigin::CombatPet] {
        let (mut app, target, mut schedule) = fixture("shadow_bolt", 1000.0);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &data::CLASSES[7];
            run.milestones = Milestones {
                class: 7,
                ..Default::default()
            }
            .with_ranks(&[(0, 1002, 2)]);
            run.abilities[0].cd = 10.0;
        }
        app.world_mut().write_message(Hit {
            origin,
            source: "corruption",
            dot: 4.0,
            dot_dur: 2.0,
            ..Hit::direct(target, 0.0, false)
        });
        advance(&mut app, &mut schedule, 0.0);
        advance(&mut app, &mut schedule, 1.0);
        assert_eq!(
            app.world().resource::<Run>().abilities[0].cd,
            if origin == HitOrigin::Hero { 0.0 } else { 10.0 }
        );
        assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 996.0);
    }
}
