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
        for (key, required) in [
            ("rake", DruidForm::Cat),
            ("swipe", DruidForm::Bear),
            ("shred", DruidForm::Cat),
            ("ferocious_bite", DruidForm::Cat),
        ] {
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
    for (form, key) in [
        (DruidForm::Cat, "rake"),
        (DruidForm::Bear, "swipe"),
        (DruidForm::Cat, "shred"),
        (DruidForm::Cat, "ferocious_bite"),
    ] {
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
                    * if matches!(key, "shred" | "ferocious_bite") {
                        1.0
                    } else {
                        1.2
                    }
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
            let form = if matches!(key, "swipe" | "bash" | "demoralizing_roar") {
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
                "rejuvenation" | "regrowth" | "healing_touch" | "tranquility" => 0.9 * 0.94,
                "rake" | "swipe" | "shred" | "ferocious_bite" | "bash" | "pounce"
                | "demoralizing_roar" => 1.0,
                _ => 0.94,
            };
            let expected = if key == "tranquility" {
                (base_cooldown * factor).max(10.0)
            } else {
                base_cooldown * factor
            };
            assert!(
                (app.world().resource::<Run>().abilities[0].cd - expected).abs() < 0.001,
                "{key}/{level}"
            );
        }
    }
}

#[test]
fn permanent_form_recovery_reaches_independent_spell_and_automatic_timers() {
    for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
        for key in [
            "wrath",
            "rejuvenation",
            "rake",
            "swipe",
            "shred",
            "ferocious_bite",
        ] {
            let (mut app, _) = setup(form, key);
            if !app
                .world()
                .resource::<Run>()
                .milestones
                .ability_unlocked(key)
            {
                continue;
            }
            let baseline;
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.renown_celerity = 10;
                run.bonus.cooldown = 0.2;
                baseline = Params::new(&run.abilities[0], &run.stats(), run.milestones).cooldown;
                let mut ranks = vec![(0, 781, 3), (1, 799, 5), (2, 822, 5)];
                if let Some(f) = form {
                    ranks.push((f.tree(), f.talent(), 1));
                }
                run.milestones = run.milestones.with_ranks(&ranks);
                run.weapon_attack_cd = 0.0;
            }
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let run = app.world().resource::<Run>();
            let mut factor = if form.is_some() { 0.94 } else { 1.0 };
            if matches!(form, Some(DruidForm::Cat | DruidForm::Bear))
                && matches!(key, "rake" | "swipe" | "shred" | "ferocious_bite")
            {
                factor *= 0.85 * 0.9;
            }
            assert!(
                (run.abilities[0].cd - baseline * factor).abs() < 0.001,
                "{form:?}/{key}"
            );
            if let Some(f @ (DruidForm::Cat | DruidForm::Bear)) = form {
                let recovery = if f == DruidForm::Cat { 1.0 } else { 2.5 };
                assert!(
                    (run.weapon_attack_cd
                        - recovery * 0.94 * 0.85 * 0.9 / (1.0 + run.stats().haste).max(0.1))
                    .abs()
                        < 0.001
                );
            } else {
                assert_eq!(run.weapon_attack_cd, 0.0);
            }
        }
    }
}

#[test]
fn shred_has_no_positional_requirement_and_uses_range_and_recovery_bonuses() {
    for (x, area, should_hit) in [
        (3., 0., true),
        (-3., 0., true),
        (8., 0., false),
        (8., 1., true),
    ] {
        let (mut app, target) = setup(Some(DruidForm::Cat), "shred");
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * x;
        let expected;
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.bonus.area = area;
            run.milestones = run.milestones.with_ranks(&[(1, 1_000_001, 1), (1, 802, 2)]);
            expected =
                super::super::sheet::cooldown_time(5., run.stats().haste, run.stats().cooldown)
                    * 0.8;
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let hits = app.world().resource::<Messages<Hit>>();
        assert_eq!(hits.len(), usize::from(should_hit));
        if should_hit {
            let hit = hits.iter_current_update_messages().next().unwrap();
            assert_eq!(hit.source, "shred");
            assert_eq!(hit.color, COLOR_MELEE);
            assert_eq!(hit.dot, 0.);
            assert!((app.world().resource::<Run>().abilities[0].cd - expected).abs() < 0.001);
        }
    }
}

#[test]
fn bash_and_pounce_settle_control_without_direct_damage_and_keep_bleed_separate() {
    for (key, required) in [("bash", DruidForm::Bear), ("pounce", DruidForm::Cat)] {
        for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
            for boss in [false, true] {
                let (mut app, target) = setup(form, key);
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    let mut ranks: Vec<_> = form
                        .into_iter()
                        .map(|f| (f.tree(), f.talent(), 1))
                        .collect();
                    ranks.push((1, 797, 2));
                    run.milestones = run.milestones.with_ranks(&ranks);
                    run.abilities[0].level = 3;
                }
                app.world_mut().get_mut::<Enemy>(target).unwrap().boss = boss;
                let before = app.world().get::<Enemy>(target).unwrap().hp;
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
                assert_eq!(hits[0].damage, 0.);
                assert_eq!(hits[0].effect, Effect::Stun(3.4));
                let dot = hits[0].dot;
                let duration = hits[0].dot_dur;
                app.world_mut().run_system_once(apply_hits).unwrap();
                let enemy = app.world().get::<Enemy>(target).unwrap();
                assert_eq!(enemy.hp, before);
                assert!((enemy.stun_t - 3.4 * if boss { 0.3 } else { 1. }).abs() < 0.001);
                if key == "pounce" {
                    assert!(dot > 0.);
                    assert_eq!(enemy.dots.len(), 1);
                    assert_eq!(enemy.dots[0].dps, dot);
                    // Settlement also advances periodic effects by this fixture frame.
                    assert!((enemy.dots[0].left - (duration - 0.1)).abs() < 0.001);
                    assert!((duration - 21.6).abs() < 0.001);
                } else {
                    assert!(enemy.dots.is_empty());
                }
            }
        }
    }
}

