//! The `warcraft-survivors` launcher: the build stamp, handed to the survivors mode.

use benilla_app::BuildId;

fn main() -> benilla_app::AppExit {
    let build = BuildId {
        version: env!("CARGO_PKG_VERSION"),
        describe: env!("BENILLA_GIT_DESCRIBE"),
        sha: env!("BENILLA_GIT_SHA"),
        short: env!("BENILLA_GIT_SHORT"),
        date: env!("BENILLA_GIT_DATE"),
        profile: env!("BENILLA_PROFILE"),
        project_dir: env!("BENILLA_PROJECT_DIR"),
        ..Default::default()
    };
    if std::env::args().any(|arg| arg == "--world-preview") {
        #[cfg(feature = "dev")]
        {
            return benilla_app::run_worldview(build);
        }
        #[cfg(not(feature = "dev"))]
        {
            eprintln!("--world-preview requires a development build");
            return benilla_app::AppExit::error();
        }
    }
    benilla_app::run_survivors(build)
}
