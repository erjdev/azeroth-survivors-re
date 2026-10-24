//! Opt-in read-only movement queries against the actual streamed world.
use benilla_assets::coords::wow_to_bevy;
use benilla_world::terrain_stream::WorldLoadProgress;
use bevy::prelude::*;

#[derive(Resource)]
struct Probe {
    points: Vec<(String, Vec3)>,
    settled: u32,
    done: bool,
}

pub(super) fn plugin(app: &mut App) {
    let Some(path) = std::env::var_os("WOW_SURVIVORS_COLLISION_PROBE") else {
        return;
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        warn!("survivors collision probe: cannot read point file");
        return;
    };
    let mut points = Vec::new();
    for line in text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let words: Vec<_> = line.split_whitespace().collect();
        let coords: Vec<f32> = words
            .iter()
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        if words.len() != 4 || coords.len() != 3 || !coords.iter().all(|v| v.is_finite()) {
            warn!("survivors collision probe: expected label wow_x wow_y wow_z");
            return;
        }
        points.push((
            words[0].to_owned(),
            wow_to_bevy([coords[0], coords[1], coords[2]]),
        ));
    }
    app.insert_resource(Probe {
        points,
        settled: 0,
        done: false,
    })
    .add_systems(Update, probe);
}

fn probe(
    mut probe: ResMut<Probe>,
    progress: Res<WorldLoadProgress>,
    loading: Res<crate::loading_screen::LoadingScreen>,
    floor: crate::survivors::Floor,
) {
    if probe.done {
        return;
    }
    if !progress.presentable() || loading.covering() {
        probe.settled = 0;
        return;
    }
    // Let the physics query pipeline observe the final collider attachments.
    probe.settled += 1;
    if probe.settled < 60 {
        return;
    }
    probe.done = true;
    info!("survivors collision probe: settled focus {:?}, points {}; read-only queries, not hero movement", progress.focus_tile, probe.points.len());
    for (label, center) in &probe.points {
        for direction in [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z] {
            let start = *center - direction * 6.0;
            let Some(height) = floor.height(start) else {
                warn!("survivors collision probe: {label} direction {direction:?}: no start floor");
                continue;
            };
            let start = Vec3::new(start.x, height, start.z);
            let desired = start + direction * 12.0;
            let slide = floor.slide(start, desired, crate::survivors::HERO_RADIUS);
            let travel = floor.travel(start, desired, crate::survivors::HERO_RADIUS);
            let clear = floor.line_clear(start, desired);
            info!("survivors collision probe: {label} direction {direction:?}, start {start:?}, slide advance {:.3}, travel advance {:.3}, requested 12, line clear {clear}", (slide - start).dot(direction), (travel - start).dot(direction));
        }
    }
}
