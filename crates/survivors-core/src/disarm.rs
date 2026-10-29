//! Target-owned disarm clocks, independent of damage policy and rendered appearance.
use crate::enemy_equipment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Eligibility {
    Weapon,
    NoMainHandWeapon,
    Immune,
    Unknown,
}

/// Survivors policy: a verified main-hand weapon, excluding explicit mechanic immunity.
/// Natural attacks, offhand-only props and ranged-only equipment cannot establish eligibility.
pub fn eligibility(template: u32) -> Eligibility {
    let (Some(items), Some(mask)) = (
        enemy_equipment::for_template(template),
        enemy_equipment::mechanic_immunity(template),
    ) else {
        return Eligibility::Unknown;
    };
    // MaNGOS SharedDefines: MECHANIC_DISARM = 3, masks use 1 << (mechanic - 1).
    if mask & (1 << 2) != 0 {
        Eligibility::Immune
    } else if items.iter().any(|item| item.0 == 0 && item.2 == 2) {
        Eligibility::Weapon
    } else {
        Eligibility::NoMainHandWeapon
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Disarm,
    Riposte,
}

impl Source {
    fn index(self) -> usize {
        match self {
            Self::Disarm => 0,
            Self::Riposte => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Effect {
    applied: f32,
    expires: f32,
}

/// Each source retains its own expiry and ordering watermark until the target is removed.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Disarm([Option<Effect>; 2]);

impl Disarm {
    /// Caller supplies active run time, so pause does not advance these clocks.
    /// Same-source refresh replaces; a second source never borrows or shortens its duration.
    pub fn apply(&mut self, target: Eligibility, source: Source, seconds: f32, now: f32) -> bool {
        let expires = now + seconds;
        let slot = &mut self.0[source.index()];
        if target != Eligibility::Weapon
            || !now.is_finite()
            || now < 0.0
            || !seconds.is_finite()
            || !expires.is_finite()
            || expires <= now
            || slot.is_some_and(|old| now < old.applied)
        {
            return false;
        }
        *slot = Some(Effect {
            applied: now,
            expires,
        });
        true
    }

    pub fn remaining(self, now: f32) -> f32 {
        if !now.is_finite() || now < 0.0 {
            return 0.0;
        }
        self.0
            .into_iter()
            .flatten()
            .filter(|effect| now >= effect.applied && now < effect.expires)
            .map(|effect| effect.expires - now)
            .fold(0.0, f32::max)
    }

    pub fn active(self, now: f32) -> bool {
        self.remaining(now) > 0.0
    }

    /// Dispel one source without reviving an older delayed application of that source.
    pub fn remove(&mut self, source: Source) {
        if let Some(effect) = &mut self.0[source.index()] {
            effect.expires = effect.applied;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_and_immunity_are_independent_of_creature_name_or_type() {
        assert_eq!(eligibility(116), Eligibility::Weapon); // Defias Bandit
        assert_eq!(eligibility(98), Eligibility::Weapon); // sword/shield gnoll
        assert_eq!(eligibility(4306), Eligibility::Weapon); // legacy raw weapon
        assert_eq!(eligibility(3975), Eligibility::Immune); // Herod
        assert_eq!(eligibility(30), Eligibility::NoMainHandWeapon); // Forest Spider
        assert_eq!(eligibility(u32::MAX), Eligibility::Unknown);
    }

    #[test]
    fn overlapping_sources_refresh_and_expire_independently() {
        let mut status = Disarm::default();
        assert!(status.apply(Eligibility::Weapon, Source::Disarm, 10., 0.));
        assert!(status.apply(Eligibility::Weapon, Source::Riposte, 6., 1.));
        assert_eq!(status.remaining(1.), 9.);
        status.remove(Source::Disarm);
        assert_eq!(status.remaining(1.), 6.);
        assert!(status.apply(Eligibility::Weapon, Source::Disarm, 2., 2.));
        assert_eq!(status.remaining(4.), 3.);
        assert!(!status.active(7.));
        assert!(status.apply(Eligibility::Weapon, Source::Riposte, 6., 7.));
        assert_eq!(status.remaining(7.), 6.);
        assert!(!status.apply(Eligibility::Weapon, Source::Riposte, 100., 6.));
        assert!(!status.active(13.));
    }

    #[test]
    fn rejected_targets_and_invalid_times_cannot_mutate_status() {
        let mut status = Disarm::default();
        status.apply(Eligibility::Weapon, Source::Disarm, 10., 2.);
        let before = status;
        for target in [
            Eligibility::Immune,
            Eligibility::Unknown,
            Eligibility::NoMainHandWeapon,
        ] {
            assert!(!status.apply(target, Source::Disarm, 10., 3.));
        }
        for seconds in [0., -1., f32::NAN, f32::INFINITY] {
            assert!(!status.apply(Eligibility::Weapon, Source::Riposte, seconds, 3.));
        }
        for now in [-1., f32::NAN, f32::INFINITY, f32::MAX] {
            assert!(!status.apply(Eligibility::Weapon, Source::Riposte, 10., now));
        }
        assert_eq!(status, before);
        assert!(!status.active(f32::NAN));
        assert!(!status.active(1.));
        assert!(status.active(2.));
        assert!(!status.active(12.));
    }

    #[test]
    fn pause_target_isolation_and_removal_preserve_the_clock_contract() {
        let mut first = Disarm::default();
        let second = Disarm::default();
        first.apply(Eligibility::Weapon, Source::Riposte, 6., 10.);
        for _ in 0..10 {
            assert_eq!(first.remaining(10.), 6.);
        }
        assert!(!second.active(10.));
        first.remove(Source::Riposte);
        assert!(!first.active(10.));
        assert!(!first.apply(Eligibility::Weapon, Source::Riposte, 6., 9.));
        assert!(first.apply(Eligibility::Weapon, Source::Riposte, 6., 11.));
        assert_eq!(first.remaining(11.), 6.);
    }
}
