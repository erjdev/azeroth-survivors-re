//! Analog stick filtering shared by movement, camera and future menu gestures.
//! Callers supply normalized raw axes, before any per-axis dead zone.

pub const DEFAULT_DEAD_ZONE: f32 = 0.25;
pub const MAX_DEAD_ZONE: f32 = 0.5;

/// Ignore circular center drift, then continuously rescale the remaining travel.
/// Direction is preserved and diagonal travel cannot exceed full deflection.
pub fn stick(axes: [f32; 2], dead_zone: f32) -> [f32; 2] {
    if !axes.iter().all(|axis| axis.is_finite()) {
        return [0.0; 2];
    }
    let dead_zone = if dead_zone.is_finite() {
        dead_zone.clamp(0.0, MAX_DEAD_ZONE)
    } else {
        DEFAULT_DEAD_ZONE
    };
    let [x, y] = axes.map(|axis| axis.clamp(-1.0, 1.0));
    let length = x.hypot(y);
    if length <= dead_zone || length == 0.0 {
        return [0.0; 2];
    }
    let strength = (length.min(1.0) - dead_zone) / (1.0 - dead_zone);
    [x / length * strength, y / length * strength]
}

/// One device's current sample. Inactive frames discard every action; a caller
/// must sample again on resume rather than queueing gestures behind a modal.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frame {
    pub movement: [f32; 2],
    pub look: [f32; 2],
    pub zoom: f32,
    pub jump: bool,
}

impl Frame {
    pub fn filtered(self, dead_zone: f32, active: bool) -> Self {
        if !active {
            return Self::default();
        }
        Self {
            movement: stick(self.movement, dead_zone),
            look: stick(self.look, dead_zone),
            zoom: if self.zoom.is_finite() {
                self.zoom.clamp(-1.0, 1.0)
            } else {
                0.0
            },
            jump: self.jump,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drift_boundary_and_analog_travel_are_continuous() {
        assert_eq!(stick([0.15, 0.15], 0.25), [0.0, 0.0]);
        assert_eq!(stick([0.25, 0.0], 0.25), [0.0, 0.0]);
        let just_outside = stick([0.25001, 0.0], 0.25);
        assert!(just_outside[0] > 0.0 && just_outside[0] < 0.0001);
        assert_eq!(stick([0.625, 0.0], 0.25), [0.5, 0.0]);
        assert_eq!(stick([-1.0, 0.0], 0.25), [-1.0, 0.0]);
        assert_eq!(stick([0.0, 0.0], 0.0), [0.0, 0.0]);
    }

    #[test]
    fn diagonal_directions_keep_their_angle_and_never_gain_speed() {
        for x in -10..=10 {
            for y in -10..=10 {
                let input = [x as f32 / 10.0, y as f32 / 10.0];
                let [a, b] = stick(input, 0.25);
                assert!(a.hypot(b) <= 1.000001);
                assert!((a * input[1] - b * input[0]).abs() < 0.000001);
            }
        }
        let [x, y] = stick([1.0, 1.0], 0.25);
        assert!((x.hypot(y) - 1.0).abs() < 0.000001);
    }

    #[test]
    fn invalid_device_data_and_preferences_stay_finite() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(stick([value, 1.0], 0.25), [0.0, 0.0]);
            assert_eq!(stick([1.0, value], 0.25), [0.0, 0.0]);
            assert_eq!(stick([0.625, 0.0], value), [0.5, 0.0]);
        }
        assert_eq!(stick([0.5, 0.0], 1.0), [0.0, 0.0]);
        assert_eq!(stick([0.5, 0.0], -1.0), [0.5, 0.0]);
        assert_eq!(stick([100.0, 0.0], 0.25), [1.0, 0.0]);
    }

    #[test]
    fn blocked_samples_cannot_leak_movement_camera_zoom_or_jump() {
        let raw = Frame {
            movement: [0.625, 0.0],
            look: [0.0, -0.625],
            zoom: 2.0,
            jump: true,
        };
        assert_eq!(raw.filtered(0.25, false), Frame::default());
        assert_eq!(
            raw.filtered(0.25, true),
            Frame {
                movement: [0.5, 0.0],
                look: [0.0, -0.5],
                zoom: 1.0,
                jump: true,
            }
        );
        assert_eq!(Frame::default().filtered(0.25, true), Frame::default());
        assert_eq!(
            Frame {
                zoom: f32::NAN,
                ..raw
            }
            .filtered(0.25, true)
            .zoom,
            0.0
        );
    }
}
