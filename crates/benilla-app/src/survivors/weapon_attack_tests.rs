//! Real auto-cast and damage settlement; no unattended game session.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::gear_loadout::{MeleeKind, MeleeWeapon};

fn setup(two_handed: bool) -> (App, Entity, Entity) {
    let (mut app, hero, target) = super::earthbind_tests::setup(true);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[1];
        run.milestones = survivors_core::talents::Milestones {
            class: 1,
            weapon: Some(MeleeWeapon {
                kind: MeleeKind::Mace,
                two_handed,
            }),
            ..Default::default()
        };
        run.abilities.clear();
        run.bonus.targets = 0;
    }
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 2.;
    (app, hero, target)
}

#[test]
fn paladin_weapon_attack_without_slots_settles_and_obeys_cadence() {
    for (two_handed, base, seconds) in [(false, 24., 2.), (true, 36., 3.)] {
        let (mut app, _, target) = setup(two_handed);
        let expected = base * app.world().resource::<Run>().stats().might;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(app.world().resource::<Run>().weapon_attack_cd, seconds);
        assert!(app.world().resource::<Run>().abilities.is_empty());
        app.world_mut().run_system_once(apply_hits).unwrap();
        let damage = 1000. - app.world().get::<Enemy>(target).unwrap().hp;
        // The fixture RNG may crit; verify settlement against the recorded real hit.
        assert!(damage >= expected && damage <= expected * 2.01);
        assert_eq!(
            app.world().resource::<Run>().damage_sources[0].0,
            "Weapon Attack"
        );
        app.world_mut().resource_mut::<Messages<Hit>>().clear();
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
    }
}

#[test]
fn paladin_weapon_attack_rejects_unarmed_hidden_distant_dead_and_other_classes() {
    for case in 0..6 {
        let (mut app, _, target) = setup(false);
        match case {
            0 => app.world_mut().resource_mut::<Run>().milestones.weapon = None,
            1 => {
                app.world_mut()
                    .entity_mut(target)
                    .insert(super::super::perception::Concealed);
            }
            2 => {
                app.world_mut()
                    .get_mut::<Transform>(target)
                    .unwrap()
                    .translation = Vec3::Y * 8.
            }
            3 => app.world_mut().resource_mut::<Run>().hp = 0.,
            4 => app.world_mut().get_mut::<Enemy>(target).unwrap().dead = true,
            _ => app.world_mut().resource_mut::<Run>().class = &data::CLASSES[0],
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert!(
            app.world().resource::<Messages<Hit>>().is_empty(),
            "case {case}"
        );
        assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 0.);
    }
}

#[test]
fn paladin_weapon_attack_requires_clear_geometry_and_does_not_spend_empty_swing() {
    use avian3d::prelude::*;
    let (mut app, _, _) = setup(false);
    let wall = app
        .world_mut()
        .spawn((
            RigidBody::Static,
            Collider::cuboid(0.2, 5., 5.),
            Transform::from_xyz(1., 1., 0.),
        ))
        .id();
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 0.);
    app.world_mut().despawn(wall);
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(app.world().resource::<Messages<Hit>>().len(), 1);
}

#[test]
fn paladin_weapon_attack_preserves_six_slots_and_freezes_when_paused() {
    let (mut app, _, _) = setup(false);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        for _ in 0..6 {
            run.abilities.push(crate::survivors::Owned {
                def: data::ability("holy_shock"),
                level: 1,
                cd: 100.,
            });
        }
        run.bonus.might = 0.5;
        run.weapon_attack_cd = 1.;
    }
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::ZERO);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 1.);
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs(1));
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(app.world().resource::<Run>().abilities.len(), 6);
    let hits: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<Hit>>()
        .drain()
        .collect();
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0].damage >= 36.,
        "permanent equipment power reaches the basic swing"
    );
    assert_eq!(hits[0].source, "Weapon Attack");
}

