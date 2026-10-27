//! Scrollbar siblings stay fixed while their content scrolls inside its viewport.
use bevy::{
    input::ButtonState,
    prelude::*,
    ui::RelativeCursorPosition,
    window::{PrimaryWindow, WindowEvent},
};

#[derive(Component)]
pub(in crate::survivors) struct Track(Entity);
#[derive(Component)]
pub(in crate::survivors) struct Thumb(Entity);

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, viewport: Entity) {
    parent
        .spawn((
            Track(viewport),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                bottom: Val::Px(14.0),
                right: Val::Px(4.0),
                width: Val::Px(8.0),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.02, 0.01, 0.85)),
            Visibility::Hidden,
            Interaction::default(),
            RelativeCursorPosition::default(),
            bevy::ui::FocusPolicy::Block,
            ZIndex(2),
        ))
        .with_children(|track| {
            track.spawn((
                Thumb(viewport),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.68, 0.51, 0.18)),
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

// Fractions of the track: thumb size, thumb top and maximum logical scroll offset.
fn geometry(node: &ComputedNode, position: f32) -> Option<(f32, f32, f32)> {
    let view = node.size.y;
    let content = node.content_size.y;
    if view <= 0.0 || !content.is_finite() || content <= view + 1.0 {
        return None;
    }
    let max = (content - view) * node.inverse_scale_factor;
    if max <= 0.0 {
        return None;
    }
    let size = (view / content).clamp(0.06, 1.0);
    Some((size, position.clamp(0.0, max) / max * (1.0 - size), max))
}

pub(in crate::survivors) fn update(
    mut tracks: Query<(&Track, &Interaction, &mut Visibility)>,
    mut thumbs: Query<(&Thumb, &mut Node, &mut BackgroundColor)>,
    viewports: Query<(&ComputedNode, &ScrollPosition)>,
) {
    for (track, interaction, mut visibility) in &mut tracks {
        let Ok((node, position)) = viewports.get(track.0) else {
            continue;
        };
        let Some((size, top, _max)) = geometry(node, position.0.y) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        for (thumb, mut style, mut color) in &mut thumbs {
            if thumb.0 != track.0 {
                continue;
            }
            let top = Val::Percent(top * 100.0);
            let height = Val::Percent(size * 100.0);
            if style.top != top || style.height != height {
                style.top = top;
                style.height = height;
            }
            let tint = if *interaction != Interaction::None {
                Color::srgb(0.93, 0.75, 0.32)
            } else {
                Color::srgb(0.68, 0.51, 0.18)
            };
            color.set_if_neq(BackgroundColor(tint));
        }
    }
}

#[derive(Default)]
pub(in crate::survivors) struct Drag {
    point: Option<Vec2>,
    active: Option<(Entity, f32)>,
}

// Read the ordered stream so a whole short drag survives within one frame.
pub(in crate::survivors) fn input(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut events: MessageReader<WindowEvent>,
    tracks: Query<(
        Entity,
        &Track,
        &ComputedNode,
        &UiGlobalTransform,
        &Visibility,
    )>,
    mut viewports: Query<(&ComputedNode, &mut ScrollPosition)>,
    mut drag: Local<Drag>,
) {
    let Ok((window_entity, window)) = windows.single() else {
        events.clear();
        *drag = Drag::default();
        return;
    };
    if drag.point.is_none() {
        drag.point = window.cursor_position().map(|p| p * window.scale_factor());
    }
    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(event) if event.window == window_entity => {
                drag.point = Some(event.position * window.scale_factor());
            }
            WindowEvent::MouseButtonInput(event)
                if event.window == window_entity && event.button == MouseButton::Left =>
            {
                if event.state == ButtonState::Released {
                    drag.active = None;
                } else if let Some(point) = drag.point {
                    for (entity, track, node, transform, visibility) in &tracks {
                        if *visibility == Visibility::Hidden
                            || !node.contains_point(*transform, point)
                        {
                            continue;
                        }
                        let Ok((view, position)) = viewports.get(track.0) else {
                            continue;
                        };
                        let Some((size, top, _)) = geometry(view, position.0.y) else {
                            continue;
                        };
                        let Some(pointer) =
                            node.normalize_point(*transform, point).map(|p| p.y + 0.5)
                        else {
                            continue;
                        };
                        let grab = if (top..=top + size).contains(&pointer) {
                            pointer - top
                        } else {
                            size * 0.5
                        };
                        drag.active = Some((entity, grab));
                        break;
                    }
                }
            }
            WindowEvent::WindowFocused(event)
                if event.window == window_entity && !event.focused =>
            {
                *drag = Drag::default();
            }
            WindowEvent::CursorLeft(event) if event.window == window_entity => {
                *drag = Drag::default();
            }
            _ => continue,
        }
        if let (Some((entity, grab)), Some(point)) = (drag.active, drag.point) {
            let Ok((_, track, node, transform, _)) = tracks.get(entity) else {
                drag.active = None;
                continue;
            };
            let Ok((view, mut position)) = viewports.get_mut(track.0) else {
                continue;
            };
            let Some((size, _, max)) = geometry(view, position.0.y) else {
                continue;
            };
            if let Some(pointer) = node.normalize_point(*transform, point).map(|p| p.y + 0.5) {
                position.0.y = (pointer - grab).clamp(0.0, 1.0 - size) / (1.0 - size) * max;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_drag_in_one_frame_reaches_the_bottom_and_releases() {
        use bevy::{input::mouse::MouseButtonInput, window::CursorMoved};
        let mut app = App::new();
        app.add_message::<WindowEvent>().add_systems(Update, input);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        let viewport = app
            .world_mut()
            .spawn((
                ComputedNode {
                    size: Vec2::splat(100.),
                    content_size: Vec2::new(100., 500.),
                    inverse_scale_factor: 1.,
                    ..default()
                },
                ScrollPosition::default(),
            ))
            .id();
        app.world_mut().spawn((
            Track(viewport),
            ComputedNode {
                size: Vec2::new(10., 100.),
                ..default()
            },
            UiGlobalTransform::default(),
            Visibility::Inherited,
        ));
        for event in [
            WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(0., -40.),
                delta: None,
            }),
            WindowEvent::MouseButtonInput(MouseButtonInput {
                window,
                button: MouseButton::Left,
                state: ButtonState::Pressed,
            }),
            WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(0., 40.),
                delta: None,
            }),
            WindowEvent::MouseButtonInput(MouseButtonInput {
                window,
                button: MouseButton::Left,
                state: ButtonState::Released,
            }),
        ] {
            app.world_mut().write_message(event);
        }
        app.update();
        let released_position = app.world().get::<ScrollPosition>(viewport).unwrap().0.y;
        assert!((released_position - 400.).abs() < 0.001);
        app.world_mut()
            .write_message(WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(0., -40.),
                delta: None,
            }));
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(viewport).unwrap().0.y,
            released_position
        );
    }

    #[test]
    fn thumb_tracks_viewport_fraction_and_logical_scroll_at_high_dpi() {
        let node = ComputedNode {
            size: Vec2::new(600., 400.),
            content_size: Vec2::new(600., 1600.),
            inverse_scale_factor: 0.5,
            ..default()
        };
        assert_eq!(geometry(&node, 0.), Some((0.25, 0., 600.)));
        assert_eq!(geometry(&node, 300.), Some((0.25, 0.375, 600.)));
        assert_eq!(geometry(&node, 900.), Some((0.25, 0.75, 600.)));
        assert_eq!(geometry(&ComputedNode::default(), 20.), None);
        let fitted = ComputedNode {
            content_size: node.size,
            ..node
        };
        assert_eq!(geometry(&fitted, 20.), None);
    }
}
