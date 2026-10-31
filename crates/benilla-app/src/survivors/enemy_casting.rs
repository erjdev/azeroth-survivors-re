//! First roster-backed enemy caster. The chase supplies attention and connectivity.
use bevy::prelude::*;
use survivors_core::casting::{Casting, School, Spell};

pub(super) const FROSTBOLT: Spell = Spell {
    id: 13322,
    school: School::Frost,
    seconds: 3.,
    interruptible: true,
};
pub(super) const RANGE: f32 = 40.;
pub(super) const SLOW: survivors_core::slows::Source =
    survivors_core::slows::Source::Spell("enemy_frostbolt");

#[derive(Component, Default)]
pub(super) struct Caster {
    pub state: Casting<Entity>,
    ready_at: f32,
}

impl Caster {
    /// Pinned creature 1522 → spell list 15220 → spell 13322. Other templates
    /// remain melee until their own spells are authored and validated.
    pub fn enabled(template: u32) -> bool {
        template == 1522
    }

    /// Return (started, completed, cancelled). A target switch cancels instead
    /// of redirecting an already-started spell onto a different victim.
    pub fn step(&mut self, target: Entity, now: f32, connected: bool) -> (bool, bool, bool) {
        let before = self.state.active();
        let completed = self
            .state
            .poll(now, |original| connected && original == target)
            .is_some();
        let cancelled = before.is_some() && self.state.active().is_none() && !completed;
        if completed || cancelled {
            // Survivors recovery follows settlement/cancellation; no burst catch-up casts.
            self.ready_at = now + 2.;
        }
        let started = connected && now >= self.ready_at && self.state.start(FROSTBOLT, target, now);
        (started, completed, cancelled)
    }
}
