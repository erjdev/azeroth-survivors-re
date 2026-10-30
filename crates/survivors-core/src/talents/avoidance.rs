//! Authored talent avoidance is separate from armor and flat damage reduction.
use super::{Milestones, TalentStat};

impl Milestones {
    pub fn melee_parry(self) -> f32 {
        // Shaman learns weapon-dependent Parry, unlike the other classes' Deflection bonuses.
        if self.class == 5 && self.weapon.is_none() {
            return 0.;
        }
        self.trained_value(TalentStat::MeleeParry)
    }

    pub fn melee_dodge(self) -> f32 {
        self.trained_value(TalentStat::MeleeDodge)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_rank_and_refund_is_class_scoped_without_armor_or_flat_reduction() {
        for (owner, tree, node, dodge) in [
            (0, 0, 130, false),
            (1, 2, 1403, false),
            (2, 2, 1311, false),
            (3, 1, 187, false),
            (3, 1, 186, true),
        ] {
            for class in 0..9 {
                for rank in [0, 1, 2, 3, 4, 5, 255] {
                    let ranks = [(tree, node, rank)];
                    let trained = Milestones {
                        class,
                        ..Default::default()
                    }
                    .with_ranks(&ranks);
                    let expected = if class == owner {
                        f32::from(rank.min(5)) * 0.01
                    } else {
                        0.
                    };
                    assert!(
                        (trained.melee_parry() - if dodge { 0. } else { expected }).abs() < 0.00001
                    );
                    assert!(
                        (trained.melee_dodge() - if dodge { expected } else { 0. }).abs() < 0.00001
                    );
                    let bonus = super::super::stat_adjustment(class, &ranks);
                    assert_eq!((bonus.armor, bonus.reduction), (0., 0.));
                    let reset = trained.with_ranks(&[]);
                    assert_eq!((reset.melee_dodge(), reset.melee_parry()), (0., 0.));
                }
            }
            let def = super::super::talent_definition(owner, node).unwrap();
            let text = def.inline_description(4).unwrap();
            assert!(text.contains("4 » 5 percentage point"), "{text}");
            assert!(text.contains(if dodge { "dodge" } else { "parry" }));
        }
    }
}

#[cfg(test)]
mod shaman_tests {
    use super::*;
    use crate::gear_loadout::{MeleeKind, MeleeWeapon};

    #[test]
    fn shaman_parry_requires_training_and_worn_melee_and_refunds_remove_it() {
        for class in 0..9 {
            for rank in [0, 1, 2, 255] {
                for kind in [
                    None,
                    Some(MeleeKind::Axe),
                    Some(MeleeKind::Mace),
                    Some(MeleeKind::Sword),
                    Some(MeleeKind::Polearm),
                    Some(MeleeKind::Staff),
                    Some(MeleeKind::Fist),
                    Some(MeleeKind::Dagger),
                ] {
                    let t = Milestones {
                        class,
                        weapon: kind.map(|kind| MeleeWeapon {
                            kind,
                            two_handed: false,
                        }),
                        ..Default::default()
                    }
                    .with_ranks(&[(1, 616, rank)]);
                    assert_eq!(
                        t.melee_parry(),
                        if class == 5 && rank > 0 && kind.is_some() {
                            0.05
                        } else {
                            0.
                        }
                    );
                    assert_eq!(t.melee_dodge(), 0.);
                    assert_eq!(t.with_ranks(&[]).melee_parry(), 0.);
                    assert_eq!(
                        crate::talents::stat_adjustment(class, &[(1, 616, rank)]).reduction,
                        0.
                    );
                }
            }
        }
        let node = crate::talents::talent_definition(5, 616).unwrap();
        let (text, ranges) = node.inline_description_with_values(0).unwrap();
        assert!(ranges
            .iter()
            .any(|r| &text[r.clone()] == "+0 » 5 percentage points melee parry chance"));
    }
}
