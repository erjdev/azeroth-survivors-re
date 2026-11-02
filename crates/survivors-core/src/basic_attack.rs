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
