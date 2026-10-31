//! Independent, cast-snapshotted healing effects advanced only by active gameplay time.

#[derive(Clone, Debug, PartialEq)]
struct Effect {
    source: &'static str,
    rate: f32,
    remaining: f32,
    unpaid: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HealingOverTime {
    effects: Vec<Effect>,
}

impl HealingOverTime {
    /// Recasting one source replaces its unpaid healing; other spells keep their own clocks.
    /// Invalid casts leave an existing effect intact. The caller enforces life/cast eligibility.
    pub fn refresh(&mut self, source: &'static str, total: f32, duration: f32) -> bool {
        let rate = total / duration;
        if source.is_empty()
            || !total.is_finite()
            || total <= 0.0
            || !duration.is_finite()
            || duration <= 0.0
            || !rate.is_finite()
            || rate <= 0.0
        {
            return false;
        }
        let effect = Effect {
            source,
            rate,
            remaining: duration,
            unpaid: total,
        };
        if let Some(existing) = self.effects.iter_mut().find(|e| e.source == source) {
            *existing = effect;
        } else {
            self.effects.push(effect);
        }
        true
    }

    pub fn contains(&self, source: &str) -> bool {
        self.effects.iter().any(|e| e.source == source)
    }

    /// Returns scheduled healing, including only the unexpired part of a long frame.
    /// The caller caps effective healing to missing health; overhealing is never banked.
    pub fn advance(&mut self, seconds: f32) -> f32 {
        if !seconds.is_finite() || seconds <= 0.0 {
            return 0.0;
        }
        let mut healing = 0.0;
        for effect in &mut self.effects {
            let paid = if seconds >= effect.remaining {
                effect.unpaid
            } else {
                (effect.rate * seconds).min(effect.unpaid)
            };
            effect.unpaid = (effect.unpaid - paid).max(0.0);
            effect.remaining = (effect.remaining - seconds).max(0.0);
            healing += paid;
        }
        self.effects.retain(|e| e.remaining > 0.0 && e.unpaid > 0.0);
        healing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_sources_refresh_without_replaying_or_erasing_each_other() {
        let mut healing = HealingOverTime::default();
        assert!(healing.refresh("rejuvenation", 30.0, 10.0));
        assert_eq!(healing.advance(2.0), 6.0);
        assert!(healing.refresh("regrowth", 12.0, 6.0));
        assert_eq!(healing.advance(2.0), 10.0);
        assert!(healing.refresh("rejuvenation", 60.0, 10.0));
        assert_eq!(healing.advance(4.0), 32.0);
        assert!(!healing.contains("regrowth"));
        assert!(healing.contains("rejuvenation"));
        assert_eq!(healing.advance(20.0), 36.0);
        assert_eq!(healing.advance(20.0), 0.0);
    }

    #[test]
    fn frame_partition_and_subsecond_duration_preserve_total_payment() {
        for steps in [vec![50.0], vec![1.0; 50], vec![0.01; 5000]] {
            let mut healing = HealingOverTime::default();
            healing.refresh("long", 37.0, 13.0);
            healing.refresh("short", 7.0, 0.25);
            let paid: f32 = steps.into_iter().map(|dt| healing.advance(dt)).sum();
            assert!((paid - 44.0).abs() < 0.002, "{paid}");
            assert_eq!(healing, HealingOverTime::default());
        }
    }

    #[test]
    fn pause_and_invalid_input_do_not_consume_or_replace_effects() {
        let mut healing = HealingOverTime::default();
        healing.refresh("rejuvenation", 30.0, 10.0);
        let before = healing.clone();
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(healing.advance(value), 0.0);
            assert!(!healing.refresh("rejuvenation", value, 10.0));
            assert!(!healing.refresh("rejuvenation", 30.0, value));
            assert_eq!(healing, before);
        }
        assert!(!healing.refresh("", 30.0, 10.0));
        assert_eq!(healing, before);
    }
}
