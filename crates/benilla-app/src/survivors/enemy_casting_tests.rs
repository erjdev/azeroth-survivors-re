//! Actual chase, spell settlement and control state, without unattended combat.
use super::*;
use crate::survivors::{enemies::move_enemies, enemy_casting};
use bevy::ecs::system::RunSystemOnce;

fn setup() -> (App, Entity, Entity) {
    let (mut app, hero, enemy) = super::earthbind_tests::setup(true);
    let def = (0..data::MAPS.len())
        .flat_map(|map| data::roster(map).enemies.iter())
        .find(|def| def.template_entry == 1522)
        .unwrap();
    {
        let mut e = app.world_mut().get_mut::<Enemy>(enemy).unwrap();
        e.def = def;
        e.damage = 10.;
        e.attack_cd = 0.;
    }
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.hp = 1000.;
        run.bonus.armor = 100.;
        run.abilities.clear();
        run.thorns = 50.;
        run.thorns_t = 20.;
        run.thorns_spell = data::ability("thorns").spell;
    }
    (app, hero, enemy)
}

fn tick(app: &mut App, now: f32) {
    app.world_mut().resource_mut::<Run>().elapsed = now;
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    app.insert_resource(time);
    app.world_mut().run_system_once(move_enemies).unwrap();
}

#[test]
fn enemy_frostbolt_casts_then_settles_magic_once_and_slows_without_melee_reflection() {
    let (mut app, hero, enemy) = setup();
    let position = app.world().get::<Transform>(enemy).unwrap().translation;
    tick(&mut app, 0.);
    let active = app
        .world()
        .get::<enemy_casting::Caster>(enemy)
        .unwrap()
        .state
        .active()
        .unwrap();
    assert_eq!(
        (active.spell.id, active.target, active.finishes),
        (13322, hero, 3.)
    );
    assert_eq!(
        app.world().get::<Transform>(enemy).unwrap().translation,
        position
    );
    assert_eq!(app.world().resource::<Run>().hp, 1000.);
    tick(&mut app, 2.99);
    assert_eq!(app.world().resource::<Run>().hp, 1000.);
    tick(&mut app, 3.);
    let run = app.world().resource::<Run>();
    assert_eq!(run.hp, 970.); // 100 flat armor must not reduce magic.
    assert_eq!(run.incoming_slows.state(3.), (0.5, 2.));
    assert_eq!(run.incoming_slows.state(5.), (1., 0.));
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    tick(&mut app, 3.);
    assert_eq!(app.world().resource::<Run>().hp, 970.);
    tick(&mut app, 4.99);
    assert!(app
        .world()
        .get::<enemy_casting::Caster>(enemy)
        .unwrap()
        .state
        .active()
        .is_none());
    tick(&mut app, 5.);
    assert_eq!(
        app.world()
            .get::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .active()
            .unwrap()
            .finishes,
        8.
    );
}

#[test]
fn enemy_frostbolt_cancels_on_control_death_range_visibility_and_target_loss() {
    use avian3d::prelude::*;
    for case in 0..8 {
        let (mut app, hero, enemy) = setup();
        tick(&mut app, 0.);
        match case {
            0 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().stun_t = 5.,
            1 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().fear_t = 5.,
            2 => {
                app.world_mut()
                    .get_mut::<Enemy>(enemy)
                    .unwrap()
                    .incapacitate_t = 5.
            }
            3 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().dead = true,
            4 => app.world_mut().resource_mut::<Run>().hp = 0.,
            5 => {
                app.world_mut()
                    .get_mut::<Transform>(hero)
                    .unwrap()
                    .translation = Vec3::X * -50.
            }
            6 => app.world_mut().resource_mut::<Run>().mobile.vanish_t = 5.,
            _ => {
                app.world_mut().spawn((
                    RigidBody::Static,
                    Collider::cuboid(0.2, 5., 5.),
                    Transform::from_xyz(2., 1., 0.),
                ));
                app.update();
            }
        }
        let hp = app.world().resource::<Run>().hp;
        tick(&mut app, 1.);
        assert!(
            app.world()
                .get::<enemy_casting::Caster>(enemy)
                .unwrap()
                .state
                .active()
                .is_none(),
            "case {case}"
        );
        tick(&mut app, 3.);
        assert_eq!(app.world().resource::<Run>().hp, hp, "case {case}");
    }
}

