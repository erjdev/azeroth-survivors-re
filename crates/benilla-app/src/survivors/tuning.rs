//! Native development tuning, polled independently of virtual time so pause does not stop edits.

use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use survivors_core::tuning::Tuning;

pub(super) fn plugin(app: &mut App) {
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.init_resource::<Live>().add_systems(
        Update,
        reload.before(benilla_world::schedule::WorldStage::Input),
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