#[test]
fn bite_needs_no_setup_points_or_facing_and_area_extends_melee_reach() {
    for (x, area, should_hit) in [
        (3., 0., true),
        (-3., 0., true),
        (8., 0., false),
        (8., 1., true),
    ] {
        let (mut app, target) = setup(Some(DruidForm::Cat), "ferocious_bite");
        let automatic_after_tick = 100. - app.world().resource::<Time>().delta_secs();
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * x;
        app.world_mut().resource_mut::<Run>().bonus.area = area;
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let hits = app.world().resource::<Messages<Hit>>();
        assert_eq!(hits.len(), usize::from(should_hit));
        if should_hit {
            let hit = hits.iter_current_update_messages().next().unwrap();
            assert_eq!(hit.source, "ferocious_bite");
            assert_eq!(hit.color, COLOR_MELEE);
            assert_eq!(hit.dot, 0.);
            assert!(hit.damage > 0.);
            assert_eq!(
                app.world().resource::<Run>().weapon_attack_cd,
                automatic_after_tick
            );
        }
    }
}

#[test]
fn roar_is_bear_only_and_settles_without_damage_or_replacing_other_debuffs() {
    for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
        for boss in [false, true] {
            let (mut app, target) = setup(form, "demoralizing_roar");
            let before;
            {
                let mut enemy = app.world_mut().get_mut::<Enemy>(target).unwrap();
                enemy.boss = boss;
                enemy.weaken = 0.5;
                enemy.weaken_t = 20.;
                before = enemy.hp;
            }
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let hits = app.world().resource::<Messages<Hit>>();
            assert_eq!(hits.len(), usize::from(form == Some(DruidForm::Bear)));
            if form != Some(DruidForm::Bear) {
                continue;
            }
            let hit = hits.iter_current_update_messages().next().unwrap();
            assert_eq!((hit.damage, hit.dot), (0., 0.));
            assert!(matches!(hit.effect, Effect::MeleeWeaken(n, t) if n == 0.2 && t == 10.));
            app.world_mut().run_system_once(apply_hits).unwrap();
            assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, before);
            assert_eq!(app.world().get::<Enemy>(target).unwrap().weaken, 0.5);
            assert_eq!(
                app.world()
                    .get::<crate::survivors::roar::Roar>(target)
                    .unwrap()
                    .0
                    .melee_multiplier(1.),
                0.8
            );
        }
    }
}

#[test]
fn roar_uses_one_area_and_waits_for_clear_line_of_sight() {
    use avian3d::prelude::{Collider, RigidBody};
    for (distance, area, wall, expected) in [
        (3., 0., false, 1),
        (15., 0., false, 0),
        (15., 1., false, 1),
        (3., 0., true, 0),
    ] {
        let (mut app, target) = setup(Some(DruidForm::Bear), "demoralizing_roar");
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * distance;
        if wall {
            app.world_mut().spawn((
                RigidBody::Static,
                Collider::cuboid(0.5, 6., 6.),
                Transform::from_xyz(1.5, 2., 0.),
            ));
            app.update();
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            app.insert_resource(time);
        }
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.bonus.area = area;
            run.bonus.targets = 20;
        }
        app.world_mut().run_system_once(cast_abilities).unwrap();
        assert_eq!(
            app.world().resource::<Messages<Hit>>().len(),
            expected,
            "{distance}/{area}/{wall}"
        );
        if expected == 0 {
            // Shared empty-target retry, rather than spending the full Roar recovery.
            assert_eq!(app.world().resource::<Run>().abilities[0].cd, 0.2);
        }
    }
}

#[test]
fn feral_aggression_reaches_bite_hits_and_roar_settlement_at_every_rank() {
    for rank in 0..=5 {
        for (form, key) in [
            (DruidForm::Cat, "ferocious_bite"),
            (DruidForm::Bear, "demoralizing_roar"),
        ] {
            let mut damages = Vec::new();
            for trained in [false, true] {
                let (mut app, target) = setup(Some(form), key);
                let mut ranks = vec![(form.tree(), form.talent(), 1)];
                if trained {
                    ranks.push((1, 795, rank));
                }
                app.world_mut().resource_mut::<Run>().milestones =
                    app.world().resource::<Run>().milestones.with_ranks(&ranks);
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let hits = app.world().resource::<Messages<Hit>>();
                assert_eq!(hits.len(), 1);
                damages.push(hits.iter_current_update_messages().next().unwrap().damage);
                app.world_mut().run_system_once(apply_hits).unwrap();
                if key == "demoralizing_roar" {
                    let expected =
                        1. - 0.2 * (1. + if trained { f32::from(rank) * 0.08 } else { 0. });
                    let actual = app
                        .world()
                        .get::<crate::survivors::roar::Roar>(target)
                        .unwrap()
                        .0
                        .melee_multiplier(1.);
                    assert!((actual - expected).abs() < 0.00001);
                }
            }
            if key == "ferocious_bite" {
                assert!((damages[1] - damages[0] * (1. + f32::from(rank) * 0.03)).abs() < 0.001);
            } else {
                assert_eq!(damages, vec![0., 0.]);
            }
        }
    }
}
