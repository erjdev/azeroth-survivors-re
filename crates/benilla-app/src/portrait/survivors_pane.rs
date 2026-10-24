//! Survivors' Bevy UI bridge to the existing full-body inspect renderer.
use super::{BoothPanes, InspectBooth, PortraitImages, PortraitSource, INSPECT_SLOT};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

#[derive(Component)]
pub(crate) struct ModelPane;

#[derive(Component)]
pub(crate) struct ModelTurn(pub f32);

#[derive(Resource, Default)]
pub(crate) struct ModelSubject(pub Option<Entity>);

pub(crate) struct SurvivorsPanePlugin;
impl Plugin for SurvivorsPanePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModelSubject>().add_systems(
            Update,
            feed.after(crate::ui_script::UiPaint)
                .before(super::sync_inspect_booth),
        );
    }
}

fn feed(
    subject: Res<ModelSubject>,
    images: Res<PortraitImages>,
    mut panes: ResMut<BoothPanes>,
    mut inspect: ResMut<InspectBooth>,
    mut nodes: Query<(&ComputedNode, &RelativeCursorPosition, &mut ImageNode), With<ModelPane>>,
    mouse: Res<ButtonInput<MouseButton>>,
    clicks: Res<crate::glue::GlueClicks>,
    turns: Query<(Entity, &ModelTurn)>,
    mut previous: Local<Option<f32>>,
    mut dragging: Local<bool>,
    mut yaw: Local<Option<f32>>,
    mut owned: Local<bool>,
) {
    let Ok((node, cursor, mut image)) = nodes.single_mut() else {
        if *owned {
            inspect.unit = None;
            panes.0.remove(INSPECT_SLOT);
            *owned = false;
        }
        *dragging = false;
        return;
    };
    if node.size.x <= 0.0 || node.size.y <= 0.0 {
        inspect.unit = None;
        panes.0.remove(INSPECT_SLOT);
        *dragging = false;
        return;
    }
    *owned = true;
    inspect.unit = subject.0;
    image.color = if subject.0.is_some() {
        Color::WHITE
    } else {
        Color::NONE
    };
    panes
        .0
        .insert(INSPECT_SLOT.into(), node.size.x / node.size.y);
    if let Some(PortraitSource::Live(target)) = images.0.get(INSPECT_SLOT) {
        if image.image != *target {
            image.image = target.clone();
        }
    }
    let angle = yaw.get_or_insert(0.61);
    for (entity, turn) in &turns {
        if clicks.hit(entity) {
            *angle += turn.0;
        }
    }
    let x = cursor.normalized.map(|p| p.x);
    if mouse.just_pressed(MouseButton::Left) {
        *dragging = cursor.cursor_over;
        *previous = x;
    }
    if *dragging {
        if let (Some(last), Some(now)) = (*previous, x) {
            *angle += (now - last) * std::f32::consts::TAU;
        }
        *previous = x;
    }
    if !mouse.pressed(MouseButton::Left) {
        *dragging = false;
        *previous = None;
    }
    *angle = angle.rem_euclid(std::f32::consts::TAU);
    inspect.yaw = *angle;
}
