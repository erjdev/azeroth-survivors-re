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
        app.init_resource::<ModelSubject>()
            .add_systems(
                Update,
                feed.after(crate::ui_script::UiPaint)
                    .before(super::sync_inspect_booth),
            )
            .add_systems(
                PostUpdate,
                fit_equipment
                    .after(bevy::transform::TransformSystems::Propagate)
                    .after(bevy::camera::visibility::VisibilitySystems::CalculateBounds)
                    .before(bevy::camera::visibility::VisibilitySystems::UpdateFrusta),
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

/// Keep the authored body composition, backing away only when rigid equipment needs room.
/// Skinned bounds are bind-pose bounds, so they must not determine the live body's framing.
fn fit_equipment(
    nodes: Query<(), With<ModelPane>>,
    subject: Res<ModelSubject>,
    booths: Res<super::Booths>,
    panes: Res<BoothPanes>,
    creatures: Option<Res<crate::entities::Creatures>>,
    entities: Query<&crate::net::NetEntity>,
    children: Query<&Children>,
    parts: Query<
        (&bevy::camera::primitives::Aabb, &GlobalTransform),
        (
            Without<super::BoothCam>,
            Without<benilla_world::rig_palette::RigPart>,
        ),
    >,
    mut cameras: Query<(&super::BoothCam, &mut Transform, &mut GlobalTransform)>,
) {
    if nodes.is_empty() {
        return;
    }
    let Some(unit) = subject.0 else { return };
    let Some(booth) = booths.0.get(INSPECT_SLOT) else {
        return;
    };
    let Some(&aspect) = panes.0.get(INSPECT_SLOT) else {
        return;
    };
    let display = entities.get(unit).ok().and_then(|n| n.display_id);
    let Some(anchors) = super::booth_anchors(creatures.as_deref(), display) else {
        return;
    };
    let (mut camera, projection) = super::framing::body_frame(&anchors, aspect);
    let clip = projection.get_clip_from_view();
    let mut retreat = 0.0_f32;
    for entity in children.iter_descendants(booth.root) {
        let Ok((bounds, world)) = parts.get(entity) else {
            continue;
        };
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    let local = Vec3::from(bounds.center)
                        + Vec3::from(bounds.half_extents) * Vec3::new(x, y, z);
                    let point = camera.rotation.inverse()
                        * (world.transform_point(local) - camera.translation);
                    retreat = retreat.max(required_retreat(point, clip));
                }
            }
        }
    }
    camera.translation += camera.rotation * Vec3::Z * retreat;
    for (slot, mut local, mut global) in &mut cameras {
        if slot.0 == INSPECT_SLOT {
            // Booth cameras have no parent. Propagation already ran for the equipment;
            // update both transforms before camera/frustum extraction.
            *local = camera;
            *global = GlobalTransform::from(camera);
        }
    }
}

fn required_retreat(point: Vec3, clip: Mat4) -> f32 {
    // Leave 8% of each half-axis clear for small attachment animation/glow excursions.
    let depth =
        (point.x.abs() * clip.x_axis.x.abs()).max(point.y.abs() * clip.y_axis.y.abs()) / 0.92;
    (point.z + depth.max(0.1)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_corners_fit_across_aspects_and_depths() {
        for aspect in [0.5, 1.0, 1.6, 2.4] {
            let clip = Mat4::perspective_infinite_reverse_rh(0.7, aspect, 0.03);
            for point in [
                Vec3::new(3.0, 1.0, -2.0),
                Vec3::new(-2.0, 4.0, 0.5),
                Vec3::new(0.0, 0.0, -5.0),
            ] {
                let retreat = required_retreat(point, clip);
                let fitted = point - Vec3::Z * retreat;
                let projected = clip * fitted.extend(1.0);
                let ndc = projected.truncate() / projected.w;
                assert!(ndc.x.abs() <= 0.92001 && ndc.y.abs() <= 0.92001);
                assert!(fitted.z < -0.03);
            }
        }
    }

    #[test]
    fn equipment_already_inside_the_frame_does_not_zoom_out() {
        let clip = Mat4::perspective_infinite_reverse_rh(0.7, 1.6, 0.03);
        assert_eq!(required_retreat(Vec3::new(0.1, 0.2, -5.0), clip), 0.0);
    }
}
