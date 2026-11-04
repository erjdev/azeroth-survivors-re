//! Moving area healing; the single-player run owns its hero and combat pets.
use super::{
    hero::Hero,
    hunter_pet::HunterPet,
    units::{Fx, COLOR_HEAL},
    Floor, Run,
};
use bevy::prelude::*;

pub(super) struct Active {
    caster: Entity,
    last: f32,
    radius: f32,
    healing: survivors_core::healing::HealingOverTime,
}

impl Active {
    pub fn new(caster: Entity, now: f32, total: f32, radius: f32) -> Option<Self> {
        if !now.is_finite() || !radius.is_finite() || radius <= 0.0 {
            return None;
        }
        let mut healing = survivors_core::healing::HealingOverTime::default();
        healing
            .refresh_pulsed("tranquility", total, 2.0, 5)
            .then_some(Self {
                caster,
                last: now,
                radius,
                healing,
            })
    }
}

/// Called only during active play. Re-evaluate range/life/line of sight at every payment.
/// Aggregated long-frame payments use current positions; no stale target list follows a pet.
pub(super) fn tick(
    mut run: ResMut<Run>,
    heroes: Query<&Transform, With<Hero>>,
    mut pets: Query<(Entity, &Transform, &mut HunterPet), Without<Hero>>,
    floor: Floor,
    mut fx: Fx,
) {
    let Some(mut active) = run.tranquility.take() else {
        return;
    };
    if !run.alive() {
        return;
    }
    let Ok(hero) = heroes.get(active.caster) else {
        return;
    };
    let elapsed = (run.elapsed - active.last).max(0.0);
    active.last = run.elapsed;
    let amount = active.healing.advance(elapsed);
    if amount > 0.0 {
        let max_hp = run.stats().max_hp;
        let restored = run.heal(amount, max_hp);
        if restored > 0.0 {
            fx.number(active.caster, restored, false, COLOR_HEAL);
        }
        let mut recipients = vec![active.caster];
        for (entity, transform, mut pet) in &mut pets {
            if pet.vitality.hp > 0.0
                && super::hero::reach(hero.translation, transform.translation) <= active.radius
                && floor.line_clear(hero.translation, transform.translation)
            {
                let restored = pet.vitality.heal(amount);
                if restored > 0.0 {
                    fx.number(entity, restored, false, COLOR_HEAL);
                }
                recipients.push(entity);
            }
        }
        fx.cast(active.caster, 740, &recipients, None);
    }
    if active.healing.contains("tranquility") {
        run.tranquility = Some(active);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivors::{abilities, data, Owned};
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn actual_cast_pulses_follow_the_caster_and_never_revive_or_bank_healing() {
        let (mut app, hero, _) = abilities::earthbind_tests::setup(true);
        let mut companion = HunterPet::default();
        companion.vitality = survivors_core::combat_pet::Vitality::new(100., 10.);
        let pet = app
            .world_mut()
            .spawn((Transform::from_xyz(3., 0., 0.), companion))
            .id();
        app.world_mut()
            .get_mut::<HunterPet>(pet)
            .unwrap()
            .vitality
            .hp = 10.;
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = &data::CLASSES[8];
            run.milestones = survivors_core::talents::Milestones {
                class: 8,
                ..Default::default()
            };
            run.hp = 10.;
            run.abilities = vec![Owned {
                def: data::ability("tranquility"),
                level: 1,
                cd: 0.,
            }];
            run.abilities.push(Owned {
                def: data::ability("wrath"),
                level: 1,
                cd: 0.,
            });
            run.healing.refresh("rejuvenation", 30., 10.);
        }
        app.world_mut()
            .run_system_once(abilities::cast_abilities)
            .unwrap();
        let expected = {
            let run = app.world().resource::<Run>();
            assert!(run.tranquility.is_some());
            assert!(
                run.abilities[1].cd > 0.,
                "Wrath fires alongside the healing channel"
            );
            assert!(run.healing.contains("rejuvenation"));
            abilities::Params::new(&run.abilities[0], &run.stats(), run.milestones).dmg / 5.
        };
        for (now, hero_x, expected_pet) in [
            (1., 0., 10.),
            (2., 0., 10. + expected),
            (4., 50., 10. + expected),
            (6., 0., 10. + expected * 2.),
        ] {
            app.world_mut().resource_mut::<Run>().elapsed = now;
            app.world_mut()
                .get_mut::<Transform>(hero)
                .unwrap()
                .translation
                .x = hero_x;
            app.world_mut().run_system_once(tick).unwrap();
            assert!(
                (app.world().get::<HunterPet>(pet).unwrap().vitality.hp - expected_pet).abs()
                    < 0.001
            );
            if now == 2. {
                app.world_mut().resource_mut::<Run>().abilities[0].cd = 0.;
                app.world_mut()
                    .run_system_once(abilities::cast_abilities)
                    .unwrap();
            }
        }
        let hp = app.world().resource::<Run>().hp;
        assert!((hp - (10. + expected * 3.)).abs() < 0.001);
        app.world_mut().run_system_once(tick).unwrap();
        assert_eq!(
            app.world().resource::<Run>().hp,
            hp,
            "unchanged run time cannot repeat a pulse"
        );
        app.world_mut()
            .get_mut::<HunterPet>(pet)
            .unwrap()
            .vitality
            .hurt(1000., 6.);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.hp = run.stats().max_hp;
            run.elapsed = 100.;
        }
        app.world_mut().run_system_once(tick).unwrap();
        assert_eq!(app.world().get::<HunterPet>(pet).unwrap().vitality.hp, 0.);
        assert!(app.world().resource::<Run>().tranquility.is_none());
        app.world_mut().resource_mut::<Run>().hp = 10.;
        app.world_mut().run_system_once(tick).unwrap();
        assert_eq!(app.world().resource::<Run>().hp, 10.);
        app.world_mut().resource_mut::<Run>().tranquility = Active::new(hero, 100., 60., 20.);
        app.world_mut().resource_mut::<Run>().hp = 0.;
        app.world_mut().run_system_once(tick).unwrap();
        assert_eq!(app.world().resource::<Run>().hp, 0.);
        assert!(app.world().resource::<Run>().tranquility.is_none());
    }
}
