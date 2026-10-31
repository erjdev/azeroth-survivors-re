//! Visual-only local template answers; combat bonuses remain in Survivors rules.
use benilla_protocol::ItemInfo;
use survivors_core::gear_visuals::GearVisual;

pub(super) fn template(name: &str, visual: GearVisual) -> ItemInfo {
    ItemInfo {
        class: u32::from(visual.item_class),
        subclass: u32::from(visual.subclass),
        name: name.into(),
        display_info_id: visual.display_id,
        quality: 1,
        flags: 0,
        buy_price: 0,
        sell_price: 0,
        inventory_type: u32::from(visual.inventory_type),
        allowable_class: -1,
        allowable_race: -1,
        item_level: 0,
        required_level: 0,
        required_skill: 0,
        required_skill_rank: 0,
        required_spell: 0,
        required_honor_rank: 0,
        required_city_rank: 0,
        required_rep_faction: 0,
        required_rep_rank: 0,
        max_count: 0,
        stackable: 1,
        container_slots: 0,
        stats: Vec::new(),
        damages: Vec::new(),
        dmg_min: 0.0,
        dmg_max: 0.0,
        dmg_type: 0,
        armor: 0,
        resistances: [0; 6],
        delay_ms: 0,
        ammo_type: 0,
        ranged_mod_range: 0.0,
        spells: Vec::new(),
        spell_charges_0: 0,
        use_spell: None,
        bonding: 0,
        description: String::new(),
        page_text: 0,
        language_id: 0,
        page_material: 0,
        start_quest: 0,
        lock_id: 0,
        material: visual.material as u32,
        sheath: u32::from(visual.sheath),
        random_property: 0,
        block: 0,
        item_set: 0,
        max_durability: 0,
        area: 0,
        map: 0,
        bag_family: 0,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn visual_template_preserves_weapon_type_without_importing_power() {
        let axe = survivors_core::gear_visuals::named("Whirlwind Axe").unwrap();
        let item = super::template("Whirlwind Axe", axe);
        assert_eq!(
            (item.class, item.subclass, item.inventory_type, item.sheath),
            (2, 1, 17, 1)
        );
        assert_eq!(item.display_info_id, 22734);
        assert!(item.stats.is_empty() && item.spells.is_empty());
        assert_eq!((item.armor, item.dmg_min, item.dmg_max), (0, 0.0, 0.0));
    }
}
