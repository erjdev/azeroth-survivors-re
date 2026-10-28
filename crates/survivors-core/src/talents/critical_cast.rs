//! Class-scoped next-cast critical charges advance only with run time.
use super::{Milestones, TalentStat};

#[derive(Clone, Copy, Debug, Default)]
pub struct CriticalCastClock {
    next: f32,
}

impl CriticalCastClock {
    fn charge(talents: Milestones, key: &str) -> Option<(f32, f32)> {
        let (stat, recovery) = match talents.class {
            1 if key == "holy_shock" => (TalentStat::DivineFavor, 120.0),
            3 if matches!(
                key,
                "backstab" | "sinister_strike" | "eviscerate" | "gouge" | "kidney_shot"
            ) =>
            {
                (TalentStat::ColdBlood, 180.0)
            }
            4 if matches!(key, "smite" | "holy_fire" | "holy_nova" | "mind_blast") => {
                (TalentStat::InnerFocus, 180.0)
            }
            5 if super::shaman::CASTS.contains(&key) => (TalentStat::ElementalMastery, 180.0),
            _ => return None,
        };
        Some((talents.trained_value(stat), recovery))
    }

    pub fn bonus(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if !elapsed.is_finite() || elapsed < self.next {
            return 0.0;
        }
        Self::charge(talents, key).map_or(0.0, |(bonus, _)| bonus)
    }

