//! Cast and settlement coverage for permanent-form starting abilities.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::{forms::DruidForm, Milestones};

fn setup(form: Option<DruidForm>, key: &'static str) -> (App, Entity) {
    let (mut app, _, target) = super::earthbind_tests::setup(true);
    let ranks: Vec<_> = form
        .into_iter()
        .map(|f| (f.tree(), f.talent(), 1))
        .collect();
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[8];
        run.milestones = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&ranks);
        run.abilities = vec![super::super::Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.0,
        }];
        run.weapon_attack_cd = 100.0;
        run.bonus.targets = 0;
    }
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 3.0;
    (app, target)
}

#[test]
fn forms_gate_actual_casts_and_rake_settles_a_separate_noncritical_bleed() {
    for form in [
        None,
        Some(DruidForm::Cat),
        Some(DruidForm::Bear),
        Some(DruidForm::Moonkin),
        Some(DruidForm::TreeOfLife),
    ] {
        for (key, required) in [("rake", DruidForm::Cat), ("swipe", DruidForm::Bear)] {
            let (mut app, target) = setup(form, key);
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let hits: Vec<_> = app
                .world()
                .resource::<Messages<Hit>>()
                .iter_current_update_messages()
                .cloned()
                .collect();
            assert_eq!(hits.len(), usize::from(form == Some(required)));
            if hits.is_empty() {
                continue;
            }
            assert_eq!(hits[0].source, key);
            assert_eq!(hits[0].origin, HitOrigin::Hero);
            assert_eq!(hits[0].color, COLOR_MELEE);
            let expected_damage = hits[0].damage;
            let expected_dot = hits[0].dot;
            let before = app.world().get::<Enemy>(target).unwrap().hp;
            app.world_mut().run_system_once(apply_hits).unwrap();
            let enemy = app.world().get::<Enemy>(target).unwrap();
            assert!((enemy.hp - (before - expected_damage)).abs() < 0.001);
            if key == "rake" {
                assert_eq!(enemy.dots.len(), 1);
                assert_eq!(enemy.dots[0].source, "rake");
                assert_eq!(enemy.dots[0].origin, HitOrigin::Hero);
                assert!((enemy.dots[0].dps - expected_dot).abs() < 0.001);
            } else {
                assert!(enemy.dots.is_empty());
            }
        }
    }
}

#[test]
fn swipe_uses_area_radius_and_never_duplicates_hits_for_extra_targets() {
    for targets in [0, 10] {
        let (mut app, target) = setup(Some(DruidForm::Bear), "swipe");
        app.world_mut().resource_mut::<Run>().bonus.targets = targets;
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * 8.0;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.bonus.area = 1.0;
            // Isolate radius eligibility from the empty-target retry timer.
            run.abilities[0].cd = 0.0;
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(app.world().resource::<Messages<Hit>>().len(), 1);
    }
}

#[test]
fn ferocity_and_fury_affect_real_casts_bleeds_and_automatic_attacks() {
    for (form, key, source, damage, recovery) in [
        (DruidForm::Cat, "rake", "Claw", 12.0, 1.0),
        (DruidForm::Bear, "swipe", "Maul", 30.0, 2.5),
    ] {
        for rank in [0, 1, 2, 5, 255] {
            for automatic in [false, true] {
                let (mut app, target) = setup(Some(form), key);
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.milestones = run.milestones.with_ranks(&[
                        (form.tree(), form.talent(), 1),
                        (1, 796, rank),
                        (1, 805, rank),
                    ]);
                    run.renown_celerity = 10;
                    if automatic {
                        run.abilities.clear();
                        run.weapon_attack_cd = 0.0;
                    }
                }
                let stats = app.world().resource::<Run>().stats();
                let fury = 1.0 + 0.10 * f32::from(rank.min(2));
                let ferocity = 1.0 - 0.04 * f32::from(rank.min(5));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let hits: Vec<_> = app
                    .world()
                    .resource::<Messages<Hit>>()
                    .iter_current_update_messages()
                    .cloned()
                    .collect();
                assert_eq!(hits.len(), 1);
                let hit = &hits[0];
                assert_eq!(hit.source, if automatic { source } else { key });
                let base = if automatic {
                    damage
                } else {
                    data::ability(key).damage
                };
                let expected =
                    base * stats.might * fury * if hit.crit { stats.crit_damage } else { 1.0 };
                assert!((hit.damage - expected).abs() < 0.001);
                let run = app.world().resource::<Run>();
                let actual_cd = if automatic {
                    run.weapon_attack_cd
                } else {
                    run.abilities[0].cd
                };
                let expected_cd = if automatic {
                    recovery * ferocity / (1.0 + stats.haste)
                } else {
                    super::super::sheet::cooldown_time(
                        data::ability(key).cooldown,
                        stats.haste,
                        stats.cooldown,
                    ) * ferocity
                };
                assert!((actual_cd - expected_cd).abs() < 0.001);
                let before = app.world().get::<Enemy>(target).unwrap().hp;
                app.world_mut().run_system_once(apply_hits).unwrap();
                let enemy = app.world().get::<Enemy>(target).unwrap();
                assert!((before - enemy.hp - expected).abs() < 0.001);
                if !automatic && key == "rake" {
                    assert_eq!(enemy.dots.len(), 1);
                    assert!((enemy.dots[0].dps - 3.0 * stats.might * fury).abs() < 0.001);
                } else {
                    assert!(enemy.dots.is_empty());
                }
            }
        }
    }
}

