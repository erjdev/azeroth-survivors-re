//! Incoming contact resolution. Zero health loss alone is never an avoidance event.
use bevy::prelude::Entity;

use super::Run;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Outcome {
    Invalid,
    Inactive,
    Immune,
    Dodged,
    Parried,
    /// A connected attack may be wholly absorbed; that is still not a dodge or parry.
    Landed {
        absorbed: f32,
        health_removed: f32,
    },
}

impl Outcome {
    pub(super) fn health_removed(self) -> f32 {
        match self {
            Self::Landed { health_removed, .. } => health_removed,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Strike {
    pub attacker: Entity,
    pub outcome: Outcome,
}

pub(super) fn contact(
    run: &mut Run,
    attacker: Entity,
    raw: f32,
    roll: impl FnOnce() -> f32,
) -> Strike {
    if raw.is_finite() && raw > 0.0 && run.alive() && run.immune_t <= 0.0 {
        let aspect_dodge = if run.class.id == 3 {
            run.abilities
                .iter()
                .find(|o| o.def.kind == super::data::Kind::Aspect)
                .map_or(0.0, |owned| {
                    super::abilities::Params::new(owned, &run.stats(), run.milestones).dodge_chance
                })
        } else {
            0.
        };
        let deterrence = run.milestones.deterrence_avoidance(run.elapsed);
        let parry = deterrence + run.milestones.melee_parry();
        let dodge = (aspect_dodge + deterrence + run.milestones.melee_dodge()).clamp(0.0, 1.0);
        if dodge + parry > 0.0 {
            let roll = roll();
            if roll.is_finite() && roll >= 0.0 && roll < (dodge + parry).min(1.0) {
                return Strike {
                    attacker,
                    outcome: if roll < dodge {
                        Outcome::Dodged
                    } else {
                        Outcome::Parried
                    },
                };
            }
        }
    }
    Strike {
        attacker,
        outcome: hurt(run, raw),
    }
}

/// Survivors consumes the parry immediately; cooldown blocks further responses, not avoidance.
pub(super) fn counterattack(run: &mut Run, strike: Strike) -> Option<super::abilities::Hit> {
    let base = run.milestones.counterattack_damage();
    let now = run.elapsed;
    if strike.outcome != Outcome::Parried
        || run.class.id != 3
        || !run.alive()
        || base <= 0.0
        || !now.is_finite()
        || now < 0.0
        || !run.counterattack_ready_at.is_finite()
        || now < run.counterattack_ready_at
    {
        return None;
    }
    let damage = base * run.stats().might;
    if !damage.is_finite() || damage <= 0.0 || now + 5.0 <= now {
        return None;
    }
    run.counterattack_ready_at = now + 5.0;
    Some(super::abilities::Hit {
        occurred: Some(now),
        source: "Counterattack",
        effect: super::data::Effect::Root(5.0),
        ..super::abilities::Hit::direct(strike.attacker, damage, false)
    })
}

pub(super) fn hurt(run: &mut Run, raw: f32) -> Outcome {
    if !raw.is_finite() || raw <= 0.0 {
        return Outcome::Invalid;
    }
    if !run.alive() {
        return Outcome::Inactive;
    }
    if run.immune_t > 0.0 {
        return Outcome::Immune;
    }
    let stats = run.stats();
    let retaliation =
        run.thorns_t > 0.0 && run.thorns_spell == super::data::ability("retaliation").spell;
    let raw = raw * run.milestones.retaliation_risk(retaliation);
    let mut damage = run.milestones.after_flat_absorption(super::sheet::mitigate(
        raw,
        stats.armor,
        stats.reduction,
    ));
    let absorbed = if run.shield_t > 0.0 && run.shield > 0.0 {
        let absorbed = damage.min(run.shield);
        run.shield -= absorbed;
        damage -= absorbed;
        absorbed
    } else {
        0.0
    };
    damage = run.milestones.incoming(damage, run.hp, stats.max_hp);
    let health_removed = damage.min(run.hp);
    run.hp -= health_removed;
    if run.alive() {
        run.blessed_recovery.hit(run.milestones, health_removed);
    }
    if health_removed > 0.0 && run.hp > 0.0 && run.milestones.enrage_damage(true) > 1.0 {
        run.enrage_t = 4.0;
    }
    if health_removed > 0.0 && run.hp > 0.0 && run.milestones.blood_craze_fraction() > 0.0 {
        run.blood_craze_t = 6.0;
    }
    Outcome::Landed {
        absorbed,
        health_removed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunter() -> Run {
        Run {
            class: &super::super::data::CLASSES[2],
            milestones: survivors_core::talents::Milestones {
                class: 2,
                ..Default::default()
            }
            .with_ranks(&[(2, 1308, 1), (2, 1312, 1)]),
            hp: 100.,
            shield: 10.,
            shield_t: 10.,
            ..Default::default()
        }
    }

    #[test]
    fn trained_class_avoidance_prevents_contact_before_shields_and_retains_attacker() {
        let attacker = Entity::from_bits(42);
        for (class, tree, node, outcome) in [
            (0, 0, 130, Outcome::Parried),
            (1, 2, 1403, Outcome::Parried),
            (3, 1, 187, Outcome::Parried),
            (3, 1, 186, Outcome::Dodged),
        ] {
            for rank in 1..=5 {
                let setup = || Run {
                    class: &super::super::data::CLASSES[class],
                    milestones: survivors_core::talents::Milestones {
                        class,
                        ..Default::default()
                    }
                    .with_ranks(&[(tree, node, rank)]),
                    hp: 100.,
                    shield: 10.,
                    shield_t: 10.,
                    ..Default::default()
                };
                let mut run = setup();
                let strike = contact(&mut run, attacker, 20., || f32::from(rank) * 0.01 - 0.0001);
                assert_eq!(strike, Strike { attacker, outcome });
                assert_eq!((run.hp, run.shield), (100., 10.));
                assert!(
                    counterattack(&mut run, strike).is_none(),
                    "Hunter reaction must not leak across classes"
                );
                let mut run = setup();
                assert_eq!(
                    contact(&mut run, attacker, 20., || f32::from(rank) * 0.01 + 0.0001).outcome,
                    Outcome::Landed {
                        absorbed: 10.,
                        health_removed: 10.
                    }
                );
                let mut run = setup();
                assert_eq!(
                    hurt(&mut run, 20.),
                    Outcome::Landed {
                        absorbed: 10.,
                        health_removed: 10.
                    },
                    "non-contact damage does not roll avoidance"
                );
            }
        }
    }

    #[test]
    fn rogue_dodge_and_parry_are_exclusive_and_inactive_contacts_never_roll() {
        let attacker = Entity::from_bits(42);
        let setup = || Run {
            class: &super::super::data::CLASSES[3],
            milestones: survivors_core::talents::Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(1, 186, 5), (1, 187, 5)]),
            hp: 100.,
            shield: 10.,
            shield_t: 10.,
            ..Default::default()
        };
        for (roll, expected) in [
            (0.0499, Outcome::Dodged),
            (0.0501, Outcome::Parried),
            (0.0999, Outcome::Parried),
            (
                0.1001,
                Outcome::Landed {
                    absorbed: 10.,
                    health_removed: 10.,
                },
            ),
        ] {
            assert_eq!(
                contact(&mut setup(), attacker, 20., || roll).outcome,
                expected
            );
        }
        let mut run = setup();
        run.immune_t = 1.;
        assert_eq!(
            contact(&mut run, attacker, 20., || panic!("immune rolls")).outcome,
            Outcome::Immune
        );
        run.hp = 0.;
        assert_eq!(
            contact(&mut run, attacker, 20., || panic!("dead rolls")).outcome,
            Outcome::Inactive
        );
        assert_eq!(
            contact(&mut run, attacker, f32::NAN, || panic!("invalid rolls")).outcome,
            Outcome::Invalid
        );
        let mut run = setup();
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(matches!(
            contact(&mut run, attacker, 20., || panic!("refunded rolls")).outcome,
            Outcome::Landed { .. }
        ));
    }

    #[test]
    fn deterrence_uses_one_exclusive_attack_table_without_flat_damage_reduction() {
        let attacker = Entity::from_bits(42);
        for (roll, expected) in [
            (0., Outcome::Dodged),
            (0.2499, Outcome::Dodged),
            (0.25, Outcome::Parried),
            (0.4999, Outcome::Parried),
        ] {
            let mut run = hunter();
            assert_eq!(contact(&mut run, attacker, 20., || roll).outcome, expected);
            assert_eq!((run.hp, run.shield), (100., 10.));
        }
        for roll in [0.5, 0.9999, 1., -1., f32::NAN] {
            let mut run = hunter();
            assert_eq!(
                contact(&mut run, attacker, 20., || roll).outcome,
                Outcome::Landed {
                    absorbed: 10.,
                    health_removed: 10.
                }
            );
        }
        let mut run = hunter();
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("aspect_of_the_monkey"),
            level: 1,
            cd: 0.,
        });
        assert_eq!(
            contact(&mut run, attacker, 20., || 0.3299).outcome,
            Outcome::Dodged
        );
        assert_eq!(
            contact(&mut run, attacker, 20., || 0.33).outcome,
            Outcome::Parried
        );
        assert_eq!(
            contact(&mut run, attacker, 20., || 0.5799).outcome,
            Outcome::Parried
        );
        run.elapsed = 10.;
        assert!(matches!(
            contact(&mut run, attacker, 20., || 0.25).outcome,
            Outcome::Landed { .. }
        ));
        run.elapsed = 180.;
        assert_eq!(
            contact(&mut run, attacker, 20., || 0.33).outcome,
            Outcome::Parried
        );
        run.immune_t = 1.;
        assert_eq!(
            contact(&mut run, attacker, 20., || panic!("immune must not roll")).outcome,
            Outcome::Immune
        );
    }

