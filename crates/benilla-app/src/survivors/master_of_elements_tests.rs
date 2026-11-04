//! Real hit settlement applies recovery only to the eligible owned spell.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::Milestones;

fn setup(rank: u8) -> (App, Entity) {
    let (mut app, _, target) = super::earthbind_tests::setup(true);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[6];
        run.milestones = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(1, 1639, rank)]);
        run.abilities = ["fireball", "frostbolt", "arcane_missiles"]
            .map(|key| super::super::Owned {
                def: data::ability(key),
                level: 1,
                cd: 10.0,
            })
            .into();
    }
    (app, target)
}

fn land(
    app: &mut App,
    target: Entity,
    source: &'static str,
    origin: HitOrigin,
    critical: bool,
    damage: f32,
) {
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        origin,
        ..Hit::direct(target, damage, critical)
    });
    app.world_mut().run_system_once(apply_hits).unwrap();
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
}

#[test]
fn critical_refunds_are_ranked_scoped_and_shared_between_spells() {
    for rank in 0..=4 {
        let (mut app, target) = setup(rank);
        land(&mut app, target, "fireball", HitOrigin::Hero, true, 1.0);
        let expected = 10.0 * (1.0 - rank.min(3) as f32 * 0.1);
        assert!((app.world().resource::<Run>().abilities[0].cd - expected).abs() < 0.0001);
        land(&mut app, target, "frostbolt", HitOrigin::Hero, true, 1.0);
        assert_eq!(app.world().resource::<Run>().abilities[1].cd, 10.0);
        app.world_mut().resource_mut::<Run>().elapsed = 1.0;
        land(&mut app, target, "frostbolt", HitOrigin::Hero, true, 1.0);
        assert!((app.world().resource::<Run>().abilities[1].cd - expected).abs() < 0.0001);
        assert_eq!(app.world().resource::<Run>().abilities[2].cd, 10.0);
    }
}

#[test]
fn ineligible_hits_and_ready_or_unowned_spells_do_not_consume_the_deadline() {
    for (source, origin, critical, damage) in [
        ("fireball", HitOrigin::Hero, false, 1.0),
        ("fireball", HitOrigin::Hero, true, 0.0),
        ("fireball", HitOrigin::CombatPet, true, 1.0),
        ("fireball", HitOrigin::TriggeredHero, true, 1.0),
        ("fireball", HitOrigin::Reflection, true, 1.0),
        ("arcane_missiles", HitOrigin::Hero, true, 1.0),
        ("pyroblast", HitOrigin::Hero, true, 1.0),
    ] {
        let (mut app, target) = setup(3);
        land(&mut app, target, source, origin, critical, damage);
        assert!(app
            .world()
            .resource::<Run>()
            .abilities
            .iter()
            .all(|a| a.cd == 10.0));
        land(&mut app, target, "fireball", HitOrigin::Hero, true, 1.0);
        assert!((app.world().resource::<Run>().abilities[0].cd - 7.0).abs() < 0.0001);
    }
    let (mut app, target) = setup(3);
    app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.0;
    land(&mut app, target, "fireball", HitOrigin::Hero, true, 1.0);
    land(&mut app, target, "frostbolt", HitOrigin::Hero, true, 1.0);
    assert!((app.world().resource::<Run>().abilities[1].cd - 7.0).abs() < 0.0001);
}
