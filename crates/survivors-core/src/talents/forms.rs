//! Permanent Survivors progression forms; IDs above the installed DBC range are authored nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DruidForm {
    Cat,
    Bear,
    Moonkin,
    TreeOfLife,
}

impl DruidForm {
    pub const ALL: [Self; 4] = [Self::Cat, Self::Bear, Self::Moonkin, Self::TreeOfLife];

    pub const fn talent(self) -> u32 {
        match self {
            Self::Cat => 1_000_001,
            Self::Bear => 1_000_002,
            Self::Moonkin => 793,
            Self::TreeOfLife => 1_000_003,
        }
    }

    pub const fn tree(self) -> u8 {
        match self {
            Self::Cat | Self::Bear => 1,
            Self::Moonkin => 0,
            Self::TreeOfLife => 2,
        }
    }

    pub fn from_talent(id: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|form| form.talent() == id)
    }

    /// Tauren retain their faction skins; other supported races use the Night Elf forms.
    pub fn display(self, race: u8) -> u32 {
        match (self, race == 6) {
            (Self::Cat, false) => 892,
            (Self::Cat, true) => 8571,
            (Self::Bear, false) => 2281,
            (Self::Bear, true) => 2289,
            (Self::Moonkin, false) => 15374,
            (Self::Moonkin, true) => 15375,
            (Self::TreeOfLife, _) => 864,
        }
    }
}

/// Imported conflicting choices are retained for repair, never silently resolved by order.
pub fn valid_forms(class: usize, ranks: &[(u8, u32, u8)]) -> bool {
    let mut selected = false;
    for &(tree, id, rank) in ranks {
        let Some(form) = DruidForm::from_talent(id) else {
            continue;
        };
        if rank == 0 {
            continue;
        }
        if class != 8 || selected || rank != 1 || tree != form.tree() {
            return false;
        }
        selected = true;
    }
    true
}

pub fn can_learn_form(class: usize, ranks: &[(u8, u32, u8)], id: u32) -> bool {
    let Some(form) = DruidForm::from_talent(id) else {
        return true;
    };
    class == 8
        && !ranks.iter().any(|&(_, other, rank)| {
            rank > 0 && other != form.talent() && DruidForm::from_talent(other).is_some()
        })
}

impl super::Milestones {
    /// Spell.dbc 17002/24866: Cat-only outdoor speed; caller supplies body environment.
    pub fn feline_movement_multiplier(self, indoors: bool) -> f32 {
        if !indoors && self.druid_form() == Some(DruidForm::Cat) {
            1. + self.trained_value(super::TalentStat::FelineSwiftness)
        } else {
            1.
        }
    }

    /// Spell.dbc 24867/24864: two/four points of dodge, also while indoors.
    pub fn feline_dodge(self) -> f32 {
        if self.druid_form() == Some(DruidForm::Cat) {
            self.trained_value(super::TalentStat::FelineSwiftness) * (2. / 15.)
        } else {
            0.
        }
    }

    /// Installed level-scaled attack power, scoped to the selected permanent feral form.
    pub fn feral_attack_power(self, level: u32) -> f32 {
        if matches!(self.druid_form(), Some(DruidForm::Cat | DruidForm::Bear)) {
            level as f32 * self.trained_value(super::TalentStat::PredatoryStrikes)
        } else {
            0.
        }
    }

