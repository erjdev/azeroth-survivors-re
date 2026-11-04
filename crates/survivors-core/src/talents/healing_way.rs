//! Installed spell 29203: +6% per stack, three stacks, one refreshed 15-second expiry.
use super::{Milestones, TalentStat};

pub(super) fn chance_percent(rank: u8) -> u8 {
    match rank {
        0 => 0,
        1 => 33,
        2 => 66,
        _ => 100,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HealingWay {
    stacks: u8,
    expires: f32,
}

impl HealingWay {
    pub fn chance(talents: Milestones, source: &str) -> f32 {
        if talents.class == 5 && source == "healing_wave" {
            f32::from(chance_percent(
                talents.trained_value(TalentStat::HealingWay) as u8,
            )) / 100.
        } else {
            0.
        }
    }

    pub fn multiplier(self, talents: Milestones, source: &str, elapsed: f32) -> f32 {
        if Self::chance(talents, source) > 0.
            && elapsed.is_finite()
            && elapsed >= 0.
            && elapsed < self.expires
        {
            1. + 0.06 * f32::from(self.stacks)
        } else {
            1.
        }
    }

    /// Settle after the heal, so this proc cannot amplify its own triggering cast.
    pub fn record(
        &mut self,
        talents: Milestones,
        source: &str,
        healed: f32,
        elapsed: f32,
        roll: f32,
    ) {
        if healed.is_finite()
            && healed > 0.
            && elapsed.is_finite()
            && elapsed >= 0.
            && (0. ..1.).contains(&roll)
            && roll < Self::chance(talents, source)
        {
            if elapsed >= self.expires {
                self.stacks = 0;
            }
            self.stacks = (self.stacks + 1).min(3);
            self.expires = elapsed + 15.;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn talents(rank: u8) -> Milestones {
        Milestones {
            class: 5,
            ..Default::default()
        }
        .with_ranks(&[(2, 1648, rank)])
    }
    #[test]
    fn stacks_refresh_expire_and_never_cross_sources_or_specs() {
        let t = talents(3);
        let mut buff = HealingWay::default();
        for n in 1..=4 {
            buff.record(t, "healing_wave", 1., n as f32, 0.);
            assert!(
                (buff.multiplier(t, "healing_wave", n as f32) - (1. + 0.06 * n.min(3) as f32))
                    .abs()
                    < 0.00001
            );
        }
        assert_eq!(buff.multiplier(t, "healing_wave", 19.), 1.);
        buff.record(t, "healing_wave", 1., 19., 0.);
        assert_eq!(buff.multiplier(t, "healing_wave", 19.), 1.06);
        for key in [
            "holy_light",
            "searing_totem",
            "magma_totem",
            "healing_touch",
        ] {
            let before = buff;
            buff.record(t, key, 40., 20., 0.);
            assert_eq!(buff, before);
            assert_eq!(buff.multiplier(t, key, 20.), 1.);
        }
        assert_eq!(buff.multiplier(talents(0), "healing_wave", 20.), 1.);
        assert_eq!(
            buff.multiplier(Milestones { class: 1, ..t }, "healing_wave", 20.),
            1.
        );
        assert_eq!(buff.multiplier(t, "healing_wave", f32::NAN), 1.);
    }
    #[test]
    fn every_rank_has_exact_chance_and_invalid_or_failed_heals_do_not_refresh() {
        for (rank, percent) in [(0, 0), (1, 33), (2, 66), (3, 100), (255, 100)] {
            let t = talents(rank);
            assert_eq!(HealingWay::chance(t, "healing_wave"), percent as f32 / 100.);
            for roll in 0..100 {
                let mut buff = HealingWay::default();
                buff.record(t, "healing_wave", 10., 0., roll as f32 / 100.);
                assert_eq!(buff.stacks > 0, roll < percent);
            }
        }
        let t = talents(3);
        let mut buff = HealingWay::default();
        buff.record(t, "healing_wave", 1., 0., 0.);
        let before = buff;
        for heal in [0., -1., f32::NAN, f32::INFINITY] {
            buff.record(t, "healing_wave", heal, 14., 0.);
        }
        for time in [-1., f32::NAN, f32::INFINITY] {
            buff.record(t, "healing_wave", 1., time, 0.);
        }
        for roll in [-1., 1., f32::NAN] {
            buff.record(t, "healing_wave", 1., 14., roll);
        }
        assert_eq!(buff, before);
        assert_eq!(buff.multiplier(t, "healing_wave", 15.), 1.);
    }
    #[test]
    fn tooltip_highlights_the_non_linear_proc_chance() {
        let def = super::super::talent_definition(5, 1648).unwrap();
        for (rank, expected) in [
            (0, "0 » 33% chance"),
            (1, "33 » 66% chance"),
            (2, "66 » 100% chance"),
            (3, "100% chance"),
        ] {
            let (text, ranges) = def.inline_description_with_values(rank).unwrap();
            assert!(ranges.iter().any(|range| &text[range.clone()] == expected));
        }
    }
}
