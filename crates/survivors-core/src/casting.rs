//! Active-run spell casting. Each caster owns one state; targets retain their identity.
//! The caller supplies installed/authored timings, validates immunity and connection,
//! and settles completed spells. This module does not supply damage or proc policy.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum School {
    Holy,
    Fire,
    Nature,
    Frost,
    Shadow,
    Arcane,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spell {
    pub id: u32,
    pub school: School,
    pub seconds: f32,
    /// Cast interruption immunity is separate from silence immunity, owned by the caller.
    pub interruptible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cast<T> {
    pub spell: Spell,
    pub target: T,
    pub started: f32,
    pub finishes: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interrupt {
    Invalid,
    NotCasting,
    TooLate,
    Immune,
    Interrupted(u32),
}

#[derive(Clone, Debug)]
pub struct Casting<T> {
    active: Option<Cast<T>>,
    observed: f32,
    locked_until: [f32; 6],
    silenced_until: f32,
}

impl<T> Default for Casting<T> {
    fn default() -> Self {
        Self {
            active: None,
            observed: 0.,
            locked_until: [0.; 6],
            silenced_until: 0.,
        }
    }
}

fn deadline(now: f32, seconds: f32) -> Option<f32> {
    let end = now + seconds;
    (now.is_finite()
        && now >= 0.
        && seconds.is_finite()
        && seconds > 0.
        && end.is_finite()
        && end > now)
        .then_some(end)
}

impl<T: Copy> Casting<T> {
    fn observe(&mut self, now: f32) -> bool {
        if !now.is_finite() || now < self.observed {
            return false;
        }
        self.observed = now;
        true
    }

    pub fn active(&self) -> Option<Cast<T>> {
        self.active
    }

    /// A read-only eligibility query. Pending completions must be polled before a new cast.
    pub fn available(&self, school: School, now: f32) -> bool {
        now.is_finite()
            && now >= self.observed
            && self.active.is_none()
            && now >= self.silenced_until
            && now >= self.locked_until[school as usize]
    }

    /// No recovery is spent here. A rejected start leaves the current cast untouched.
    /// Instant spells finish on their first poll, exactly once.
    pub fn start(&mut self, spell: Spell, target: T, now: f32) -> bool {
        let finishes = if spell.seconds == 0. {
            now
        } else if let Some(end) = deadline(now, spell.seconds) {
            end
        } else {
            return false;
        };
        if spell.id == 0 || !self.available(spell.school, now) {
            return false;
        }
        self.observed = now;
        self.active = Some(Cast {
            spell,
            target,
            started: now,
            finishes,
        });
        true
    }

    /// Call every active update with current caster/target validity, including line of sight.
    /// Losing connection cancels permanently; a late frame emits only one completion,
    /// retaining its scheduled timestamp instead of pretending the cast finished now.
    pub fn poll(&mut self, now: f32, mut connected: impl FnMut(T) -> bool) -> Option<Cast<T>> {
        if !self.observe(now) {
            return None;
        }
        let cast = self.active?;
        if !connected(cast.target) {
            self.active = None;
            return None;
        }
        if now < cast.finishes {
            return None;
        }
        self.active.take()
    }

    /// An interrupt locks only the interrupted school, and only if it succeeds.
    /// At the exact completion time it is too late, independent of system call order.
    pub fn interrupt(&mut self, now: f32, lock_seconds: f32) -> Interrupt {
        let end = if lock_seconds == 0. {
            now
        } else if let Some(end) = deadline(now, lock_seconds) {
            end
        } else {
            return Interrupt::Invalid;
        };
        if !self.observe(now) {
            return Interrupt::Invalid;
        }
        let Some(cast) = self.active else {
            return Interrupt::NotCasting;
        };
        if now >= cast.finishes {
            return Interrupt::TooLate;
        }
        if !cast.spell.interruptible {
            return Interrupt::Immune;
        }
        let until = &mut self.locked_until[cast.spell.school as usize];
        *until = until.max(end);
        self.active = None;
        Interrupt::Interrupted(cast.spell.id)
    }

    /// Silence blocks all magic schools even while idle. A shorter reapplication cannot
    /// shorten an existing silence. Already completed pending spells are not erased.
    /// Apply only after the caller resolves silence immunity/resistance.
    pub fn silence(&mut self, now: f32, seconds: f32) -> bool {
        let Some(end) = deadline(now, seconds) else {
            return false;
        };
        if !self.observe(now) {
            return false;
        }
        self.silenced_until = self.silenced_until.max(end);
        if self.active.is_some_and(|cast| now < cast.finishes) {
            self.active = None;
        }
        true
    }

    /// Death/despawn/mode exit discards unpaid work. Control expiry is not refreshed.
    pub fn cancel(&mut self) {
        self.active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spell(school: School) -> Spell {
        Spell {
            id: 123,
            school,
            seconds: 2.,
            interruptible: true,
        }
    }

    #[test]
    fn completion_keeps_target_and_deadline_and_is_paid_once() {
        let mut state = Casting::default();
        assert!(state.start(spell(School::Fire), 7, 10.));
        assert!(!state.start(spell(School::Frost), 8, 10.));
        for now in [10., 10., 11.99] {
            assert!(state.poll(now, |_| true).is_none());
        }
        let paid = state.poll(20., |target| target == 7).unwrap();
        assert_eq!((paid.target, paid.started, paid.finishes), (7, 10., 12.));
        assert!(state.poll(20., |_| true).is_none());
        assert!(state.available(School::Fire, 20.));
    }

    #[test]
    fn interruption_locks_only_successfully_interrupted_school() {
        let mut state = Casting::default();
        assert_eq!(state.interrupt(0., 10.), Interrupt::NotCasting);
        assert!(state.start(spell(School::Fire), 7, 0.));
        assert_eq!(state.interrupt(1., 10.), Interrupt::Interrupted(123));
        assert!(state.poll(2., |_| true).is_none());
        assert!(!state.available(School::Fire, 10.99));
        assert!(state.available(School::Fire, 11.));
        for school in [
            School::Holy,
            School::Nature,
            School::Frost,
            School::Shadow,
            School::Arcane,
        ] {
            assert!(state.available(school, 2.));
        }
        assert!(state.start(spell(School::Frost), 8, 2.));
        assert_eq!(state.poll(4., |_| true).unwrap().target, 8);
    }

    #[test]
    fn silence_blocks_all_schools_without_inventing_a_school_lock() {
        let mut state = Casting::default();
        assert!(state.silence(0., 4.));
        assert!(state.silence(1., 1.));
        for school in [
            School::Holy,
            School::Fire,
            School::Nature,
            School::Frost,
            School::Shadow,
            School::Arcane,
        ] {
            assert!(!state.start(spell(school), 7, 3.99));
            assert!(state.available(school, 4.));
        }
        let mut immune = spell(School::Fire);
        immune.interruptible = false;
        assert!(state.start(immune, 7, 4.));
        assert_eq!(state.interrupt(5., 10.), Interrupt::Immune);
        assert!(state.silence(5., 1.));
        assert!(state.poll(6., |_| true).is_none());
        assert!(state.available(School::Fire, 6.));
    }

    #[test]
    fn school_lock_and_silence_expire_independently() {
        let mut state = Casting::default();
        assert!(state.start(spell(School::Shadow), 1, 0.));
        assert_eq!(state.interrupt(1., 10.), Interrupt::Interrupted(123));
        assert!(state.silence(2., 4.));
        assert!(!state.available(School::Fire, 5.));
        assert!(state.available(School::Fire, 6.));
        assert!(!state.available(School::Shadow, 6.));
        assert!(state.available(School::Shadow, 11.));
    }

    #[test]
    fn deadline_controls_cannot_retroactively_cancel_completion() {
        for silence in [false, true] {
            let mut state = Casting::default();
            assert!(state.start(spell(School::Fire), 7, 0.));
            if silence {
                assert!(state.silence(2., 4.));
            } else {
                assert_eq!(state.interrupt(2., 10.), Interrupt::TooLate);
            }
            assert_eq!(state.poll(2., |_| true).unwrap().target, 7);
            assert_eq!(state.available(School::Fire, 2.), !silence);
        }
    }

    #[test]
    fn lost_targets_cancel_and_cannot_reappear_in_this_cast() {
        let mut state = Casting::default();
        assert!(state.start(spell(School::Fire), 7, 0.));
        assert!(state.poll(1., |_| false).is_none());
        assert!(state.poll(2., |_| true).is_none());
        assert!(state.start(spell(School::Fire), 8, 2.));
        state.cancel();
        assert!(state.poll(4., |_| true).is_none());
    }

    #[test]
    fn invalid_and_stale_operations_cannot_erase_or_replay_casts() {
        let mut state = Casting::default();
        assert!(state.start(spell(School::Fire), 7, 10.));
        for now in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1., 9.] {
            assert!(state.poll(now, |_| false).is_none());
            assert_eq!(state.interrupt(now, 3.), Interrupt::Invalid);
            assert!(!state.silence(now, 3.));
        }
        for seconds in [f32::NAN, f32::INFINITY, -1.] {
            assert_eq!(state.interrupt(11., seconds), Interrupt::Invalid);
            assert!(!state.silence(11., seconds));
        }
        assert_eq!(state.poll(12., |_| true).unwrap().target, 7);
        assert!(!state.start(spell(School::Fire), 7, 11.));
    }

    #[test]
    fn instant_and_zero_lockout_casts_do_not_invent_delays() {
        let mut state = Casting::default();
        let instant = Spell {
            seconds: 0.,
            ..spell(School::Holy)
        };
        assert!(state.start(instant, 7, 0.));
        assert_eq!(state.interrupt(0., 10.), Interrupt::TooLate);
        assert!(state.poll(0., |_| true).is_some());
        assert!(state.poll(0., |_| true).is_none());
        assert!(state.start(spell(School::Holy), 7, 0.));
        assert_eq!(state.interrupt(1., 0.), Interrupt::Interrupted(123));
        assert!(state.start(spell(School::Holy), 7, 1.));
    }

    #[test]
    fn invalid_casts_and_unrepresentable_deadlines_are_rejected() {
        let mut state = Casting::default();
        for seconds in [-1., f32::NAN, f32::INFINITY] {
            assert!(!state.start(
                Spell {
                    seconds,
                    ..spell(School::Fire)
                },
                7,
                0.
            ));
        }
        assert!(!state.start(
            Spell {
                id: 0,
                ..spell(School::Fire)
            },
            7,
            0.
        ));
        assert!(!state.start(spell(School::Fire), 7, f32::MAX));
        assert!(!state.silence(f32::MAX, 1.));
        assert!(state.start(spell(School::Fire), 7, 0.));
    }
}
