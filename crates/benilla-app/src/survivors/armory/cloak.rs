//! Cloak requirements beside the equipment they supplement.
use super::{campaign, hud, Account, Selection, GOLD};
use bevy::prelude::*;
use survivors_core::layout::Layout;

pub(super) fn build(
    strings: &crate::survivors::localization::Strings,
    ui: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    account: &Account,
    selection: &Selection,
    layout: &Layout,
) {
    let Some(hero) = selection.character.and_then(|id| account.character(id)) else {
        return;
    };
    let Some(progress) = account.classes.get(hero.class) else {
        return;
    };
    let feats = account.feats(hero.class, hero.id);
    let quality = campaign::cloak_quality(feats);
    super::typography::heading(ui, font, "Cloak", layout.title_size, GOLD);
    ui.spawn(hud::text_bundle(
        font,
        quality.map_or_else(
            || "Class cloak: not yet earned".to_owned(),
            |quality| format!("Class cloak: {}", campaign::quality_name(quality)),
        ),
        layout.body_size,
        GOLD,
    ));
    if let Some(quality) = quality {
        ui.spawn(hud::text_bundle(
            font,
            super::door::bonus_line(strings, campaign::cloak_bonus(quality)),
            layout.body_size,
            hud::card_growth::GAIN,
        ));
    }
    let (seen, total) = account.legendary_progress(hero.class);
    let raid = campaign::ZONES
        .last()
        .map_or("Final raid", |zone| zone.name);
    for (label, complete) in [
        (
            format!(
                "Talent cap: {}/{}",
                campaign::talent_level(hero.talent_xp),
                campaign::TALENT_CAP
            ),
            feats.talent_max,
        ),
        (
            format!(
                "{raid}: {}",
                if feats.raid { "cleared" } else { "not cleared" }
            ),
            feats.raid,
        ),
        (
            format!(
                "Gear collection: {}/{}",
                campaign::owned_pieces(&progress.pieces),
                campaign::PIECES
            ),
            feats.collection,
        ),
        (format!("Legendary amulets: {seen}/{total}"), feats.amulets),
    ] {
        ui.spawn(hud::text_bundle(
            font,
            label,
            layout.body_size,
            if complete {
                hud::card_growth::GAIN
            } else {
                Color::srgb(0.64, 0.58, 0.4)
            },
        ));
    }
    ui.spawn(hud::text_bundle(font, "One feat earns a cloak; two grant Rare. A raid clear grants Rare directly. All four grant Epic.", layout.body_size - 1.0, Color::srgb(0.64, 0.58, 0.4)));
    let epics = account.epic_cloaks();
    ui.spawn(hud::text_bundle(
        font,
        format!("Account legendary cloak: {epics}/9 epic class cloaks"),
        layout.body_size,
        GOLD,
    ));
    if campaign::legendary_cloak(epics) {
        ui.spawn(hud::text_bundle(
            font,
            super::door::bonus_line(strings, campaign::cloak_bonus(campaign::Quality::Legendary)),
            layout.body_size,
            hud::card_growth::GAIN,
        ));
    }
}
