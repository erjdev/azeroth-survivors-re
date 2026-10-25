//! Camera-to-hero cutaway geometry. Rendering and collision policy stay separate.
//! CPU reference for the Survivors scenery shader; actor classification belongs to the renderer.

/// A projected circular reveal region, measured in world yards at the hero's focus plane.
#[derive(Clone, Copy, Debug)]
pub struct Cutaway {
    pub camera: [f32; 3],
    pub focus: [f32; 3],
    /// Outer radius at the focus plane. Zero disables the cutaway.
    pub radius: f32,
    /// Fraction of the radius used by the soft edge, clamped to 0..1.
    pub feather: f32,
    /// Preserve geometry this close to the focus plane and everything behind it.
    pub focus_clearance: f32,
}

impl Cutaway {
    /// Scenery visibility: 0 fully revealed through, 1 unchanged. Only fragments between
    /// camera and focus can be removed. Radius scales with depth to hold a circular screen
    /// footprint as the obstruction moves toward the camera. The caller selects scenery;
    /// this must never be applied to actors or used to remove physical colliders.
    pub fn visibility(self, fragment: [f32; 3]) -> f32 {
        if !self
            .camera
            .into_iter()
            .chain(self.focus)
            .chain(fragment)
            .all(f32::is_finite)
            || ![self.radius, self.feather, self.focus_clearance]
                .into_iter()
                .all(f32::is_finite)
            || self.radius <= 0.0
        {
            return 1.0;
        }
        let axis = sub(self.focus, self.camera);
        let length = dot(axis, axis).sqrt();
        if length <= 0.001 {
            return 1.0;
        }
        let direction = axis.map(|v| v / length);
        let relative = sub(fragment, self.camera);
        let depth = dot(relative, direction);
        if depth <= 0.001 || depth >= length - self.focus_clearance.max(0.0) {
            return 1.0;
        }
        let offset = sub(relative, direction.map(|v| v * depth));
        let distance = dot(offset, offset).sqrt();
        let radius = self.radius * depth / length;
        let inner = radius * (1.0 - self.feather.clamp(0.0, 1.0));
        if distance >= radius {
            return 1.0;
        }
        if distance <= inner {
            return 0.0;
        }
        let t = (distance - inner) / (radius - inner);
        t * t * (3.0 - 2.0 * t)
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cutaway() -> Cutaway {
        Cutaway {
            camera: [0., 0., 0.],
            focus: [0., 0., 20.],
            radius: 4.,
            feather: 0.25,
            focus_clearance: 0.5,
        }
    }

    #[test]
    fn only_occluding_depth_and_projected_circle_are_removed() {
        let c = cutaway();
        assert_eq!(c.visibility([0., 0., 10.]), 0.);
        for p in [
            [0., 0., -1.],
            [0., 0., 0.],
            [0., 0., 19.5],
            [0., 0., 20.],
            [0., 0., 30.],
            [3., 0., 10.],
        ] {
            assert_eq!(c.visibility(p), 1., "preserve {p:?}");
        }
        assert_eq!(c.visibility([1.5, 0., 10.]), 0.);
        assert!((c.visibility([1.75, 0., 10.]) - 0.5).abs() < 1e-5);
        assert_eq!(c.visibility([2., 0., 10.]), 1.);
    }

    #[test]
    fn footprint_is_depth_scaled_and_camera_orientation_independent() {
        let c = cutaway();
        let rotated = Cutaway {
            camera: [8., 3., 4.],
            focus: [28., 3., 4.],
            ..c
        };
        for offset in [0., 1., 1.6, 1.75, 2., 5.] {
            let expected = c.visibility([offset, 0., 10.]);
            assert!((c.visibility([offset / 2., 0., 5.]) - expected).abs() < 1e-5);
            assert!((rotated.visibility([18., 3. + offset, 4.]) - expected).abs() < 1e-5);
        }
    }

    #[test]
    fn disabled_invalid_and_degenerate_focus_preserve_scenery() {
        let c = cutaway();
        for invalid in [
            Cutaway { radius: 0., ..c },
            Cutaway {
                radius: f32::NAN,
                ..c
            },
            Cutaway {
                focus: c.camera,
                ..c
            },
            Cutaway {
                focus_clearance: 30.,
                ..c
            },
        ] {
            assert_eq!(invalid.visibility([0., 0., 10.]), 1.);
        }
        assert_eq!(c.visibility([f32::INFINITY, 0., 10.]), 1.);
        assert_eq!(Cutaway { feather: 0., ..c }.visibility([1.99, 0., 10.]), 0.);
    }
}
