use super::poison_tests::{advance, fixture};
use super::*;
use crate::survivors::defense::{contact, Outcome};

#[test]
fn only_original_positive_hero_hits_grant_dodge_and_lethal_hits_still_count() {
    for (source, origin, damage, hp, expected) in [
        ("ghostly_strike", HitOrigin::Hero, 25., 100., true),
        ("ghostly_strike", HitOrigin::Hero, 25., 10., true),
        ("ghostly_strike", HitOrigin::Hero, 0., 100., false),
        ("ghostly_strike", HitOrigin::CombatPet, 25., 100., false),
        ("sword_specialization", HitOrigin::Hero, 25., 100., false),
        ("sinister_strike", HitOrigin::Hero, 25., 100., false),
    ] {
        let (mut app, target, mut schedule) = fixture("ghostly_strike", hp);
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source,
            origin,
            ..Hit::direct(target, damage, false)
        });
        advance(&mut app, &mut schedule, 0.);
        let mut run = app.world_mut().resource_mut::<Run>();
        assert_eq!(
            run.ghostly_strike.dodge(run.elapsed),
            if expected { 0.15 } else { 0. }
        );
        if expected {
            run.shield = 30.;
            run.shield_t = 10.;
        }
        let hp = run.hp;
        let result = contact(&mut run, target, 10., || 0.1);
        assert_eq!(result.outcome == Outcome::Dodged, expected);
        if expected {
            assert_eq!(run.hp, hp);
            assert_eq!(run.shield, 30.);
        }
        run.elapsed = 7.;
        assert_ne!(
            contact(&mut run, target, 10., || 0.1).outcome,
            Outcome::Dodged
        );
    }
}

#[test]
fn ghostly_training_changes_only_damage_and_joins_builder_rules() {
    use survivors_core::talents::Milestones;
    for class in 0..9 {
        for rank in [0, 1, 255] {
            let base = Milestones {
                class,
                ..Default::default()
            };
            let trained = base.with_ranks(&[(2, 303, rank)]);
            let mut expected = base.cast("ghostly_strike");
            if class == 3 && rank > 0 {
                expected.damage *= 1.2;
            }
            assert_eq!(trained.cast("ghostly_strike"), expected);
            assert_eq!(
                trained.cast("sinister_strike"),
                base.cast("sinister_strike")
            );
            assert_eq!(trained.with_ranks(&[]), base);
        }
    }
    let rogue = Milestones {
        class: 3,
        ..Default::default()
    }
    .with_ranks(&[(0, 283, 5)]);
    assert_eq!(rogue.seal_fate_chance("ghostly_strike", 25., true), 1.);
    assert_eq!(rogue.seal_fate_chance("ghostly_strike", 25., false), 0.);
}

#[test]
fn poison_eligibility_requires_original_owned_weapon_contact() {
    use survivors_core::{
        gear_loadout::{MeleeKind, MeleeWeapon},
        talents::{Coating, Milestones},
    };
    let rogue = Milestones {
        class: 3,
        weapon: Some(MeleeWeapon {
            kind: MeleeKind::Sword,
            two_handed: false,
        }),
        ..Default::default()
    };
    assert!(Coating::eligible(rogue, "ghostly_strike", 25., true));
    assert!(!Coating::eligible(rogue, "ghostly_strike", 25., false));
    assert!(!Coating::eligible(rogue, "ghostly_strike", 0., true));
}
