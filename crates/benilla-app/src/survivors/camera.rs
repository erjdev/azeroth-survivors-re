//! Survivors camera preferences, separate from the stock client's CVars.
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct CameraSettings {
    pub auto_push: bool,
    pub cutaway: bool,
    pub silhouette_enemies: bool,
    pub silhouette_friendlies: bool,
    pub invert_x: bool,
    pub invert_y: bool,
    pub camera_speed: f32,
    pub zoom_speed: f32,
    pub stick_dead_zone: f32,
    resolved_distance: Option<f32>,
}

impl Default for CameraSettings {
    fn default() -> Self {
        let text = crate::local_state::home()
            .and_then(|p| crate::local_state::read_to_string(&p.join("survivors-camera.txt")).ok())
            .unwrap_or_default();
        Self::parse(&text)
    }
}

impl CameraSettings {
    pub(super) fn parse(text: &str) -> Self {
        let mut settings = Self {
            auto_push: true,
            cutaway: true,
            silhouette_enemies: false,
            silhouette_friendlies: false,
            invert_x: false,
            invert_y: false,
            camera_speed: 1.,
            zoom_speed: 1.,
            stick_dead_zone: survivors_core::controller::DEFAULT_DEAD_ZONE,
            resolved_distance: None,
        };
        for line in text.lines() {
            match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                ["auto_push", "0"] => settings.auto_push = false,
                ["auto_push", "1"] => settings.auto_push = true,
                ["cutaway", "0"] => settings.cutaway = false,
                ["cutaway", "1"] => settings.cutaway = true,
                ["silhouette_enemies", "0"] => settings.silhouette_enemies = false,
                ["silhouette_enemies", "1"] => settings.silhouette_enemies = true,
                ["silhouette_friendlies", "0"] => settings.silhouette_friendlies = false,
                ["silhouette_friendlies", "1"] => settings.silhouette_friendlies = true,
                ["invert_x", "0"] => settings.invert_x = false,
                ["invert_x", "1"] => settings.invert_x = true,
                ["invert_y", "0"] => settings.invert_y = false,
                ["invert_y", "1"] => settings.invert_y = true,
                ["stick_dead_zone", value] => {
                    if let Ok(value) = value.parse::<f32>() {
                        if value.is_finite() {
                            settings.stick_dead_zone =
                                value.clamp(0., survivors_core::controller::MAX_DEAD_ZONE);
                        }
                    }
                }
                [key @ ("camera_speed" | "zoom_speed"), value] => {
                    if let Ok(value) = value.parse::<f32>() {
                        if value.is_finite() {
                            if *key == "camera_speed" {
                                settings.camera_speed = value.clamp(0.25, 3.);
                            } else {
                                settings.zoom_speed = value.clamp(0.25, 3.);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        settings
    }

    fn encode(&self) -> String {
        format!(
            "auto_push {}\ncutaway {}\nsilhouette_enemies {}\nsilhouette_friendlies {}\ninvert_x {}\ninvert_y {}\ncamera_speed {}\nzoom_speed {}\nstick_dead_zone {}\n",
            u8::from(self.auto_push),
            u8::from(self.cutaway),
            u8::from(self.silhouette_enemies),
            u8::from(self.silhouette_friendlies),
            u8::from(self.invert_x),
            u8::from(self.invert_y),
            self.camera_speed,
            self.zoom_speed,
            self.stick_dead_zone
        )
    }

    pub fn toggle(&mut self) {
        self.auto_push = !self.auto_push;
        self.resolved_distance = None;
        self.save();
    }

    pub fn toggle_cutaway(&mut self) {
        self.cutaway = !self.cutaway;
        self.save();
    }

    pub fn toggle_silhouette(&mut self, enemies: bool) {
        if enemies {
            self.silhouette_enemies = !self.silhouette_enemies;
        } else {
            self.silhouette_friendlies = !self.silhouette_friendlies;
        }
        self.save();
    }

    pub fn silhouette_filters(&self) -> survivors_core::silhouette::Filters {
        survivors_core::silhouette::Filters {
            enemies: self.silhouette_enemies,
            friendlies: self.silhouette_friendlies,
        }
    }

    /// Reset Graphics preferences only, preserving the independent Controls category.
    pub fn restore_defaults(&mut self) {
        self.auto_push = true;
        self.cutaway = true;
        self.silhouette_enemies = false;
        self.silhouette_friendlies = false;
        self.resolved_distance = None;
        self.save();
    }

    pub fn toggle_inversion(&mut self, horizontal: bool) {
        if horizontal {
            self.invert_x = !self.invert_x;
        } else {
            self.invert_y = !self.invert_y;
        }
        self.save();
    }

    pub fn restore_controls(&mut self) {
        self.invert_x = false;
        self.invert_y = false;
        self.camera_speed = 1.;
        self.zoom_speed = 1.;
        self.stick_dead_zone = survivors_core::controller::DEFAULT_DEAD_ZONE;
        self.save();
    }

    pub fn set_dead_zone(&mut self, value: f32) {
        if !value.is_finite() {
            return;
        }
        let value = value.clamp(0., survivors_core::controller::MAX_DEAD_ZONE);
        if self.stick_dead_zone != value {
            self.stick_dead_zone = value;
            self.save();
        }
    }

    pub fn set_speed(&mut self, zoom: bool, value: f32) {
        if !value.is_finite() {
            return;
        }
        let speed = if zoom {
            &mut self.zoom_speed
        } else {
            &mut self.camera_speed
        };
        let value = value.clamp(0.25, 3.);
        if *speed != value {
            *speed = value;
            self.save();
        }
    }

    pub fn mouse_delta(&self, delta: Vec2) -> Vec2 {
        delta
            * self.camera_speed
            * Vec2::new(
                if self.invert_x { -1. } else { 1. },
                if self.invert_y { -1. } else { 1. },
            )
    }

    fn save(&self) {
        if let Some(home) = crate::local_state::home() {
            if let Err(error) =
                crate::local_state::write_atomic(&home.join("survivors-camera.txt"), &self.encode())
            {
                warn!("survivors: camera settings save failed: {error}");
            }
        }
    }

    /// Pull inward immediately for safety; ease outward without changing requested zoom.
    pub fn resolve(&mut self, look: Vec3, desired: Vec3, safe: Vec3, dt: f32) -> Vec3 {
        if !self.auto_push {
            self.resolved_distance = None;
            return desired;
        }
        let arm = desired - look;
        let length = arm.length();
        if length < 1e-5 {
            return desired;
        }
        let limit = safe.distance(look).min(length);
        let previous = self.resolved_distance.unwrap_or(limit);
        let distance = if limit <= previous {
            limit
        } else {
            previous + (limit - previous) * (1.0 - (-8.0 * dt.max(0.0)).exp())
        };
        self.resolved_distance = Some(distance);
        look + arm / length * distance
    }
}

/// Opt-in, one-second camera samples for reproducing obstruction without changing the solver.
/// Coordinates are raw WoW yards so the source-geometry audit can inspect the same location.
pub(super) struct CameraTrace {
    enabled: bool,
    next: f64,
}

impl Default for CameraTrace {
    fn default() -> Self {
        Self {
            enabled: std::env::var("WOW_SURVIVORS_CAMERA_TRACE").is_ok_and(|v| v == "1"),
            next: 0.0,
        }
    }
}

impl CameraTrace {
    pub fn record(&mut self, now: f64, look: Vec3, desired: Vec3, safe: Vec3, resolved: Vec3) {
        if !self.enabled || now < self.next {
            return;
        }
        self.next = now + 1.0;
        info!(
            "survivors camera: look_wow={:?} desired_wow={:?} safe_wow={:?} resolved_wow={:?} requested={:.3} safe={:.3} resolved={:.3}",
            benilla_assets::coords::bevy_to_wow(look),
            benilla_assets::coords::bevy_to_wow(desired),
            benilla_assets::coords::bevy_to_wow(safe),
            benilla_assets::coords::bevy_to_wow(resolved),
            desired.distance(look),
            safe.distance(look),
            resolved.distance(look),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_zone_loads_old_saves_bounds_values_and_round_trips() {
        for (text, expected) in [
            ("", 0.25),
            ("stick_dead_zone 0.3", 0.3),
            ("stick_dead_zone -1", 0.),
            ("stick_dead_zone 3", 0.5),
            ("stick_dead_zone NaN", 0.25),
            ("stick_dead_zone inf", 0.25),
        ] {
            let settings = CameraSettings::parse(text);
            assert_eq!(settings.stick_dead_zone, expected);
            assert_eq!(
                CameraSettings::parse(&settings.encode()).stick_dead_zone,
                expected
            );
        }
    }

    #[test]
    fn speeds_load_safely_and_scale_mouse_input_without_changing_inversion() {
        for (text, expected) in [
            ("", (1., 1.)),
            ("camera_speed 1.5\nzoom_speed 0.5", (1.5, 0.5)),
            ("camera_speed -1\nzoom_speed 20", (0.25, 3.)),
            ("camera_speed NaN\nzoom_speed inf", (1., 1.)),
        ] {
            let settings = CameraSettings::parse(text);
            assert_eq!((settings.camera_speed, settings.zoom_speed), expected);
            let loaded = CameraSettings::parse(&settings.encode());
            assert_eq!((loaded.camera_speed, loaded.zoom_speed), expected);
        }
        let settings = CameraSettings::parse("camera_speed 2\ninvert_x 1");
        assert_eq!(
            settings.mouse_delta(Vec2::new(3., -4.)),
            Vec2::new(-6., -8.)
        );
    }

    #[test]
    fn saved_inversion_changes_only_the_selected_mouse_axis() {
        let delta = Vec2::new(12., -7.);
        for (text, expected) in [
            ("", delta),
            ("invert_x 1", Vec2::new(-12., -7.)),
            ("invert_y 1", Vec2::new(12., 7.)),
            ("invert_x 1\ninvert_y 1", -delta),
            ("invert_x broken\ninvert_y 9", delta),
        ] {
            let settings = CameraSettings::parse(text);
            assert_eq!(settings.mouse_delta(delta), expected);
            assert_eq!(
                CameraSettings::parse(&settings.encode()).mouse_delta(delta),
                expected
            );
            assert!(settings.auto_push && settings.cutaway);
        }
    }

    #[test]
    fn silhouette_filters_are_independent_persistent_and_graphics_scoped() {
        for enemies in [false, true] {
            for friendlies in [false, true] {
                let mut settings = CameraSettings::parse("invert_x 1\ncamera_speed 2");
                if enemies {
                    settings.toggle_silhouette(true);
                }
                if friendlies {
                    settings.toggle_silhouette(false);
                }
                let mut loaded = CameraSettings::parse(&settings.encode());
                assert_eq!(
                    loaded.silhouette_filters(),
                    survivors_core::silhouette::Filters {
                        enemies,
                        friendlies
                    }
                );
                loaded.restore_controls();
                assert_eq!(loaded.silhouette_filters(), settings.silhouette_filters());
                settings.restore_defaults();
                assert!(!settings.silhouette_enemies && !settings.silhouette_friendlies);
                assert!(settings.invert_x);
                assert_eq!(settings.camera_speed, 2.);
            }
        }
        let old =
            CameraSettings::parse("cutaway 0\nsilhouette_enemies broken\nsilhouette_friendlies 9");
        assert!(!old.silhouette_enemies && !old.silhouette_friendlies && !old.cutaway);
    }

    #[test]
    fn preferences_round_trip_and_old_or_invalid_files_keep_collision_enabled() {
        for enabled in [false, true] {
            let settings = CameraSettings {
                auto_push: enabled,
                cutaway: !enabled,
                silhouette_enemies: enabled,
                silhouette_friendlies: !enabled,
                invert_x: enabled,
                invert_y: !enabled,
                camera_speed: 1.5,
                zoom_speed: 0.5,
                stick_dead_zone: 0.3,
                resolved_distance: Some(4.0),
            };
            let loaded = CameraSettings::parse(&settings.encode());
            assert_eq!(loaded.auto_push, enabled);
            assert_eq!(loaded.cutaway, !enabled);
            assert_eq!(loaded.silhouette_enemies, enabled);
            assert_eq!(loaded.silhouette_friendlies, !enabled);
            assert_eq!(loaded.resolved_distance, None);
            assert_eq!(loaded.invert_x, enabled);
            assert_eq!(loaded.invert_y, !enabled);
            assert_eq!((loaded.camera_speed, loaded.zoom_speed), (1.5, 0.5));
        }
        assert!(CameraSettings::parse("").auto_push);
        assert!(CameraSettings::parse("auto_push 0").cutaway);
        assert!(CameraSettings::parse("cutaway broken").cutaway);
        assert!(CameraSettings::parse("auto_push broken\nunknown 0").auto_push);
    }

    #[test]
    fn obstruction_is_immediate_return_is_smooth_and_disabled_is_unconstrained() {
        let mut settings = CameraSettings::parse("");
        let desired = Vec3::Z * 30.0;
        assert_eq!(
            settings.resolve(Vec3::ZERO, desired, desired, 0.016),
            desired
        );
        assert_eq!(
            settings
                .resolve(Vec3::ZERO, desired, Vec3::Z * 5.0, 0.016)
                .z,
            5.0
        );
        let returning = settings.resolve(Vec3::ZERO, desired, desired, 0.016).z;
        assert!(returning > 5.0 && returning < 30.0);
        assert_eq!(
            settings
                .resolve(Vec3::ZERO, desired, Vec3::Z * 2.0, 0.016)
                .z,
            2.0
        );
        settings.auto_push = false;
        assert_eq!(
            settings.resolve(Vec3::ZERO, desired, Vec3::Z * 2.0, 0.016),
            desired
        );
        assert_eq!(settings.resolved_distance, None);
    }
}
