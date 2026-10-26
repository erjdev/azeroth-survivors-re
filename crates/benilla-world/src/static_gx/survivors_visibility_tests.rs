//! Exercise the same public query used by Survivors' camera reveal. These synthetic
//! triangles verify retained geometry admission, not texture-alpha or GPU rendering.

use benilla_formats::ModelBlend;
use bevy::{ecs::system::SystemState, platform::collections::HashSet, prelude::*};

use super::{
    render::GxDoodadVis,
    testkit::{batch_of, object, tri},
    StaticGx,
};
use crate::interact::{ObjectHit, WorldPick};

fn cast(world: &mut World, origin: Vec3) -> Vec<ObjectHit> {
    let mut query = SystemState::<WorldPick>::new(world);
    query
        .get(world)
        .cast(Ray3d::new(origin, Dir3::NEG_Y), &HashSet::default(), true)
}

#[test]
fn retained_cutout_foliage_answers_without_entities_or_colliders() {
    let mut world = World::new();
    let mut gx = StaticGx::default();
    let tree = object(4242);
    assert!(gx.divert(batch_of(
        &tree,
        &tri([0.0; 3]),
        Vec3::ZERO,
        None,
        ModelBlend::AlphaTest
    )));
    world.insert_resource(gx);
    let eye = Vec3::new(-0.3, 5.0, -0.3);
    assert!(
        cast(&mut world, eye).is_empty(),
        "culled geometry must not occlude"
    );

    world
        .resource_mut::<StaticGx>()
        .world
        .visible
        .push(GxDoodadVis::Cell((0, 0)));
    let hits = cast(&mut world, eye);
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0].entity.is_none(),
        "no entity pick set or collider is needed"
    );
    assert_eq!(hits[0].object.as_ref().unwrap().id, tree.id);
    assert_eq!(hits[0].hit.distance, 5.0);
    assert!(
        cast(&mut world, Vec3::new(-2.0, 5.0, -2.0)).is_empty(),
        "clear sightline"
    );
    assert!(
        cast(&mut world, Vec3::new(-0.3, -1.0, -0.3)).is_empty(),
        "foliage behind the camera"
    );

    world.resource_mut::<StaticGx>().world.visible.clear();
    assert!(
        cast(&mut world, eye).is_empty(),
        "previous visibility must not stick"
    );
    world.remove_resource::<StaticGx>();
    assert!(
        cast(&mut world, eye).is_empty(),
        "entity-only mode remains supported"
    );
}
