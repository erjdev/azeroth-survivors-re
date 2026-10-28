//! Active-time channel pulses. The caller owns damage snapshots and validates each target.

#[derive(Clone, Debug, PartialEq)]
pub struct Channel<T> {
    targets: Vec<T>,
    started: f32,
    observed: f32,
    interval: f32,
    ticks: u8,
    emitted: u8,
}

impl<T: Copy + Eq> Channel<T> {
    /// A channel fixes its targets at cast time; duplicates cannot multiply damage.
    pub fn new(
        targets: impl IntoIterator<Item = T>,
        now: f32,
        interval: f32,
        ticks: u8,
    ) -> Option<Self> {
        let end = now + interval * f32::from(ticks);
        if !now.is_finite()
            || now < 0.0
            || !interval.is_finite()
            || interval <= 0.0
            || ticks == 0
            || !end.is_finite()
            || now + interval <= now
        {
            return None;
        }
        let mut unique = Vec::new();
        for target in targets {
            if !unique.contains(&target) {
                unique.push(target);
            }
        }
        if unique.is_empty() {
            return None;
        }
        Some(Self {
            targets: unique,
            started: now,
            observed: now,
            interval,
            ticks,
            emitted: 0,
        })
    }

    /// First pulse is after one interval, never at cast time. Catch-up pulses retain
    /// their scheduled timestamp, allowing damage consumers to evaluate timed effects.
    /// Losing range/visibility or dying permanently removes that target from this cast.
    /// Invalid or backwards time is ignored rather than replaying already emitted pulses.
    pub fn advance(&mut self, now: f32, mut connected: impl FnMut(T) -> bool) -> Vec<(T, f32)> {
        if !now.is_finite() || now < self.observed {
            return Vec::new();
        }
        self.observed = now;
        self.targets.retain(|&target| connected(target));
        let mut pulses = Vec::new();
        while self.emitted < self.ticks {
            let at = self.started + self.interval * f32::from(self.emitted + 1);
            if at > now {
                break;
            }
            for &target in &self.targets {
                pulses.push((target, at));
            }
            self.emitted += 1;
        }
        pulses
    }

    pub fn finished(&self) -> bool {
        self.targets.is_empty() || self.emitted == self.ticks
    }

    /// Caster death, mode exit or explicit interruption stops unpaid pulses immediately.
    pub fn cancel(&mut self) {
        self.targets.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_second_channel_has_no_initial_hit_and_pays_each_pulse_once() {
        let mut channel = Channel::new([7], 10.0, 1.0, 3).unwrap();
        for now in [10.0, 10.0, 10.9] {
            assert!(channel.advance(now, |_| true).is_empty());
        }
        assert_eq!(channel.advance(11.0, |_| true), [(7, 11.0)]);
        assert!(channel.advance(11.0, |_| true).is_empty());
        assert_eq!(channel.advance(13.0, |_| true), [(7, 12.0), (7, 13.0)]);
        assert!(channel.finished());
        assert!(channel.advance(30.0, |_| true).is_empty());
    }

    #[test]
    fn lost_targets_never_rejoin_and_catchup_preserves_pulse_order() {
        let mut channel = Channel::new([1, 2, 1, 3], 0.0, 1.0, 3).unwrap();
        assert!(channel.advance(0.5, |target| target != 2).is_empty());
        assert_eq!(
            channel.advance(3.0, |_| true),
            [(1, 1.0), (3, 1.0), (1, 2.0), (3, 2.0), (1, 3.0), (3, 3.0)]
        );
        let mut lost = Channel::new([1], 0.0, 1.0, 3).unwrap();
        assert!(lost.advance(10.0, |_| false).is_empty());
        assert!(lost.finished());
    }

    #[test]
    fn invalid_clocks_cannot_consume_or_replay_pulses() {
        for (now, every, ticks) in [
            (f32::NAN, 1.0, 3),
            (-1.0, 1.0, 3),
            (0.0, f32::INFINITY, 3),
            (0.0, 0.0, 3),
            (0.0, -1.0, 3),
            (0.0, 1.0, 0),
            (f32::MAX, 1.0, 3),
        ] {
            assert!(Channel::new([1], now, every, ticks).is_none());
        }
        assert!(Channel::<u32>::new([], 0.0, 1.0, 3).is_none());
        let mut channel = Channel::new([1], 0.0, 1.0, 3).unwrap();
        assert_eq!(channel.advance(1.0, |_| true), [(1, 1.0)]);
        for now in [0.0, f32::NAN, f32::INFINITY, -1.0] {
            assert!(channel.advance(now, |_| false).is_empty());
        }
        assert_eq!(channel.advance(2.0, |_| true), [(1, 2.0)]);
        channel.cancel();
        assert!(channel.finished());
        assert!(channel.advance(3.0, |_| true).is_empty());
    }
}
