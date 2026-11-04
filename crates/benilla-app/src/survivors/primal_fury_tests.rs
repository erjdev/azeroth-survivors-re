//! Effective-hit recovery, with source/origin isolation and no stored refunds.
use super::*;
use crate::survivors::Owned;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::{forms::DruidForm, Milestones};

fn setup() -> (App, Entity) {
    let (mut app, _, target) = super::earthbind_tests::setup(true);
    app.insert_resource(Time::<()>::default());
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[8];
    run.milestones = Milestones {
        class: 8,
        ..Default::default()
    }
    .with_ranks(&[(1, DruidForm::Bear.talent(), 1), (1, 801, 2)]);
    run.abilities = ["swipe", "wrath", "bash"]
        .into_iter()
        .map(|key| Owned {
            def: data::ability(key),
            level: 1,
            cd: 5.,
        })
        .collect();
    run.weapon_attack_cd = 2.;
    run.renown_celerity = 10;
    (app, target)
}

fn land(
    app: &mut App,
    target: Entity,
    source: &'static str,
    origin: HitOrigin,
    damage: f32,
    critical: bool,
) {
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        origin,
        ..Hit::direct(target, damage, critical)
    });
    app.world_mut().run_system_once(apply_hits).unwrap();
}

#[test]
fn bear_criticals_refund_only_the_source_and_share_a_hasted_internal_cooldown() {
    let (mut app, target) = setup();
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 1.5);
    assert!(app
        .world()
        .resource::<Run>()
        .abilities
        .iter()
        .all(|a| a.cd == 5.));
    land(&mut app, target, "swipe", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().abilities[0].cd, 5.);
    let ready = 1. / (1. + app.world().resource::<Run>().stats().haste);
    app.world_mut().resource_mut::<Run>().elapsed = ready;
    land(&mut app, target, "swipe", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().abilities[0].cd, 4.5);
    land(&mut app, target, "swipe", HitOrigin::Hero, 10., true);
    let run = app.world().resource::<Run>();
    assert_eq!(
        run.abilities.iter().map(|a| a.cd).collect::<Vec<_>>(),
        [4.5, 5., 5.]
    );
    assert_eq!(run.weapon_attack_cd, 1.5);
}

#[test]
fn fury_excludes_noncriticals_foreign_sources_and_nonhero_origins() {
    for (source, origin, damage, critical) in [
        ("Maul", HitOrigin::Hero, 10., false),
        ("Maul", HitOrigin::Hero, 0., true),
        ("Maul", HitOrigin::CombatPet, 10., true),
        ("Maul", HitOrigin::TriggeredHero, 10., true),
        ("Maul", HitOrigin::Reflection, 10., true),
        ("wrath", HitOrigin::Hero, 10., true),
        ("Claw", HitOrigin::Hero, 10., true),
    ] {
        let (mut app, target) = setup();
        land(&mut app, target, source, origin, damage, critical);
        let run = app.world().resource::<Run>();
        assert_eq!(run.weapon_attack_cd, 2.);
        assert!(run.abilities.iter().all(|a| a.cd == 5.));
        // Rejected events did not consume the shared proc clock.
        land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
        assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 1.5);
    }
}

#[test]
fn zero_recovery_never_banks_fury_and_refunds_clamp_at_ready() {
    let (mut app, target) = setup();
    app.world_mut().resource_mut::<Run>().weapon_attack_cd = 0.;
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    app.world_mut().resource_mut::<Run>().weapon_attack_cd = 0.2;
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 0.);
    app.world_mut().resource_mut::<Run>().weapon_attack_cd = 2.;
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 2.);
    app.world_mut().resource_mut::<Run>().elapsed = 100.;
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    land(&mut app, target, "Maul", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 1.5);
}

fn cat_setup() -> (App, Entity) {
    let (mut app, target) = setup();
    let mut run = app.world_mut().resource_mut::<Run>();
    run.milestones = Milestones {
        class: 8,
        ..Default::default()
    }
    .with_ranks(&[(1, DruidForm::Cat.talent(), 1), (1, 800, 2)]);
    run.abilities[0].def = data::ability("rake");
    run.abilities[2].def = data::ability("shred");
    (app, target)
}

#[test]
fn cat_criticals_recover_only_the_source_with_a_shared_hasted_clock() {
    let (mut app, target) = cat_setup();
    land(&mut app, target, "Claw", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 1.5);
    land(&mut app, target, "rake", HitOrigin::Hero, 10., true);
    assert_eq!(app.world().resource::<Run>().abilities[0].cd, 5.);
    let ready = 1. / (1. + app.world().resource::<Run>().stats().haste);
    app.world_mut().resource_mut::<Run>().elapsed = ready;
    land(&mut app, target, "rake", HitOrigin::Hero, 10., true);
    app.world_mut().resource_mut::<Run>().elapsed = ready * 2.;
    land(&mut app, target, "shred", HitOrigin::Hero, 10., true);
    let run = app.world().resource::<Run>();
    assert_eq!(
        run.abilities.iter().map(|a| a.cd).collect::<Vec<_>>(),
        [4.5, 5., 4.5]
    );
    assert_eq!(run.weapon_attack_cd, 1.5);
}

#[test]
fn cat_recovery_excludes_generated_hits_and_never_banks_a_refund() {
    for (source, origin, damage, critical) in [
        ("Claw", HitOrigin::Hero, 10., false),
        ("Claw", HitOrigin::Hero, 0., true),
        ("rake", HitOrigin::CombatPet, 10., true),
        ("rake", HitOrigin::TriggeredHero, 10., true),
        ("shred", HitOrigin::Reflection, 10., true),
        ("wrath", HitOrigin::Hero, 10., true),
        ("Maul", HitOrigin::Hero, 10., true),
    ] {
        let (mut app, target) = cat_setup();
        land(&mut app, target, source, origin, damage, critical);
        assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 2.);
        assert!(app
            .world()
            .resource::<Run>()
            .abilities
            .iter()
            .all(|a| a.cd == 5.));
        app.world_mut().resource_mut::<Run>().weapon_attack_cd = 0.;
        land(&mut app, target, "Claw", HitOrigin::Hero, 10., true);
        app.world_mut().resource_mut::<Run>().weapon_attack_cd = 0.2;
        land(&mut app, target, "Claw", HitOrigin::Hero, 10., true);
        assert_eq!(app.world().resource::<Run>().weapon_attack_cd, 0.);
    }
}