#[test]
fn physical_training_scales_feral_casts_and_criticals_without_critical_bleeds() {
    for (form, key) in [(DruidForm::Cat, "rake"), (DruidForm::Bear, "swipe")] {
        for (claws, leader, bonus) in [(0, 0, 0.0), (3, 0, 0.06), (0, 1, 0.03), (3, 1, 0.09)] {
            for margin in [0.01, 0.04, 0.075, 0.10] {
                let (mut app, target) = setup(Some(form), key);
                let base_crit = app.world().resource::<Run>().stats().crit;
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.milestones = run.milestones.with_ranks(&[
                        (form.tree(), form.talent(), 1),
                        (0, 791, 5),
                        (1, 805, 2),
                        (1, 798, claws),
                        (1, 809, leader),
                    ]);
                }
                let stats = app.world().resource::<Run>().stats();
                assert_eq!(stats.crit, base_crit);
                let (seed, sample) = (1..100_000)
                    .find_map(|seed| {
                        let sample = Rng(seed).f32();
                        ((sample - base_crit - margin).abs() < 0.001).then_some((seed, sample))
                    })
                    .unwrap();
                app.world_mut().insert_resource(Rng(seed));
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let hits: Vec<_> = app
                    .world()
                    .resource::<Messages<Hit>>()
                    .iter_current_update_messages()
                    .cloned()
                    .collect();
                assert_eq!(hits.len(), 1);
                let hit = &hits[0];
                assert_eq!(hit.crit, sample < base_crit + bonus);
                let expected = data::ability(key).damage
                    * stats.might
                    * 1.1
                    * 1.2
                    * if hit.crit { stats.crit_damage } else { 1.0 };
                assert!((hit.damage - expected).abs() < 0.001);
                let before = app.world().get::<Enemy>(target).unwrap().hp;
                app.world_mut().run_system_once(apply_hits).unwrap();
                let enemy = app.world().get::<Enemy>(target).unwrap();
                assert!((before - enemy.hp - expected).abs() < 0.001);
                if key == "rake" {
                    assert_eq!(enemy.dots.len(), 1);
                    assert!((enemy.dots[0].dps - 3.0 * stats.might * 1.1 * 1.2).abs() < 0.001);
                } else {
                    assert!(enemy.dots.is_empty());
                }
            }
        }
    }
}

#[test]
fn restoration_recovery_conversions_reach_cast_timers_without_affecting_feral_attacks() {
    for &key in data::CLASSES[8].pool {
        for level in [1, 8] {
            let form = if key == "swipe" {
                DruidForm::Bear
            } else {
                DruidForm::Cat
            };
            let (mut app, _) = setup(Some(form), key);
            let base_cooldown;
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.abilities[0].level = level;
                run.renown_celerity = 10;
                run.bonus.cooldown = 0.2;
                base_cooldown =
                    Params::new(&run.abilities[0], &run.stats(), run.milestones).cooldown;
                run.milestones = run.milestones.with_ranks(&[
                    (form.tree(), form.talent(), 1),
                    (2, 841, 5),
                    (2, 829, 3),
                ]);
            }
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let factor = match key {
                "rejuvenation" | "regrowth" | "healing_touch" => 0.9 * 0.94,
                "rake" | "swipe" => 1.0,
                _ => 0.94,
            };
            assert!(
                (app.world().resource::<Run>().abilities[0].cd - base_cooldown * factor).abs()
                    < 0.001,
                "{key}/{level}"
            );
        }
    }
}
