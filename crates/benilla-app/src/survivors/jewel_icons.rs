//! Shared jewelry art from the pinned catalog in docs/SURVIVORS-GEAR-VISUALS.md.
//! Display identities only; gameplay and saved discovery names are unchanged.

pub(super) fn display(name: &str) -> Option<u32> {
    Some(match name {
        "Band of the Unicorn" => 15422,
        "Seal of Wrynn" => 9845,
        "Ring of the Moon" => 9837,
        "Magni's Will" => 9837,
        "Ring of Protection" => 26537,
        "Don Mauricio's Band" => 24022,
        "Ring of Fortitude" => 15422,
        "Band of Rumination" => 24022,
        "Ring of the Fallen" => 34336,
        "Rune of the Guard Captain" => 31631,
        "Smotts' Compass" => 6562,
        "Defender of the Timbermaw" => 33702,
        "Ramstein's Lightning Bolts" => 1236,
        // Survivors uses vanilla entry 23206 for its single Mark of the Champion.
        "Mark of the Champion" => 35649,
        "Talisman of Ephemeral Power" => 31287,
        "Zandalarian Hero Medallion" => 32431,
        "Eye of the Beast" => 24784,
        "Briarwood Reed" => 8232,
        "Drake Fang Talisman" => 31936,
        "Royal Seal of Eldre'Thalas" => 29712,
        "Mindtap Talisman" => 30723,
        "Neltharion's Tear" => 31901,
        "Amulet of the Redeemed" => 9857,
        "Medallion of Grand Marshal Morris" => 23717,
        "Pendant of the Fallen Dragon" => 31889,
        "Mark of Fordring" => 23716,
        "Onyxia Tooth Pendant" => 9860,
        "Prestor's Talisman" => 31899,
        "The Eye of Hakkar" => 32297,
        "Amulet of the Darkmoon" => 32008,
        "Choker of the Fire Lord" => 1399,
        "Gem of Nerubis" => 35817,
        "Pendant of the Qiraji Guardian" => 34204,
        "Amulet of Vek'nilash" => 34204,
        "Sadist's Collar" => 35358,
        "Stormrage's Talisman" => 35359,
        "The Burrower's Shell" => 36290,
        "Talisman of Ascendance" => 24730,
        "Master Dragonslayer's Medallion" => 31907,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn shared_jewelry_displays_have_readable_vanilla_icons() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_item_display_catalog(&mut chain).unwrap();
        let names = super::super::campaign::RING_NAMES
            .iter()
            .chain(super::super::campaign::TRINKET_NAMES.iter())
            .chain(super::super::campaign::AMULET_NAMES.iter());
        let mut verified = 0;
        for name in names {
            let display = super::display(name).expect(name);
            let row = catalog.get(display).expect(name);
            let icon = row.icon.as_ref().expect(name);
            assert!(!chain.read(&format!("{icon}.blp")).expect(name).is_empty());
            verified += 1;
        }
        assert_eq!(verified, 39);
    }
}
