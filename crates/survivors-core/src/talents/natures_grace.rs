//! One 15-second charge; automatic casting converts the 500 ms cast-time bonus to recovery.
use super::{Milestones, TalentStat};

#[derive(Clone, Copy, Debug, Default)]
pub struct NaturesGrace {
    expires: Option<f32>,
}

impl NaturesGrace {
    pub fn hit(
        &mut self,
        talents: Milestones,
        source: &str,
        removed: f32,
        critical: bool,
        elapsed: f32,
    ) {
        if talents.class == 8
            && talents.trained_value(TalentStat::NaturesGrace) > 0.0
            && critical
            && removed.is_finite()
            && removed > 0.0
            && elapsed.is_finite()
            && elapsed >= 0.0
            && matches!(
                source,
                "wrath" | "moonfire" | "starfire" | "hurricane" | "insect_swarm" | "thorns"
            )
        {
            self.expires = Some(elapsed + 15.0);
        }
    }

    pub fn reduction(self, talents: Milestones, key: &str, elapsed: f32) -> f32 {
        if talents.class == 8
            && matches!(key, "wrath" | "starfire")
            && elapsed.is_finite()
            && elapsed >= 0.0
            && self.expires.is_some_and(|expires| elapsed < expires)
        {
            talents.trained_value(TalentStat::NaturesGrace)
        } else {
            0.0
        }
    }

    pub fn recovery(self, talents: Milestones, key: &str, elapsed: f32, cooldown: f32) -> f32 {
        let reduction = self.reduction(talents, key, elapsed);
        if reduction <= 0.0 || !cooldown.is_finite() || cooldown <= 0.0 {
            return cooldown;
        }
        // Never lengthen a timer already below the ordinary floor through other talent effects.
        (cooldown - reduction).max(crate::sheet::COOLDOWN_FLOOR.min(cooldown))
    }

    /// Call only after a successful cast, so waiting for a target preserves the charge.
    pub fn consume(&mut self, talents: Milestones, key: &str, elapsed: f32) {
        if self.reduction(talents, key, elapsed) > 0.0 {
            self.expires = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trained(class: usize, rank: u8) -> Milestones {
        Milestones {
            class,
            ..Default::default()
        }
        .with_ranks(&[(0, 789, rank)])
    }

    #[test]
    fn one_charge_refreshes_expires_and_only_speeds_cast_time_spells() {
        for rank in [0, 1, 255] {
            let talent = trained(8, rank);
            let mut grace = NaturesGrace::default();
            assert_eq!(grace.recovery(talent, "wrath", 0., 2.), 2.);
            grace.hit(talent, "moonfire", 10., true, 0.);
            let expected = if rank == 0 { 2. } else { 1.5 };
            assert_eq!(grace.recovery(talent, "wrath", 0., 2.), expected);
            for key in [
                "moonfire",
                "hurricane",
                "rejuvenation",
                "thorns",
                "fireball",
            ] {
                grace.consume(talent, key, 1.);
                assert_eq!(grace.recovery(talent, key, 1., 2.), 2.);
            }
            grace.hit(talent, "hurricane", 20., true, 14.);
            assert_eq!(grace.recovery(talent, "starfire", 15., 2.), expected);
            assert_eq!(grace.recovery(talent, "starfire", 28.99, 2.), expected);
            assert_eq!(grace.recovery(talent, "starfire", 29., 2.), 2.);
            grace.hit(talent, "wrath", 2., true, 30.);
            grace.consume(talent, "starfire", 30.);
            assert_eq!(grace.recovery(talent, "wrath", 30., 2.), 2.);
        }
    }

    #[test]
    fn invalid_hits_other_classes_and_refunds_cannot_grant_recovery() {
        for class in 0..9 {
            let talent = trained(class, 1);
            for (source, damage, critical, elapsed) in [
                ("wrath", 0., true, 0.),
                ("wrath", -1., true, 0.),
                ("wrath", f32::NAN, true, 0.),
                ("wrath", f32::INFINITY, true, 0.),
                ("wrath", 1., false, 0.),
                ("wrath", 1., true, f32::NAN),
                ("wrath", 1., true, -1.),
                ("fireball", 1., true, 0.),
            ] {
                let mut grace = NaturesGrace::default();
                grace.hit(talent, source, damage, critical, elapsed);
                assert_eq!(grace.recovery(talent, "wrath", 0., 2.), 2.);
            }
            let mut grace = NaturesGrace::default();
            grace.hit(talent, "wrath", 1., true, 0.);
            assert_eq!(
                grace.recovery(talent, "wrath", 0., 2.),
                if class == 8 { 1.5 } else { 2. }
            );
            assert_eq!(grace.recovery(talent.with_ranks(&[]), "wrath", 0., 2.), 2.);
            for elapsed in [-1., f32::NAN, f32::INFINITY] {
                assert_eq!(grace.recovery(talent, "wrath", elapsed, 2.), 2.);
            }
        }
        let talent = trained(8, 1);
        let mut grace = NaturesGrace::default();
        grace.hit(talent, "starfire", 10., true, 0.);
        for (before, after) in [(2., 1.5), (0.6, 0.2), (0.2, 0.2), (0.1, 0.1)] {
            assert!((grace.recovery(talent, "wrath", 1., before) - after).abs() < 0.00001);
        }
        let definition = super::super::talent_definition(8, 789).unwrap();
        let (text, highlights) = definition.inline_description_with_values(0).unwrap();
        assert_eq!(highlights.len(), 1);
        assert_eq!(&text[highlights[0].clone()], "0 » 0.5 sec");
        assert!(!definition.inline_description(1).unwrap().contains(" » "));
    }
}
