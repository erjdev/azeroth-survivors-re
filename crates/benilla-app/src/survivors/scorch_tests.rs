//! Damage-pipeline acceptance for original Scorch triggers and per-target Fire damage.
use super::poison_tests::{advance, fixture};
use super::*;

fn setup(rank: u8, path: &std::path::Path) -> (App, Entity, Schedule) {
    let (mut app, target, schedule) = fixture("scorch", 1000.);
    let mut chain = benilla_formats::open_chain(path).unwrap();
    app.insert_resource(crate::ui_action::Spells {
        catalog: benilla_formats::load_spell_catalog(&mut chain).unwrap(),
        forms: Default::default(),
        ranges: Default::default(),
        cast_times: Default::default(),
        durations: Default::default(),
        radii: Default::default(),
    });
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[6];
    run.milestones = survivors_core::talents::Milestones {
        class: 6,
        ..Default::default()
    }
    .with_ranks(&[(1, 25, rank)]);
    (app, target, schedule)
}
fn hit(app: &mut App, target: Entity, source: &'static str, damage: f32) {
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        ..Hit::direct(target, damage, false)
    });
}
#[test]
fn scorch_applies_after_damage_and_scales_only_catalog_fire_including_ticks() {
    let path = benilla_formats::wow_data_or_skip!();
    for rank in [0, 3] {
        let (mut app, target, mut schedule) = setup(rank, &path);
        hit(&mut app, target, "scorch", 10.);
        hit(&mut app, target, "fireball", 10.);
        hit(&mut app, target, "frostbolt", 10.);
        advance(&mut app, &mut schedule, 0.);
        let expected = if rank == 3 { 969.7 } else { 970. };
        assert!((app.world().get::<Enemy>(target).unwrap().hp - expected).abs() < 0.001);
        assert_eq!(
            app.world()
                .get::<FireVulnerability>(target)
                .unwrap()
                .0
                .stacks(0.),
            u8::from(rank == 3)
        );
        app.world_mut().resource_mut::<Messages<Hit>>().clear();
        app.world_mut().get_mut::<Enemy>(target).unwrap().dots = vec![enemies::Dot {
            origin: crate::survivors::abilities::HitOrigin::Hero,
            dps: 10.,
            left: 1.,
            tick: 1.,
            source: "Ignite",
        }];
        advance(&mut app, &mut schedule, 1.);
        assert!(
            (app.world().get::<Enemy>(target).unwrap().hp
                - (expected - if rank == 3 { 10.3 } else { 10. }))
            .abs()
                < 0.001
        );
        assert_eq!(
            app.world()
                .get::<FireVulnerability>(target)
                .unwrap()
                .0
                .stacks(0.),
            u8::from(rank == 3)
        );
    }
}
#[test]
fn fire_vulnerability_expires_is_fail_closed_without_catalog_and_cleans_up_on_death() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = setup(3, &path);
    hit(&mut app, target, "scorch", 10.);
    advance(&mut app, &mut schedule, 0.);
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    app.world_mut().resource_mut::<Run>().elapsed = 30.;
    hit(&mut app, target, "fireball", 10.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 980.);
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    hit(&mut app, target, "scorch", 10.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(
        app.world()
            .get::<FireVulnerability>(target)
            .unwrap()
            .0
            .stacks(30.),
        1
    );
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    app.world_mut()
        .remove_resource::<crate::ui_action::Spells>();
    hit(&mut app, target, "fireball", 10.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 960.);
    app.world_mut().resource_mut::<Messages<Hit>>().clear();
    hit(&mut app, target, "scorch", 10000.);
    hit(&mut app, target, "scorch", 10000.);
    advance(&mut app, &mut schedule, 0.);
    assert!(app.world().get::<FireVulnerability>(target).is_none());
    assert_eq!(app.world().resource::<Run>().kills, 1);
}

#[test]
fn splash_and_multistrike_do_not_duplicate_scorch_stacks() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = setup(3, &path);
    let other = app
        .world_mut()
        .spawn((
            Guid(123),
            Transform::default(),
            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
            Marks::debuffs(),
            Enemy {
                def: &data::roster(0).enemies[0],
                hp: 1000.,
                max_hp: 1000.,
                speed: 3.0,
                damage: 1.0,
                radius: 0.6,
                xp: 1,
                boss: false,
                elite: false,
                attack_cd: 0.0,
                slows: Default::default(),
                root_t: 0.0,
                frozen_t: 0.0,
                winters_chill: Default::default(),
                shadow_weaving: Default::default(),
                kidney_t: 0.0,
                stun_t: 0.0,
                fear_t: 0.0,
                incapacitate_t: 0.0,
                weaken: 1.0,
                weaken_t: 0.0,
                poison: Default::default(),
                dots: vec![],
                dead: false,
            },
        ))
        .id();

    app.world_mut().resource_mut::<Run>().bonus.multistrike = 1.;
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source: "scorch",
        splash: 2.,
        ..Hit::direct(target, 10., false)
    });
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(
        app.world()
            .get::<FireVulnerability>(target)
            .unwrap()
            .0
            .stacks(0.),
        1
    );
    assert_eq!(
        app.world()
            .get::<FireVulnerability>(other)
            .unwrap()
            .0
            .stacks(0.),
        0
    );
    assert!(app.world().get::<Enemy>(other).unwrap().hp < 1000.);
}

#[test]
fn queued_hits_use_their_ordered_occurrence_times_before_expiry() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = setup(3, &path);
    app.world_mut().resource_mut::<Run>().elapsed = 35.;
    let mut events = app.world_mut().resource_mut::<Messages<Hit>>();
    // Deliberately reverse arrival order; settlement sorts the event timestamps.
    for (source, at) in [("fireball", 31.), ("fireball", 1.), ("scorch", 0.)] {
        events.write(Hit {
            source,
            occurred: Some(at),
            ..Hit::direct(target, 10., false)
        });
    }
    advance(&mut app, &mut schedule, 0.);
    assert!((app.world().get::<Enemy>(target).unwrap().hp - 969.7).abs() < 0.001);
    assert_eq!(
        app.world()
            .get::<FireVulnerability>(target)
            .unwrap()
            .0
            .stacks(35.),
        0
    );
}
