//! Gear progression identities, separate from the six legacy collection save bands.
use crate::campaign::{self, Bonus, Quality};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearTier {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    RaidEpic,
    Pvp,
    Legendary,
}

impl GearTier {
    /// Existing collection saves retain their original indexes. New tiers need named storage.
    pub const fn legacy_band(self) -> Option<usize> {
        match self {
            Self::Common => Some(0),
            Self::Uncommon => Some(1),
            Self::Rare => Some(2),
            Self::Epic => Some(3),
            Self::Pvp => Some(4),
            Self::Legendary => Some(5),
            Self::Poor | Self::RaidEpic => None,
        }
    }

    pub const fn from_legacy_band(band: usize) -> Option<Self> {
        match band {
            0 => Some(Self::Common),
            1 => Some(Self::Uncommon),
            2 => Some(Self::Rare),
            3 => Some(Self::Epic),
            4 => Some(Self::Pvp),
            5 => Some(Self::Legendary),
            _ => None,
        }
    }

    pub const fn quality(self) -> Quality {
        match self {
            Self::Poor => Quality::Poor,
            Self::Common => Quality::Common,
            Self::Uncommon => Quality::Uncommon,
            Self::Rare => Quality::Rare,
            Self::Epic | Self::RaidEpic | Self::Pvp => Quality::Epic,
            Self::Legendary => Quality::Legendary,
        }
    }

    /// Initial stat budgets. Raid Epic shares the six-line PvP budget, not its identity.
    /// This does not grant ownership, unlock a drop source, or apply a raid set bonus.
    pub fn piece_bonus(self, slot: usize) -> Bonus {
        if slot >= campaign::GRID_SLOTS {
            return Bonus::default();
        }
        if let Some(band) = self.legacy_band() {
            return campaign::piece_bonus(band, slot);
        }
        if self == Self::RaidEpic {
            return campaign::piece_bonus(4, slot);
        }
        // Poor keeps only the slot's primary stat, below its Common value.
        let mut bonus = Bonus::default();
        match slot {
            0 | 1 | 4 => bonus.armor = 4.,
            2 | 3 | 7 => bonus.health = 8.,
            5 => bonus.speed = 0.005,
            6 => bonus.crit = 0.005,
            8 => bonus.might = 0.01,
            _ => unreachable!(),
        }
        bonus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats(b: Bonus) -> [f32; 13] {
        [
            b.health,
            b.pickup,
            b.armor,
            b.might,
            b.crit,
            b.multistrike,
            b.speed,
            b.area,
            b.crit_damage,
            b.reduction,
            b.cooldown,
            b.gold,
            b.growth,
        ]
    }

    #[test]
    fn legacy_bands_keep_identity_and_exact_stats() {
        for band in 0..campaign::BANDS {
            let tier = GearTier::from_legacy_band(band).unwrap();
            assert_eq!(tier.legacy_band(), Some(band));
            for slot in 0..campaign::GRID_SLOTS {
                assert_eq!(
                    stats(tier.piece_bonus(slot)),
                    stats(campaign::piece_bonus(band, slot))
                );
            }
        }
        assert_eq!(GearTier::from_legacy_band(6), None);
        assert_eq!(GearTier::from_legacy_band(usize::MAX), None);
        assert_eq!(GearTier::Poor.legacy_band(), None);
        assert_eq!(GearTier::RaidEpic.legacy_band(), None);
        assert_ne!(GearTier::Epic, GearTier::RaidEpic);
        assert_ne!(GearTier::Pvp, GearTier::RaidEpic);
    }

    #[test]
    fn progression_adds_one_stat_and_strengthens_existing_lines_in_every_slot() {
        let tiers = [
            GearTier::Poor,
            GearTier::Common,
            GearTier::Uncommon,
            GearTier::Rare,
            GearTier::Epic,
            GearTier::RaidEpic,
            GearTier::Legendary,
        ];
        for slot in 0..campaign::GRID_SLOTS {
            let mut previous = [0.; 13];
            for (index, tier) in tiers.into_iter().enumerate() {
                let current = stats(tier.piece_bonus(slot));
                assert_eq!(
                    current.iter().filter(|&&value| value > 0.).count(),
                    index + 1
                );
                for (old, new) in previous.iter().zip(current) {
                    if *old > 0. {
                        assert!(new > *old, "{tier:?}, slot {slot}");
                    }
                }
                previous = current;
            }
        }
        for tier in tiers {
            assert_eq!(stats(tier.piece_bonus(usize::MAX)), [0.; 13]);
        }
        assert_eq!(GearTier::RaidEpic.quality(), Quality::Epic);
        assert_eq!(GearTier::Poor.piece_bonus(7).health, 8.);
    }
}
