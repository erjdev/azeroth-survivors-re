//! Real damage settlement, using the installed spell schools rather than name heuristics.
use super::poison_tests::{advance, fixture};
use super::*;

fn spell_data(app: &mut App, path: &std::path::Path) {
    let mut chain = benilla_formats::open_chain(path).unwrap();
    app.insert_resource(crate::ui_action::Spells {
        catalog: benilla_formats::load_spell_catalog(&mut chain).unwrap(),
        forms: Default::default(),
        ranges: Default::default(),
        cast_times: Default::default(),
        durations: Default::default(),
        radii: Default::default(),
    });
}
fn charge(app: &mut App, target: Entity) {
    let mut state = StormstrikeVulnerability::default();
    state.0.apply(0.0);
    app.world_mut().entity_mut(target).insert(state);
}
fn hit(app: &mut App, target: Entity, source: &'static str, damage: f32) {
    let mut hit = Hit::direct(target, damage, false);
    hit.source = source;
    app.world_mut().resource_mut::<Messages<Hit>>().write(hit);
}

#[test]
fn installed_nature_school_consumes_two_hits_without_affecting_other_schools() {
    let path = benilla_formats::wow_data_or_skip!();
    for source in [
        "lightning_bolt",
        "earth_shock",
        "flame_shock",
        "frost_shock",
        "wrath",
        "instant_poison",
        "heroic_strike",
    ] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        spell_data(&mut app, &path);
        charge(&mut app, target);
        let nature = matches!(
            source,
            "lightning_bolt" | "earth_shock" | "wrath" | "instant_poison"
        );
        for _ in 0..3 {
            hit(&mut app, target, source, 10.0);
        }
        advance(&mut app, &mut schedule, 0.0);
        let expected = if nature { 966.0 } else { 970.0 };
        assert!(
            (app.world().get::<Enemy>(target).unwrap().hp - expected).abs() < 0.001,
            "{source}"
        );
        assert_eq!(
            app.world()
                .get::<StormstrikeVulnerability>(target)
                .unwrap()
                .0
                .charges(0.0),
            if nature { 0 } else { 2 }
        );
    }
}

#[test]
fn delayed_hits_use_impact_expiry_and_missing_catalog_never_guesses() {
    let path = benilla_formats::wow_data_or_skip!();
    for (catalog, impact, expected) in [
        (true, 11.0, 988.0),
        (true, 12.0, 990.0),
        (false, 1.0, 990.0),
    ] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        if catalog {
            spell_data(&mut app, &path);
        }
        charge(&mut app, target);
        let mut hit = Hit::direct(target, 10.0, false);
        hit.source = "lightning_bolt";
        app.world_mut()
            .resource_mut::<PendingHits>()
            .0
            .push((impact, hit));
        app.world_mut().resource_mut::<Run>().elapsed = impact;
        advance(&mut app, &mut schedule, 0.0);
        assert!((app.world().get::<Enemy>(target).unwrap().hp - expected).abs() < 0.001);
    }
}

#[test]
fn periodic_nature_ticks_consume_charges_but_zero_damage_preserves_them() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
    spell_data(&mut app, &path);
    charge(&mut app, target);
    hit(&mut app, target, "lightning_bolt", 0.0);
    advance(&mut app, &mut schedule, 0.0);
    assert_eq!(
        app.world()
            .get::<StormstrikeVulnerability>(target)
            .unwrap()
            .0
            .charges(0.0),
        2
    );
    app.world_mut()
        .get_mut::<Enemy>(target)
        .unwrap()
        .dots
        .push(enemies::Dot {
            dps: 10.0,
            left: 3.0,
            tick: 1.0,
            source: "insect_swarm",
        });
    advance(&mut app, &mut schedule, 3.0);
    assert!((app.world().get::<Enemy>(target).unwrap().hp - 966.0).abs() < 0.001);
    assert_eq!(
        app.world()
            .get::<StormstrikeVulnerability>(target)
            .unwrap()
            .0
            .charges(0.0),
        0
    );
}

#[test]
fn multistrike_uses_one_charge_and_other_entities_keep_their_state() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
    spell_data(&mut app, &path);
    charge(&mut app, target);
    let other = app.world_mut().spawn_empty().id();
    charge(&mut app, other);
    app.world_mut().resource_mut::<Run>().bonus.multistrike = 1.0;
    hit(&mut app, target, "lightning_bolt", 10.0);
    advance(&mut app, &mut schedule, 0.0);
    assert!((app.world().get::<Enemy>(target).unwrap().hp - 976.0).abs() < 0.001);
    assert_eq!(
        app.world()
            .get::<StormstrikeVulnerability>(target)
            .unwrap()
            .0
            .charges(0.0),
        1
    );
    assert_eq!(
        app.world()
            .get::<StormstrikeVulnerability>(other)
            .unwrap()
            .0
            .charges(0.0),
        2
    );
    app.world_mut().despawn(target);
    assert!(app
        .world()
        .get::<StormstrikeVulnerability>(target)
        .is_none());
    assert_eq!(
        app.world()
            .get::<StormstrikeVulnerability>(other)
            .unwrap()
            .0
            .charges(0.0),
        2
    );
}

#[test]
fn reflection_uses_its_actual_spell_school_instead_of_the_shared_meter_name() {
    let path = benilla_formats::wow_data_or_skip!();
    for (key, expected) in [("thorns", 988.0), ("retaliation", 990.0)] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        spell_data(&mut app, &path);
        charge(&mut app, target);
        app.world_mut().resource_mut::<Run>().thorns_spell = data::ability(key).spell;
        hit(&mut app, target, "Thorns", 10.0);
        advance(&mut app, &mut schedule, 0.0);
        assert!(
            (app.world().get::<Enemy>(target).unwrap().hp - expected).abs() < 0.001,
            "{key}"
        );
    }
}

#[test]
fn weapon_poison_procs_and_periodic_poison_use_damage_school() {
    let path = benilla_formats::wow_data_or_skip!();
    for (key, expected, remaining) in [("instant_poison", 963.6, 1), ("deadly_poison", 959.4, 0)] {
        let (mut app, target, mut schedule) = fixture(key, 1000.0);
        spell_data(&mut app, &path);
        charge(&mut app, target);
        // Rng(0) deterministically procs the coating, as in the poison settlement tests.
        hit(&mut app, target, "sinister_strike", 10.0);
        advance(&mut app, &mut schedule, 0.0);
        if key == "deadly_poison" {
            advance(&mut app, &mut schedule, 9.0);
        }
        let hp = app.world().get::<Enemy>(target).unwrap().hp;
        assert!((hp - expected).abs() < 0.001, "{key}: {hp}");
        assert_eq!(
            app.world()
                .get::<StormstrikeVulnerability>(target)
                .unwrap()
                .0
                .charges(0.0),
            remaining
        );
    }
}