    /// Call only after a successful cast. Waiting never banks additional charges.
    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if self.bonus(talents, key, elapsed) > 0.0 {
            if let Some((_, recovery)) = Self::charge(talents, key) {
                self.next = elapsed + recovery;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divine_favor_only_readies_holy_shock_every_two_minutes() {
        for rank in [0, 1, 255] {
            let talents = Milestones {
                class: 1,
                ..Default::default()
            }
            .with_ranks(&[(0, 1433, rank)]);
            let expected = f32::from(rank > 0);
            let mut clock = CriticalCastClock::default();
            for key in [
                "judgement",
                "consecration",
                "exorcism",
                "holy_wrath",
                "hammer_of_justice",
                "hammer_of_wrath",
                "smite",
            ] {
                clock.consume(talents, key, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for time in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(talents, "holy_shock", time);
                assert_eq!(clock.bonus(talents, "holy_shock", time), 0.0);
            }
            assert_eq!(clock.bonus(talents, "holy_shock", 0.0), expected);
            clock.consume(talents, "holy_shock", 0.0);
            assert_eq!(clock.bonus(talents, "holy_shock", 119.99), 0.0);
            assert_eq!(clock.bonus(talents, "holy_shock", 120.0), expected);
            clock.consume(talents, "holy_shock", 1000.0);
            assert_eq!(clock.bonus(talents, "holy_shock", 1119.99), 0.0);
            assert_eq!(clock.bonus(talents, "holy_shock", 1120.0), expected);
            assert_eq!(
                clock.bonus(talents.with_ranks(&[]), "holy_shock", 1120.0),
                0.0
            );
        }
        for class in [0, 2, 3, 4, 5, 6, 7, 8] {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 1433, 1)]);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "holy_shock", 0.0),
                0.0
            );
        }
        let node = super::super::talent_definition(1, 1433).unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 100%"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }

    #[test]
    fn inner_focus_adds_a_quarter_chance_without_affecting_periodic_or_defensive_spells() {
        for rank in [0, 1, 255] {
            let talents = Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(0, 348, rank)]);
            let expected = if rank > 0 { 0.25 } else { 0.0 };
            let mut clock = CriticalCastClock::default();
            for key in [
                "shadow_word_pain",
                "power_word_shield",
                "psychic_scream",
                "renew",
                "fireball",
            ] {
                clock.consume(talents, key, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for key in ["smite", "holy_fire", "holy_nova", "mind_blast"] {
                assert_eq!(clock.bonus(talents, key, 0.0), expected);
            }
            for time in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(talents, "smite", time);
                assert_eq!(clock.bonus(talents, "smite", time), 0.0);
            }
            clock.consume(talents, "holy_nova", 0.0);
            assert_eq!(clock.bonus(talents, "mind_blast", 179.99), 0.0);
            assert_eq!(clock.bonus(talents, "mind_blast", 180.0), expected);
            clock.consume(talents, "mind_blast", 1000.0);
            assert_eq!(clock.bonus(talents, "smite", 1179.99), 0.0);
            assert_eq!(clock.bonus(talents, "smite", 1180.0), expected);
            assert_eq!(clock.bonus(talents.with_ranks(&[]), "smite", 1180.0), 0.0);
        }
        for class in [0, 1, 2, 3, 5, 6, 7, 8] {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 348, 1)]);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "smite", 0.0),
                0.0
            );
        }
        let node = super::super::talent_definition(4, 348).unwrap();
        assert!(node
            .inline_description(0)
            .unwrap()
            .contains("0 » 25 percentage points"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }

    #[test]
    fn mastery_scopes_to_shaman_casts_and_recovers_without_banked_charges() {
        for rank in [0, 1, 255] {
            let talents = Milestones {
                class: 5,
                ..Default::default()
            }
            .with_ranks(&[(0, 573, rank)]);
            let expected = f32::from(rank > 0);
            let mut clock = CriticalCastClock::default();
            for key in super::super::shaman::CASTS {
                assert_eq!(clock.bonus(talents, key, 0.0), expected);
            }
            for key in [
                "searing_totem",
                "magma_totem",
                "lightning_shield",
                "fireball",
                "sinister_strike",
                "",
            ] {
                clock.consume(talents, key, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for time in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(talents, "lightning_bolt", time);
                assert_eq!(clock.bonus(talents, "lightning_bolt", time), 0.0);
            }
            assert_eq!(clock.bonus(talents, "lightning_bolt", 0.0), expected);
            clock.consume(talents, "lightning_bolt", 0.0);
            assert_eq!(clock.bonus(talents, "earth_shock", 179.99), 0.0);
            assert_eq!(clock.bonus(talents, "earth_shock", 180.0), expected);
            clock.consume(talents, "earth_shock", 1000.0);
            assert_eq!(clock.bonus(talents, "frost_shock", 1179.99), 0.0);
            assert_eq!(clock.bonus(talents, "frost_shock", 1180.0), expected);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "lightning_bolt", 0.0),
                expected
            );
        }
        for class in [0, 1, 2, 3, 4, 6, 7, 8] {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 573, 1)]);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "lightning_bolt", 0.0),
                0.0
            );
        }
        let node = super::super::definitions(5)
            .iter()
            .find(|n| n.id == 573)
            .unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 100%"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }

    #[test]
    fn cold_blood_scopes_and_recovers_one_cast_without_banking_charges() {
        for rank in [0, 1, 255] {
            let talents = Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(0, 280, rank)]);
            let expected = f32::from(rank > 0);
            let mut clock = CriticalCastClock::default();
            for key in ["sinister_strike", "eviscerate", "gouge", "kidney_shot"] {
                assert_eq!(clock.bonus(talents, key, 0.0), expected);
            }
            for key in ["blade_flurry", "rupture", "fireball", "volley", ""] {
                clock.consume(talents, key, 0.0);
                assert_eq!(clock.bonus(talents, key, 0.0), 0.0);
            }
            for time in [-1.0, f32::NAN, f32::INFINITY] {
                clock.consume(talents, "gouge", time);
                assert_eq!(clock.bonus(talents, "gouge", time), 0.0);
            }
            assert_eq!(clock.bonus(talents, "gouge", 0.0), expected);
            clock.consume(talents, "gouge", 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 179.99), 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 180.0), expected);
            clock.consume(talents, "gouge", 1000.0);
            assert_eq!(clock.bonus(talents, "gouge", 1179.99), 0.0);
            assert_eq!(clock.bonus(talents, "gouge", 1180.0), expected);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "gouge", 0.0),
                expected
            );
        }
        for class in [0, 1, 2, 4, 5, 6, 7, 8] {
            let talents = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 280, 1)]);
            assert_eq!(
                CriticalCastClock::default().bonus(talents, "gouge", 0.0),
                0.0
            );
        }
        let node = super::super::definitions(3)
            .iter()
            .find(|n| n.id == 280)
            .unwrap();
        assert!(node.inline_description(0).unwrap().contains("0 » 100%"));
        assert!(!node.inline_description(1).unwrap().contains(" » "));
    }
}
