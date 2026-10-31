//! Initial Survivors slot identities. These are authored balance values, not vanilla item stats.
use crate::campaign::Bonus;

#[derive(Clone, Copy)]
enum Stat {
    Health,
    Pickup,
    Armor,
    Damage,
    Crit,
    Multistrike,
    Speed,
    Area,
    CritDamage,
    Reduction,
    Cooldown,
    Gold,
    Growth,
}

/// The first two lines belong to Common; every subsequent band adds the next line.
/// Gloves retain the user's explicit coefficient examples in `campaign::glove_bonus`.
const PROFILES: [[Stat; 7]; 9] = {
    use Stat::*;
    [
        [
            Armor,
            Damage,
            Crit,
            Multistrike,
            Speed,
            Reduction,
            CritDamage,
        ],
        [Armor, Crit, CritDamage, Damage, Reduction, Area, Cooldown],
        [Health, Armor, Reduction, Pickup, Damage, Crit, Multistrike],
        [Health, Pickup, Gold, Growth, Area, Reduction, Cooldown],
        [Armor, Speed, Health, Area, Reduction, Damage, Cooldown],
        [Speed, Pickup, Health, Gold, Area, Reduction, Damage],
        [
            Crit,
            Cooldown,
            Damage,
            CritDamage,
            Area,
            Health,
            Multistrike,
        ],
        [
            Health, Pickup, Area, Reduction, Damage, Cooldown, CritDamage,
        ],
        [
            Damage,
            Crit,
            CritDamage,
            Area,
            Multistrike,
            Cooldown,
            Health,
        ],
    ]
};

pub(crate) fn piece(band: usize, slot: usize) -> Bonus {
    let mut bonus = Bonus::default();
    let Some(profile) = PROFILES.get(slot).filter(|_| band < 6) else {
        return bonus;
    };
    for stat in profile.iter().take(band + 2) {
        match stat {
            Stat::Health => bonus.health = [10., 13., 17., 25., 30., 35.][band],
            Stat::Pickup => bonus.pickup = [0.08, 0.10, 0.13, 0.18, 0.21, 0.25][band],
            Stat::Armor => bonus.armor = [6., 8., 10., 14., 16., 18.][band],
            Stat::Damage => bonus.might = [0.02, 0.03, 0.04, 0.06, 0.07, 0.08][band],
            Stat::Crit => bonus.crit = [0.01, 0.02, 0.03, 0.04, 0.045, 0.05][band],
            Stat::Multistrike => bonus.multistrike = [0.01, 0.02, 0.04, 0.06, 0.07, 0.08][band],
            Stat::Speed => bonus.speed = [0.01, 0.015, 0.02, 0.03, 0.035, 0.04][band],
            Stat::Area => bonus.area = [0.01, 0.02, 0.03, 0.04, 0.05, 0.06][band],
            Stat::CritDamage => bonus.crit_damage = [0.03, 0.05, 0.08, 0.12, 0.18, 0.25][band],
            Stat::Reduction => bonus.reduction = [0.005, 0.008, 0.012, 0.02, 0.03, 0.04][band],
            Stat::Cooldown => bonus.cooldown = [0.005, 0.008, 0.012, 0.02, 0.025, 0.03][band],
            Stat::Gold => bonus.gold = [0.02, 0.03, 0.04, 0.06, 0.08, 0.10][band],
            Stat::Growth => bonus.growth = [0.01, 0.02, 0.03, 0.04, 0.05, 0.06][band],
        }
    }
    bonus
}

#[cfg(test)]
mod tests {
    use crate::{campaign, equipment::Equipment};

    #[test]
    fn slot_identity_survives_rarity_and_explicit_downgrades() {
        let chest = campaign::piece_bonus(0, 2);
        assert_eq!((chest.health, chest.armor, chest.might), (10.0, 6.0, 0.0));
        let boots = campaign::piece_bonus(0, 5);
        assert_eq!((boots.speed, boots.pickup, boots.armor), (0.01, 0.08, 0.0));
        let weapon = campaign::piece_bonus(0, 8);
        assert_eq!((weapon.might, weapon.crit, weapon.armor), (0.02, 0.01, 0.0));

        let mut owned = [0; campaign::PIECES];
        owned[2] = 1;
        owned[3 * campaign::GRID_SLOTS + 2] = 99;
        let mut equipment = Equipment::legacy(&owned);
        let epic = campaign::equipped_bonus(&equipment.worn_pieces(&owned), None);
        assert_eq!((epic.health, epic.armor, epic.might), (25.0, 14.0, 0.06));
        owned[3 * campaign::GRID_SLOTS + 2] = 1;
        let single = campaign::equipped_bonus(&equipment.worn_pieces(&owned), None);
        assert_eq!(
            (single.health, single.armor, single.might),
            (epic.health, epic.armor, epic.might)
        );
        equipment.wear(&owned, 2, Some(0));
        let common = campaign::equipped_bonus(&equipment.worn_pieces(&owned), None);
        assert_eq!(
            (common.health, common.armor, common.might),
            (10.0, 6.0, 0.0)
        );
        assert_eq!(common.pickup + common.reduction, 0.0);
    }
}