#[test]
fn paladin_weapon_attack_extra_targets_stay_in_reach_and_haste_sets_cadence() {
    let (mut app, _, first) = setup(false);
    let mut added = Vec::new();
    for x in [2.5, 3., 10.] {
        let (mut donor, target, _) = super::poison_tests::fixture("holy_shock", 1000.);
        let enemy = donor
            .world_mut()
            .entity_mut(target)
            .take::<Enemy>()
            .unwrap();
        added.push(
            app.world_mut()
                .spawn((enemy, Transform::from_xyz(x, 0., 0.)))
                .id(),
        );
    }
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.bonus.targets = 1;
        // Renown contributes through the existing haste stat.
        run.renown_celerity = 10;
    }
    let haste = app.world().resource::<Run>().stats().haste;
    app.world_mut().run_system_once(cast_abilities).unwrap();
    let hits: Vec<_> = app
        .world_mut()
        .resource_mut::<Messages<Hit>>()
        .drain()
        .collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].target, first);
    assert_eq!(hits[1].target, added[0]);
    assert!((app.world().resource::<Run>().weapon_attack_cd - 2. / (1. + haste)).abs() < 0.0001);
    assert!(hits.iter().all(|hit| hit.splash == 0. && hit.dot == 0.));
}

#[test]
fn paladin_weapon_training_reaches_actual_damage_and_physical_vulnerability() {
    for (two_handed, id, tree, cap, base) in [(false, 1429, 1, 5, 24.), (true, 1410, 2, 3, 36.)] {
        for rank in [0, 1, 3, 5, 255] {
            let (mut app, _, target) = setup(two_handed);
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.milestones = run.milestones.with_ranks(&[(tree, id, rank)]);
            }
            app.world_mut()
                .get_mut::<HemorrhageVulnerability>(target)
                .unwrap()
                .0
                .apply(0.);
            let stats = app.world().resource::<Run>().stats();
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let hits: Vec<_> = app
                .world()
                .resource::<Messages<Hit>>()
                .iter_current_update_messages()
                .cloned()
                .collect();
            assert_eq!(hits.len(), 1);
            let expected = base
                * stats.might
                * (1. + 0.02 * f32::from(rank.min(cap)))
                * if hits[0].crit { stats.crit_damage } else { 1. };
            assert!((hits[0].damage - expected).abs() < 0.001);
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert!(
                (1000. - app.world().get::<Enemy>(target).unwrap().hp - expected - 3.).abs()
                    < 0.001
            );
            assert_eq!(
                app.world()
                    .get::<HemorrhageVulnerability>(target)
                    .unwrap()
                    .0
                    .charges(0.),
                29
            );
        }
    }
}

#[test]
fn vengeance_weapon_criticals_boost_later_swings_without_stacking() {
    for two_handed in [false, true] {
        let (mut app, _, target) = setup(two_handed);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = run.milestones.with_ranks(&[(2, 1402, 5)]);
            run.bonus.crit = 1.;
        }
        let mut previous = 0.;
        for swing in 0..3 {
            app.world_mut().resource_mut::<Run>().weapon_attack_cd = 0.;
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let hit = app
                .world()
                .resource::<Messages<Hit>>()
                .iter_current_update_messages()
                .next()
                .unwrap()
                .clone();
            assert!(hit.crit);
            if swing == 1 {
                assert!((hit.damage - previous * 1.15).abs() < 0.001);
            }
            if swing == 2 {
                assert_eq!(hit.damage, previous);
            }
            previous = hit.damage;
            let before = app.world().get::<Enemy>(target).unwrap().hp;
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert!(
                (before - app.world().get::<Enemy>(target).unwrap().hp - hit.damage).abs() < 0.001
            );
            assert_eq!(app.world().resource::<Run>().vengeance_t, 8.);
            app.world_mut().resource_mut::<Messages<Hit>>().clear();
        }
    }
}

#[test]
fn vengeance_rejects_pet_noncritical_and_ineffective_weapon_hits() {
    for case in 0..6 {
        let (mut app, _, target) = setup(false);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.milestones = run.milestones.with_ranks(&[(2, 1402, 5)]);
            if case == 5 {
                run.hp = 0.;
            }
        }
        let mut hit = Hit {
            source: "Weapon Attack",
            ..Hit::direct(target, 10., true)
        };
        match case {
            0 => hit.origin = HitOrigin::CombatPet,
            1 => hit.crit = false,
            2 => hit.damage = 0.,
            3 => app.world_mut().get_mut::<Enemy>(target).unwrap().hp = 0.,
            4 => app.world_mut().get_mut::<Enemy>(target).unwrap().dead = true,
            _ => {}
        }
        app.world_mut().resource_mut::<Messages<Hit>>().write(hit);
        app.world_mut().run_system_once(apply_hits).unwrap();
        assert_eq!(app.world().resource::<Run>().vengeance_t, 0., "case {case}");
    }
}
