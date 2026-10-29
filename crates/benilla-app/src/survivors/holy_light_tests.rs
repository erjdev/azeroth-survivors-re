use super::*;
use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
use crate::survivors::Owned;
use bevy::ecs::system::RunSystemOnce;
use survivors_core::talents::Milestones;

#[test]
fn holy_light_cast_preserves_failed_readiness_and_applies_healing_crit_without_damage() {
    for critical in [false, true] {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_secs_f32(0.1));
        let mut run = Run {
            class: &data::CLASSES[1],
            milestones: Milestones {
                class: 1,
                ..Default::default()
            }
            .with_ranks(&[(0, 1627, 5), (0, 1433, 1)]),
            ..Default::default()
        };
        run.hp = run.stats().max_hp;
        run.abilities.push(Owned {
            def: data::ability("holy_light"),
            level: 1,
            cd: 0.,
        });
        run.healing.refresh("regrowth", 24., 12.);
        let periodic = run.healing.clone();
        let chance = Params::new(&run.abilities[0], &run.stats(), run.milestones).crit;
        let seed = (1..100000)
            .find(|seed| (Rng(*seed).f32() < chance) == critical)
            .unwrap();
        app.insert_resource(time)
            .insert_resource(run)
            .insert_resource(Rng(seed))
            .init_resource::<PlaySeq>()
            .init_resource::<PendingHits>()
            .add_message::<CastEvent>()
            .add_message::<SpellGoTargets>()
            .add_message::<SwingMessage>()
            .add_message::<crate::combat_text::CombatTextSpawn>()
            .add_message::<Hit>();
        app.world_mut()
            .spawn((Transform::default(), Hero { vy: 0., air: 0. }));
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert_eq!(
            run.critical_cast.bonus(run.milestones, "holy_light", 0.),
            0.0
        );
        assert_eq!(run.power_infusion.remaining(run.milestones, 0.), 0.);
        assert_eq!(app.world().resource::<Rng>().0, seed);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.hp = 1.;
            run.abilities[0].cd = 0.;
        }
        let run = app.world().resource::<Run>();
        let p = Params::new(&run.abilities[0], &run.stats(), run.milestones);
        let expected = (1. + p.dmg * if critical { 1.5 } else { 1. }).min(run.stats().max_hp);
        app.world_mut().run_system_once(cast_abilities).unwrap();
        let run = app.world().resource::<Run>();
        assert!((run.hp - expected).abs() < 0.0001);
        assert_eq!(run.inspiration_t, 0.);
        assert_eq!(
            run.critical_cast.bonus(run.milestones, "holy_light", 0.),
            0.
        );
        assert_eq!(run.power_infusion.remaining(run.milestones, 0.), 0.);
        assert_eq!(run.healing, periodic);
        assert_eq!(run.abilities[0].cd, p.cooldown);
        assert_eq!(
            run.critical_cast.bonus(run.milestones, "holy_shock", 0.),
            1.0,
            "Holy Light must preserve the existing Holy Shock-only Divine Favor conversion"
        );
        assert!(app.world().resource::<Messages<Hit>>().is_empty());
    }
}
