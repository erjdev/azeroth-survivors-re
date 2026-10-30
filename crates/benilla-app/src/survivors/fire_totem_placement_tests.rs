//! Actual fire-totem casts must use supported, reachable body space.
use super::*;
use avian3d::prelude::*;
use bevy::ecs::system::RunSystemOnce;

pub(super) fn support_cast_fixture(app: &mut App) {
    let delta = app.world().resource::<Time>().delta();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        PhysicsPlugins::new(PostUpdate),
    ));
    app.init_asset::<Mesh>();
    app.init_resource::<benilla_world::collision::MoverTraceExclusions>();
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(100., 2., 100.),
        Transform::from_xyz(0., -1., 0.),
    ));
    app.finish();
    app.cleanup();
    app.update();
    let mut time = Time::<()>::default();
    time.advance_by(delta);
    app.insert_resource(time);
}

fn setup(key: &'static str, supported: bool) -> App {
    let (mut app, _, _) = earthbind_tests::setup(supported);
    app.world_mut().resource_mut::<Run>().abilities[0].def = data::ability(key);
    app
}

fn cast(app: &mut App) -> Vec<Vec3> {
    app.world_mut().run_system_once(cast_abilities).unwrap();
    app.world_mut()
        .query_filtered::<&Transform, With<Totem>>()
        .iter(app.world())
        .map(|t| t.translation)
        .collect()
}

#[test]
fn fire_totems_do_not_cast_without_supported_ground() {
    for key in ["searing_totem", "magma_totem"] {
        let mut app = setup(key, false);
        assert!(cast(&mut app).is_empty(), "{key} floated without ground");
        assert_eq!(app.world().resource::<Run>().abilities[0].cd, 0.2);
        assert!(app
            .world()
            .resource::<Messages<crate::creature_anim::CastEvent>>()
            .is_empty());
    }
}

#[test]
fn fire_totems_ground_on_slopes_and_fall_back_from_blocked_offsets() {
    for key in ["searing_totem", "magma_totem"] {
        let mut slope = setup(key, false);
        slope.world_mut().spawn((
            RigidBody::Static,
            Collider::trimesh(
                vec![
                    Vec3::new(-10., -5., -10.),
                    Vec3::new(10., 5., -10.),
                    Vec3::new(10., 5., 10.),
                    Vec3::new(-10., -5., 10.),
                ],
                vec![[0, 2, 1], [0, 3, 2]],
            ),
            Transform::default(),
        ));
        slope.update();
        let positions = cast(&mut slope);
        assert_eq!(positions.len(), 1);
        let at = positions[0];
        assert!((at.y - at.x * 0.5).abs() < 0.01, "{key}: {at:?}");
        assert!(
            at.x > 1.9,
            "preferred offset should remain usable on the slope"
        );

        let mut blocked = setup(key, true);
        blocked.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(1., 6., 2.),
            Transform::from_xyz(2., 3., 0.),
        ));
        blocked.update();
        let positions = cast(&mut blocked);
        assert_eq!(positions.len(), 1);
        assert!(
            positions[0].length() < 0.01,
            "{key} should fall back to supported hero feet: {:?}",
            positions[0]
        );
    }
}
