//! Opt-in native UI capture for the README and visual acceptance runs.
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use std::path::PathBuf;

#[derive(Resource)]
struct CaptureDirectory {
    path: PathBuf,
    next: u32,
}

pub(super) fn plugin(app: &mut App) {
    let Some(path) = std::env::var_os("WOW_SURVIVORS_UI_SHOTS").filter(|p| !p.is_empty()) else {
        return;
    };
    let path = PathBuf::from(path);
    if let Err(error) = std::fs::create_dir_all(&path) {
        warn!("survivors: cannot create UI capture directory: {error}");
        return;
    }
    app.insert_resource(CaptureDirectory { path, next: 0 })
        .add_systems(Update, capture);
}

fn capture(
    keys: Res<ButtonInput<KeyCode>>,
    mut directory: ResMut<CaptureDirectory>,
    mut commands: Commands,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    scale: Res<UiScale>,
) {
    if !keys.just_pressed(KeyCode::F12) {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let path = loop {
        let path = directory.path.join(format!("ui-{:04}.png", directory.next));
        directory.next += 1;
        if !path.exists() {
            break path;
        }
    };
    info!(
        "Survivors UI capture requested: {} · logical {}×{} · physical {}×{} · UI scale {}",
        path.display(),
        window.width(),
        window.height(),
        window.physical_width(),
        window.physical_height(),
        scale.0
    );
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