    #[test]
    fn counterattack_requires_source_parry_and_training_and_consumes_only_active_run_cooldown() {
        let attacker = Entity::from_bits(42);
        let mut run = hunter();
        for outcome in [
            Outcome::Dodged,
            Outcome::Immune,
            Outcome::Invalid,
            Outcome::Inactive,
            Outcome::Landed {
                absorbed: 10.,
                health_removed: 0.,
            },
        ] {
            assert!(counterattack(&mut run, Strike { attacker, outcome }).is_none());
        }
        let strike = Strike {
            attacker,
            outcome: Outcome::Parried,
        };
        let hit = counterattack(&mut run, strike).unwrap();
        assert_eq!(hit.target, attacker);
        assert_eq!(hit.damage, 40. * run.stats().might);
        assert_eq!(hit.effect, super::super::data::Effect::Root(5.));
        assert_eq!(hit.source, "Counterattack");
        assert!(!hit.crit);
        assert_eq!(hit.splash, 0.);
        for now in [0., 0., 4.999, -1., f32::NAN, f32::INFINITY] {
            run.elapsed = now;
            assert!(counterattack(&mut run, strike).is_none());
        }
        run.elapsed = 5.;
        assert!(counterattack(&mut run, strike).is_some());
        run.elapsed = 10.;
        run.hp = 0.;
        assert!(counterattack(&mut run, strike).is_none());
        run.hp = 100.;
        run.milestones = run.milestones.with_ranks(&[(2, 1312, 1)]);
        assert!(counterattack(&mut run, strike).is_none());
        run.milestones = run.milestones.with_ranks(&[(2, 1308, 1)]);
        assert!(counterattack(&mut run, strike).is_none());
    }

