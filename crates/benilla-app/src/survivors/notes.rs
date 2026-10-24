//! Patch notes and restart. A configured line may name a local build to start. A URL is ignored:
//! this does not download anything, and it does not run a file it fetched.

use std::path::PathBuf;

use bevy::prelude::*;

/// What the campaign build currently tells the player.
pub(super) const NOTES: &[&str] = &[
    "Named heroes keep their own talents. Zones open after the previous clear and the talent gate.",
    "Gear, cloaks, and legendary weapons stay on the class. Chests trade three cards for an upgrade.",
    "Rings, trinkets, and amulets last one run. Legendary amulets are marked seen when dealt.",
    "Achievement points buy perks and companion ranks. Two resets refund the perk points.",
    "Restart launches this build again. A path in survivors-update.txt can name another local build. A web address is not fetched.",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RestartPlan {
    /// Start the process that is already running.
    SameProcess,
    /// Start the local file the player named.
    NamedFile,
    /// The line is a web address. Do not fetch it.
    RefuseDownload,
}

/// `line` is the first line of `survivors-update.txt`. Empty means restart in place.
pub(super) fn plan_restart(line: &str) -> RestartPlan {
    let line = line.trim();
    if line.is_empty() {
        RestartPlan::SameProcess
    } else if line.contains("://") {
        RestartPlan::RefuseDownload
    } else {
        RestartPlan::NamedFile
    }
}

/// Leave and start this same build again.
pub(super) fn restart_same(exit: &mut MessageWriter<AppExit>) -> &'static str {
    launch(None, exit)
}

/// Start the build named in `survivors-update.txt`, when that line is a file on disk.
pub(super) fn restart_configured(exit: &mut MessageWriter<AppExit>) -> &'static str {
    let line = update_line();
    match plan_restart(&line) {
        RestartPlan::RefuseDownload => "A web address is not downloaded",
        RestartPlan::SameProcess => restart_same(exit),
        RestartPlan::NamedFile => {
            let path = PathBuf::from(line.trim());
            if path.is_file() {
                launch(Some(path), exit)
            } else {
                "That build is not a file on disk"
            }
        }
    }
}

fn update_line() -> String {
    let Some(home) = crate::local_state::home() else {
        return String::new();
    };
    let path = home.join("survivors-update.txt");
    crate::local_state::read_to_string(&path)
        .ok()
        .and_then(|text| text.lines().next().map(str::to_string))
        .unwrap_or_default()
}

fn launch(path: Option<PathBuf>, exit: &mut MessageWriter<AppExit>) -> &'static str {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, exit);
        if web_sys::window().is_some_and(|window| window.location().reload().is_ok()) {
            "Reloading"
        } else {
            "Could not reload; the game is still open"
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(exe) = path.or_else(|| std::env::current_exe().ok()) else {
            return "Could not locate this build; the game is still open";
        };
        match std::process::Command::new(&exe).spawn() {
            Ok(_) => {
                exit.write(AppExit::Success);
                "Restarting"
            }
            Err(error) => {
                warn!("Could not launch {}: {error}", exe.display());
                "Could not launch that build; the game is still open"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_failed_launch_does_not_close_the_working_game() {
        use bevy::ecs::system::SystemState;

        let mut world = World::new();
        world.init_resource::<Messages<AppExit>>();
        let mut state = SystemState::<MessageWriter<AppExit>>::new(&mut world);
        // A regular source file cannot be a directory containing an executable.
        // This fails without spawning a process or touching a player's installation.
        let missing = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml/missing"));
        let message = launch(Some(missing), &mut state.get_mut(&mut world));
        assert_eq!(
            message,
            "Could not launch that build; the game is still open"
        );
        assert!(world.resource::<Messages<AppExit>>().is_empty());
    }

    #[test]
    fn a_web_address_is_not_a_build_to_launch() {
        assert_eq!(plan_restart(""), RestartPlan::SameProcess);
        assert_eq!(plan_restart("   "), RestartPlan::SameProcess);
        assert_eq!(
            plan_restart("https://example.com/survivors.exe"),
            RestartPlan::RefuseDownload
        );
        assert_eq!(
            plan_restart("C:\\Games\\survivors.exe"),
            RestartPlan::NamedFile
        );
        assert!(!NOTES.is_empty());
    }
}
