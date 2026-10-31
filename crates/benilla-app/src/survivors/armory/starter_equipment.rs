//! Compact reference-shaped equipment strip with one readable inspection line.
use super::{controller::ControllerFocus, hud};
use bevy::prelude::*;

pub(super) const EMPTY_ART: [&str; 9] = [
    "Head", "Shoulder", "Chest", "Waist", "Legs", "Feet", "Wrists", "Hands", "MainHand",
];
const HELP: &str = "Point or focus a slot; select to inspect its gear in Collection.";

#[derive(Component)]
pub(in crate::survivors) struct SlotInfo(pub String);
#[derive(Component)]
pub(in crate::survivors) struct Detail;

pub(super) fn empty(ui: &mut ChildSpawnerCommands, slot: usize) {
    ui.spawn((
        super::talent_page::Backdrop(format!(
            "Interface\\PaperDoll\\UI-PaperDoll-Slot-{}.blp",
            EMPTY_ART[slot]
        )),
        ImageNode::default(),
        Node {
            width: Val::Px(30.0),
            height: Val::Px(30.0),
            flex_shrink: 0.0,
            ..default()
        },
    ));
}

pub(super) fn detail(ui: &mut ChildSpawnerCommands, font: &Handle<Font>) {
    ui.spawn((
        Detail,
        hud::text_bundle(font, HELP, 11.0, super::GOLD),
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(30.0),
            ..default()
        },
    ));
}

pub(in crate::survivors) fn refresh(
    slots: Query<(Entity, &Interaction, &SlotInfo)>,
    focus: Res<ControllerFocus>,
    mut details: Query<&mut Text, With<Detail>>,
) {
    let selected = focus
        .focused()
        .and_then(|id| slots.get(id).ok())
        .or_else(|| {
            slots
                .iter()
                .find(|(_, interaction, _)| **interaction != Interaction::None)
        });
    let copy = selected.map_or(HELP, |(_, _, info)| info.0.as_str());
    for mut text in &mut details {
        if text.0 != copy {
            text.0 = copy.to_owned();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slot_art_resolves_from_the_install() {
        let data = benilla_formats::wow_data_or_skip!();
        let chain = benilla_formats::Chain::open(&data).unwrap();
        for art in EMPTY_ART {
            let path = format!("Interface\\PaperDoll\\UI-PaperDoll-Slot-{art}.blp");
            assert!(!chain.read(&path).expect(&path).is_empty());
        }
    }

    #[test]
    fn inspection_updates_and_clears_after_pointer_leaves_or_slot_is_removed() {
        let mut app = App::new();
        app.init_resource::<ControllerFocus>()
            .add_systems(Update, refresh);
        let slot = app
            .world_mut()
            .spawn((Interaction::Hovered, SlotInfo("Shoulders · Empty".into())))
            .id();
        let detail = app.world_mut().spawn((Detail, Text::new(HELP))).id();
        app.update();
        assert_eq!(
            app.world().get::<Text>(detail).unwrap().0,
            "Shoulders · Empty"
        );
        *app.world_mut().get_mut::<Interaction>(slot).unwrap() = Interaction::None;
        app.update();
        assert_eq!(app.world().get::<Text>(detail).unwrap().0, HELP);
        *app.world_mut().get_mut::<Interaction>(slot).unwrap() = Interaction::Pressed;
        app.update();
        app.world_mut().despawn(slot);
        app.update();
        assert_eq!(app.world().get::<Text>(detail).unwrap().0, HELP);
    }
}
