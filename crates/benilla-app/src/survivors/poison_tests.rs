//! Actual hit settlement and poison clocks, without an unattended combat session.
use super::*;
use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
use crate::survivors::{data, Owned, Phase};
use survivors_core::gear_loadout::{MeleeKind, MeleeWeapon};

pub(in crate::survivors) fn fixture(key: &str, hp: f32) -> (App, Entity, Schedule) {
    let mut app = App::new();
    let mut run = Run {
        class: &data::CLASSES[3],
        milestones: survivors_core::talents::Milestones {
            class: 3,
            weapon: Some(MeleeWeapon {
                kind: MeleeKind::Dagger,
                two_handed: false,
            }),
            ..Default::default()
        },
        hp: 50.0,
        abilities: vec![Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.0,
        }],
        ..Default::default()
    };
    run.bonus.leech = 0.1;
    app.insert_resource(Time::<()>::default())
        .insert_resource(run)
        .insert_resource(Rng(0))
        .insert_resource(crate::survivors::account::empty())
        .insert_resource(State::new(Phase::Playing))
        .init_resource::<crate::survivors::Selection>()
        .init_resource::<PlaySeq>()
        .init_resource::<PendingHits>()
        .add_message::<CastEvent>()
        .add_message::<SpellGoTargets>()
        .add_message::<SwingMessage>()
        .add_message::<crate::combat_text::CombatTextSpawn>()
        .add_message::<Hit>()
        .add_message::<Loot>()
        .add_message::<FieldChanged>();
    let target = app
        .world_mut()
        .spawn((
            Guid(123),
            Transform::default(),
            ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
            Marks::debuffs(),
            Enemy {
                def: &data::roster(0).enemies[0],
                hp,
                max_hp: hp,
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
    let mut schedule = Schedule::default();
    schedule.add_systems(apply_hits.run_if(in_state(Phase::Playing)));
    (app, target, schedule)
}
pub(in crate::survivors) fn advance(app: &mut App, schedule: &mut Schedule, dt: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(dt));
    schedule.run(app.world_mut());
}
fn hit(app: &mut App, target: Entity, source: &'static str) {
    app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
        source,
        ..Hit::direct(target, 10.0, false)
    });
}
#[test]
fn instant_poison_settles_lethal_damage_leech_and_loot_once() {
    let (mut app, target, mut schedule) = fixture("instant_poison", 20.0);
    hit(&mut app, target, "sinister_strike");
    hit(&mut app, target, "sinister_strike"); // queued overkill cannot proc again
    advance(&mut app, &mut schedule, 0.0);
    let enemy = app.world().get::<Enemy>(target).unwrap();
    assert!(enemy.dead);
    assert_eq!(enemy.hp, 0.0);
    let run = app.world().resource::<Run>();
    assert_eq!(run.kills, 1);
    assert_eq!(run.hp, 52.0);
    assert_eq!(run.damage_done, 20.0);
    assert_eq!(
        run.damage_sources,
        vec![("sinister_strike", 10.0), ("instant_poison", 10.0)]
    );
    assert_eq!(app.world().resource::<Messages<Loot>>().len(), 1);
}
#[test]
fn fresh_deadly_stack_does_not_inherit_frame_time_and_pause_freezes_ticks() {
    let (mut app, target, mut schedule) = fixture("deadly_poison", 200.0);
    hit(&mut app, target, "sinister_strike");
    advance(&mut app, &mut schedule, 3.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 190.0);
    advance(&mut app, &mut schedule, 2.0);
    hit(&mut app, target, "sinister_strike");
    advance(&mut app, &mut schedule, 0.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 180.0);
    app.world_mut().insert_resource(State::new(Phase::Paused));
    advance(&mut app, &mut schedule, 30.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 180.0);
    app.world_mut().insert_resource(State::new(Phase::Playing));
    advance(&mut app, &mut schedule, 1.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 162.0);
    advance(&mut app, &mut schedule, 11.0);
    assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 108.0);
    assert!(!app.world().get::<Enemy>(target).unwrap().poison.active());
}
#[test]
fn deadly_skipped_frames_stop_damage_and_loot_at_the_lethal_tick() {
    let (mut app, target, mut schedule) = fixture("deadly_poison", 25.0);
    hit(&mut app, target, "sinister_strike");
    advance(&mut app, &mut schedule, 0.0);
    advance(&mut app, &mut schedule, 30.0);
    let run = app.world().resource::<Run>();
    assert_eq!(run.damage_done, 25.0);
    assert_eq!(
        run.damage_sources,
        vec![("sinister_strike", 10.0), ("deadly_poison", 15.0)]
    );
    assert_eq!(run.kills, 1);
    assert_eq!(run.hp, 52.5);
    assert_eq!(app.world().resource::<Messages<Loot>>().len(), 1);
    assert!(!app.world().get::<Enemy>(target).unwrap().poison.active());
}
#[test]
fn unrelated_hits_and_unequipped_weapons_cannot_apply_coatings() {
    for key in ["instant_poison", "deadly_poison"] {
        for (source, equipped) in [
            ("fireball", true),
            ("sword_specialization", true),
            ("deadly_poison", true),
            ("sinister_strike", false),
        ] {
            let (mut app, target, mut schedule) = fixture(key, 200.0);
            if !equipped {
                app.world_mut().resource_mut::<Run>().milestones.weapon = None;
            }
            hit(&mut app, target, source);
            advance(&mut app, &mut schedule, 0.0);
            advance(&mut app, &mut schedule, 12.0);
            assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 190.0);
            assert!(!app.world().get::<Enemy>(target).unwrap().poison.active());
        }
    }
}

