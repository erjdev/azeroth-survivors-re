//! Synchronize target-owned disarm clocks with the existing stock attachment/animation flag.
use super::{enemies::Enemy, Run};
use crate::{creature_anim::UNIT_FLAG_DISARMED, net::ObjectStore};
use benilla_protocol::ObjectFields;
use bevy::prelude::*;
use survivors_core::disarm::{eligibility, Disarm, Eligibility};

#[derive(Component, Default)]
pub(super) struct DisarmState(pub Disarm);

/// Change only the disarm bit, and leave Bevy change detection untouched on stable frames.
pub(super) fn flag(mut store: Mut<ObjectStore>, active: bool) {
    const UNIT_FLAGS: u16 = 46;
    let old = store.0.unit_flags();
    let next = if active {
        old | UNIT_FLAG_DISARMED
    } else {
        old & !UNIT_FLAG_DISARMED
    };
    if old != next {
        store
            .0
            .merge(ObjectFields::from_pairs(&[(UNIT_FLAGS, next)]));
    }
}

pub(super) fn sync(
    run: Res<Run>,
    mut enemies: Query<(&Enemy, &mut DisarmState, &mut ObjectStore)>,
) {
    for (enemy, mut state, store) in &mut enemies {
        let eligible = !enemy.dead
            && enemy.hp > 0.0
            && eligibility(enemy.def.template_entry) == Eligibility::Weapon;
        if !eligible && state.0 != Disarm::default() {
            state.0 = Disarm::default();
        }
        flag(store, eligible && state.0.active(run.elapsed));
    }
}
