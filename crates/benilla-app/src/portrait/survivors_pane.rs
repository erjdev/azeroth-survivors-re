//! Survivors' Bevy UI bridge to the existing full-body inspect renderer.
use super::{BoothPanes, InspectBooth, PortraitImages, PortraitSource, INSPECT_SLOT};
use bevy::input::ButtonState;
use bevy::prelude::*;

use bevy::window::{PrimaryWindow, WindowEvent};

type RigidAttachmentFilter = (
    Without<super::BoothCam>,
    Without<benilla_world::rig_palette::RigPart>,
);

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
    loading: Res<crate::loading_screen::LoadingScreen>,
    images: Res<PortraitImages>,
    mut panes: ResMut<BoothPanes>,
    mut inspect: ResMut<InspectBooth>,
    mut nodes: Query<
        (
            &ComputedNode,
            &UiGlobalTransform,
            Option<&CalculatedClip>,
            &mut ImageNode,
        ),
        With<ModelPane>,
    >,
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut events: MessageReader<WindowEvent>,
    clicks: Res<crate::glue::GlueClicks>,
    turns: Query<(Entity, &ModelTurn)>,
    mut drag: Local<DragState>,
    mut yaw: Local<Option<f32>>,
    mut owned: Local<bool>,
) {
    let Ok((node, transform, clip, mut image)) = nodes.single_mut() else {
        if *owned {
            inspect.unit = None;
            panes.0.remove(INSPECT_SLOT);
            *owned = false;
        }
        *drag = DragState::default();
        events.clear();
        return;
    };
    if node.size.x <= 0.0 || node.size.y <= 0.0 {
        inspect.unit = None;
        panes.0.remove(INSPECT_SLOT);
        *drag = DragState::default();
        events.clear();
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
    // WindowEvent also carries OS lifecycle messages, so the global cover cannot
    // clear that mixed stream. Drop only this consumer’s pending gestures.
    if !drag.accept_input(loading.covering(), &mut events) {
        return;
    }
    let angle = yaw.get_or_insert(0.61);
    for (entity, turn) in &turns {
        if clicks.hit(entity) {
            *angle += turn.0;
        }
    }
    // Consume the ordered stream: a complete short drag can arrive in one frame,
    // when ButtonInput and RelativeCursorPosition retain only its final state.
    if let Ok((window_entity, window)) = windows.single() {
        for event in events.read() {
            match event {
                WindowEvent::CursorMoved(event) if event.window == window_entity => {
                    let point = event.position * window.scale_factor();
                    let x = node.normalize_point(*transform, point).map(|p| p.x);
                    let inside = node.contains_point(*transform, point)
                        && clip.is_none_or(|c| c.clip.contains(point));
                    *angle += drag.move_to(x, inside);
                }
                WindowEvent::MouseButtonInput(event)
                    if event.window == window_entity && event.button == MouseButton::Left =>
                {
                    drag.button(event.state == ButtonState::Pressed);
                }
                WindowEvent::WindowFocused(event)
                    if event.window == window_entity && !event.focused =>
                {
                    *drag = DragState::default();
                }
                WindowEvent::CursorLeft(event) if event.window == window_entity => {
                    *drag = DragState::default();
                }
                _ => {}
            }
        }
    } else {
        events.clear();
        *drag = DragState::default();
    }
    *angle = angle.rem_euclid(std::f32::consts::TAU);
    inspect.yaw = *angle;
}

/// Movement is measured in pane widths, independent of resolution and display scale.
#[derive(Default)]
struct DragState {
    x: Option<f32>,
    inside: bool,
    held: bool,
}
impl DragState {
    fn accept_input(&mut self, covered: bool, events: &mut MessageReader<WindowEvent>) -> bool {
        if covered {
            *self = Self::default();
            events.clear();
            false
        } else {
            true
        }
    }

    fn move_to(&mut self, x: Option<f32>, inside: bool) -> f32 {
        let delta = if self.held {
            self.x
                .zip(x)
                .map_or(0.0, |(old, new)| (new - old) * std::f32::consts::TAU)
        } else {
            0.0
        };
        self.x = x;
        self.inside = inside;
        delta
    }
    fn button(&mut self, pressed: bool) {
        self.held = pressed && self.inside && self.x.is_some();
    }
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
    parts: Query<(&bevy::camera::primitives::Aabb, &GlobalTransform), RigidAttachmentFilter>,
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
    fn survivors_drag_keeps_short_gestures_and_rejects_outside_presses() {
        let mut drag = DragState::default();
        assert_eq!(drag.move_to(Some(0.0), true), 0.0);
        drag.button(true);
        let delta = drag.move_to(Some(0.25), true);
        drag.button(false);
        assert!((delta - std::f32::consts::FRAC_PI_2).abs() < 0.0001);
        assert_eq!(drag.move_to(Some(0.4), true), 0.0);
        drag.move_to(Some(0.8), false);
        drag.button(true);
        assert_eq!(drag.move_to(Some(0.1), true), 0.0);
        drag.button(false);
        drag.button(true);
        assert!(drag.move_to(Some(-0.8), false) < 0.0);
        drag.button(false);
        assert_eq!(drag.move_to(Some(0.0), true), 0.0);
    }

    #[test]
    fn loading_discards_portrait_gestures_without_consuming_os_events() {
        use bevy::ecs::system::SystemState;
        use bevy::window::{CursorMoved, WindowCloseRequested};
        let mut world = World::new();
        world.init_resource::<Messages<WindowEvent>>();
        let window = world.spawn_empty().id();
        let mut reader = SystemState::<MessageReader<WindowEvent>>::new(&mut world);
        let mut os_reader = world.resource::<Messages<WindowEvent>>().get_cursor();
        let mut drag = DragState::default();
        drag.move_to(Some(0.0), true);
        drag.button(true);
        world.resource_mut::<Messages<WindowEvent>>().write_batch([
            WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(100.0, 20.0),
                delta: Some(Vec2::X),
            }),
            WindowEvent::WindowCloseRequested(WindowCloseRequested { window }),
        ]);
        {
            let mut events = reader.get_mut(&mut world);
            assert!(!drag.accept_input(true, &mut events));
            assert_eq!(
                events.read().count(),
                0,
                "no delayed drag after the cover drops"
            );
        }
        let messages = world.resource::<Messages<WindowEvent>>();
        assert!(os_reader
            .read(messages)
            .any(|e| matches!(e, WindowEvent::WindowCloseRequested(_))));
        {
            let mut events = reader.get_mut(&mut world);
            assert!(drag.accept_input(false, &mut events));
            assert_eq!(events.read().count(), 0);
        }
        assert_eq!(
            drag.move_to(Some(0.5), true),
            0.0,
            "old held gesture must not resume"
        );
        drag.button(true);
        assert!(
            drag.move_to(Some(0.6), true) > 0.0,
            "a fresh gesture still works"
        );
    }

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
