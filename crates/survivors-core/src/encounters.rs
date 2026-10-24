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
