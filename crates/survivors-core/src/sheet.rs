//! The run's stat sheet. Gear, talents, renown, passives, and jewelry all write these numbers.
//! Combat reads the functions here so a bonus cannot pick its own formula.

use super::data::Kind;

/// Damage reduction cannot pass this. A fully stacked sheet still takes a hit.
pub const DR_CAP: f32 = 0.75;
/// Cooldown reduction, separate from haste, cannot pass this.
pub const COOLDOWN_CAP: f32 = 0.40;
/// Haste already leaves at least this fraction of the cooldown.
pub const HASTE_FLOOR: f32 = 0.35;
/// Absolute floor under both reductions, in seconds.
pub const COOLDOWN_FLOOR: f32 = 0.2;
/// +targets on a single-target spell cannot exceed this.
pub const TARGET_CAP: u32 = 5;
/// One luck rating is one percent, and the shown percent stops at 100.
pub const LUCK_CAP: f32 = 100.0;

/// How many times one hit lands.
///
/// Below 100% multistrike, `roll` (in `[0, 1)`) is the chance to hit twice. From 100% to 200%
/// every hit lands twice, and the overflow is the chance of a third. From 200% to 300% every hit
/// lands four times, and the overflow is the chance of a fifth. Each further 100% follows that
/// step: the guaranteed count is `2 * floor(multistrike)`.
pub fn strike_count(multistrike: f32, roll: f32) -> u32 {
    let multi = multistrike.max(0.0);
    let band = multi.floor() as u32;
    let frac = (multi - band as f32).clamp(0.0, 0.999);
    if band == 0 {
        if roll < frac {
            2
        } else {
            1
        }
    } else if roll < frac {
        band * 2 + 1
    } else {
        band * 2
    }
}

/// Damage taken after flat armor, then percent reduction.
pub fn mitigate(raw: f32, armor: f32, reduction: f32) -> f32 {
    let after_armor = (raw - armor).max(1.0);
    let dr = reduction.clamp(0.0, DR_CAP);
    (after_armor * (1.0 - dr)).max(1.0)
}

/// Cooldown after haste and cooldown %. They multiply. Neither can zero the timer.
pub fn cooldown_time(base: f32, haste: f32, cooldown: f32) -> f32 {
    let after_haste = base * (1.0 - haste).max(HASTE_FLOOR);
    let cdr = cooldown.clamp(0.0, COOLDOWN_CAP);
    (after_haste * (1.0 - cdr)).max(COOLDOWN_FLOOR)
}

/// Extra targets for a spell that is single-target in its definition. Area spells gain none,
/// and a spell that already names several targets in its base count gains none.
pub fn extra_targets(kind: Kind, base_count: u32, bonus: u32) -> u32 {
    let single = matches!(
        kind,
        Kind::Interrupt
            | Kind::Channel
            | Kind::Mark
            | Kind::Curse
            | Kind::Bolt
            | Kind::Strike
            | Kind::Melee
    ) && base_count <= 1;
    if single {
        bonus.min(TARGET_CAP)
    } else {
        0
    }
}

/// Luck rating as a percent, capped at [`LUCK_CAP`].
pub fn luck_percent(rating: f32) -> f32 {
    rating.clamp(0.0, LUCK_CAP)
}

/// Gold after the gold % multiplier. A fraction rounds to the nearest coin, and a positive
/// drop never rounds down to nothing.
pub fn gold_amount(base: u32, gold_find: f32) -> u32 {
    if base == 0 {
        return 0;
    }
    let n = (base as f32 * gold_find.max(0.0)).round() as u32;
    n.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multistrike_doubles_then_steps_at_each_hundred() {
        assert_eq!(strike_count(0.0, 0.0), 1);
        assert_eq!(strike_count(0.4, 0.0), 2);
        assert_eq!(strike_count(0.4, 0.99), 1);
        assert_eq!(strike_count(1.0, 0.0), 2);
        assert_eq!(strike_count(1.2, 0.0), 3);
        assert_eq!(strike_count(1.2, 0.99), 2);
        assert_eq!(strike_count(2.0, 0.99), 4);
        assert_eq!(strike_count(2.4, 0.0), 5);
        assert_eq!(strike_count(2.4, 0.99), 4);
    }

    #[test]
    fn damage_reduction_follows_armor_and_stops_at_three_quarters() {
        assert_eq!(mitigate(20.0, 5.0, 0.0), 15.0);
        assert!((mitigate(20.0, 0.0, 0.5) - 10.0).abs() < 0.01);
        assert!((mitigate(20.0, 0.0, 0.9) - 5.0).abs() < 0.01);
        assert!(mitigate(0.2, 0.0, 0.0) >= 1.0);
    }

    #[test]
    fn cooldown_stacks_haste_and_percent_and_keeps_a_floor() {
        let base = 10.0;
        let haste_only = cooldown_time(base, 0.0, 0.0);
        let both = cooldown_time(base, 0.2, 0.1);
        assert!(both < haste_only);
        assert!(cooldown_time(base, 0.9, 0.9) >= COOLDOWN_FLOOR);
    }

    #[test]
    fn extra_targets_reach_only_single_target_spells() {
        assert_eq!(extra_targets(Kind::Bolt, 1, 2), 2);
        assert_eq!(extra_targets(Kind::Strike, 1, 9), TARGET_CAP);
        assert_eq!(extra_targets(Kind::Melee, 1, 1), 1);
        assert_eq!(extra_targets(Kind::Bolt, 3, 2), 0);
        assert_eq!(extra_targets(Kind::Nova, 1, 4), 0);
        assert_eq!(extra_targets(Kind::Cone, 1, 4), 0);
    }

    #[test]
    fn luck_is_a_percent_and_gold_rounds_up_from_a_scrap() {
        assert_eq!(luck_percent(12.0), 12.0);
        assert_eq!(luck_percent(140.0), 100.0);
        assert_eq!(gold_amount(1, 1.4), 1);
        assert_eq!(gold_amount(10, 1.5), 15);
        assert_eq!(gold_amount(1, 0.1), 1);
    }
}
