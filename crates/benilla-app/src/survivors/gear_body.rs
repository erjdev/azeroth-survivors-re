//! Feed local loadouts into the stock player compositor and attachment resolver.
use super::{account::Account, hero::Hero, Selection};
use crate::entities::{CharCreate, Creatures};
use crate::net::{NetEntity, ObjectStore};
use benilla_protocol::{EntityKind, ObjectFields};
use bevy::prelude::*;
use survivors_core::{campaign, gear_visuals};

// 1.12.1 public descriptor layout, shared with ObjectFields' player readers.
const VISIBLE_ENTRY: u16 = 260;
const PLAYER_BYTES: u16 = 193;
const PLAYER_BYTES_2: u16 = 194;
const DISPLAY: u16 = 131;

pub(super) fn sync(
    selection: Res<Selection>,
    account: Res<Account>,
    creatures: Option<Res<Creatures>>,
    bodies: Option<Res<CharCreate>>,
    mut items: ResMut<crate::items::Items>,
    mut heroes: Query<
        (
            &mut NetEntity,
            &mut ObjectStore,
            Option<&crate::entities::DressKey>,
        ),
        With<Hero>,
    >,
) {
    let (Some(class), Some((preset, race, sex))) = (
        selection.class,
        selection.look.and_then(|i| selection.looks.get(i)).copied(),
    ) else {
        return;
    };
    let (Some(creatures), Some(bodies)) = (creatures.as_deref(), bodies.as_deref()) else {
        return;
    };
    let (Some(look), Some(display)) = (
        creatures.survivors_appearance(preset),
        bodies.0.body_display(race, sex),
    ) else {
        return;
    };
    let Some(progress) = account.classes.get(class) else {
        return;
    };
    let visible = progress.equipment.visible_bands(&progress.pieces);
    let mut entries = [0; 19];
    let mut ranged = false;
    for (slot, band) in visible.into_iter().enumerate() {
        let Some(band) = band else { continue };
        let Some(visual) = gear_visuals::piece(class, slot, usize::from(band)) else {
            continue;
        };
        let target = render_slot(slot, visual.inventory_type);
        let Some(target) = target else { continue };
        if items.template_cached(visual.item_id).is_none() {
            let name = campaign::piece_name(class, slot, usize::from(band)).unwrap();
            items.insert_template(
                visual.item_id,
                Some(super::gear_template::template(name, visual)),
            );
        }
        entries[target] = visual.item_id;
        ranged |= target == 17;
    }
    let bytes = u32::from(look.skin)
        | u32::from(look.face) << 8
        | u32::from(look.hair_style) << 16
        | u32::from(look.hair_color) << 24;
    let mut fields = vec![
        (2, 0x19),
        (DISPLAY, display),
        (132, display),
        (PLAYER_BYTES, bytes),
        (PLAYER_BYTES_2, u32::from(look.facial_hair)),
        (164, if ranged { 2 } else { 1 }),
    ];
    fields.extend(
        entries
            .iter()
            .enumerate()
            .map(|(i, &entry)| (VISIBLE_ENTRY + 12 * i as u16, entry)),
    );
    for (mut net, mut store, dress) in &mut heroes {
        // Local GUIDs retain their existing identity; renderer dispatch uses EntityKind.
        if net.kind != EntityKind::Player {
            net.kind = EntityKind::Player;
        }
        if net.display_id != Some(display) {
            net.display_id = Some(display);
        }
        if fields
            .iter()
            .any(|&(field, value)| !store.0.raw_fields().any(|pair| pair == (field, value)))
        {
            store.0.merge(ObjectFields::from_pairs(&fields));
        } else if dress.is_some_and(|d| !d.held_ready) {
            // Local templates arrive before their model assets. Keep the descriptor-driven
            // resolver awake until it records ready attachments; there is no server update
            // to wake it when a newly selected appearance finishes loading.
            store.set_changed();
        }
    }
}

fn render_slot(slot: usize, inventory: u8) -> Option<usize> {
    match slot {
        0..=7 => Some([0, 2, 4, 5, 6, 7, 8, 9][slot]),
        8 => Some(if matches!(inventory, 15 | 25 | 26) {
            17
        } else {
            15
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn armor_and_ranged_weapons_use_player_descriptor_slots() {
        assert_eq!(super::render_slot(0, 1), Some(0));
        assert_eq!(super::render_slot(2, 20), Some(4));
        assert_eq!(super::render_slot(6, 9), Some(8));
        assert_eq!(super::render_slot(8, 17), Some(15));
        assert_eq!(super::render_slot(8, 15), Some(17));
        assert_eq!(super::render_slot(8, 26), Some(17));
        assert_eq!(super::render_slot(9, 14), None);
    }
}
