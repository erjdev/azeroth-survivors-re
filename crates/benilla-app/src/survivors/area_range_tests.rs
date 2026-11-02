//! Area scales reach as well as radius, without adding targets or changing recovery.
use super::*;
use bevy::ecs::system::RunSystemOnce;

#[test]
fn area_scales_every_owned_range_and_radius_without_changing_other_parameters() {
    let run = Run::default();
    let stats = run.stats();
    let mut enlarged = stats;
    enlarged.area *= 1.5;
    for def in data::ABILITIES {
        for level in [1, 8] {
            let owned = super::super::Owned { def, level, cd: 0. };
            let base = Params::new(&owned, &stats, run.milestones);
            let scaled = Params::new(&owned, &enlarged, run.milestones);
            assert!(
                (scaled.range - base.range * 1.5).abs() < 0.0001,
                "{}",
                def.key
            );
            assert!(
                (scaled.radius - base.radius * 1.5).abs() < 0.0001,
                "{}",
                def.key
            );
            assert_eq!(
                (scaled.count, scaled.dmg, scaled.cooldown),
                (base.count, base.dmg, base.cooldown),
                "{}",
                def.key
            );
        }
    }
}

#[test]
fn area_extends_actual_melee_cast_range_without_adding_splash_or_extra_victims() {
    for (area, distance, expected) in [(0., 7., false), (0.5, 7., true), (0.5, 10., false)] {
        let (mut app, target, _) = super::poison_tests::fixture("backstab", 1000.);
        app.world_mut()
            .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
        app.world_mut()
            .get_mut::<Transform>(target)
            .unwrap()
            .translation = Vec3::X * distance;
        app.world_mut().resource_mut::<Run>().bonus.area = area;
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f32(0.1));
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let hits: Vec<_> = app
            .world()
            .resource::<Messages<Hit>>()
            .iter_current_update_messages()
            .collect();
        assert_eq!(
            hits.len(),
            usize::from(expected),
            "area={area}, distance={distance}"
        );
        if let Some(hit) = hits.first() {
            assert_eq!(hit.target, target);
            assert_eq!(hit.splash, 0.);
        }
    }
}

#[test]
fn area_composes_with_flat_and_percent_range_talents_and_scoped_amulets() {
    for (class, node, key, expected) in [
        (2, 1352, "arcane_shot", (28. + 6.) * 1.5),
        (4, 1635, "smite", 20. * 1.2 * 1.5),
    ] {
        let run = Run {
            milestones: survivors_core::talents::Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(1, node, 3)]),
            ..Default::default()
        };
        let mut stats = run.stats();
        stats.area = 1.5;
        let owned = super::super::Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let plain = Params::new(&owned, &stats, run.milestones);
        assert!(
            (plain.range - expected).abs() < 0.0001,
            "{key}: {}",
            plain.range
        );
        stats.legendary = Some((
            key,
            super::super::campaign::Bonus {
                area: 0.25,
                ..Default::default()
            },
        ));
        let scoped = Params::new(&owned, &stats, run.milestones);
        assert!((scoped.range - expected / 1.5 * 1.75).abs() < 0.0001);
        stats.legendary.as_mut().unwrap().0 = "unrelated_spell";
        assert_eq!(
            Params::new(&owned, &stats, run.milestones).range,
            plain.range
        );
    }
}
