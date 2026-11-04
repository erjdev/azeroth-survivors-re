//! Native Survivors instruments; compiled only through the development capture root.
use bevy::prelude::*;
mod collision_probe;
mod screenshots;
mod tuning;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins((tuning::plugin, screenshots::plugin, collision_probe::plugin));
}