    #[test]
    fn monkey_dodges_before_absorption_without_spending_invalid_or_immune_rolls() {
        let mut run = Run {
            class: &super::super::data::CLASSES[2],
            milestones: survivors_core::talents::Milestones {
                class: 2,
                ..Default::default()
            }
            .with_ranks(&[(0, 1381, 5)]),
            hp: 20.,
            shield: 10.,
            shield_t: 5.,
            ..Default::default()
        };
        let attacker = Entity::from_bits(42);
        assert_eq!(
            contact(&mut run, attacker, 1., || panic!("no aspect must not roll")).outcome,
            Outcome::Landed {
                absorbed: 1.,
                health_removed: 0.
            }
        );
        run.shield = 10.;
        run.abilities.push(super::super::Owned {
            def: super::super::data::ability("aspect_of_the_monkey"),
            level: 1,
            cd: 0.,
        });
        let chance = run.milestones.monkey_dodge(1, 0.);
        let strike = contact(&mut run, attacker, 100., || chance - 0.0001);
        assert_eq!(
            strike,
            Strike {
                attacker,
                outcome: Outcome::Dodged
            }
        );
        assert_eq!(
            (run.hp, run.shield, run.enrage_t, run.blood_craze_t),
            (20., 10., 0., 0.)
        );
        for invalid in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(
                contact(&mut run, attacker, invalid, || panic!(
                    "invalid input must not roll"
                ))
                .outcome,
                Outcome::Invalid
            );
        }
        run.immune_t = 1.;
        assert_eq!(
            contact(&mut run, attacker, 100., || panic!(
                "immunity must not roll"
            ))
            .outcome,
            Outcome::Immune
        );
        run.immune_t = 0.;
        assert_eq!(
            contact(&mut run, attacker, 100., || chance).outcome,
            Outcome::Landed {
                absorbed: 10.,
                health_removed: 20.
            }
        );
        assert_eq!(
            contact(&mut run, attacker, 100., || panic!(
                "dead hero must not roll"
            ))
            .outcome,
            Outcome::Inactive
        );
    }

    #[test]
    fn connected_absorbed_immune_and_inactive_hits_are_distinct_and_keep_the_attacker() {
        let attacker = Entity::from_bits(42);
        let mut run = Run {
            hp: 20.0,
            shield: 10.0,
            shield_t: 5.0,
            ..Default::default()
        };
        let result = contact(&mut run, attacker, 6.0, || 0.99);
        assert_eq!(result.attacker, attacker);
        assert_eq!(
            result.outcome,
            Outcome::Landed {
                absorbed: 6.0,
                health_removed: 0.0
            }
        );
        assert_eq!((run.hp, run.shield), (20.0, 4.0));
        run.immune_t = 1.0;
        assert_eq!(
            contact(&mut run, attacker, 6.0, || 0.99).outcome,
            Outcome::Immune
        );
        assert_eq!((run.hp, run.shield), (20.0, 4.0));
        run.immune_t = 0.0;
        assert_eq!(
            contact(&mut run, attacker, 6.0, || 0.99).outcome,
            Outcome::Landed {
                absorbed: 4.0,
                health_removed: 2.0
            }
        );
        assert_eq!((run.hp, run.shield), (18.0, 0.0));
        assert_eq!(
            contact(&mut run, attacker, 100.0, || 0.99).outcome,
            Outcome::Landed {
                absorbed: 0.0,
                health_removed: 18.0
            }
        );
        assert_eq!(run.hp, 0.0);
        assert_eq!(
            contact(&mut run, attacker, 100.0, || 0.99).outcome,
            Outcome::Inactive
        );
    }

    #[test]
    fn invalid_damage_cannot_heal_consume_a_shield_or_poison_health() {
        for raw in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut run = Run {
                hp: 20.0,
                shield: 10.0,
                shield_t: 5.0,
                ..Default::default()
            };
            assert_eq!(hurt(&mut run, raw), Outcome::Invalid);
            assert_eq!((run.hp, run.shield, run.shield_t), (20.0, 10.0, 5.0));
        }
    }

    #[test]
    fn expired_shields_and_lethal_hits_do_not_create_recovery() {
        let mut run = Run {
            hp: 5.0,
            shield: 10.0,
            shield_t: 0.0,
            ..Default::default()
        };
        assert_eq!(
            hurt(&mut run, 50.0),
            Outcome::Landed {
                absorbed: 0.0,
                health_removed: 5.0
            }
        );
        assert_eq!(
            (run.hp, run.shield, run.enrage_t, run.blood_craze_t),
            (0.0, 10.0, 0.0, 0.0)
        );
    }
}
