//! Name hit testing uses the rendered font's advances, including spaces, at physical scale.
use super::{name_input::Cursor, Act, Armory, Page};
use crate::survivors::{hud, Phase, Typing};
use bevy::{
    input::ButtonState,
    prelude::*,
    text::ComputedTextBlock,
    window::{PrimaryWindow, WindowEvent},
};

/// The byte range represented by a rendered segment; the caret/placeholder has an empty range.
#[derive(Component)]
pub(in crate::survivors) struct Segment {
    pub slot: u8,
    pub start: usize,
    pub end: usize,
}

#[derive(Default)]
pub(in crate::survivors) struct Pointer {
    point: Option<Vec2>,
    drag: Option<Drag>,
}

struct Drag {
    slot: u8,
    anchor: usize,
    // Snapshot on press: rebuilding centered text must not move the drag's hit targets.
    stops: Vec<(f32, usize)>,
}

fn nearest(stops: &[(f32, usize)], x: f32) -> Option<usize> {
    stops
        .iter()
        .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()))
        .map(|stop| stop.1)
}

impl Drag {
    fn cursor(&self, x: f32) -> Option<Cursor> {
        let at = nearest(&self.stops, x)?;
        Some(Cursor {
            slot: self.slot,
            at,
            start: self.anchor,
            end: at,
        })
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(in crate::survivors) fn input(
    windows: Query<(Entity, &Window), With<PrimaryWindow>>,
    mut events: MessageReader<WindowEvent>,
    fields: Query<(
        &Act,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        Option<&CalculatedClip>,
    )>,
    segments: Query<(
        &Segment,
        &ComputedNode,
        &UiGlobalTransform,
        &ComputedTextBlock,
    )>,
    phase: Res<State<Phase>>,
    settings: Res<hud::PausePage>,
    mut armory: ResMut<Armory>,
    mut typing: ResMut<Typing>,
    mut pointer: Local<Pointer>,
) {
    if *phase.get() != Phase::Menu
        || settings.open
        || !(armory.page == Page::Heroes || armory.rename.is_some())
    {
        events.clear();
        *pointer = Pointer::default();
        return;
    }
    let Ok((window_id, window)) = windows.single() else {
        events.clear();
        return;
    };
    if pointer.point.is_none() {
        pointer.point = window.physical_cursor_position();
    }
    if pointer
        .drag
        .as_ref()
        .is_some_and(|drag| armory.focus != drag.slot)
    {
        pointer.drag = None;
    }
    for event in events.read() {
        match event {
            WindowEvent::CursorMoved(event) if event.window == window_id => {
                pointer.point = Some(event.position * window.scale_factor());
                if let Some(drag) = &pointer.drag {
                    armory.name_cursor = drag.cursor(event.position.x * window.scale_factor());
                }
            }
            WindowEvent::MouseButtonInput(event)
                if event.window == window_id && event.button == MouseButton::Left =>
            {
                if event.state == ButtonState::Released {
                    pointer.drag = None;
                    continue;
                }
                pointer.drag = None;
                let Some(point) = pointer.point else { continue };
                for (act, node, transform, visible, clip) in &fields {
                    let Act::Focus(slot) = act else { continue };
                    if !visible.get()
                        || !node.contains_point(*transform, point)
                        || clip.is_some_and(|clip| !clip.clip.contains(point))
                    {
                        continue;
                    }
                    let mut stops = Vec::new();
                    for (segment, text_node, text_transform, block) in &segments {
                        if segment.slot != *slot {
                            continue;
                        }
                        let left = text_transform.translation.x - text_node.size.x * 0.5;
                        stops.push((left, segment.start));
                        for run in block.buffer().0.layout_runs() {
                            for glyph in run.glyphs {
                                stops.push((
                                    left + glyph.x,
                                    (segment.start + glyph.start).min(segment.end),
                                ));
                                stops.push((
                                    left + glyph.x + glyph.w,
                                    (segment.start + glyph.end).min(segment.end),
                                ));
                            }
                        }
                        stops.push((left + text_node.size.x, segment.end));
                    }
                    let Some(anchor) = nearest(&stops, point.x) else {
                        continue;
                    };
                    let drag = Drag {
                        slot: *slot,
                        anchor,
                        stops,
                    };
                    armory.focus = *slot;
                    armory.name_cursor = drag.cursor(point.x);
                    typing.0 = true;
                    pointer.drag = Some(drag);
                    break;
                }
            }
            WindowEvent::WindowFocused(event) if event.window == window_id && !event.focused => {
                *pointer = Pointer::default();
            }
            WindowEvent::CursorLeft(event) if event.window == window_id => {
                *pointer = Pointer::default();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_press_drag_release_keeps_selection_and_ignores_later_motion() {
        use bevy::{input::mouse::MouseButtonInput, window::CursorMoved};
        let mut app = App::new();
        app.add_message::<WindowEvent>()
            .insert_resource(State::new(Phase::Menu))
            .init_resource::<hud::PausePage>()
            .init_resource::<Typing>()
            .insert_resource(Armory {
                page: Page::Heroes,
                ..default()
            })
            .add_systems(Update, input);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.world_mut().spawn((
            Act::Focus(1),
            ComputedNode {
                size: Vec2::new(100., 20.),
                ..default()
            },
            UiGlobalTransform::default(),
            InheritedVisibility::VISIBLE,
        ));
        app.world_mut().spawn((
            Segment {
                slot: 1,
                start: 0,
                end: 4,
            },
            ComputedNode {
                size: Vec2::new(40., 20.),
                ..default()
            },
            UiGlobalTransform::default(),
            ComputedTextBlock::default(),
        ));
        let movement = |x| {
            WindowEvent::CursorMoved(CursorMoved {
                window,
                position: Vec2::new(x, 0.),
                delta: None,
            })
        };
        let button = |state| {
            WindowEvent::MouseButtonInput(MouseButtonInput {
                window,
                button: MouseButton::Left,
                state,
            })
        };
        for event in [
            movement(-20.),
            button(ButtonState::Pressed),
            movement(20.),
            button(ButtonState::Released),
        ] {
            app.world_mut().write_message(event);
        }
        app.update();
        let expected = Some(Cursor {
            slot: 1,
            at: 4,
            start: 0,
            end: 4,
        });
        assert_eq!(app.world().resource::<Armory>().name_cursor, expected);
        assert!(app.world().resource::<Typing>().0);
        app.world_mut().write_message(movement(-20.));
        app.update();
        assert_eq!(app.world().resource::<Armory>().name_cursor, expected);
        // A settings overlay must not let the hidden name field take a new click.
        app.world_mut().resource_mut::<hud::PausePage>().open = true;
        app.world_mut().write_message(button(ButtonState::Pressed));
        app.update();
        assert_eq!(app.world().resource::<Armory>().name_cursor, expected);
    }

    #[test]
    fn variable_width_letters_and_spaces_use_rendered_stops() {
        let stops = [(40., 0), (46., 1), (62., 2), (66., 3), (73., 4)];
        assert_eq!(nearest(&stops, 48.), Some(1));
        assert_eq!(nearest(&stops, 64.5), Some(3));
        assert_eq!(nearest(&stops, -100.), Some(0));
        assert_eq!(nearest(&stops, 900.), Some(4));
        assert_eq!(nearest(&[], 0.), None);
    }

    #[test]
    fn drag_reverses_through_anchor_and_clamps_outside_field() {
        let drag = Drag {
            slot: 4,
            anchor: 2,
            stops: vec![(100., 0), (110., 1), (120., 2), (130., 3)],
        };
        assert_eq!(
            drag.cursor(999.),
            Some(Cursor {
                slot: 4,
                at: 3,
                start: 2,
                end: 3
            })
        );
        assert_eq!(
            drag.cursor(0.),
            Some(Cursor {
                slot: 4,
                at: 0,
                start: 2,
                end: 0
            })
        );
        assert_eq!(
            drag.cursor(120.),
            Some(Cursor {
                slot: 4,
                at: 2,
                start: 2,
                end: 2
            })
        );
    }

    #[test]
    fn physical_scale_does_not_change_selected_byte() {
        let stops = [(10., 0), (15., 1), (27., 2), (34., 3)];
        for scale in [0.75, 1., 1.25, 2.] {
            let scaled: Vec<_> = stops.iter().map(|&(x, at)| (x * scale, at)).collect();
            assert_eq!(nearest(&scaled, 25. * scale), Some(2));
        }
    }
}