#[test]
fn enemy_frostbolt_respects_immunity_absorption_and_paused_time() {
    for immune in [true, false] {
        let (mut app, _, enemy) = setup();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.immune_t = if immune { 10. } else { 0. };
            run.shield = 50.;
            run.shield_t = 10.;
        }
        tick(&mut app, 0.);
        app.insert_resource(Time::<()>::default());
        app.world_mut().run_system_once(move_enemies).unwrap();
        assert_eq!(
            app.world()
                .get::<enemy_casting::Caster>(enemy)
                .unwrap()
                .state
                .active()
                .unwrap()
                .finishes,
            3.
        );
        tick(&mut app, 3.);
        let run = app.world().resource::<Run>();
        assert_eq!(run.hp, 1000.);
        assert_eq!(run.shield, if immune { 50. } else { 20. });
        assert_eq!(
            run.incoming_slows.state(3.).0,
            if immune { 1. } else { 0.5 }
        );
    }
}

#[test]
fn enemy_frostbolt_school_lockout_stops_casts_but_does_not_prevent_melee_fallback() {
    let (mut app, _, enemy) = setup();
    tick(&mut app, 0.);
    assert_eq!(
        app.world_mut()
            .get_mut::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .interrupt(1., 10.),
        survivors_core::casting::Interrupt::Interrupted(13322)
    );
    app.world_mut()
        .get_mut::<Transform>(enemy)
        .unwrap()
        .translation = Vec3::X * 0.5;
    tick(&mut app, 1.);
    assert!(app
        .world()
        .get::<enemy_casting::Caster>(enemy)
        .unwrap()
        .state
        .active()
        .is_none());
    assert!(app.world().resource::<Run>().hp < 1000.);
    assert_eq!(app.world().resource::<Run>().incoming_slows.state(1.).0, 1.);
}

#[test]
fn enemy_frostbolt_stoneclaw_switch_cancels_old_target_and_new_cast_hits_only_totem() {
    use crate::survivors::stoneclaw::Stoneclaw;
    let (mut app, hero, enemy) = setup();
    tick(&mut app, 0.);
    assert_eq!(
        app.world()
            .get::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .active()
            .unwrap()
            .target,
        hero
    );
    let totem = app
        .world_mut()
        .spawn((
            Transform::from_xyz(4., 0., 0.),
            crate::net::ObjectStore::default(),
            Stoneclaw {
                life: survivors_core::stoneclaw::Stoneclaw::deploy(1., 100.).unwrap(),
                radius: 8.,
                captured: Default::default(),
            },
        ))
        .id();
    tick(&mut app, 1.);
    assert!(app
        .world()
        .get::<enemy_casting::Caster>(enemy)
        .unwrap()
        .state
        .active()
        .is_none());
    tick(&mut app, 3.);
    assert_eq!(
        app.world()
            .get::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .active()
            .unwrap()
            .target,
        totem
    );
    tick(&mut app, 6.);
    assert_eq!(
        app.world().get::<Stoneclaw>(totem).unwrap().life.health(),
        70.
    );
    assert_eq!(app.world().resource::<Run>().hp, 1000.);
    assert!(app
        .world()
        .get::<Stoneclaw>(totem)
        .unwrap()
        .captured
        .contains(&enemy));
}

#[test]
fn enemy_frostbolt_pet_attention_keeps_damage_and_slow_on_pet() {
    use crate::survivors::hunter_pet::HunterPet;
    let (mut app, _, _) = setup();
    let mut pet = HunterPet::default();
    pet.vitality = survivors_core::combat_pet::Vitality::new(100., 10.);
    let pet_entity = app
        .world_mut()
        .spawn((
            pet,
            Transform::from_xyz(4., 0., 0.),
            crate::net::ObjectStore::default(),
        ))
        .id();
    tick(&mut app, 0.);
    tick(&mut app, 3.);
    let pet = app.world().get::<HunterPet>(pet_entity).unwrap();
    assert_eq!(pet.vitality.hp, 70.);
    assert_eq!(pet.incoming_slows.state(3.), (0.5, 2.));
    assert_eq!(app.world().resource::<Run>().hp, 1000.);
    assert_eq!(app.world().resource::<Run>().incoming_slows.state(3.).0, 1.);
}

#[test]
fn enemy_frostbolt_missing_hero_cancels_pending_cast() {
    let (mut app, hero, enemy) = setup();
    tick(&mut app, 0.);
    app.world_mut().despawn(hero);
    tick(&mut app, 1.);
    assert!(app
        .world()
        .get::<enemy_casting::Caster>(enemy)
        .unwrap()
        .state
        .active()
        .is_none());
    assert_eq!(app.world().resource::<Run>().hp, 1000.);
}

