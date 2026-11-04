//! Independent solo-caster movement slows, measured against active run time.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Spell(&'static str),
    Secondary(&'static str),
    Aftermath,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Slow {
    source: Source,
    speed: f32,
    applied: f32,
    expires: f32,
}

/// One entry per authored source. Expired entries retain their ordering watermark,
/// so a delayed older application cannot replace a newer, already-expired effect.
/// Storage grows only when a distinct spell/talent source first affects this target.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Slows {
    entries: Vec<Slow>,
    cleared_at: Option<f32>,
}

impl Slows {
    pub fn apply(&mut self, source: Source, speed: f32, seconds: f32, now: f32) -> bool {
        let expires = now + seconds;
        if !speed.is_finite()
            || !(0.0..=1.0).contains(&speed)
            || !now.is_finite()
            || now < 0.0
            || self.cleared_at.is_some_and(|cleared| now < cleared)
            || !expires.is_finite()
            || expires <= now
        {
            return false;
        }
        let next = Slow {
            source,
            speed,
            applied: now,
            expires,
        };
        if let Some(old) = self.entries.iter_mut().find(|slow| slow.source == source) {
            if now < old.applied {
                return false;
            }
            *old = next;
        } else {
            self.entries.push(next);
        }
        true
    }

    /// Remove current snares and zero-speed roots without granting future immunity.
    /// Keep a watermark for delayed applications, including previously unseen sources.
    pub fn cleanse(&mut self, now: f32) -> bool {
        if !now.is_finite() || now < 0.0 || self.cleared_at.is_some_and(|last| now < last) {
            return false;
        }
        self.cleared_at = Some(now);
        for slow in &mut self.entries {
            if slow.applied <= now {
                slow.expires = slow.expires.min(now);
            }
        }
        true
    }

    /// Strongest active speed factor and its own remaining duration. Equal-strength
    /// sources remain effective until the later expiry; unrelated durations never mix.
    pub fn state(&self, now: f32) -> (f32, f32) {
        if !now.is_finite() || now < 0.0 {
            return (1.0, 0.0);
        }
        self.entries
            .iter()
            .filter(|slow| now >= slow.applied && now < slow.expires && slow.speed < 1.0)
            .fold((1.0, 0.0), |(speed, left), slow| {
                if slow.speed < speed {
                    (slow.speed, slow.expires - now)
                } else if slow.speed == speed {
                    (speed, left.max(slow.expires - now))
                } else {
                    (speed, left)
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FROST: Source = Source::Spell("frostbolt");
    const CLAP: Source = Source::Spell("thunder_clap");

    #[test]
    fn cleanse_removes_overlaps_and_roots_rejects_stale_hits_but_allows_new_effects() {
        let mut slows = Slows::default();
        slows.apply(FROST, 0.5, 20., 1.);
        slows.apply(CLAP, 0.8, 30., 2.);
        slows.apply(Source::Secondary("root"), 0., 10., 3.);
        assert_eq!(slows.state(4.), (0., 9.));
        assert!(slows.cleanse(4.));
        assert_eq!(slows.state(4.), (1., 0.));
        assert!(!slows.apply(FROST, 0.5, 20., 3.));
        assert!(!slows.apply(Source::Aftermath, 0.5, 20., 3.));
        let saved = slows.clone();
        for now in [f32::NAN, f32::INFINITY, -1., 3.] {
            assert!(!slows.cleanse(now));
            assert_eq!(slows, saved);
        }
        assert!(slows.apply(FROST, 0.5, 2., 4.));
        assert_eq!(slows.state(4.), (0.5, 2.));
        assert_eq!(slows.state(6.), (1., 0.));
        assert!(slows.cleanse(6.));
        assert!(slows.apply(CLAP, 0.8, 5., 7.));
        assert_eq!(slows.state(7.), (0.8, 5.));
    }

    #[test]
    fn overlapping_strengths_expire_independently_in_both_application_orders() {
        for reverse in [false, true] {
            let mut slows = Slows::default();
            let mut applications = [(FROST, 0.5, 2.5), (CLAP, 0.8, 10.)];
            if reverse {
                applications.reverse();
            }
            for (source, speed, duration) in applications {
                assert!(slows.apply(source, speed, duration, 0.));
            }
            assert_eq!(slows.state(0.), (0.5, 2.5));
            assert_eq!(slows.state(2.5), (0.8, 7.5));
            assert_eq!(slows.state(10.), (1., 0.));
        }
    }

    #[test]
    fn refresh_replaces_only_its_source_and_stale_applications_cannot_revive_it() {
        let mut slows = Slows::default();
        slows.apply(FROST, 0.2, 20., 0.);
        slows.apply(CLAP, 0.8, 30., 0.);
        slows.apply(FROST, 0.5, 2., 1.);
        assert_eq!(slows.state(1.), (0.5, 2.));
        assert_eq!(slows.state(3.), (0.8, 27.));
        assert!(!slows.apply(FROST, 0.1, 50., 0.5));
        assert_eq!(slows.state(3.), (0.8, 27.));
        for _ in 0..100 {
            slows.apply(FROST, 0.5, 2., 4.);
        }
        assert_eq!(slows.entries.len(), 2);
        assert_eq!(slows.state(4.), (0.5, 2.)); // A paused clock does not age state.
        assert_eq!(slows.state(6.), (0.8, 24.));
    }

    #[test]
    fn secondary_and_proc_sources_are_independent_and_invalid_inputs_do_not_mutate() {
        let mut slows = Slows::default();
        slows.apply(FROST, 0.5, 2., 0.);
        slows.apply(Source::Secondary("frostbolt"), 0.8, 5., 0.);
        slows.apply(Source::Aftermath, 0.5, 3., 0.);
        assert_eq!(slows.state(0.), (0.5, 3.));
        assert_eq!(slows.state(3.), (0.8, 2.));
        let saved = slows.clone();
        for (speed, secs, now) in [
            (f32::NAN, 1., 0.),
            (-0.1, 1., 0.),
            (1.1, 1., 0.),
            (0.5, 0., 0.),
            (0.5, -1., 0.),
            (0.5, f32::INFINITY, 0.),
            (0.5, 1., f32::NAN),
            (0.5, 1., -1.),
            (0.5, 1., f32::MAX),
        ] {
            assert!(!slows.apply(FROST, speed, secs, now));
            assert_eq!(slows, saved);
        }
        for now in [f32::NAN, f32::INFINITY, -1.] {
            assert_eq!(slows.state(now), (1., 0.));
        }
        assert_eq!(Slows::default().state(0.), (1., 0.));
    }
}
