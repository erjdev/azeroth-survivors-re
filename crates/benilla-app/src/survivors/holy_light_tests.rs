use super::*;
use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
use crate::survivors::Owned;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::Milestones;

#[test]
fn paladin_heal_cast_preserves_failed_readiness_and_applies_healing_crit_without_damage() {
    for key in ["holy_light", "flash_of_light"] {
        for rank in [0, 3] {
            for critical in [false, true] {
                let mut app = App::new();
                let mut time = Time::<()>::default();
                time.advance_by(std::time::Duration::from_secs_f32(0.1));
                let mut run = Run {
                    class: &data::CLASSES[1],
                    milestones: Milestones {
                        class: 1,
                        ..Default::default()
                    }
                    .with_ranks(&[(0, 1627, 5), (0, 1433, 1), (0, 1444, rank)]),
                    ..Default::default()
                };
                run.hp = run.stats().max_hp;
                run.abilities.push(Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.,
                });
                run.healing.refresh("regrowth", 24., 12.);
                let periodic = run.healing.clone();
                let chance = Params::new(&run.abilities[0], &run.stats(), run.milestones).crit;
                let seed = (1..100000)
                    .find(|seed| (Rng(*seed).f32() < chance) == critical)
                    .unwrap();
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
                let run = app.world().resource::<Run>();
                assert_eq!(run.critical_cast.bonus(run.milestones, key, 0.), 0.0);
                assert_eq!(run.power_infusion.remaining(run.milestones, 0.), 0.);
                assert_eq!(app.world().resource::<Rng>().0, seed);
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.hp = 1.;
                    run.abilities[0].cd = 0.;
                }
                let run = app.world().resource::<Run>();
                let p = Params::new(&run.abilities[0], &run.stats(), run.milestones);
                let base_heal = if key == "holy_light" { 40.0 } else { 18.0 };
                assert!((p.dmg - base_heal * (1.0 + 0.04 * f32::from(rank))).abs() < 0.0001);
                let expected =
                    (1. + p.dmg * if critical { 1.5 } else { 1. }).min(run.stats().max_hp);
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let run = app.world().resource::<Run>();
                assert!((run.hp - expected).abs() < 0.0001);
                assert_eq!(run.inspiration_t, 0.);
                assert_eq!(run.critical_cast.bonus(run.milestones, key, 0.), 0.);
                assert_eq!(run.power_infusion.remaining(run.milestones, 0.), 0.);
                assert_eq!(run.healing, periodic);
                assert_eq!(run.abilities[0].cd, p.cooldown);
                assert_eq!(
                    run.critical_cast.bonus(run.milestones, "holy_shock", 0.),
                    1.0,
                    "Holy Light must preserve the existing Holy Shock-only Divine Favor conversion"
                );
                assert!(app.world().resource::<Messages<Hit>>().is_empty());
            }
        }
    }
}

#[test]
fn healing_wave_cast_heals_without_a_target_and_never_deals_damage() {
    for rank in 0..=5 {
        for critical in [false, true] {
            let mut app = App::new();
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            let mut run = Run {
                class: &data::CLASSES[5],
                ..Default::default()
            };
            run.milestones.class = 5;
            run.milestones = run.milestones.with_ranks(&[
                (2, 592, rank),
                (2, 594, rank),
                (2, 586, rank),
                (2, 593, rank),
            ]);
            run.hp = run.stats().max_hp;
            run.abilities.push(Owned {
                def: data::ability("healing_wave"),
                level: 1,
                cd: 0.,
            });
            let p = Params::new(&run.abilities[0], &run.stats(), run.milestones);
            let base = Params::new(
                &run.abilities[0],
                &run.stats(),
                run.milestones.with_ranks(&[]),
            );
            assert!((p.dmg - 40. * (1. + 0.02 * f32::from(rank))).abs() < 0.0001);
            assert!(
                (p.cooldown
                    - base.cooldown
                        * (1. - 0.04 * f32::from(rank))
                        * (1. - 0.02 * f32::from(rank)))
                .abs()
                    < 0.0001
            );
            assert!((p.crit - base.crit - 0.01 * f32::from(rank)).abs() < 0.0001);
            let seed = (1..100000)
                .find(|seed| {
                    let roll = Rng(*seed).f32();
                    (roll < p.crit) == critical && (!critical || rank == 0 || roll >= base.crit)
                })
                .unwrap();
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
            assert_eq!(app.world().resource::<Rng>().0, seed);
            assert!(app.world().resource::<Messages<CastEvent>>().is_empty());
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.hp = 1.;
                run.abilities[0].cd = 0.;
            }
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let run = app.world().resource::<Run>();
            assert_eq!(run.hp, 1. + p.dmg * if critical { 1.5 } else { 1. });
            assert_eq!(run.abilities[0].cd, p.cooldown);
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            assert!(!app.world().resource::<Messages<CastEvent>>().is_empty());
            assert_eq!(run.inspiration_t, 0.);
        }
    }
}