fn counterspell_setup(rank: u8) -> (App, Entity, Entity) {
    let (mut app, hero, enemy) = setup();
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[6];
        run.milestones = survivors_core::talents::Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(0, 88, rank)]);
        run.bonus.targets = 0;
        run.abilities = vec![crate::survivors::Owned {
            def: data::ability("counterspell"),
            level: 1,
            cd: 0.,
        }];
    }
    tick(&mut app, 0.);
    app.world_mut().resource_mut::<Run>().elapsed = 1.;
    (app, hero, enemy)
}

#[test]
fn counterspell_interrupts_without_damage_and_silence_is_independent_of_school_lock() {
    for rank in 0..=2 {
        for seed in [0, 1, 2, 3, u64::MAX] {
            let (mut app, _, enemy) = counterspell_setup(rank);
            app.insert_resource(Rng(seed));
            let silenced = rank == 2 || (rank == 1 && Rng(seed).f32() < 0.5);
            let hp = app.world().get::<Enemy>(enemy).unwrap().hp;
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let caster = app.world().get::<enemy_casting::Caster>(enemy).unwrap();
            use survivors_core::casting::School;
            assert!(caster.state.active().is_none());
            assert!(!caster.state.available(School::Frost, 10.99));
            assert!(caster.state.available(School::Frost, 11.));
            assert_eq!(caster.state.available(School::Fire, 4.99), !silenced);
            assert!(caster.state.available(School::Fire, 5.));
            assert_eq!(app.world().resource::<Run>().abilities[0].cd, 30.);
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            assert_eq!(app.world().get::<Enemy>(enemy).unwrap().hp, hp);
            tick(&mut app, 3.);
            assert_eq!(app.world().resource::<Run>().hp, 1000.);
        }
    }
}

#[test]
fn counterspell_waits_without_spending_recovery_for_ineligible_casts() {
    use avian3d::prelude::*;
    for case in 0..10 {
        let (mut app, _, enemy) = counterspell_setup(0);
        match case {
            0 => app
                .world_mut()
                .get_mut::<enemy_casting::Caster>(enemy)
                .unwrap()
                .state
                .cancel(),
            1 => {
                app.world_mut()
                    .entity_mut(enemy)
                    .insert(crate::survivors::perception::Concealed);
            }
            2 => {
                app.world_mut()
                    .get_mut::<Transform>(enemy)
                    .unwrap()
                    .translation = Vec3::X * 35.
            }
            3 => {
                app.world_mut().spawn((
                    RigidBody::Static,
                    Collider::cuboid(0.2, 5., 5.),
                    Transform::from_xyz(2., 1., 0.),
                ));
                app.update();
                let mut time = Time::<()>::default();
                time.advance_by(std::time::Duration::from_secs_f32(0.1));
                app.insert_resource(time);
            }
            4 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().boss = true,
            5 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().dead = true,
            6 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().hp = 0.,
            7 => app.world_mut().resource_mut::<Run>().hp = 0.,
            8 => app.world_mut().resource_mut::<Run>().elapsed = 3.,
            _ => {
                let mut caster = app
                    .world_mut()
                    .get_mut::<enemy_casting::Caster>(enemy)
                    .unwrap();
                let mut old = caster.state.active().unwrap();
                old.spell.interruptible = false;
                caster.state.cancel();
                assert!(caster.state.start(old.spell, old.target, 0.));
            }
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(
            app.world().resource::<Run>().abilities[0].cd,
            0.,
            "case {case}"
        );
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
    }
}

#[test]
fn counterspell_silence_can_stop_uninterruptible_cast_without_granting_school_lock() {
    let (mut app, _, enemy) = counterspell_setup(2);
    {
        let mut caster = app
            .world_mut()
            .get_mut::<enemy_casting::Caster>(enemy)
            .unwrap();
        let mut old = caster.state.active().unwrap();
        old.spell.interruptible = false;
        caster.state.cancel();
        assert!(caster.state.start(old.spell, old.target, 0.));
    }
    app.world_mut().run_system_once(cast_abilities).unwrap();
    let caster = app.world().get::<enemy_casting::Caster>(enemy).unwrap();
    assert!(caster.state.active().is_none());
    assert!(!caster
        .state
        .available(survivors_core::casting::School::Frost, 4.99));
    assert!(caster
        .state
        .available(survivors_core::casting::School::Frost, 5.));
}

#[test]
fn counterspell_parameters_keep_control_durations_fixed_and_suppress_damage_scaling() {
    let (app, _, _) = counterspell_setup(2);
    let run = app.world().resource::<Run>();
    let mut stats = run.stats();
    stats.targets = 2;
    stats.might = 10.;
    stats.crit = 1.;
    for level in 1..=data::MAX_ABILITY_LEVEL {
        let p = Params::new(
            &crate::survivors::Owned {
                def: data::ability("counterspell"),
                level,
                cd: 0.,
            },
            &stats,
            run.milestones,
        );
        assert_eq!(
            (p.dmg, p.dot, p.crit, p.duration, p.silence_chance),
            (0., 0., 0., 10., 1.)
        );
        assert_eq!(
            p.count,
            3 + [3, 5, 7].iter().filter(|&&rank| level >= rank).count()
        );
        assert!(p.cooldown <= 30.);
    }
}

fn silence_setup(trained: bool) -> (App, Entity, Entity) {
    let (mut app, hero, enemy) = counterspell_setup(0);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[4];
        run.milestones = survivors_core::talents::Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(if trained { &[(2, 541, 1)] } else { &[] });
        run.abilities[0].def = data::ability("silence");
    }
    (app, hero, enemy)
}

