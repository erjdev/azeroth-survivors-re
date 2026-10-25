//! Opt-in live controls for the production menu, independent of gameplay and saves.
use super::{Armory, Page, Phase, Typing};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub(crate) fn plugin(app: &mut App) {
    let Some(path) = std::env::var_os("WOW_SURVIVORS_UI_PREVIEW") else {
        return;
    };
    app.insert_resource(Preview {
        path: path.into(),
        seen: String::new(),
        poll: Timer::from_seconds(0.25, TimerMode::Repeating),
    })
    .add_systems(
        Update,
        reload.before(super::input).run_if(in_state(Phase::Menu)),
    );
}

#[derive(Resource)]
struct Preview {
    path: std::path::PathBuf,
    seen: String,
    poll: Timer,
}

struct Request {
    page: Page,
    width: u32,
    height: u32,
}

impl Request {
    fn parse(text: &str) -> Option<Self> {
        let (mut page, mut width, mut height) = (None, None, None);
        for line in text.lines() {
            let line = line.split('#').next()?.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line.split_once('=')?;
            let value = value.trim();
            match key.trim() {
                "page" if page.is_none() => page = Some(Page::preview(value)?),
                "width" if width.is_none() => width = Some(value.parse::<u32>().ok()?),
                "height" if height.is_none() => height = Some(value.parse::<u32>().ok()?),
                _ => return None,
            }
        }
        let request = Self {
            page: page?,
            width: width?,
            height: height?,
        };
        ((640..=7680).contains(&request.width) && (480..=4320).contains(&request.height))
            .then_some(request)
    }

    fn apply(&self, armory: &mut Armory, typing: &mut Typing, window: &mut Window) {
        armory.page = self.page;
        armory.focus = 0;
        typing.0 = false;
        window.resolution.set(self.width as f32, self.height as f32);
    }
}

fn reload(
    time: Res<Time<Real>>,
    mut preview: ResMut<Preview>,
    mut armory: ResMut<Armory>,
    mut typing: ResMut<Typing>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !preview.poll.tick(time.delta()).just_finished() {
        return;
    }
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let Ok(text) = std::fs::read_to_string(&preview.path) else {
        return;
    };
    if text == preview.seen {
        return;
    }
    let request = Request::parse(&text);
    preview.seen = text;
    let Some(request) = request else {
        warn!("Invalid Survivors UI preview; keeping current page and size");
        return;
    };
    request.apply(&mut armory, &mut typing, &mut window);
    info!("Survivors UI preview applied: {}", preview.seen.trim());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_unknown_duplicate_and_unsafe_sizes_are_rejected() {
        for text in [
            "page=collection\nwidth=1280",
            "page=playing\nwidth=1280\nheight=720",
            "page=collection\nwidth=1280\nheight=0",
            "page=collection\nwidth=NaN\nheight=720",
            "page=collection\nwidth=99999\nheight=720",
            "page=collection\nwidth=1280\nheight=720\nwidth=1024",
            "page=collection\nwidth=1280\nheight=720\nunlock=true",
        ] {
            assert!(Request::parse(text).is_none(), "{text}");
        }
    }

    #[test]
    fn live_navigation_resizes_and_releases_typing_without_resetting_drafts() {
        let request =
            Request::parse("# Logical pixels\npage = talents\nwidth = 1024\nheight = 768 # 4:3\n")
                .unwrap();
        let mut armory = Armory {
            page: Page::Heroes,
            focus: 1,
            draft_first: "Preview".into(),
            ..default()
        };
        let mut typing = Typing(true);
        let mut window = Window::default();
        request.apply(&mut armory, &mut typing, &mut window);
        assert!(armory.page == Page::Talents);
        assert_eq!(armory.focus, 0);
        assert!(!typing.0);
        assert_eq!(armory.draft_first, "Preview");
        assert_eq!(window.resolution.width(), 1024.);
        assert_eq!(window.resolution.height(), 768.);
    }
}
