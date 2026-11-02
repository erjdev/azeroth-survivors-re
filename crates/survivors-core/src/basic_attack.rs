//! Authored automatic attacks outside the six acquired ability slots.
use crate::talents::{forms::DruidForm, Milestones};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BasicAttack {
    pub source: &'static str,
    pub damage: f32,
    pub recovery: f32,
    pub reach: f32,
}

impl Milestones {
    /// Survivors tuning: 14 attack power adds one base DPS, using un-hasted cadence.
    pub fn basic_attack_at_level(self, level: u32) -> Option<BasicAttack> {
        self.basic_attack().map(|mut attack| {
            attack.damage += self.feral_attack_power(level) / 14. * attack.recovery;
            attack
        })
    }

    /// Uses the frozen run loadout/form. Cosmetic weapons never supply attack identity.
    pub fn basic_attack(self) -> Option<BasicAttack> {
        let (source, damage, recovery) = match self.class {
            1 => {
                let weapon = self.weapon?;
                if weapon.two_handed {
                    ("Weapon Attack", 36., 3.)
                } else {
                    ("Weapon Attack", 24., 2.)
                }
            }
            8 => match self.druid_form()? {
                DruidForm::Cat => ("Claw", 12., 1.),
                DruidForm::Bear => ("Maul", 30., 2.5),
                DruidForm::Moonkin | DruidForm::TreeOfLife => return None,
            },
            _ => return None,
        };
        Some(BasicAttack {
            source,
            damage,
            recovery,
            reach: 3.,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn predatory_strikes_scales_run_level_and_base_cadence_only_in_feral_forms() {
        for form in DruidForm::ALL {
            for rank in [0, 1, 2, 3, 255] {
                for level in [0, 1, 14, 60, 100] {
                    let trained = Milestones {
                        class: 8,
                        ..Default::default()
                    }
                    .with_ranks(&[(form.tree(), form.talent(), 1), (1, 803, rank)]);
                    let feral = matches!(form, DruidForm::Cat | DruidForm::Bear);
                    let power = if feral {
                        level as f32 * 0.5 * f32::from(rank.min(3))
                    } else {
                        0.
                    };
                    assert_eq!(trained.feral_attack_power(level), power);
                    if let Some(attack) = trained.basic_attack_at_level(level) {
                        assert!(
                            (attack.damage / attack.recovery - 12. - power / 14.).abs() < 0.00001
                        );
                    }
                    let untrained = trained.with_ranks(&[(form.tree(), form.talent(), 1)]);
                    for spell in ["wrath", "moonfire", "thorns", "rejuvenation", "regrowth"] {
                        assert_eq!(trained.cast(spell), untrained.cast(spell));
                    }
                    assert_eq!(untrained.feral_attack_power(level), 0.);
                }
            }
        }
        for class in 0..9 {
            let no_form = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(1, 803, 3)]);
            assert_eq!(no_form.feral_attack_power(60), 0.);
        }
        let definition = crate::talents::talent_definition(8, 803).unwrap();
        let (text, spans) = definition.inline_description_with_values(1).unwrap();
        assert!(text.contains("50 » 100%"), "{text}");
        assert!(
            text.contains("Each 14 attack power adds 1 base damage"),
            "{text}"
        );
        assert!(spans
            .iter()
            .any(|span| text[span.clone()].contains("50 » 100%")));
    }

    #[test]
    fn feral_attacks_require_one_matching_form_and_do_not_require_weapons() {
        let base = Milestones {
            class: 8,
            ..Default::default()
        };
        assert_eq!(base.basic_attack(), None);
        for form in DruidForm::ALL {
            let trained = base.with_ranks(&[(form.tree(), form.talent(), 1)]);
            let attack = trained.basic_attack();
            assert_eq!(
                attack.is_some(),
                matches!(form, DruidForm::Cat | DruidForm::Bear)
            );
            if let Some(attack) = attack {
                assert_eq!(attack.damage / attack.recovery, 12.);
                for rank in [0, 1, 5, 255] {
                    let trained =
                        trained.with_ranks(&[(form.tree(), form.talent(), 1), (0, 791, rank)]);
                    assert!(
                        (trained.cast(attack.source).damage - (1. + 0.02 * f32::from(rank.min(5))))
                            .abs()
                            < 0.00001
                    );
                    for spell in ["wrath", "moonfire", "thorns", "rejuvenation"] {
                        assert_eq!(trained.cast(spell), base.cast(spell));
                    }
                }
            }
        }
        let conflict = base.with_ranks(&[
            (1, DruidForm::Cat.talent(), 1),
            (1, DruidForm::Bear.talent(), 1),
        ]);
        assert_eq!(conflict.basic_attack(), None);
    }
}
