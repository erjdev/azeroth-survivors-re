use super::*;
use crate::survivors::Owned;
use survivors_core::talents::Milestones;

#[test]
fn regrowth_direct_critical_does_not_multiply_periodic_healing_or_other_sources() {
    for critical in [false, true] {
        let mut run = Run {
            hp: 20.,
            ..Default::default()
        };
        let mut stats = run.stats();
        stats.might = 1.;
        stats.crit = if critical { 1. } else { 0. };
        stats.crit_damage = 9.; // Offensive critical damage never changes healing criticals.
        let p = Params::new(
            &Owned {
                def: data::ability("regrowth"),
                level: 1,
                cd: 0.,
            },
            &stats,
            Milestones::default(),
        );
        assert_eq!(p.crit_damage, 1.5);
        run.healing.refresh("rejuvenation", 30., 10.);
        let (healed, actual_critical) = cast_regrowth(&mut run, &p, 100., &mut Rng(123)).unwrap();
        assert_eq!(actual_critical, critical);
        assert_eq!(healed, if critical { 36. } else { 24. });
        assert_eq!(run.healing.advance(2.), 10.); // 6 Rejuvenation + 4 Regrowth.
        let before = run.healing.clone();
        run.hp = 85.;
        let mut rng = Rng(123);
        assert!(cast_regrowth(&mut run, &p, 100., &mut rng).is_none());
        assert_eq!(rng.0, 123, "failed casts do not consume a critical roll");
        assert_eq!(run.healing, before);
        run.hp = 0.;
        assert!(cast_regrowth(&mut run, &p, 100., &mut rng).is_none());
        assert_eq!(run.hp, 0.);
        assert_eq!(run.healing, before);
    }
}

#[test]
fn regrowth_training_reaches_cast_chance_and_refreshes_only_its_periodic_effect() {
    let mut run = Run {
        hp: 20.,
        ..Default::default()
    };
    let mut stats = run.stats();
    stats.might = 1.;
    stats.crit = 0.5;
    let trained = Milestones {
        class: 8,
        ..Default::default()
    }
    .with_ranks(&[(2, 825, 5)]);
    let p = Params::new(
        &Owned {
            def: data::ability("regrowth"),
            level: 1,
            cd: 0.,
        },
        &stats,
        trained,
    );
    assert_eq!(p.crit, 1.);
    assert_eq!(
        data::CLASSES[8]
            .ability_keys()
            .iter()
            .filter(|&&k| k == "regrowth")
            .count(),
        1
    );
    run.healing.refresh("rejuvenation", 30., 10.);
    cast_regrowth(&mut run, &p, 100., &mut Rng(123)).unwrap();
    run.healing.advance(2.);
    run.hp = 80.;
    assert_eq!(
        cast_regrowth(&mut run, &p, 100., &mut Rng(123)),
        Some((20., true))
    );
    assert_eq!(run.healing.advance(20.), 48.); // 24 remaining Rejuvenation + refreshed 24 Regrowth.
    assert_eq!(run.healing.advance(20.), 0.);
}
