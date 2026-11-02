use super::*;
use crate::survivors::Owned;
use survivors_core::talents::Milestones;

#[test]
fn direct_heal_caps_health_preserves_periodic_sources_and_rejects_failed_casts() {
    for critical in [false, true] {
        let mut run = Run {
            hp: 10.,
            ..Default::default()
        };
        let mut stats = run.stats();
        stats.might = 1.;
        stats.crit = if critical { 1. } else { 0. };
        stats.crit_damage = 9.;
        let p = Params::new(
            &Owned {
                def: data::ability("healing_touch"),
                level: 1,
                cd: 0.,
            },
            &stats,
            Milestones::default(),
        );
        assert_eq!(p.crit_damage, 1.5);
        run.healing.refresh("regrowth", 24., 12.);
        let periodic = run.healing.clone();
        assert_eq!(
            cast_direct_heal(&mut run, &p, 100., &mut Rng(123)),
            Some((if critical { 60. } else { 40. }, critical))
        );
        assert_eq!(run.healing, periodic);
        run.hp = 80.;
        assert_eq!(
            cast_direct_heal(&mut run, &p, 100., &mut Rng(123)),
            Some((20., critical))
        );
        for (hp, dead) in [(85., None), (100., None), (0., None), (30., Some(0.))] {
            run.hp = hp;
            run.dead_t = dead;
            let mut rng = Rng(123);
            assert!(cast_direct_heal(&mut run, &p, 100., &mut rng).is_none());
            assert_eq!(rng.0, 123);
            assert_eq!(run.hp, hp);
            assert_eq!(run.healing, periodic);
        }
    }
}

#[test]
fn healing_touch_training_reaches_real_healing_and_recovery_but_not_regrowth_crit() {
    let mut run = Run {
        hp: 10.,
        ..Default::default()
    };
    let mut stats = run.stats();
    stats.might = 1.;
    stats.crit = 0.;
    let owned = Owned {
        def: data::ability("healing_touch"),
        level: 1,
        cd: 0.,
    };
    let base = Milestones {
        class: 8,
        ..Default::default()
    };
    let plain = Params::new(&owned, &stats, base);
    let trained = base.with_ranks(&[
        (2, 824, 5),
        (2, 843, 5),
        (2, 831, 1),
        (0, 783, 3),
        (2, 828, 5),
        (2, 823, 5),
        (2, 825, 5),
        (2, 830, 3),
    ]);
    let p = Params::new(&owned, &stats, trained);
    assert!((p.cooldown - plain.cooldown * 0.85 * 0.9 * 0.85 * 0.91).abs() < 0.00001);
    assert_eq!(p.crit, 0.);
    let (healed, critical) = cast_direct_heal(&mut run, &p, 100., &mut Rng(123)).unwrap();
    assert!(!critical);
    assert!((healed - 48.4).abs() < 0.00001);
    assert_eq!(
        Params::new(&owned, &stats, trained.with_ranks(&[])).dmg,
        plain.dmg
    );
    assert_eq!(
        data::CLASSES[8]
            .ability_keys()
            .iter()
            .filter(|&&k| k == "healing_touch")
            .count(),
        1
    );
}

#[test]
fn healing_wave_is_acquired_and_keeps_totem_only_training_separate() {
    let mut run = Run {
        class: &data::CLASSES[5],
        ..Default::default()
    };
    run.milestones.class = 5;
    assert_eq!(
        data::CLASSES[5]
            .ability_keys()
            .iter()
            .filter(|&&k| k == "healing_wave")
            .count(),
        1
    );
    let owned = Owned {
        def: data::ability("healing_wave"),
        level: 1,
        cd: 0.,
    };
    let mut stats = run.stats();
    stats.might = 1.;
    stats.crit = 0.;
    let base = Params::new(&owned, &stats, run.milestones);
    let trained = Params::new(
        &owned,
        &stats,
        run.milestones.with_ranks(&[(2, 595, 5), (2, 588, 5)]),
    );
    assert_eq!(base.dmg, 40.);
    assert_eq!(base.cooldown, 12.);
    assert_eq!(trained.dmg, base.dmg);
    assert_eq!(trained.cooldown, base.cooldown);
    let upgraded = Params::new(&Owned { level: 2, ..owned }, &stats, run.milestones);
    assert!((upgraded.dmg - 52.).abs() < 0.0001);
    assert!(upgraded.cooldown < base.cooldown);
}

#[test]
fn healing_wave_criticals_refresh_ancestral_armor_without_totem_or_overheal_triggers() {
    use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
    use bevy::ecs::system::RunSystemOnce;
    for rank in 0..=3 {
        for critical in [false, true] {
            let mut run = Run {
                class: &data::CLASSES[5],
                milestones: Milestones {
                    class: 5,
                    ..Default::default()
                }
                .with_ranks(&[(2, 581, rank)]),
                ..Default::default()
            };
            run.bonus.armor = 80.;
            run.bonus.crit = 0.5;
            run.hp = run.stats().max_hp;
            let base_armor = run.stats().armor;
            run.abilities.push(Owned {
                def: data::ability("healing_wave"),
                level: 1,
                cd: 0.,
            });
            let chance = Params::new(&run.abilities[0], &run.stats(), run.milestones).crit;
            let seed = (1..10000)
                .find(|seed| (Rng(*seed).f32() < chance) == critical)
                .unwrap();
            let mut app = App::new();
            let mut time = Time::<()>::default();
            time.advance_by(std::time::Duration::from_secs_f32(0.1));
            app.insert_resource(run)
                .insert_resource(time)
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
            assert_eq!(app.world().resource::<Run>().ancestral_healing_t, 0.);
            assert_eq!(app.world().resource::<Rng>().0, seed);
            for remaining in [0., 4.] {
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.hp = 1.;
                    run.abilities[0].cd = 0.;
                    run.ancestral_healing_t = remaining;
                }
                app.world_mut().resource_mut::<Rng>().0 = seed;
                app.world_mut().run_system_once(cast_abilities).unwrap();
                let run = app.world().resource::<Run>();
                assert!(run.hp > 1.);
                let active = critical && rank > 0;
                assert_eq!(
                    run.ancestral_healing_t,
                    if active { 15. } else { remaining }
                );
                let bonus = if rank > 0 && (active || remaining > 0.) {
                    [0., 0.08, 0.16, 0.25][usize::from(rank)]
                } else {
                    0.
                };
                assert!((run.stats().armor - base_armor * (1. + bonus)).abs() < 0.0001);
                assert!(app.world().resource::<Messages<Hit>>().is_empty());
            }
        }
    }
}
