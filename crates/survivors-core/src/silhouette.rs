//! Reference contract for the occluded-actor compositor, independent of scenery cutaways.
//! The actor depth must be its nearest rendered surface, not each mesh fragment's depth.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Enemy,
    Friendly,
    /// Neutral/unclassified actors still occupy the mask but receive no highlight.
    Unclassified,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Filters {
    pub enemies: bool,
    pub friendlies: bool,
}

impl Filters {
    /// Reverse-Z depth: larger values are nearer. Zero is cleared background.
    /// `tolerance` accounts for the chosen mask/depth formats; equality never reveals.
    /// Gameplay-hidden actors must be excluded before generating the actor mask.
    pub fn color(
        self,
        side: Side,
        actor_depth: f32,
        scene_depth: f32,
        tolerance: f32,
    ) -> Option<[f32; 3]> {
        if ![actor_depth, scene_depth, tolerance]
            .into_iter()
            .all(f32::is_finite)
            || !(0.0..=1.0).contains(&scene_depth)
            || !(0.0..=1.0).contains(&actor_depth)
            || actor_depth == 0.0
            || tolerance < 0.0
            || scene_depth - actor_depth <= tolerance
        {
            return None;
        }
        match side {
            Side::Enemy if self.enemies => Some([1.0, 0.0, 0.0]),
            Side::Friendly if self.friendlies => Some([0.0, 1.0, 0.0]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTH: Filters = Filters {
        enemies: true,
        friendlies: true,
    };

    #[test]
    fn only_scenery_in_front_of_the_nearest_actor_surface_reveals_it() {
        // A visible actor writes the same front depth into the world and actor mask.
        assert_eq!(BOTH.color(Side::Enemy, 0.6, 0.6, 0.0001), None);
        // Scenery behind the actor must not reveal it, nor may the cleared background.
        assert_eq!(BOTH.color(Side::Enemy, 0.6, 0.3, 0.0001), None);
        assert_eq!(BOTH.color(Side::Enemy, 0.6, 0.0, 0.0001), None);
        assert_eq!(
            BOTH.color(Side::Enemy, 0.6, 0.8, 0.0001),
            Some([1., 0., 0.])
        );
        assert_eq!(
            BOTH.color(Side::Friendly, 0.6, 0.8, 0.0001),
            Some([0., 1., 0.])
        );
        assert_eq!(BOTH.color(Side::Unclassified, 0.6, 0.8, 0.0001), None);
        // A masked-out texel has no actor even when the scene contains a wall.
        assert_eq!(BOTH.color(Side::Enemy, 0., 0.8, 0.0001), None);
    }

    #[test]
    fn each_side_can_be_disabled_independently_including_both() {
        for enemies in [false, true] {
            for friendlies in [false, true] {
                let filters = Filters {
                    enemies,
                    friendlies,
                };
                assert_eq!(filters.color(Side::Enemy, 0.2, 0.8, 0.).is_some(), enemies);
                assert_eq!(
                    filters.color(Side::Friendly, 0.2, 0.8, 0.).is_some(),
                    friendlies
                );
            }
        }
    }

    #[test]
    fn invalid_depth_and_precision_noise_do_not_create_highlights() {
        for invalid in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert_eq!(BOTH.color(Side::Enemy, invalid, 0.8, 0.001), None);
            assert_eq!(BOTH.color(Side::Enemy, 0.2, invalid, 0.001), None);
        }
        for invalid in [f32::NAN, f32::INFINITY, -0.001] {
            assert_eq!(BOTH.color(Side::Enemy, 0.2, 0.8, invalid), None);
        }
        assert_eq!(BOTH.color(Side::Enemy, 0.5, 0.50001, 0.0001), None);
        assert_eq!(
            BOTH.color(Side::Enemy, 0.5, 0.501, 0.0001),
            Some([1., 0., 0.])
        );
    }
}