    pub fn druid_form(self) -> Option<DruidForm> {
        if self.class != 8 {
            return None;
        }
        let mut selected = None;
        for (index, definition) in super::definitions(self.class).iter().enumerate() {
            if self.training[index] > 0 {
                if let Some(form) = DruidForm::from_talent(definition.id) {
                    if selected.is_some() {
                        return None;
                    }
                    selected = Some(form);
                }
            }
        }
        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn form_benefits_are_scoped_and_descriptions_highlight_the_upgrade() {
        use crate::talents::{stat_adjustment, talent_definition, Milestones};
        for form in DruidForm::ALL {
            let ranks = [(form.tree(), form.talent(), 1)];
            let trained = Milestones {
                class: 8,
                ..Default::default()
            }
            .with_ranks(&ranks);
            assert_eq!(trained.druid_form(), Some(form));
            assert_eq!(trained.with_ranks(&[]).druid_form(), None);
            assert!(
                (trained.health_multiplier() - if form == DruidForm::Bear { 1.25 } else { 1. })
                    .abs()
                    < 0.00001
            );
            assert!(
                (stat_adjustment(8, &ranks).speed - if form == DruidForm::Cat { 0.2 } else { 0. })
                    .abs()
                    < 0.00001
            );
            assert!(
                (trained.cast("wrath").crit - if form == DruidForm::Moonkin { 0.05 } else { 0. })
                    .abs()
                    < 0.00001
            );
            assert!(
                (trained.cast("regrowth").damage
                    - if form == DruidForm::TreeOfLife {
                        1.2
                    } else {
                        1.
                    })
                .abs()
                    < 0.00001
            );
            assert_eq!(trained.cast("fireball").crit, 0.);
            let (text, spans) = talent_definition(8, form.talent())
                .unwrap()
                .inline_description_with_values(0)
                .unwrap();
            assert!(
                spans.iter().any(|span| text[span.clone()].contains('»')),
                "{text}"
            );
        }
    }

    #[test]
    fn forms_reject_every_conflicting_pair_in_either_order_and_wrong_class_tree_rank() {
        for form in DruidForm::ALL {
            let ranks = [(form.tree(), form.talent(), 1)];
            assert!(valid_forms(8, &ranks));
            for class in 0..8 {
                assert!(!valid_forms(class, &ranks));
            }
            assert!(!valid_forms(8, &[(3, form.talent(), 1)]));
            assert!(!valid_forms(8, &[(form.tree(), form.talent(), 2)]));
            for other in DruidForm::ALL {
                if other != form {
                    assert!(!can_learn_form(8, &ranks, other.talent()));
                    assert!(!valid_forms(
                        8,
                        &[ranks[0], (other.tree(), other.talent(), 1)]
                    ));
                }
            }
        }
        assert!(valid_forms(8, &[]));
        assert!(can_learn_form(8, &[], DruidForm::Cat.talent()));
    }
}

#[cfg(test)]
mod feline_tests {
    use super::*;
    use crate::talents::{stat_adjustment, talent_definition, Milestones};

    #[test]
    fn feline_benefits_require_cat_and_speed_also_requires_outdoors() {
        for class in 0..9 {
            for form in [None].into_iter().chain(DruidForm::ALL.map(Some)) {
                for rank in [0, 1, 2, 3, 255] {
                    let mut ranks = vec![(1, 807, rank)];
                    if let Some(form) = form {
                        ranks.push((form.tree(), form.talent(), 1));
                    }
                    let m = Milestones {
                        class,
                        ..Default::default()
                    }
                    .with_ranks(&ranks);
                    let active = class == 8 && form == Some(DruidForm::Cat);
                    let rank = f32::from(rank.min(2));
                    assert!(
                        (m.feline_movement_multiplier(false)
                            - if active { 1. + rank * 0.15 } else { 1. })
                        .abs()
                            < 1e-6
                    );
                    assert_eq!(m.feline_movement_multiplier(true), 1.);
                    assert!((m.melee_dodge() - if active { rank * 0.02 } else { 0. }).abs() < 1e-6);
                    assert_eq!(m.with_ranks(&[]).melee_dodge(), 0.);
                    assert_eq!(m.with_ranks(&[]).feline_movement_multiplier(false), 1.);
                    // No unconditional speed or defensive stat leaks out of the talent.
                    let bonus = stat_adjustment(class, &[(1, 807, rank as u8)]);
                    assert_eq!((bonus.speed, bonus.armor, bonus.reduction), (0., 0., 0.));
                }
            }
        }
        let conflict = Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[
            (1, 807, 2),
            (1, DruidForm::Cat.talent(), 1),
            (1, DruidForm::Bear.talent(), 1),
        ]);
        assert_eq!(conflict.feline_movement_multiplier(false), 1.);
        assert_eq!(conflict.melee_dodge(), 0.);
    }

    #[test]
    fn feline_inline_upgrade_colors_both_changes_with_one_unit_each() {
        let node = talent_definition(8, 807).unwrap();
        for rank in [0, 1, 2, 255] {
            let (text, spans) = node.inline_description_with_values(rank).unwrap();
            assert_eq!(spans.len(), 2);
            assert_eq!(text.matches(" » ").count(), if rank < 2 { 2 } else { 0 });
            assert_eq!(text.matches("% movement speed").count(), 1);
            assert_eq!(
                text.matches("percentage points melee dodge chance").count(),
                1
            );
            assert!(text.contains("outdoors"));
        }
        let (text, spans) = node.inline_description_with_values(1).unwrap();
        assert_eq!(&text[spans[0].clone()], "+15 » 30% movement speed");
        assert_eq!(
            &text[spans[1].clone()],
            "+2 » 4 percentage points melee dodge chance"
        );
    }
}
