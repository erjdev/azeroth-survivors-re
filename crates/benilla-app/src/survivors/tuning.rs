//! Runtime layout shared by the player UI and development editor.
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct UiLayout {
    pub value: survivors_core::layout::Layout,
    pub version: u64,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UiLayout>();
}
