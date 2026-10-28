//! Incoming contact resolution. Zero health loss alone is never an avoidance event.
use bevy::prelude::Entity;

use super::Run;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Outcome {
    Invalid,
    Inactive,
    Immune,
    Dodged,
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
    if run.class.id == 3 && raw.is_finite() && raw > 0.0 && run.alive() && run.immune_t <= 0.0 {
        let chance = run
            .abilities
            .iter()
            .find(|o| o.def.kind == super::data::Kind::Aspect)
            .map_or(0.0, |owned| {
                super::abilities::Params::new(owned, &run.stats(), run.milestones).dodge_chance
            });
        if chance > 0.0 {
            let roll = roll();
            if roll.is_finite() && roll >= 0.0 && roll < chance {
                return Strike {
                    attacker,
                    outcome: Outcome::Dodged,
                };
            }
        }
    }
    Strike {
        attacker,
        outcome: hurt(run, raw),
    }
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
    let raw = raw
        * run.milestones.retaliation_risk(retaliation)
        * run.milestones.deterrence_multiplier(run.elapsed);
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
