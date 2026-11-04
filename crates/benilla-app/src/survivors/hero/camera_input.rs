//! Apply camera gestures independently of rendering and world collision.
use super::*;

pub(super) fn apply<'a>(
    rig: &mut CamRig,
    settings: &super::super::camera::CameraSettings,
    phase: Phase,
    covered: bool,
    toggle: bool,
    drag: Option<Vec2>,
    wheel: impl Iterator<Item = &'a MouseWheel>,
) {
    let active = phase == Phase::Playing && !covered;
    if active && toggle {
        rig.mode = match rig.mode {
            CamMode::Fixed => CamMode::Orbit,
            CamMode::Orbit => {
                rig.yaw = 0.;
                rig.pitch = FIXED_PITCH;
                CamMode::Fixed
            }
        };
    }
    // Consume even blocked events, so UI scrolling cannot become zoom on resume.
    for event in wheel {
        if active {
            let notches = match event.unit {
                MouseScrollUnit::Line => event.y,
                MouseScrollUnit::Pixel => event.y * 0.02,
            };
            rig.dist = (rig.dist * (1. - 0.12 * settings.zoom_speed * notches).clamp(0.5, 1.5))
                .clamp(ZOOM_MIN, ZOOM_MAX);
        }
    }
    if active && rig.mode == CamMode::Orbit {
        if let Some(delta) = drag {
            let delta = settings.mouse_delta(delta);
            rig.yaw -= delta.x * ORBIT_RATE;
            rig.pitch = (rig.pitch + delta.y * ORBIT_RATE).clamp(PITCH_MIN, PITCH_MAX);
        }
    }
}

