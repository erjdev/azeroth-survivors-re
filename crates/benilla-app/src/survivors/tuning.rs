//! Native development tuning, polled independently of virtual time so pause does not stop edits.

use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use survivors_core::tuning::Tuning;

#[derive(Resource, Default)]
pub(super) struct UiLayout {
    pub value: survivors_core::layout::Layout,
    pub version: u64,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<UiLayout>();
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.init_resource::<Live>().add_systems(
        Update,
        (reload, reload_layout).before(benilla_world::schedule::WorldStage::Input),
    );
    #[cfg(not(all(feature = "dev", not(target_arch = "wasm32"))))]
    let _ = app;
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
#[derive(Resource, Default)]
struct Live {
    current: Tuning,
    text: Option<String>,
    next: f64,
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
fn reload(time: Res<Time<Real>>, mut live: ResMut<Live>, mut run: ResMut<super::Run>) {
    if time.elapsed_secs_f64() >= live.next {
        live.next = time.elapsed_secs_f64() + 0.5;
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../survivors-core/tuning.txt");
        if std::fs::metadata(&path).is_ok_and(|metadata| metadata.len() <= 16_384) {
            if let Ok(text) = std::fs::read_to_string(path) {
                if live.text.as_ref() != Some(&text) {
                    match Tuning::parse(&text) {
                        Ok(next) => {
                            live.current = next;
                            info!("Survivors tuning reloaded: {next:?}");
                        }
                        Err(error) => warn!("Survivors tuning unchanged: {error}"),
                    }
                    live.text = Some(text);
                }
            }
        }
    }
    if run.tuning != live.current {
        run.tuning = live.current;
        run.version = run.version.wrapping_add(1);
    }
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
fn reload_layout(
    time: Res<Time<Real>>,
    mut layout: ResMut<UiLayout>,
    mut next: Local<f64>,
    mut previous: Local<Option<String>>,
) {
    if time.elapsed_secs_f64() < *next {
        return;
    }
    *next = time.elapsed_secs_f64() + 0.5;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../survivors-core/ui.txt");
    if !std::fs::metadata(&path).is_ok_and(|m| m.len() <= 16_384) {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    if previous.as_ref() == Some(&text) {
        return;
    }
    match survivors_core::layout::Layout::parse(&text) {
        Ok(value) => {
            if layout.value != value {
                layout.value = value;
                layout.version = layout.version.wrapping_add(1);
            }
            info!("Survivors UI layout reloaded: {value:?}");
        }
        Err(error) => warn!("Survivors UI layout unchanged: {error}"),
    }
    *previous = Some(text);
}
