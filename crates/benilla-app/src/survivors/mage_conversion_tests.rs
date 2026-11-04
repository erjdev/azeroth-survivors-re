//! Authored resource conversions must reach real cast payloads and recovery timers.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::Milestones;

const RANKS: &[(u8, u32, u8)] = &[(0, 85, 1), (0, 1142, 3), (0, 77, 5), (1, 23, 2)];

#[test]
fn mage_conversion_parameters_preserve_defenses_durations_and_other_classes() {
    let run = Run::default();
    let stats = run.stats();
    let base = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = base.with_ranks(RANKS);
    for def in data::ABILITIES {
        for level in [1, 8] {
            let owned = super::super::Owned {
                def,
                level,
                cd: 0.0,
            };
            let before = Params::new(&owned, &stats, base);
            let after = Params::new(&owned, &stats, trained);
            let mage = data::CLASSES[6].pool.contains(&def.key);
            let offense = mage && !matches!(def.key, "mana_shield" | "counterspell");
            let fire = matches!(
                def.key,
                "fireball" | "fire_blast" | "scorch" | "flamestrike" | "pyroblast" | "blast_wave"
            );
            let damage = if offense { 1.05 } else { 1.0 };
            let recovery = if mage {
                0.94 * 0.90 * if fire { 0.90 } else { 1.0 }
            } else {
                1.0
            };
            assert!(
                (after.dmg - before.dmg * damage).abs() < 0.001,
                "{}",
                def.key
            );
            assert!(
                (after.dot - before.dot * damage).abs() < 0.001,
                "{}",
                def.key
            );
            assert!(
                (after.cooldown - before.cooldown * recovery).abs() < 0.001,
                "{}",
                def.key
            );
            assert_eq!(
                (after.duration, after.radius, after.range, after.count),
                (before.duration, before.radius, before.range, before.count)
            );
        }
    }
}

#[test]
fn real_mage_casts_use_new_damage_and_recovery_without_changing_pulse_or_absorption() {
    for key in ["fireball", "pyroblast", "flamestrike", "mana_shield"] {
        let mut outcomes = Vec::new();
        for trained in [false, true] {
            let (mut app, _, _) = super::earthbind_tests::setup(true);
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.class = &data::CLASSES[6];
                run.milestones = Milestones {
                    class: 6,
                    ..Default::default()
                }
                .with_ranks(if trained { RANKS } else { &[] });
                run.bonus = Default::default();
                run.abilities = vec![super::super::Owned {
                    def: data::ability(key),
                    level: 1,
                    cd: 0.0,
                }];
            }
            app.insert_resource(Rng(0));
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let (cd, shield) = {
                let run = app.world().resource::<Run>();
                (run.abilities[0].cd, run.shield)
            };
            let hits = app.world().resource::<Messages<Hit>>();
            let direct: f32 = hits
                .iter_current_update_messages()
                .map(|hit| hit.damage)
                .sum();
            let dot: f32 = hits.iter_current_update_messages().map(|hit| hit.dot).sum();
            let area = app
                .world_mut()
                .query::<&GroundArea>()
                .iter(app.world())
                .map(|area| (area.damage, area.every, area.left))
                .next();
            outcomes.push((cd, shield, direct, dot, area));
        }
        let (before, after) = (&outcomes[0], &outcomes[1]);
        let recovery = 0.94 * 0.90 * if key == "mana_shield" { 1.0 } else { 0.90 };
        assert!((after.0 - before.0 * recovery).abs() < 0.001, "{key}");
        assert_eq!(before.1, after.1, "absorption changed for {key}");
        assert!((after.2 - before.2 * 1.05).abs() < 0.001, "direct {key}");
        assert!((after.3 - before.3 * 1.05).abs() < 0.001, "periodic {key}");
        if key == "mana_shield" {
            assert!(after.1 > 0.0);
        } else if key == "flamestrike" {
            let (old, new) = (before.4.unwrap(), after.4.unwrap());
            assert!(old.0 > 0.0);
            assert!((new.0 - old.0 * 1.05).abs() < 0.001);
            assert_eq!((old.1, old.2), (new.1, new.2));
        } else {
            assert!(after.2 > 0.0);
        }
    }
}
