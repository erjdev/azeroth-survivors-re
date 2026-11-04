//! Independent, cast-snapshotted healing effects advanced only by active gameplay time.

#[derive(Clone, Debug, PartialEq)]
struct Effect {
    source: &'static str,
    rate: f32,
    remaining: f32,
    unpaid: f32,
    pulses: Option<Pulses>,
}

/// Fixed-count healing pulses; elapsed time uses f64 to keep frame partitioning stable.
#[derive(Clone, Debug, PartialEq)]
struct Pulses {
    interval: f64,
    elapsed: f64,
    paid: u32,
    count: u32,
    amount: f32,
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
            pulses: None,
        };
        if let Some(existing) = self.effects.iter_mut().find(|e| e.source == source) {
            *existing = effect;
        } else {
            self.effects.push(effect);
        }
        true
    }

    /// A moving channel may schedule a finite number of equal healing pulses.
    /// The first pulse occurs after one interval, never immediately. Recasting replaces
    /// only this source. Invalid schedules preserve every existing effect.
    pub fn refresh_pulsed(
        &mut self,
        source: &'static str,
        total: f32,
        interval: f32,
        count: u32,
    ) -> bool {
        let duration = interval * count as f32;
        if count == 0 || !self.refresh(source, total, duration) {
            return false;
        }
        let effect = self
            .effects
            .iter_mut()
            .find(|e| e.source == source)
            .unwrap();
        effect.pulses = Some(Pulses {
            interval: f64::from(interval),
            elapsed: 0.0,
            paid: 0,
            count,
            amount: total / count as f32,
        });
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
            let paid = if let Some(pulses) = &mut effect.pulses {
                pulses.elapsed += f64::from(seconds);
                // A small tolerance handles f32 frame deltas at an exact pulse boundary.
                let due =
                    ((pulses.elapsed / pulses.interval + 1e-6).floor() as u32).min(pulses.count);
                let newly_due = due.saturating_sub(pulses.paid);
                pulses.paid = due;
                if due == pulses.count {
                    effect.unpaid
                } else {
                    (newly_due as f32 * pulses.amount).min(effect.unpaid)
                }
            } else if seconds >= effect.remaining {
                effect.unpaid
            } else {
                (effect.rate * seconds).min(effect.unpaid)
            };
            effect.unpaid = (effect.unpaid - paid).max(0.0);
            effect.remaining = (effect.remaining - seconds).max(0.0);
            healing += paid;
        }
        self.effects.retain(|e| {
            e.unpaid > 0.0
                && e.pulses
                    .as_ref()
                    .map_or(e.remaining > 0.0, |p| p.paid < p.count)
        });
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

#[cfg(test)]
mod pulse_tests {
    use super::*;

    #[test]
    fn pulses_wait_for_boundaries_and_coexist_with_continuous_healing() {
        let mut healing = HealingOverTime::default();
        healing.refresh("rejuvenation", 30., 10.);
        assert!(healing.refresh_pulsed("tranquility", 50., 2., 5));
        assert_eq!(healing.advance(1.), 3.);
        assert_eq!(healing.advance(1.), 13.);
        assert_eq!(healing.advance(3.), 19.);
        assert_eq!(healing.advance(0.5), 1.5);
        assert_eq!(healing.advance(0.5), 11.5);
        assert_eq!(healing.advance(100.), 32.);
        assert_eq!(healing, HealingOverTime::default());
        assert_eq!(healing.advance(100.), 0.);
    }

    #[test]
    fn pulse_totals_are_frame_partition_independent_and_do_not_bank_overrun() {
        for steps in [
            vec![100.],
            vec![2.; 5],
            vec![0.01; 1000],
            vec![1. / 60.; 600],
        ] {
            let mut healing = HealingOverTime::default();
            healing.refresh_pulsed("tranquility", 37., 2., 5);
            let paid: f32 = steps.into_iter().map(|dt| healing.advance(dt)).sum();
            assert!((paid - 37.).abs() < 0.001, "{paid}");
            assert!(!healing.contains("tranquility"));
            assert_eq!(healing.advance(10.), 0.);
        }
    }

    #[test]
    fn pulse_refresh_and_invalid_schedules_preserve_other_sources() {
        let mut healing = HealingOverTime::default();
        healing.refresh("regrowth", 20., 10.);
        healing.refresh_pulsed("tranquility", 50., 2., 5);
        assert_eq!(healing.advance(3.), 16.);
        let before = healing.clone();
        for (total, interval, count) in [
            (50., 2., 0),
            (50., 0., 5),
            (50., -2., 5),
            (f32::NAN, 2., 5),
            (50., f32::INFINITY, 5),
            (50., f32::NAN, 5),
            (0., 2., 5),
        ] {
            assert!(!healing.refresh_pulsed("tranquility", total, interval, count));
            assert_eq!(healing, before);
        }
        for dt in [0., -1., f32::NAN, f32::INFINITY] {
            assert_eq!(healing.advance(dt), 0.);
            assert_eq!(healing, before);
        }
        healing.refresh_pulsed("tranquility", 100., 2., 5);
        assert_eq!(
            healing.advance(1.),
            2.,
            "recast restarts only its own pulse clock"
        );
        assert_eq!(healing.advance(1.), 22.);
        assert_eq!(healing.advance(100.), 90.);
        assert_eq!(healing, HealingOverTime::default());
    }
}