#[test]
fn healing_way_boosts_only_subsequent_effective_wave_casts() {
    let mut app = App::new();
    let mut time = Time::<()>::default();
    time.advance_by(std::time::Duration::from_secs_f32(0.1));
    let mut run = Run {
        class: &data::CLASSES[5],
        ..Default::default()
    };
    run.milestones = Milestones {
        class: 5,
        ..Default::default()
    }
    .with_ranks(&[(2, 1648, 3)]);
    run.abilities.push(Owned {
        def: data::ability("healing_wave"),
        level: 1,
        cd: 0.,
    });
    let p = Params::new(&run.abilities[0], &run.stats(), run.milestones);
    let seed = (1..100000).find(|seed| Rng(*seed).f32() >= p.crit).unwrap();
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
    for (elapsed, bonus) in [
        (0., 0.),
        (1., 0.06),
        (2., 0.12),
        (3., 0.18),
        (4., 0.18),
        (19., 0.),
    ] {
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.hp = 1.;
            run.elapsed = elapsed;
            run.abilities[0].cd = 0.;
        }
        app.world_mut().resource_mut::<Rng>().0 = seed;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert!((run.hp - (1. + p.dmg * (1. + bonus))).abs() < 0.0001);
        assert_eq!(run.abilities[0].cd, p.cooldown);
    }
    let before = app.world().resource::<Run>().healing_way;
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.hp = run.stats().max_hp;
        run.elapsed = 20.;
        run.abilities[0].cd = 0.;
    }
    app.world_mut().resource_mut::<Rng>().0 = seed;
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert_eq!(app.world().resource::<Run>().healing_way, before);
    assert_eq!(app.world().resource::<Rng>().0, seed);
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
}

#[test]
fn illumination_refunds_half_the_actual_recovery_only_after_eligible_critical_healing() {
    for key in ["holy_light", "flash_of_light"] {
        for rank in 0..=5 {
            for critical in [false, true] {
                for refund in [false, true] {
                    if refund && (!critical || rank == 0) || !refund && critical && rank == 5 {
                        continue;
                    }
                    let mut app = App::new();
                    let mut time = Time::<()>::default();
                    time.advance_by(std::time::Duration::from_secs_f32(0.1));
                    let mut run = Run {
                        class: &data::CLASSES[1],
                        ..Default::default()
                    };
                    run.milestones = Milestones {
                        class: 1,
                        ..Default::default()
                    }
                    .with_ranks(&[(0, 1461, rank)]);
                    run.hp = run.stats().max_hp;
                    run.abilities.push(Owned {
                        def: data::ability(key),
                        level: 4,
                        cd: 0.,
                    });
                    let p = Params::new(&run.abilities[0], &run.stats(), run.milestones);
                    let chance = run.milestones.illumination_chance(key, 1., critical);
                    let seed = (1..100000)
                        .find(|seed| {
                            let mut rng = Rng(*seed);
                            let crit = rng.f32() < p.crit;
                            let proc = chance > 0. && rng.f32() < chance;
                            crit == critical && proc == refund
                        })
                        .unwrap();
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
                        app.world().resource::<Rng>().0,
                        seed,
                        "full-health cast spends no RNG"
                    );
                    {
                        let mut run = app.world_mut().resource_mut::<Run>();
                        run.hp = 1.;
                        run.abilities[0].cd = 0.;
                    }
                    app.world_mut().run_system_once(cast_abilities).unwrap();
                    let run = app.world().resource::<Run>();
                    let expected_heal =
                        (1. + p.dmg * if critical { 1.5 } else { 1. }).min(run.stats().max_hp);
                    assert!((run.hp - expected_heal).abs() < 0.0001);
                    assert_eq!(
                        run.abilities[0].cd,
                        p.cooldown * if refund { 0.5 } else { 1. },
                        "{key} rank {rank} critical {critical} refund {refund}"
                    );
                    assert!(app.world().resource::<Messages<Hit>>().is_empty());
                    assert_eq!(
                        Params::new(&run.abilities[0], &run.stats(), run.milestones).cooldown,
                        p.cooldown,
                        "refund does not permanently change the spell"
                    );
                }
            }
        }
    }
}
