//! Campaign encounters use the same creature and clock as the drop table.

use super::{campaign, data};

/// Scheduled creatures are distinct even when one creature has several drop rows.
fn build_schedule(map: usize) -> Vec<(&'static data::EnemyDef, u32)> {
    let Some(map_def) = data::MAPS.get(map) else {
        return Vec::new();
    };
    let Some(zone) = campaign::zone_index(map_def.name) else {
        return Vec::new();
    };
    let roster = data::roster(map);
    let mut rows = Vec::new();
    for row in campaign::DROPS.iter().filter(|row| row.zone == zone) {
        if rows
            .iter()
            .any(|(enemy, _): &(&data::EnemyDef, u32)| enemy.name == row.creature)
        {
            continue;
        }
        if let Some(enemy) = roster
            .bosses
            .iter()
            .find(|enemy| enemy.name == row.creature)
        {
            rows.push((enemy, row.at_secs));
        }
    }
    // Campaigns without collection drops still need a complete victory schedule.
    // Space their authored bosses through 16:00, matching campaign finales;
    // the final fight gates victory after the base 15-minute survival clock.
    if rows.is_empty() && !roster.bosses.is_empty() {
        return roster
            .bosses
            .iter()
            .enumerate()
            .map(|(i, enemy)| (enemy, ((i + 1) * 960 / roster.bosses.len()) as u32))
            .collect();
    }
    // Keep the existing boss roster as well as the drop encounters: timed drops do not
    // shrink the horde or remove its other boss fights.
    if !rows.is_empty() {
        for (index, enemy) in roster.bosses.iter().enumerate() {
            if !rows
                .iter()
                .any(|(scheduled, _)| scheduled.name == enemy.name)
            {
                rows.push((enemy, ((index + 1) as f32 * data::BOSS_EVERY_SECS) as u32));
            }
        }
    }
    rows.sort_by_key(|(_, at)| *at);
    rows
}

type Scheduled = (&'static data::EnemyDef, u32);
static SCHEDULES: std::sync::LazyLock<Vec<Vec<Scheduled>>> =
    std::sync::LazyLock::new(|| (0..data::MAPS.len()).map(build_schedule).collect());

fn schedule(map: usize) -> &'static [Scheduled] {
    SCHEDULES.get(map).map(Vec::as_slice).unwrap_or(&[])
}

/// Whether this source resolves to the scheduled boss that combat actually spawns.
pub fn drop_is_boss(drop: &campaign::DropRow) -> bool {
    data::MAPS.iter().enumerate().any(|(map, def)| {
        campaign::zone_index(def.name) == Some(drop.zone)
            && schedule(map)
                .iter()
                .any(|(enemy, at)| enemy.name == drop.creature && *at == drop.at_secs)
    })
}

/// A campaign table is enterable only when every advertised drop has a scheduled boss.
pub fn ready(map: usize) -> bool {
    let Some(zone) = data::MAPS
        .get(map)
        .and_then(|map| campaign::zone_index(map.name))
    else {
        return false;
    };
    let rows = schedule(map);
    !rows.is_empty()
        && campaign::DROPS
            .iter()
            .filter(|row| row.zone == zone)
            .all(|row| {
                row.chance <= 100
                    && row.at_secs > 0
                    && campaign::piece_index(row.band, row.slot).is_some()
                    && rows
                        .iter()
                        .any(|(enemy, at)| enemy.name == row.creature && *at == row.at_secs)
                    && (0..data::CLASSES.len())
                        .all(|class| campaign::piece_name(class, row.slot, row.band).is_some())
            })
}

/// The next scheduled boss, or the existing arena cadence for maps without campaign drops.
pub fn next(map: usize, spawned: u32, elapsed: f32) -> Option<&'static data::EnemyDef> {
    let rows = schedule(map);
    if rows.is_empty() {
        let at = (spawned.saturating_add(1) as f32) * data::BOSS_EVERY_SECS;
        return (elapsed >= at)
            .then(|| data::roster(map).bosses.get(spawned as usize))
            .flatten();
    }
    let &(enemy, at) = rows.get(spawned as usize)?;
    (elapsed >= at as f32).then_some(enemy)
}

