//! Actual settlement → proc → extra cast, without an unattended game session.
use super::*;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::Milestones;

fn setup() -> (App, Entity, Entity) {
    let (mut app, hero, target) = super::earthbind_tests::setup(true);
    {
        let mut run = app.world_mut().resource_mut::<Run>();
        run.class = &data::CLASSES[6];
        run.milestones = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(0, 75, 5)]);
        run.abilities.clear();
        run.bonus.targets = 0;
    }
    app.insert_resource(Rng(0));
    (app, hero, target)
}

fn land(app: &mut App, target: Entity, source: &'static str, origin: HitOrigin, damage: f32) {
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        origin,
        ..Hit::direct(target, damage, false)
    });
    app.world_mut().run_system_once(apply_hits).unwrap();
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
}

#[test]
fn successful_damage_fires_slotless_missiles_and_generated_damage_never_recurses() {
    let (mut app, _, target) = setup();
    land(&mut app, target, "fireball", HitOrigin::Hero, 10.0);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    let hits: Vec<_> = app
        .world()
        .resource::<Messages<Hit>>()
        .iter_current_update_messages()
        .cloned()
        .collect();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].source, "arcane_missiles");
    assert_eq!(hits[0].origin, HitOrigin::TriggeredHero);
    assert!(hits[0].damage > 0.0);
    assert!(app.world().resource::<Run>().abilities.is_empty());
    let before = app.world().get::<Enemy>(target).unwrap().hp;
    // Let even an unusually delayed generated missile settle after the ICD expires.
    app.world_mut().resource_mut::<Run>().elapsed = 10.0;
    app.world_mut().run_system_once(apply_hits).unwrap();
    assert!(app.world().get::<Enemy>(target).unwrap().hp < before);
    assert!(!app.world_mut().resource_mut::<Run>().clearcasting.take());
    land(&mut app, target, "arcane_missiles", HitOrigin::Hero, 10.0);
    assert!(app.world_mut().resource_mut::<Run>().clearcasting.take());
}

#[test]
fn owned_rank_scales_extra_cast_without_changing_six_slots_or_their_timers() {
    for rank in [1, 8] {
        let (mut app, _, target) = setup();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            for key in [
                "arcane_missiles",
                "fireball",
                "frostbolt",
                "frost_nova",
                "arcane_explosion",
                "cone_of_cold",
            ] {
                run.abilities.push(super::super::Owned {
                    def: data::ability(key),
                    level: rank,
                    cd: 20.0,
                });
            }
        }
        land(&mut app, target, "fireball", HitOrigin::Hero, 10.0);
        let expected = {
            let run = app.world().resource::<Run>();
            Params::new(&run.abilities[0], &run.stats(), run.milestones).dmg
        };
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert_eq!(run.abilities.len(), 6);
        assert!(run.abilities.iter().all(|a| (a.cd - 19.9).abs() < 0.0001));
        let hits = app.world().resource::<Messages<Hit>>();
        let hit = hits.iter_current_update_messages().next().unwrap();
        assert!((hit.damage - expected * run.stats().crit_damage).abs() < 0.001);
    }
}

#[test]
fn pets_reflections_zero_damage_and_foreign_spells_cannot_queue_clearcasting() {
    for (origin, source, damage) in [
        (HitOrigin::CombatPet, "fireball", 10.0),
        (HitOrigin::Reflection, "fireball", 10.0),
        (HitOrigin::TriggeredHero, "fireball", 10.0),
        (HitOrigin::Hero, "fireball", 0.0),
        (HitOrigin::Hero, "smite", 10.0),
    ] {
        let (mut app, _, target) = setup();
        land(&mut app, target, source, origin, damage);
        assert!(!app.world_mut().resource_mut::<Run>().clearcasting.take());
    }
}

#[test]
fn paused_dispatch_waits_and_missing_targets_do_not_bank_a_charge() {
    let (mut app, _, target) = setup();
    land(&mut app, target, "fireball", HitOrigin::Hero, 10.0);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::ZERO);
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 1000.0;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.1));
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
    assert!(!app.world_mut().resource_mut::<Run>().clearcasting.take());
    app.world_mut()
        .get_mut::<Transform>(target)
        .unwrap()
        .translation = Vec3::X * 5.0;
    app.world_mut().run_system_once(cast_abilities).unwrap();
    assert!(app.world().resource::<Messages<Hit>>().is_empty());
}
