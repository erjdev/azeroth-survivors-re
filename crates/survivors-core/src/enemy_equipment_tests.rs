use crate::{data, enemy_equipment::for_template};

#[test]
fn every_roster_has_audited_equipment_without_duplicate_slots() {
    let mut armed = 0;
    let mut empty = 0;
    for map in 0..data::MAPS.len() {
        let roster = data::roster(map);
        for enemy in roster.enemies.iter().chain(&roster.bosses) {
            let items = for_template(enemy.template_entry).expect(enemy.name);
            if items.is_empty() {
                empty += 1;
            } else {
                armed += 1;
            }
            let mut slots = std::collections::HashSet::new();
            for &(slot, display, _, _, _, _, _) in items {
                assert!(slot < 3 && slots.insert(slot), "{}", enemy.name);
                assert_ne!(display, 0, "{}", enemy.name);
            }
        }
    }
    assert!(armed > 0 && empty > 0);
    assert!(for_template(u32::MAX).is_none());
}

#[test]
fn ordinary_dual_wield_shield_and_legacy_equipment_keep_exact_descriptors() {
    assert_eq!(for_template(116), Some(&[(0, 7487, 2, 7, 1, 13, 3)][..]));
    assert_eq!(
        for_template(61),
        Some(&[(0, 7492, 2, 7, 1, 13, 3), (1, 7492, 2, 7, 1, 13, 3)][..])
    );
    assert_eq!(for_template(98).unwrap()[1], (1, 1755, 4, 6, 1, 14, 4));
    // Legacy raw values are preserved, not replaced by player item metadata.
    assert_eq!(for_template(4306).unwrap()[0].1, 8078);
    assert_eq!(for_template(4306).unwrap()[0].5, 1);
}
