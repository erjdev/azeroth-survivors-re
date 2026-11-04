//! Exercise deployed totems through the actual area system across frame partitions.
use super::poison_tests::fixture;
use super::*;
use bevy::ecs::system::RunSystemOnce;

fn deployed(pulse: bool) -> (App, Entity, Entity) {
    let (mut app, target, _) = fixture("searing_totem", 10000.0);
    app.world_mut()
        .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
    let totem = app
        .world_mut()
        .spawn((
            Transform::default(),
            Totem {
                cast: 0,
                damage: 10.0,
                range: 20.0,
                radius: 10.0,
                left: 5.25,
                tick: 0.3,
                pulse,
                source: if pulse {
                    "magma_totem"
                } else {
                    "searing_totem"
                },
            },
        ))
        .id();
    (app, target, totem)
}

fn tick(app: &mut App, dt: f32) {
    app.world_mut().resource_mut::<Run>().elapsed += dt;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(dt));
    app.world_mut().run_system_once(tick_areas).unwrap();
}

#[test]
fn totems_deliver_every_due_attack_before_expiry_across_long_frames() {
    for pulse in [false, true] {
        let mut totals = Vec::new();
        for frames in [vec![0.1; 60], vec![0.5; 12], vec![6.0]] {
            let (mut app, target, totem) = deployed(pulse);
            tick(&mut app, 0.0);
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
            for dt in frames {
                tick(&mut app, dt);
            }
            let hits: Vec<_> = app
                .world_mut()
                .resource_mut::<Messages<Hit>>()
                .drain()
                .collect();
            assert_eq!(hits.len(), if pulse { 3 } else { 5 });
            assert!(hits.iter().all(|hit| hit.target == target));
            totals.push(hits.iter().map(|hit| hit.damage).sum::<f32>());
            assert!(app.world().get::<Totem>(totem).is_none());
            assert!(app
                .world()
                .get::<benilla_world::model_fade::DespawnFade>(totem)
                .is_some());
            tick(&mut app, 10.0);
            assert!(app.world().resource::<Messages<Hit>>().is_empty());
        }
        assert_eq!(totals[0], totals[1]);
        assert_eq!(totals[1], totals[2]);
    }
}

#[test]
fn empty_totem_pulses_are_not_banked_for_a_later_target() {
    for pulse in [false, true] {
        let (mut app, target, _) = deployed(pulse);
        app.world_mut().get_mut::<Enemy>(target).unwrap().dead = true;
        tick(&mut app, 2.8);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        app.world_mut().get_mut::<Enemy>(target).unwrap().dead = false;
        tick(&mut app, 0.1);
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
        tick(&mut app, 1.4);
        assert_eq!(
            app.world().resource::<Messages<Hit>>().len(),
            if pulse { 1 } else { 2 }
        );
    }
}
