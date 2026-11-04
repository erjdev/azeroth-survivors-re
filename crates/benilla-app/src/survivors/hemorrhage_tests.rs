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
fn hit(app: &mut App, target: Entity, source: &'static str, damage: f32) {
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        ..Hit::direct(target, damage, false)
    });
}
#[test]
fn hemorrhage_applies_after_the_strike_and_only_physical_damage_consumes_charges() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = fixture("hemorrhage", 1000.);
    spell_data(&mut app, &path);
    hit(&mut app, target, "hemorrhage", 20.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 980.);
    assert_eq!(
        app.world()
            .get::<HemorrhageVulnerability>(target)
            .unwrap()
            .0
            .charges(0.),
        30
    );
    for source in ["instant_poison", "fireball", "unknown"] {
        hit(&mut app, target, source, 10.);
    }
    hit(&mut app, target, "sinister_strike", 10.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 937.);
    assert_eq!(
        app.world()
            .get::<HemorrhageVulnerability>(target)
            .unwrap()
            .0
            .charges(0.),
        29
    );
    hit(&mut app, target, "hemorrhage", 20.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 914.);
    assert_eq!(
        app.world()
            .get::<HemorrhageVulnerability>(target)
            .unwrap()
            .0
            .charges(0.),
        30
    );
    app.world_mut().resource_mut::<Run>().elapsed = 15.;
    hit(&mut app, target, "sinister_strike", 10.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 904.);
}
#[test]
fn physical_bleeds_and_pet_bites_share_charges_and_lethal_hits_do_not_apply() {
    let path = benilla_formats::wow_data_or_skip!();
    let (mut app, target, mut schedule) = fixture("hemorrhage", 100.);
    spell_data(&mut app, &path);
    hit(&mut app, target, "hemorrhage", 20.);
    advance(&mut app, &mut schedule, 0.);
    app.world_mut()
        .get_mut::<Enemy>(target)
        .unwrap()
        .dots
        .push(enemies::Dot {
            origin: HitOrigin::Hero,
            dps: 10.,
            left: 1.,
            tick: 1.,
            source: "rupture",
        });
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        origin: HitOrigin::CombatPet,
        source: crate::survivors::hunter_pet::SOURCE,
        ..Hit::direct(target, 10., false)
    });
    advance(&mut app, &mut schedule, 1.);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 54.);
    assert_eq!(
        app.world()
            .get::<HemorrhageVulnerability>(target)
            .unwrap()
            .0
            .charges(0.),
        28
    );
    let (mut app, target, mut schedule) = fixture("hemorrhage", 10.);
    hit(&mut app, target, "hemorrhage", 20.);
    hit(&mut app, target, "hemorrhage", 20.);
    advance(&mut app, &mut schedule, 0.);
    assert_eq!(
        app.world()
            .get::<HemorrhageVulnerability>(target)
            .unwrap()
            .0
            .charges(0.),
        0
    );
    assert_eq!(app.world().resource::<Run>().kills, 1);
}

#[test]
fn hemorrhage_training_scales_strike_and_existing_weapon_builds_reach_the_new_attack() {
    use crate::survivors::Owned;
    use survivors_core::{
        gear_loadout::{MeleeKind, MeleeWeapon},
        talents::{talent_definition, Milestones},
    };
    let run = Run::default();
    let mut stats = run.stats();
    stats.might = 1.;
    let owned = Owned {
        def: data::ability("hemorrhage"),
        level: 1,
        cd: 0.,
    };
    let base = Milestones {
        class: 3,
        weapon: Some(MeleeWeapon {
            kind: MeleeKind::Dagger,
            two_handed: false,
        }),
        ..Default::default()
    };
    for rank in [0, 1, 255] {
        let trained = base.with_ranks(&[(2, 681, rank)]);
        let p = Params::new(&owned, &stats, trained);
        assert_eq!(p.dmg, if rank == 0 { 20. } else { 24. });
        assert_eq!(p.cooldown, 2.5);
        assert_eq!(p.count, 1);
        assert_eq!(
            trained.cast("sinister_strike"),
            base.cast("sinister_strike")
        );
        assert_eq!(trained.with_ranks(&[]), base);
    }
    let trained = base.with_ranks(&[(0, 283, 5)]);
    assert_eq!(trained.seal_fate_chance("hemorrhage", 20., true), 1.);
    assert!(survivors_core::talents::Coating::eligible(
        trained,
        "hemorrhage",
        20.,
        true
    ));
    assert!(!survivors_core::talents::Coating::eligible(
        trained,
        "hemorrhage",
        20.,
        false
    ));
    let (text, spans) = talent_definition(3, 681)
        .unwrap()
        .inline_description_with_values(0)
        .unwrap();
    assert!(text.contains("0 » 20%"), "{text}");
    assert!(!spans.is_empty());
}

#[test]
fn generated_weapon_hits_consume_without_reapplying_and_labels_do_not_turn_pet_spells_physical() {
    let mut state = HemorrhageVulnerability::default();
    state.0.apply(0.);
    assert_eq!(
        state.bonus("sword_specialization", HitOrigin::Hero, 20., 0., None, 0),
        3.
    );
    assert_eq!(
        state.bonus(
            "sword_specialization",
            HitOrigin::CombatPet,
            20.,
            0.,
            None,
            0
        ),
        0.
    );
    assert_eq!(
        state.bonus("Hunter Pet", HitOrigin::Hero, 20., 0., None, 0),
        0.
    );
    assert_eq!(state.0.charges(0.), 29);
}
