//! Independent enemy-owned Roar; only the physical-contact path consumes it.
use bevy::prelude::*;

#[derive(Component, Default)]
pub(super) struct Roar(pub survivors_core::roar::Roar);
