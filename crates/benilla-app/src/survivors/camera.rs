//! Survivors camera preferences, separate from the stock client's CVars.
use bevy::prelude::*;

#[derive(Resource)]
pub(super) struct CameraSettings {
    pub auto_push: bool,
    pub cutaway: bool,
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
    fn parse(text: &str) -> Self {
        let mut settings = Self {
            auto_push: true,
            cutaway: true,
            resolved_distance: None,
        };
        for line in text.lines() {
            match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                ["auto_push", "0"] => settings.auto_push = false,
                ["auto_push", "1"] => settings.auto_push = true,
                ["cutaway", "0"] => settings.cutaway = false,
                ["cutaway", "1"] => settings.cutaway = true,
                _ => {}
            }
        }
        settings
    }

    fn encode(&self) -> String {
        format!(
            "auto_push {}\ncutaway {}\n",
            u8::from(self.auto_push),
            u8::from(self.cutaway)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip_and_old_or_invalid_files_keep_collision_enabled() {
        for enabled in [false, true] {
            let settings = CameraSettings {
                auto_push: enabled,
                cutaway: !enabled,
                resolved_distance: Some(4.0),
            };
            let loaded = CameraSettings::parse(&settings.encode());
            assert_eq!(loaded.auto_push, enabled);
            assert_eq!(loaded.cutaway, !enabled);
            assert_eq!(loaded.resolved_distance, None);
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
