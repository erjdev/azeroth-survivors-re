//! Real hit settlement with target-owned marks and cast-owned coefficients.
use super::poison_tests::{advance, fixture};
use super::*;
use crate::survivors::{data, marking, Owned};
use bevy::ecs::system::RunSystemOnce;

fn hunter(app: &mut App) -> Entity {
    let mut run = app.world_mut().resource_mut::<Run>();
    run.class = &data::CLASSES[2];
    run.milestones = survivors_core::talents::Milestones {
        class: 2,
        ..Default::default()
    };
    run.abilities.clear();
    app.world_mut()
        .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()))
        .id()
}

fn mark(app: &mut App, target: Entity, caster: Entity, duration: f32) {
    let mut mark = marking::Marked::default();
    assert!(mark.0.apply(caster, 20.0, duration, 0.0));
    app.world_mut().entity_mut(target).insert(mark);
}

fn shot(app: &App, target: Entity, source: &'static str, level: u8, critical: bool) -> Hit {
    let run = app.world().resource::<Run>();
    let def = data::ability(source);
    let p = Params::new(
        &Owned {
            def,
            level,
            cd: 0.0,
        },
        &run.stats(),
        run.milestones,
    );
    hit_for(
        def,
        &p,
        target,
        p.dmg
            * if critical {
                p.crit_damage.max(1.0)
            } else {
                1.0
            },
        critical,
    )
}

fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
}

#[test]
fn marks_only_add_to_normalized_weapon_shots_and_keep_cast_scaling() {
    for source in ["aimed_shot", "multi_shot", "arcane_shot", "raptor_strike"] {
        for level in [1, 4, 8] {
            for critical in [false, true] {
                let (mut app, target, mut schedule) = fixture("instant_poison", 10000.0);
                let caster = hunter(&mut app);
                mark(&mut app, target, caster, 120.0);
                let input = shot(&app, target, source, level, critical);
                let eligible = matches!(source, "aimed_shot" | "multi_shot");
                assert_eq!(input.ranged_power_scale > 0.0, eligible);
                let expected = input.damage + 20.0 * input.ranged_power_scale;
                // A stat change after launch must not alter the cast coefficient.
                app.world_mut().resource_mut::<Run>().bonus.might = 5.0;
                app.world_mut().resource_mut::<Messages<Hit>>().write(input);
                advance(&mut app, &mut schedule, 0.0);
                near(
                    app.world().get::<Enemy>(target).unwrap().hp,
                    10000.0 - expected,
                );
                near(app.world().resource::<Run>().damage_done as f32, expected);
            }
        }
    }
}

#[test]
fn delayed_hits_check_mark_expiry_and_caster_presence_at_impact() {
    for (impact, remove_caster) in [(0.5, false), (1.0, false), (0.5, true)] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        let caster = hunter(&mut app);
        mark(&mut app, target, caster, 1.0);
        let input = shot(&app, target, "multi_shot", 1, false);
        let expected = input.damage
            + if impact < 1.0 && !remove_caster {
                20.0 * input.ranged_power_scale
            } else {
                0.0
            };
        app.world_mut()
            .resource_mut::<PendingHits>()
            .0
            .push((impact, input));
        advance(&mut app, &mut schedule, 0.0);
        assert_eq!(app.world().get::<Enemy>(target).unwrap().hp, 1000.0);
        if remove_caster {
            app.world_mut().despawn(caster);
        }
        app.world_mut().resource_mut::<Run>().elapsed = impact;
        advance(&mut app, &mut schedule, 0.0);
        near(
            app.world().get::<Enemy>(target).unwrap().hp,
            1000.0 - expected,
        );
        app.world_mut().run_system_once(marking::prune).unwrap();
        assert_eq!(
            app.world().get::<marking::Marked>(target).is_some(),
            impact < 1.0 && !remove_caster
        );
    }
}

#[test]
fn splash_reads_its_own_mark_and_scales_the_bonus_once() {
    for primary_marked in [false, true] {
        let (mut app, target, mut schedule) = fixture("instant_poison", 1000.0);
        let caster = hunter(&mut app);
        let (mut other, other_target, _) = fixture("instant_poison", 1000.0);
        let mut bundle = other
            .world_mut()
            .entity_mut(other_target)
            .take::<(Enemy, Guid, Transform, ObjectStore, Marks)>()
            .unwrap();
        bundle.1 = Guid(124);
        bundle.2.translation = Vec3::X;
        let secondary = app.world_mut().spawn(bundle).id();
        mark(
            &mut app,
            if primary_marked { target } else { secondary },
            caster,
            120.0,
        );
        let mut input = shot(&app, target, "multi_shot", 1, false);
        input.splash = 2.0;
        let base = input.damage;
        let bonus = 20.0 * input.ranged_power_scale;
        app.world_mut().resource_mut::<Messages<Hit>>().write(input);
        advance(&mut app, &mut schedule, 0.0);
        near(
            app.world().get::<Enemy>(target).unwrap().hp,
            1000.0 - base - if primary_marked { bonus } else { 0.0 },
        );
        near(
            app.world().get::<Enemy>(secondary).unwrap().hp,
            1000.0 - 0.7 * (base + if primary_marked { 0.0 } else { bonus }),
        );
    }
}

