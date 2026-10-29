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
        run.milestones
            .with_ranks(&[(2, 586, 5), (2, 593, 5), (2, 588, 5)]),
    );
    assert_eq!(base.dmg, 40.);
    assert_eq!(base.cooldown, 12.);
    assert_eq!(trained.dmg, base.dmg);
    assert_eq!(trained.cooldown, base.cooldown);
    let upgraded = Params::new(&Owned { level: 2, ..owned }, &stats, run.milestones);
    assert!((upgraded.dmg - 52.).abs() < 0.0001);
    assert!(upgraded.cooldown < base.cooldown);
}
