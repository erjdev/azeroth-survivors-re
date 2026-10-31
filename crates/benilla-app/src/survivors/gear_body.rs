//! Feed local loadouts into the stock player compositor and attachment resolver.
use super::{account::Account, hero::Hero, Selection};
use crate::entities::{CharCreate, Creatures};
use crate::net::{NetEntity, ObjectStore};
use benilla_protocol::{EntityKind, ObjectFields};
use bevy::prelude::*;
use survivors_core::campaign;

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
    let mut entries = [0; 19];
    let mut ranged = false;
    for slot in 0..campaign::GRID_SLOTS {
        let Some(piece) =
            progress
                .equipment
                .visible(class, &progress.pieces, &account.supplemental, slot)
        else {
            continue;
        };
        let visual = piece.visual;
        let target = render_slot(slot, visual.inventory_type);
        let Some(target) = target else { continue };
        if items.template_cached(visual.item_id).is_none() {
            let name = piece.name;
            items.insert_template(
                visual.item_id,
                Some(super::gear_template::template(name, visual)),
            );
        }
        entries[target] = visual.item_id;
        ranged |= target == 17;
    }
    apply_tabard(account.tabard, &mut entries, &mut items);
    let custom = selection
        .character
        .and_then(|id| account.character(id))
        .filter(|hero| hero.class == class && hero.body == Some((race, sex)))
        .and_then(|hero| hero.features);
    let [skin, face, hair_style, hair_color, facial_hair] = custom
        .zip(bodies.0.ranges(race, sex))
        .map(|(values, ranges)| {
            super::appearance::bounded(values, super::appearance::counts(ranges))
        })
        .unwrap_or([
            look.skin,
            look.face,
            look.hair_style,
            look.hair_color,
            look.facial_hair,
        ]);
    let bytes = u32::from(skin)
        | u32::from(face) << 8
        | u32::from(hair_style) << 16
        | u32::from(hair_color) << 24;
    let mut fields = vec![
        (2, 0x19),
        (DISPLAY, display),
        (132, display),
        (PLAYER_BYTES, bytes),
        (PLAYER_BYTES_2, u32::from(facial_hair)),
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

// Tabards are account cosmetics, outside the nine power-bearing collection slots.
// Feed the same public descriptor used by both the menu hero and the run hero.
fn apply_tabard(index: u8, entries: &mut [u32; 19], items: &mut crate::items::Items) {
    entries[18] = 0;
    let Some(tabard) = index
        .checked_sub(1)
        .and_then(|i| campaign::TABARDS.get(i as usize))
    else {
        return;
    };
    // The shipped Guild Tabard: item 5976, ItemDisplayInfo 20621.
    let visual = survivors_core::gear_visuals::GearVisual {
        item_id: 5976,
        display_id: tabard.display,
        inventory_type: 19,
        item_class: 4,
        subclass: 0,
        material: 7,
        sheath: 0,
    };
    if items.template_cached(visual.item_id).is_none() {
        items.insert_template(
            visual.item_id,
            Some(super::gear_template::template(tabard.name, visual)),
        );
    }
    entries[18] = visual.item_id;
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
    fn tabard_descriptor_uses_cosmetic_template_and_clears_without_replacing_gear() {
        let mut items = crate::items::Items::default();
        let mut entries = [42; 19];
        super::apply_tabard(1, &mut entries, &mut items);
        assert_eq!(entries[18], 5976);
        assert_eq!(&entries[..18], &[42; 18]);
        let item = items.template_cached(5976).unwrap();
        assert_eq!((item.display_info_id, item.inventory_type), (20621, 19));
        assert!(item.stats.is_empty() && item.spells.is_empty());
        for index in [0, 255] {
            super::apply_tabard(index, &mut entries, &mut items);
            assert_eq!(entries[18], 0);
            assert_eq!(&entries[..18], &[42; 18]);
        }
    }

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