/// Time-based controller motion shares the mouse camera's inversion and limits.
pub(super) fn analog(
    rig: &mut CamRig,
    settings: &super::super::camera::CameraSettings,
    frame: survivors_core::controller::Frame,
    dt: f32,
) {
    if !dt.is_finite() || dt <= 0.0 {
        return;
    }
    let look = Vec2::new(frame.look[0], -frame.look[1]);
    if look != Vec2::ZERO {
        rig.mode = CamMode::Orbit;
        let delta = settings.mouse_delta(look) * (1.8 * dt);
        rig.yaw -= delta.x;
        rig.pitch = (rig.pitch + delta.y).clamp(PITCH_MIN, PITCH_MAX);
    }
    rig.dist =
        (rig.dist * (-1.5 * settings.zoom_speed * frame.zoom * dt).exp()).clamp(ZOOM_MIN, ZOOM_MAX);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controller_camera_is_time_based_inverted_and_bounded() {
        let mut settings = super::super::super::camera::CameraSettings::default();
        settings.camera_speed = 1.0;
        settings.zoom_speed = 1.0;
        settings.invert_x = false;
        settings.invert_y = false;
        let frame = survivors_core::controller::Frame {
            look: [0.4, -0.2],
            zoom: 0.5,
            ..default()
        };
        let mut fine = CamRig::default();
        let mut batch = CamRig::default();
        for _ in 0..10 {
            analog(&mut fine, &settings, frame, 0.01);
        }
        analog(&mut batch, &settings, frame, 0.1);
        assert!((fine.yaw - batch.yaw).abs() < 0.00001);
        assert!((fine.pitch - batch.pitch).abs() < 0.00001);
        assert!((fine.dist - batch.dist).abs() < 0.0001);
        assert!(matches!(batch.mode, CamMode::Orbit));
        settings.invert_x = true;
        settings.invert_y = true;
        let mut inverse = CamRig::default();
        analog(&mut inverse, &settings, frame, 0.1);
        assert!((inverse.yaw + batch.yaw).abs() < 0.00001);
        assert!((inverse.pitch + batch.pitch - 2.0 * FIXED_PITCH).abs() < 0.00001);
        let mut idle = CamRig::default();
        analog(&mut idle, &settings, Default::default(), 0.1);
        assert!(matches!(idle.mode, CamMode::Fixed));
        assert_eq!(idle.dist, 30.0);
        analog(&mut batch, &settings, frame, 100.0);
        assert!((PITCH_MIN..=PITCH_MAX).contains(&batch.pitch));
        assert_eq!(batch.dist, ZOOM_MIN);
    }

    #[test]
    fn zoom_speed_changes_distance_without_changing_camera_rotation() {
        let mut settings = super::super::super::camera::CameraSettings::default();
        for (speed, expected) in [(0.5, 28.2), (1., 26.4), (2., 22.8)] {
            settings.zoom_speed = speed;
            let mut rig = CamRig::default();
            apply(
                &mut rig,
                &settings,
                Phase::Playing,
                false,
                false,
                None,
                [wheel(MouseScrollUnit::Line, 1.)].iter(),
            );
            assert!((rig.dist - expected).abs() < 0.0001);
            assert_eq!((rig.yaw, rig.pitch), (0., FIXED_PITCH));
        }
    }

    fn wheel(unit: MouseScrollUnit, y: f32) -> MouseWheel {
        MouseWheel {
            unit,
            x: 0.,
            y,
            window: Entity::PLACEHOLDER,
        }
    }

    #[test]
    fn menus_rewards_pause_and_loading_drain_input_without_changing_the_rig() {
        let settings = super::super::super::camera::CameraSettings::default();
        for (phase, covered) in [
            (Phase::Menu, false),
            (Phase::Paused, false),
            (Phase::LevelUp, false),
            (Phase::GameOver, false),
            (Phase::Playing, true),
        ] {
            let mut rig = CamRig {
                mode: CamMode::Orbit,
                yaw: 0.7,
                pitch: 0.8,
                ..default()
            };
            let events = [
                wheel(MouseScrollUnit::Line, 1.),
                wheel(MouseScrollUnit::Pixel, -50.),
            ];
            let mut consumed = 0;
            apply(
                &mut rig,
                &settings,
                phase,
                covered,
                true,
                Some(Vec2::splat(100.)),
                events.iter().inspect(|_| consumed += 1),
            );
            assert_eq!(consumed, 2);
            assert!(rig.mode == CamMode::Orbit);
            assert_eq!((rig.yaw, rig.pitch, rig.dist), (0.7, 0.8, 30.));
        }
    }

    #[test]
    fn orbit_uses_each_inverted_axis_and_locking_restores_fixed_view() {
        let mut settings = super::super::super::camera::CameraSettings::default();
        settings.camera_speed = 1.;
        for (invert_x, invert_y) in [(false, false), (true, false), (false, true), (true, true)] {
            settings.invert_x = invert_x;
            settings.invert_y = invert_y;
            let mut rig = CamRig {
                pitch: 0.8,
                ..default()
            };
            apply(
                &mut rig,
                &settings,
                Phase::Playing,
                false,
                true,
                Some(Vec2::new(10., 5.)),
                std::iter::empty(),
            );
            assert!(rig.mode == CamMode::Orbit);
            assert!((rig.yaw - if invert_x { 0.08 } else { -0.08 }).abs() < 1e-6);
            assert!((rig.pitch - if invert_y { 0.76 } else { 0.84 }).abs() < 1e-6);
            apply(
                &mut rig,
                &settings,
                Phase::Playing,
                false,
                true,
                Some(Vec2::splat(99.)),
                std::iter::empty(),
            );
            assert!(rig.mode == CamMode::Fixed);
            assert_eq!((rig.yaw, rig.pitch), (0., FIXED_PITCH));
        }
    }

    #[test]
    fn zoom_units_match_and_extreme_gestures_stay_inside_limits() {
        let mut settings = super::super::super::camera::CameraSettings::default();
        settings.camera_speed = 1.;
        settings.zoom_speed = 1.;
        settings.invert_x = false;
        settings.invert_y = false;
        let mut line = CamRig::default();
        let mut pixel = CamRig::default();
        apply(
            &mut line,
            &settings,
            Phase::Playing,
            false,
            false,
            None,
            [wheel(MouseScrollUnit::Line, 1.)].iter(),
        );
        apply(
            &mut pixel,
            &settings,
            Phase::Playing,
            false,
            false,
            None,
            [wheel(MouseScrollUnit::Pixel, 50.)].iter(),
        );
        assert_eq!(line.dist, pixel.dist);
        assert!(line.dist < 30.);
        for (scroll, pitch, expected_zoom, expected_pitch) in [
            (100., 1e6, ZOOM_MIN, PITCH_MAX),
            (-100., -1e6, ZOOM_MAX, PITCH_MIN),
        ] {
            line.mode = CamMode::Orbit;
            let events = (0..30)
                .map(|_| wheel(MouseScrollUnit::Line, scroll))
                .collect::<Vec<_>>();
            apply(
                &mut line,
                &settings,
                Phase::Playing,
                false,
                false,
                Some(Vec2::new(0., pitch)),
                events.iter(),
            );
            assert_eq!((line.dist, line.pitch), (expected_zoom, expected_pitch));
        }
    }
}
