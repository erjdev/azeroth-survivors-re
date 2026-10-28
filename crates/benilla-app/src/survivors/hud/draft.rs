//! Identity and exclusion checks for replacing a card without repeating the hand.
use super::*;
use crate::survivors::campaign::{JewelKind, TradeCard};

pub(super) fn same(a: Choice, b: Choice) -> bool {
    match (a, b) {
        (Choice::NewAbility(a), Choice::NewAbility(b)) => a.key == b.key,
        (Choice::NewPassive(a), Choice::NewPassive(b)) => a.key == b.key,
        (Choice::Upgrade(a), Choice::Upgrade(b)) | (Choice::UpPassive(a), Choice::UpPassive(b)) => {
            a == b
        }
        (Choice::Jewel(a), Choice::Jewel(b)) => a == b,
        (Choice::Heal, Choice::Heal) => true,
        _ => false,
    }
}

pub(super) fn trade(card: Choice, run: &Run) -> Option<TradeCard> {
    Some(match card {
        Choice::Upgrade(i) => TradeCard::Upgrade(i),
        Choice::Jewel(JewelKind::Ring(i)) => TradeCard::Ring(i as usize),
        Choice::Jewel(JewelKind::Trinket(i)) => TradeCard::Trinket(i as usize),
        Choice::Jewel(JewelKind::Amulet(i)) => TradeCard::Amulet(i as usize),
        Choice::Jewel(JewelKind::Legend(key)) => TradeCard::Legend(
            run.abilities
                .iter()
                .position(|owned| owned.def.key == key)?,
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn banish_replaces_in_place_without_repeating_any_original_option() {
        for seed in 1..65 {
            let mut run = Run {
                banishes: 2,
                ..default()
            };
            let mut rng = Rng(seed);
            let mut offer = Offer {
                choices: choices_for(&run, &mut rng, 3),
            };
            let original = offer.choices.clone();
            assert!(banish_card(&mut run, &mut offer, &mut rng, 1));
            assert_eq!(run.banishes, 1);
            assert_eq!(offer.len(), 3);
            assert!(same(offer.choices[0], original[0]));
            assert!(same(offer.choices[2], original[2]));
            assert!(!original.iter().any(|card| same(*card, offer.choices[1])));
        }
    }

    #[test]
    fn exhausted_level_hand_and_invalid_banish_keep_their_charge() {
        let mut run = Run {
            banishes: 1,
            ..default()
        };
        for key in run.class.pool.iter().take(data::MAX_ABILITIES) {
            run.abilities.push(Owned {
                def: data::ability(key),
                level: data::MAX_ABILITY_LEVEL,
                cd: 0.0,
            });
        }
        for passive in data::PASSIVES.iter().take(data::MAX_PASSIVES) {
            run.passives.push((passive, data::MAX_PASSIVE_LEVEL));
        }
        let mut rng = Rng(17);
        let mut offer = Offer {
            choices: choices_for(&run, &mut rng, 3),
        };
        assert_eq!(offer.len(), 1);
        assert!(matches!(offer.choices[0], Choice::Heal));
        assert!(!banish_card(&mut run, &mut offer, &mut rng, 4));
        assert!(!banish_card(&mut run, &mut offer, &mut rng, 0));
        assert_eq!(run.banishes, 1);
        assert_eq!(offer.len(), 1);
        assert!(matches!(offer.choices[0], Choice::Heal));
    }

    #[test]
    fn chest_banish_keeps_single_card_and_charge_when_no_replacement_rolls() {
        let mut saw_exhausted = false;
        let mut saw_jewelry = false;
        for seed in 1..129 {
            let mut run = Run {
                trading: true,
                banishes: 1,
                banish_armed: true,
                ..default()
            };
            run.abilities.push(Owned {
                def: data::ability("arcane_shot"),
                level: 1,
                cd: 0.0,
            });
            let mut offer = Offer {
                choices: vec![Choice::Upgrade(0)],
            };
            let mut rng = Rng(seed);
            let changed = banish_card(&mut run, &mut offer, &mut rng, 0);
            assert_eq!(offer.len(), 1);
            assert!(!run.banish_armed);
            if changed {
                saw_jewelry = true;
                assert_eq!(run.banishes, 0);
                assert!(matches!(offer.choices[0], Choice::Jewel(_)));
            } else {
                saw_exhausted = true;
                assert_eq!(rng.0, seed);
                assert!(!banish_card(&mut run, &mut offer, &mut rng, 0));
                assert_eq!(rng.0, seed);
                assert_eq!(run.banishes, 1);
                assert!(matches!(offer.choices[0], Choice::Upgrade(0)));
            }
        }
        assert!(saw_exhausted && saw_jewelry);
    }
}

#[cfg(test)]
mod backstab_offer_tests {
    use super::*;
    use survivors_core::gear_loadout::{MeleeKind, MeleeWeapon};
    #[test]
    fn backstab_offers_and_selection_require_the_worn_dagger() {
        for kind in [None, Some(MeleeKind::Sword), Some(MeleeKind::Dagger)] {
            let mut run = Run {
                class: &data::CLASSES[3],
                ..Default::default()
            };
            run.milestones.weapon = kind.map(|kind| MeleeWeapon {
                kind,
                two_handed: false,
            });
            let mut rng = Rng(3);
            let choices = choices_for(&run, &mut rng, 100);
            let allowed = kind == Some(MeleeKind::Dagger);
            assert_eq!(
                choices
                    .iter()
                    .any(|c| matches!(c, Choice::NewAbility(a) if a.key == "backstab")),
                allowed
            );
            let mut offer = Offer {
                choices: vec![Choice::NewAbility(data::ability("backstab"))],
            };
            pick(0, &mut run, &mut offer, &mut rng, &mut NextState::default());
            assert_eq!(run.abilities.len(), usize::from(allowed));
            if allowed {
                let mut offer = Offer {
                    choices: choices_for(&run, &mut rng, 100),
                };
                assert!(!offer
                    .choices
                    .iter()
                    .any(|c| matches!(c, Choice::NewAbility(a) if a.key == "backstab")));
                let slot = offer
                    .choices
                    .iter()
                    .position(|c| matches!(c, Choice::Upgrade(0)))
                    .unwrap();
                pick(
                    slot,
                    &mut run,
                    &mut offer,
                    &mut rng,
                    &mut NextState::default(),
                );
                assert_eq!(run.abilities[0].level, 2);
            }
        }
    }
}
