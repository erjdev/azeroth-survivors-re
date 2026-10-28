//! Collection identity, icon decoding and scheduled-source evidence, without a renderer.
use anyhow::{bail, Context, Result};
use benilla_formats::{Chain, ItemDisplayCatalog};
use std::collections::HashSet;
use survivors_core::{
    campaign, data, encounters, gear_tier::GearTier, gear_visuals, supplemental_gear,
};

fn icon(
    chain: &mut Chain,
    displays: &ItemDisplayCatalog,
    checked: &mut HashSet<u32>,
    visual: gear_visuals::GearVisual,
) -> Result<()> {
    if checked.contains(&visual.display_id) {
        return Ok(());
    }
    let display = displays
        .get(visual.display_id)
        .context("missing display row")?;
    let path = format!(
        "{}.blp",
        display.icon.as_ref().context("missing icon name")?
    );
    let decoded = benilla_blp::decode(&chain.read_file(&path)?)
        .with_context(|| format!("invalid icon {path}"))?;
    if decoded.width == 0 || decoded.height == 0 {
        bail!("empty icon {path}");
    }
    checked.insert(visual.display_id);
    Ok(())
}

pub fn report(chain: &mut Chain) -> Result<()> {
    let displays = benilla_formats::load_item_display_catalog(chain)?;
    let mut checked = HashSet::new();
    let mut gaps = Vec::new();
    for (class, definition) in data::CLASSES.iter().enumerate() {
        let mut verified = 0;
        for band in 0..campaign::BANDS {
            for slot in 0..campaign::GRID_SLOTS {
                let result = (|| -> Result<()> {
                    let name = campaign::piece_name(class, slot, band).context("missing name")?;
                    let visual = gear_visuals::piece(class, slot, band)
                        .context("missing visual identity")?;
                    icon(chain, &displays, &mut checked, visual).with_context(|| {
                        format!(
                            "{name}, item {}, display {}",
                            visual.item_id, visual.display_id
                        )
                    })?;
                    if !campaign::DROPS.iter().any(|row| {
                        row.band == band
                            && row.slot == slot
                            && row.chance > 0
                            && row.chance <= 100
                            && encounters::drop_is_boss(row)
                    }) {
                        bail!("no positive-chance source matching an actual scheduled encounter");
                    }
                    Ok(())
                })();
                match result {
                    Ok(()) => verified += 1,
                    Err(error) => gaps.push(format!(
                        "{} band {band} slot {slot}: {error:#}",
                        definition.name
                    )),
                }
            }
        }
        let mut supplemental = 0;
        for (tier, slots) in [(GearTier::Poor, 9), (GearTier::RaidEpic, 8)] {
            for slot in 0..slots {
                let result = supplemental_gear::piece(class, slot, tier)
                    .context("missing supplemental identity")
                    .and_then(|piece| icon(chain, &displays, &mut checked, piece.visual));
                match result {
                    Ok(()) => supplemental += 1,
                    Err(error) => gaps.push(format!(
                        "{} {tier:?} slot {slot}: {error:#}",
                        definition.name
                    )),
                }
            }
        }
        println!("COLLECTION: {} · {verified}/54 named identities with decoded icons and scheduled sources · {supplemental}/17 supplemental identities with decoded icons", definition.name);
    }
    for weapon in survivors_core::weapon_alternatives::choices(0) {
        match icon(chain, &displays, &mut checked, weapon.visual) {
            Ok(()) => println!("WEAPON ALTERNATIVE: {} · item {} · display {} · icon decoded; gameplay acquisition and model rendering require separate acceptance", weapon.name, weapon.visual.item_id, weapon.visual.display_id),
            Err(error) => gaps.push(format!("{} alternative: {error:#}", weapon.name)),
        }
    }
    println!("{} unique display icons decoded. Supplemental checks establish identity/art only; Raid Epic kill settlement is verified separately; zone access remains gated by world readiness.", checked.len());
    println!("Scope excludes equipped model rendering, stat balance, zone unlocks, traversal and live acquisition. Item IDs/names use the repository's pinned template mappings; the MPQ verifies display rows and icon data, not server item templates.");
    if !gaps.is_empty() {
        bail!("{} collection gaps:\n{}", gaps.len(), gaps.join("\n"));
    }
    Ok(())
}