#[test]
fn splash_echoes_and_generated_attacks_do_not_multiply_coatings() {
    for key in ["instant_poison", "deadly_poison"] {
        let (mut app, target, mut schedule) = fixture(key, 1000.0);
        let (mut other, source, _) = fixture(key, 1000.0);
        let enemy = other
            .world_mut()
            .entity_mut(source)
            .take::<Enemy>()
            .unwrap();
        let neighbor = app
            .world_mut()
            .spawn((
                Guid(124),
                Transform::from_xyz(1.0, 0.0, 0.0),
                ObjectStore(benilla_protocol::messages::ObjectFields::from_pairs(&[])),
                Marks::debuffs(),
                enemy,
            ))
            .id();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.bonus.multistrike = 2.0;
            run.milestones.weapon = Some(MeleeWeapon {
                kind: MeleeKind::Sword,
                two_handed: false,
            });
            run.milestones = run.milestones.with_ranks(&[(1, 242, 5)]);
        }
        app.world_mut().resource_mut::<Messages<Hit>>().write(Hit {
            source: "sinister_strike",
            splash: 2.0,
            ..Hit::direct(target, 10.0, false)
        });
        advance(&mut app, &mut schedule, 0.0);
        if key == "deadly_poison" {
            advance(&mut app, &mut schedule, 3.0);
        }
        let run = app.world().resource::<Run>();
        let poison = run
            .damage_sources
            .iter()
            .find(|row| row.0 == key)
            .unwrap()
            .1;
        assert_eq!(poison, if key == "instant_poison" { 22.0 } else { 9.0 });
        assert!(!app.world().get::<Enemy>(neighbor).unwrap().poison.active());
        // A second actual target gets its own application, independently of earlier splash.
        hit(&mut app, neighbor, "sinister_strike");
        advance(&mut app, &mut schedule, 0.0);
        if key == "deadly_poison" {
            advance(&mut app, &mut schedule, 3.0);
        }
        let run = app.world().resource::<Run>();
        let poison = run
            .damage_sources
            .iter()
            .find(|row| row.0 == key)
            .unwrap()
            .1;
        assert_eq!(poison, if key == "instant_poison" { 44.0 } else { 27.0 });
    }
}