#[test]
fn priest_silence_cancels_even_interrupt_immune_cast_and_expires_without_school_lock() {
    use survivors_core::casting::School;
    for interruptible in [false, true] {
        let (mut app, _, enemy) = silence_setup(true);
        {
            let mut caster = app
                .world_mut()
                .get_mut::<enemy_casting::Caster>(enemy)
                .unwrap();
            let mut old = caster.state.active().unwrap();
            old.spell.interruptible = interruptible;
            caster.state.cancel();
            assert!(caster.state.start(old.spell, old.target, 0.));
        }
        let position = app.world().get::<Transform>(enemy).unwrap().translation;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let caster = app.world().get::<enemy_casting::Caster>(enemy).unwrap();
        assert!(caster.state.active().is_none());
        for school in [
            School::Holy,
            School::Fire,
            School::Nature,
            School::Frost,
            School::Shadow,
            School::Arcane,
        ] {
            assert!(!caster.state.available(school, 5.99));
            assert!(caster.state.available(school, 6.));
        }
        assert_eq!(app.world().resource::<Run>().abilities[0].cd, 45.);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        tick(&mut app, 3.);
        assert_eq!(app.world().resource::<Run>().hp, 1000.);
        assert!(
            app.world()
                .get::<Transform>(enemy)
                .unwrap()
                .translation
                .distance(Vec3::ZERO)
                < position.length(),
            "silence must permit pursuit"
        );
        tick(&mut app, 5.99);
        assert!(app
            .world()
            .get::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .active()
            .is_none());
        tick(&mut app, 6.);
        assert!(app
            .world()
            .get::<enemy_casting::Caster>(enemy)
            .unwrap()
            .state
            .active()
            .is_some());
    }
}

#[test]
fn priest_silence_waits_for_training_live_targets_and_clear_casts() {
    use avian3d::prelude::*;
    for case in 0..9 {
        let (mut app, _, enemy) = silence_setup(case != 0);
        match case {
            0 => {}
            1 => app
                .world_mut()
                .get_mut::<enemy_casting::Caster>(enemy)
                .unwrap()
                .state
                .cancel(),
            2 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().boss = true,
            3 => app.world_mut().get_mut::<Enemy>(enemy).unwrap().hp = 0.,
            4 => {
                app.world_mut()
                    .get_mut::<Transform>(enemy)
                    .unwrap()
                    .translation = Vec3::X * 21.
            }
            5 => {
                app.world_mut()
                    .entity_mut(enemy)
                    .insert(crate::survivors::perception::Concealed);
            }
            6 => app.world_mut().resource_mut::<Run>().elapsed = 3.,
            7 => app.world_mut().resource_mut::<Run>().hp = 0.,
            _ => {
                app.world_mut().spawn((
                    RigidBody::Static,
                    Collider::cuboid(0.2, 5., 5.),
                    Transform::from_xyz(2., 1., 0.),
                ));
                app.update();
            }
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(
            app.world().resource::<Run>().abilities[0].cd,
            0.,
            "case {case}"
        );
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
    }
}

#[test]
fn priest_silence_rank_growth_never_extends_control_or_adds_damage() {
    let (app, _, _) = silence_setup(true);
    let run = app.world().resource::<Run>();
    let mut stats = run.stats();
    stats.targets = 2;
    stats.might = 10.;
    stats.crit = 1.;
    for level in 1..=data::MAX_ABILITY_LEVEL {
        let p = Params::new(
            &crate::survivors::Owned {
                def: data::ability("silence"),
                level,
                cd: 0.,
            },
            &stats,
            run.milestones,
        );
        assert_eq!(
            (p.dmg, p.dot, p.crit, p.duration, p.silence_chance),
            (0., 0., 0., 5., 1.)
        );
        assert_eq!(
            p.count,
            3 + [3, 5, 7].iter().filter(|&&rank| level >= rank).count()
        );
        assert!(p.cooldown <= 45.);
    }
}