/// Campaign victory waits for all scheduled bosses to die; arena runs keep their timer.
pub fn won(map: usize, elapsed: f32, defeated: u32) -> bool {
    let rows = schedule(map);
    let last = rows.last().map(|(_, at)| *at as f32).unwrap_or(0.0);
    elapsed >= last.max(data::VICTORY_SECS) && defeated as usize >= rows.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arathi_survival_bosses_supply_rare_slots_and_gate_victory() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Arathi Basin")
            .unwrap();
        assert_eq!(data::MAPS[map].map, 529);
        assert_eq!(data::roster(map).enemies.len(), 10);
        assert!(ready(map));
        let rows: Vec<_> = campaign::DROPS.iter().filter(|row| row.zone == 4).collect();
        assert_eq!(rows.len(), 9);
        assert_eq!(schedule(map).len(), 9);
        for (index, row) in rows.iter().enumerate() {
            assert_eq!((row.slot, row.band, row.chance), (index, 2, 60));
            assert!(drop_is_boss(row));
            assert!(next(map, index as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, index as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, row.slot, row.band).is_some());
            }
        }
        assert_eq!(
            schedule(map).last().unwrap().0.name,
            campaign::ZONES[4].boss
        );
        assert!(!won(map, 960., 8));
        assert!(!won(map, 959., 9));
        assert!(won(map, 960., 9));
        assert!(next(map, 9, 10_000.).is_none());
        assert_eq!(
            campaign::zone_lock_reason(4, 51, &[true; campaign::CLEAR_COUNT]).as_deref(),
            Some("Arathi Basin world content is not ready")
        );
    }

    #[test]
    fn warsong_faction_encounters_schedule_all_uncommon_sources() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Warsong Gulch")
            .unwrap();
        assert_eq!(data::MAPS[map].map, 489);
        assert_eq!(data::roster(map).enemies.len(), 10);
        assert!(ready(map));
        let rows: Vec<_> = campaign::DROPS.iter().filter(|row| row.zone == 2).collect();
        assert_eq!(rows.len(), 9);
        assert_eq!(schedule(map).len(), 9);
        for (index, row) in rows.iter().enumerate() {
            assert_eq!((row.slot, row.band, row.chance), (index, 1, 60));
            assert!(drop_is_boss(row));
            assert!(next(map, index as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, index as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, row.slot, row.band).is_some());
            }
        }
        assert_eq!(
            schedule(map).last().unwrap().0.name,
            campaign::ZONES[2].boss
        );
        assert!(!won(map, 960., 8));
        assert!(!won(map, 959., 9));
        assert!(won(map, 960., 9));
        assert!(next(map, 9, 10_000.).is_none());
        assert!(!campaign::zone_open(2, 51, &[true; campaign::CLEAR_COUNT]));
    }

    #[test]
    fn scarlet_bosses_without_collection_loot_still_gate_victory() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Scarlet Monastery")
            .unwrap();
        assert_eq!(data::MAPS[map].map, 189);
        assert_eq!(data::roster(map).enemies.len(), 18);
        assert!(ready(map));
        assert_eq!(schedule(map).len(), 8);
        for (i, &(enemy, at)) in schedule(map).iter().enumerate() {
            assert_eq!(at, (i as u32 + 1) * 120);
            assert!(next(map, i as u32, at as f32 - 0.01).is_none());
            assert_eq!(next(map, i as u32, at as f32).unwrap().name, enemy.name);
        }
        assert_eq!(
            schedule(map).last().unwrap().0.name,
            "High Inquisitor Whitemane"
        );
        assert!(next(map, 8, 10_000.).is_none());
        assert!(!won(map, 960., 7));
        assert!(!won(map, 959., 8));
        assert!(won(map, 960., 8));
        assert!(!campaign::DROPS.iter().any(|row| row.zone == 7));
        assert!(!campaign::zone_open(7, 51, &[true; campaign::CLEAR_COUNT]));
    }

    #[test]
    fn scheduled_sources_use_the_same_capped_boss_probability_as_combat() {
        for drop in campaign::DROPS
            .iter()
            .filter(|d| matches!(d.zone, 0 | 1 | 3))
        {
            assert!(drop_is_boss(drop), "{}", drop.creature);
            for (rank, expected) in [(0, 0.60), (1, 0.65), (2, 0.70), (255, 0.70)] {
                assert!(
                    (campaign::drop_chance(drop.chance, drop_is_boss(drop), rank) - expected).abs()
                        < 0.00001
                );
            }
        }
        assert_eq!(campaign::drop_chance(60, false, 2), 0.60);
        assert_eq!(campaign::drop_chance(98, true, 2), 1.0);
        let missing = campaign::DropRow {
            zone: 0,
            slot: 0,
            band: 0,
            creature: "Unscheduled",
            at_secs: 180,
            chance: 60,
        };
        assert!(!drop_is_boss(&missing));
    }

    fn elwynn() -> usize {
        data::MAPS
            .iter()
            .position(|map| map.name == "Elwynn Forest")
            .unwrap()
    }

    #[test]
    fn campaign_spawns_follow_the_displayed_drop_times() {
        let map = elwynn();
        for row in campaign::DROPS.iter().filter(|row| row.zone == 0) {
            let spawned = schedule(map)
                .iter()
                .position(|(enemy, _)| enemy.name == row.creature)
                .unwrap() as u32;
            assert!(next(map, spawned, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, spawned, row.at_secs as f32).unwrap().name,
                row.creature
            );
        }
        assert!(next(map, schedule(map).len() as u32, 10_000.0).is_none());
    }

    #[test]
    fn a_large_time_step_delivers_each_encounter_once() {
        let map = elwynn();
        let mut spawned = 0;
        let mut names = Vec::new();
        while let Some(enemy) = next(map, spawned, 1_000.0) {
            assert!(!names.contains(&enemy.name));
            names.push(enemy.name);
            spawned += 1;
        }
        assert_eq!(names.len(), data::roster(map).bosses.len());
        assert_eq!(names[0], "Hogger");
        assert_eq!(names.last(), Some(&"Fedfennel"));
    }

    #[test]
    fn final_campaign_drop_is_obtainable_before_victory() {
        let map = elwynn();
        assert!(!won(map, 900.0, 5));
        assert!(!won(map, 960.0, 5));
        assert!(won(map, 960.0, 6));
        assert!(!won(map, 959.0, 6));
    }

    #[test]
    fn ungoro_drops_follow_nine_bosses_but_do_not_unlock_unverified_world_content() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Fire Plume Ridge")
            .unwrap();
        assert!(ready(map));
        let mut slots: Vec<_> = campaign::DROPS
            .iter()
            .filter(|row| row.zone == 3)
            .map(|row| {
                assert_eq!(row.band, 1);
                row.slot
            })
            .collect();
        slots.sort_unstable();
        assert_eq!(slots, (0..campaign::GRID_SLOTS).collect::<Vec<_>>());
        assert_eq!(schedule(map).len(), 9);
        for (index, row) in campaign::DROPS.iter().filter(|d| d.zone == 3).enumerate() {
            assert!(next(map, index as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, index as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            assert!(drop_is_boss(row));
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, row.slot, row.band).is_some());
                assert!(crate::gear_visuals::piece(class, row.slot, row.band).is_some());
            }
        }
        assert!(next(map, 9, 10_000.0).is_none());
        assert!(!won(map, 959.0, 9));
        assert!(!won(map, 960.0, 8));
        assert!(won(map, 960.0, 9));
        assert!(!campaign::zone_open(3, 51, &[true; campaign::CLEAR_COUNT]));
        assert_eq!(
            campaign::zone_lock_reason(3, 51, &[true; campaign::CLEAR_COUNT]).as_deref(),
            Some("Un'Goro Crater world content is not ready")
        );
    }

    #[test]
    fn feralas_rare_schedule_is_complete_but_world_acceptance_still_blocks_entry() {
        let map = data::MAPS.iter().position(|m| m.name == "Feralas").unwrap();
        assert_eq!(data::MAPS[map].map, 1);
        assert!(ready(map));
        assert_eq!(schedule(map).len(), 9);
        for (slot, row) in campaign::DROPS
            .iter()
            .filter(|row| row.zone == 5)
            .enumerate()
        {
            assert_eq!((row.slot, row.band), (slot, 2));
            assert!(next(map, slot as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, slot as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            assert!(drop_is_boss(row));
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, slot, 2).is_some());
                assert!(crate::gear_visuals::piece(class, slot, 2).is_some());
            }
        }
        assert!(next(map, 9, 10_000.0).is_none());
        assert!(!won(map, 959.0, 9));
        assert!(!won(map, 960.0, 8));
        assert!(won(map, 960.0, 9));
        assert!(!campaign::zone_open(5, 51, &[true; campaign::CLEAR_COUNT]));
        assert_eq!(
            campaign::zone_lock_reason(5, 51, &[true; campaign::CLEAR_COUNT]).as_deref(),
            Some("Feralas world content is not ready")
        );
    }

    #[test]
    fn gurubashi_epic_schedule_is_complete_but_world_acceptance_still_blocks_entry() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Gurubashi Arena")
            .unwrap();
        assert_eq!(data::MAPS[map].map, 0);
        assert!(ready(map));
        assert_eq!(schedule(map).len(), 9);
        for (slot, row) in campaign::DROPS
            .iter()
            .filter(|row| row.zone == 6)
            .enumerate()
        {
            assert_eq!((row.slot, row.band), (slot, 3));
            assert!(next(map, slot as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, slot as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            assert!(drop_is_boss(row));
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, slot, 3).is_some());
                assert!(crate::gear_visuals::piece(class, slot, 3).is_some());
            }
        }
        assert!(next(map, 9, 10_000.0).is_none());
        assert!(!won(map, 959.0, 9));
        assert!(!won(map, 960.0, 8));
        assert!(won(map, 960.0, 9));
        assert!(!campaign::zone_open(6, 51, &[true; campaign::CLEAR_COUNT]));
        assert_eq!(
            campaign::zone_lock_reason(6, 51, &[true; campaign::CLEAR_COUNT]).as_deref(),
            Some("Gurubashi Arena world content is not ready")
        );
    }

    #[test]
    fn aq_legendary_armor_schedule_is_complete_but_world_acceptance_still_blocks_entry() {
        let map = data::MAPS
            .iter()
            .position(|m| m.name == "Gates of Ahn'Qiraj")
            .unwrap();
        assert_eq!(data::MAPS[map].map, 1);
        assert!(ready(map));
        assert_eq!(schedule(map).len(), 8);
        for (slot, row) in campaign::DROPS
            .iter()
            .filter(|row| row.zone == 10)
            .enumerate()
        {
            assert_eq!((row.slot, row.band), (slot, 5));
            assert!(next(map, slot as u32, row.at_secs as f32 - 0.01).is_none());
            assert_eq!(
                next(map, slot as u32, row.at_secs as f32).unwrap().name,
                row.creature
            );
            assert!(drop_is_boss(row));
            for class in 0..data::CLASSES.len() {
                assert!(campaign::piece_name(class, slot, 5).is_some());
                assert!(crate::gear_visuals::piece(class, slot, 5).is_some());
            }
        }
        assert!(next(map, 8, 10_000.0).is_none());
        assert!(!won(map, 959.0, 8));
        assert!(!won(map, 960.0, 7));
        assert!(won(map, 960.0, 8));
        assert!(!campaign::zone_open(10, 51, &[true; campaign::CLEAR_COUNT]));
        assert_eq!(
            campaign::zone_lock_reason(10, 51, &[true; campaign::CLEAR_COUNT]).as_deref(),
            Some("Gates of Ahn'Qiraj world content is not ready")
        );
    }

    #[test]
    fn non_campaign_maps_keep_the_arena_clock() {
        let map = data::MAPS
            .iter()
            .position(|map| campaign::zone_index(map.name).is_none())
            .unwrap();
        assert!(next(map, 0, data::BOSS_EVERY_SECS - 0.01).is_none());
        assert!(next(map, 0, data::BOSS_EVERY_SECS).is_some());
        assert!(!won(map, data::VICTORY_SECS - 0.01, 0));
        assert!(won(map, data::VICTORY_SECS, 0));
    }
}
