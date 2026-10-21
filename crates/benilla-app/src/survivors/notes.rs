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
pub(super) fn restart_same(exit: &mut MessageWriter<AppExit>) {
    launch(None, exit);
}

/// Start the build named in `survivors-update.txt`, when that line is a file on disk.
pub(super) fn restart_configured(exit: &mut MessageWriter<AppExit>) -> &'static str {
    let line = update_line();
    match plan_restart(&line) {
        RestartPlan::RefuseDownload => "A web address is not downloaded",
        RestartPlan::SameProcess => {
            launch(None, exit);
            "Restarting"
        }
        RestartPlan::NamedFile => {
            let path = PathBuf::from(line.trim());
            if path.is_file() {
                launch(Some(path), exit);
                "Starting that build"
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

fn launch(path: Option<PathBuf>, exit: &mut MessageWriter<AppExit>) {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = path;
        if let Some(window) = web_sys::window() {
            let _ = window.location().reload();
        }
        let _ = exit;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let exe = path.or_else(|| std::env::current_exe().ok());
        if let Some(exe) = exe {
            let _ = std::process::Command::new(exe).spawn();
        }
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
