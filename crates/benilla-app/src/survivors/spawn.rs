//! Spawn candidates stay inside the arena, require floor support and reject body overlaps.
use super::{data::MapDef, floor::Floor, hero::arena, units::flat};
use bevy::prelude::*;

pub(super) fn point(
    world: &Floor,
    map: &MapDef,
    hero: Vec3,
    angle: f32,
    distance: f32,
    radius: f32,
) -> Option<Vec3> {
    choose(
        arena(map),
        map.leash,
        hero,
        angle,
        distance,
        radius,
        (|p| world.height(p), |p| world.can_spawn(p, radius)),
    )
}

fn choose(
    center: Vec3,
    leash: f32,
    hero: Vec3,
    angle: f32,
    distance: f32,
    radius: f32,
    (height, clear): (impl Fn(Vec3) -> Option<f32>, impl Fn(Vec3) -> bool),
) -> Option<Vec3> {
    let edge = leash - radius - 0.25;
    if edge <= 0.0 {
        return None;
    }
    for attempt in 0..16 {
        let a = angle + attempt as f32 * std::f32::consts::TAU / 16.;
        let mut p = hero + Vec3::new(a.cos(), 0., a.sin()) * distance;
        let offset = flat(p - center);
        if offset.length() > edge {
            let bounded = center + offset.normalize() * edge;
            p.x = bounded.x;
            p.z = bounded.z;
        }
        // Clamping an outward ring from a hero at the edge must not spawn on their feet.
        if flat(p - hero).length() < (radius + 2.).max(6.) {
            continue;
        }
        let Some(y) = height(p).filter(|y| y.is_finite()) else {
            continue;
        };
        p.y = y;
        let supported = [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
            .iter()
            .all(|axis| {
                height(p + *axis * radius)
                    .is_some_and(|h| h.is_finite() && (h - y).abs() <= radius.max(0.75))
            });
        if supported && clear(p) {
            return Some(p);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_arena_spawns_fit_their_bodies_and_do_not_land_on_an_edge_hero() {
        let hero = Vec3::new(29., 10., 0.);
        for i in 0..64 {
            for radius in [0.6, 3.0] {
                let p = choose(
                    Vec3::Y * 10.,
                    30.,
                    hero,
                    i as f32 * std::f32::consts::TAU / 64.,
                    42.,
                    radius,
                    (|_| Some(10.), |_| true),
                )
                .unwrap();
                assert!(flat(p).length() + radius <= 30.);
                assert!(flat(p - hero).length() >= 6.);
                assert_eq!(p.y, 10.);
            }
        }
    }
    #[test]
    fn missing_floor_or_unsupported_footprint_cannot_produce_a_spawn() {
        assert!(choose(
            Vec3::ZERO,
            30.,
            Vec3::ZERO,
            0.,
            20.,
            1.,
            (|_| None, |_| true)
        )
        .is_none());
        // Only the nominal ring has support: all bodies would overhang its edge.
        assert!(choose(
            Vec3::ZERO,
            30.,
            Vec3::ZERO,
            0.,
            20.,
            1.,
            (
                |p| { ((flat(p).length() - 20.).abs() < 0.01).then_some(0.) },
                |_| true
            )
        )
        .is_none());
    }
    #[test]
    fn rejected_direction_can_retry_on_supported_ground() {
        let p = choose(
            Vec3::ZERO,
            30.,
            Vec3::ZERO,
            0.,
            20.,
            1.,
            (|p: Vec3| (p.x < 0.).then_some(0.), |_| true),
        )
        .unwrap();
        assert!(p.x < 0.);
    }
    #[test]
    fn supported_but_occupied_spawns_retry_and_fully_blocked_rings_wait() {
        let candidate = choose(
            Vec3::ZERO,
            30.,
            Vec3::ZERO,
            0.,
            20.,
            1.,
            (
                |_| Some(4.),
                |p: Vec3| {
                    assert_eq!(p.y, 4.);
                    p.x < 0.
                },
            ),
        )
        .unwrap();
        assert!(candidate.x < 0.);
        assert!(choose(
            Vec3::ZERO,
            30.,
            Vec3::ZERO,
            0.,
            20.,
            1.,
            (|_| Some(0.), |_| false)
        )
        .is_none());
    }
}