#[test]
fn target_death_clears_gameplay_mark() {
    let (mut app, target, mut schedule) = fixture("instant_poison", 1.0);
    let caster = hunter(&mut app);
    mark(&mut app, target, caster, 120.0);
    let input = shot(&app, target, "aimed_shot", 1, false);
    app.world_mut().resource_mut::<Messages<Hit>>().write(input);
    advance(&mut app, &mut schedule, 0.0);
    assert!(app.world().get::<Enemy>(target).unwrap().dead);
    assert!(app.world().get::<marking::Marked>(target).is_none());
}

#[test]
fn concealed_targets_require_our_live_mark_for_direct_casts_but_not_area_damage() {
    use crate::survivors::perception::Concealed;
    for source in [
        "arcane_shot",
        "serpent_sting",
        "chain_lightning",
        "frost_nova",
    ] {
        for (concealed, own_mark, elapsed) in [
            (false, false, 0.0),
            (true, false, 0.0),
            (true, true, 0.0),
            (true, true, 120.0),
        ] {
            let (mut app, target, _) = fixture(source, 10000.0);
            let caster = hunter(&mut app);
            app.world_mut().resource_mut::<Run>().abilities.push(Owned {
                def: data::ability(source),
                level: 1,
                cd: 0.0,
            });
            if concealed {
                app.world_mut().entity_mut(target).insert(Concealed);
            }
            let marker = if own_mark {
                caster
            } else {
                app.world_mut().spawn_empty().id()
            };
            mark(&mut app, target, marker, 120.0);
            app.world_mut().resource_mut::<Run>().elapsed = elapsed;
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(0.1));
            app.world_mut().run_system_once(cast_abilities).unwrap();
            let expected = source == "frost_nova" || !concealed || (own_mark && elapsed < 120.0);
            assert_eq!(
                !app.world().resource::<Messages<Hit>>().is_empty(),
                expected,
                "{source}: concealed={concealed}, own={own_mark}, time={elapsed}"
            );
            if !expected {
                assert_eq!(
                    app.world().resource::<Run>().abilities[0].cd,
                    0.2,
                    "a hidden target only schedules the existing short retry, not a cast cooldown"
                );
            }
        }
    }
}

#[test]
fn render_concealment_restores_prior_visibility_on_reveal_and_removal() {
    use crate::survivors::perception::{self, Concealed};
    for original in [
        Visibility::Inherited,
        Visibility::Visible,
        Visibility::Hidden,
    ] {
        let (mut app, target, _) = fixture("arcane_shot", 100.0);
        let caster = hunter(&mut app);
        app.world_mut()
            .entity_mut(target)
            .insert((original, Concealed));
        app.world_mut().run_system_once(perception::sync).unwrap();
        assert_eq!(
            *app.world().get::<Visibility>(target).unwrap(),
            Visibility::Hidden
        );
        mark(&mut app, target, caster, 120.0);
        app.world_mut().run_system_once(perception::sync).unwrap();
        assert_eq!(*app.world().get::<Visibility>(target).unwrap(), original);
        app.world_mut().resource_mut::<Run>().elapsed = 120.0;
        app.world_mut().run_system_once(perception::sync).unwrap();
        assert_eq!(
            *app.world().get::<Visibility>(target).unwrap(),
            Visibility::Hidden
        );
        app.world_mut().entity_mut(target).remove::<Concealed>();
        app.world_mut().run_system_once(perception::sync).unwrap();
        assert_eq!(*app.world().get::<Visibility>(target).unwrap(), original);
        app.world_mut().entity_mut(target).insert(Concealed);
        app.world_mut().resource_mut::<Run>().elapsed = 0.0;
        app.world_mut().despawn(caster);
        app.world_mut().run_system_once(perception::sync).unwrap();
        assert_eq!(
            *app.world().get::<Visibility>(target).unwrap(),
            Visibility::Hidden
        );
    }
}

#[test]
fn ranged_totems_require_reveal_but_pulses_hit_concealed_victims() {
    use crate::survivors::perception::Concealed;
    for pulse in [false, true] {
        for revealed in [false, true] {
            let (mut app, target, _) = fixture("arcane_shot", 100.0);
            let caster = hunter(&mut app);
            app.world_mut().entity_mut(target).insert(Concealed);
            if revealed {
                mark(&mut app, target, caster, 120.0);
            }
            app.world_mut().spawn((
                Transform::default(),
                Totem {
                    cast: 0,
                    damage: 10.0,
                    range: 20.0,
                    radius: 10.0,
                    left: 10.0,
                    tick: 0.0,
                    pulse,
                    source: "searing_totem",
                },
            ));
            app.world_mut()
                .resource_mut::<Time>()
                .advance_by(std::time::Duration::from_secs_f32(0.1));
            app.world_mut().run_system_once(tick_areas).unwrap();
            assert_eq!(
                !app.world().resource::<Messages<Hit>>().is_empty(),
                pulse || revealed
            );
        }
    }
}
