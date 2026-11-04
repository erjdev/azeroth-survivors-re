//! One native persistent beam per connected victim; never a victim-to-victim chain.
use super::{channeling::Channels, hero::Hero, Phase, Run, RunEntity};
use crate::creature_anim::{ChainProcPlay, FxClass, SpellKitFx, SpellVisuals};
use crate::net::ObjectStore;
use benilla_protocol::messages::ObjectFields;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct BeamAnchor {
    caster: Entity,
    target: Entity,
    spell: u32,
}

pub(super) fn sync(
    mut commands: Commands,
    run: Res<Run>,
    phase: Res<State<Phase>>,
    channels: Res<Channels>,
    spells: Option<Res<crate::ui_action::Spells>>,
    visuals: Option<Res<SpellVisuals>>,
    mut heroes: Query<
        (
            Entity,
            &Transform,
            Option<&crate::entities::OverheadFallback>,
            &mut ObjectStore,
        ),
        With<Hero>,
    >,
    mut anchors: Query<(Entity, &BeamAnchor, &mut Transform), Without<Hero>>,
    mut plays: MessageWriter<ChainProcPlay>,
    mut reaps: MessageWriter<SpellKitFx>,
) {
    let enabled =
        run.alive() && matches!(phase.get(), Phase::Playing | Phase::Paused | Phase::LevelUp);
    let mut wanted = Vec::new();
    for (caster, transform, height, mut store) in &mut heroes {
        let channel = enabled
            .then(|| {
                channels
                    .active
                    .iter()
                    .find(|channel| channel.caster == caster)
            })
            .flatten();
        let spell = channel.map_or(0, |channel| super::data::ability(channel.source).spell);
        // The real caster owns the hold animation and one sound. Beam-only anchors below
        // carry no channel fields, avoiding duplicated channel kit sounds or animations.
        if store.0.unit_channel_spell() != spell || store.0.unit_channel_object().is_some() {
            store
                .0
                .merge(ObjectFields::from_pairs(&[(144, spell), (20, 0), (21, 0)]));
        }
        if let Some(channel) = channel {
            let origin = transform.translation
                + Vec3::Y * height.map_or(1.0, |height| height.0 * transform.scale.y * 0.75);
            for &target in channel.clock.targets() {
                wanted.push((caster, target, spell, origin));
            }
        }
    }
    let mut present = std::collections::HashSet::new();
    for (entity, anchor, mut transform) in &mut anchors {
        if let Some(&(_, _, _, origin)) = wanted.iter().find(|&&(caster, target, spell, _)| {
            (caster, target, spell) == (anchor.caster, anchor.target, anchor.spell)
        }) {
            transform.translation = origin;
            present.insert((anchor.caster, anchor.target, anchor.spell));
        } else {
            reaps.write(SpellKitFx::Reap {
                entity,
                spell_id: anchor.spell,
                class: FxClass::Hold,
            });
            commands.entity(entity).despawn();
        }
    }
    let (Some(spells), Some(visuals)) = (spells, visuals) else {
        return;
    };
    for (caster, target, spell, origin) in wanted {
        if present.contains(&(caster, target, spell)) {
            continue;
        }
        let Some(proc) = spells
            .catalog
            .get(spell)
            .and_then(|spell| visuals.0.stages(spell.visual))
            .and_then(|stages| visuals.0.kit(stages.channel))
            .and_then(|kit| kit.chain_proc())
        else {
            continue;
        };
        let entity = commands
            .spawn((
                BeamAnchor {
                    caster,
                    target,
                    spell,
                },
                Transform::from_translation(origin),
                ObjectStore(ObjectFields::from_pairs(&[])),
                crate::entities::ChainHops(vec![target]),
                RunEntity,
            ))
            .id();
        plays.write(ChainProcPlay {
            entity,
            spell_id: spell,
            proc,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn installed_channel_uses_independent_silent_anchors_and_reaps_them_on_exit() {
        let path = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::open_chain(&path).unwrap();
        let mut app = App::new();
        app.insert_resource(crate::ui_action::Spells {
            catalog: benilla_formats::load_spell_catalog(&mut chain).unwrap(),
            forms: Default::default(),
            ranges: Default::default(),
            cast_times: Default::default(),
            durations: Default::default(),
            radii: Default::default(),
        })
        .insert_resource(SpellVisuals(
            benilla_formats::load_spell_visual_catalog(&mut chain).unwrap(),
        ))
        .insert_resource(Run {
            hp: 100.0,
            ..Default::default()
        })
        .insert_resource(State::new(Phase::Playing))
        .init_resource::<Channels>()
        .add_message::<ChainProcPlay>()
        .add_message::<SpellKitFx>();
        let caster = app
            .world_mut()
            .spawn((
                Hero { vy: 0.0, air: 0.0 },
                Transform::default(),
                crate::entities::OverheadFallback(2.0),
                ObjectStore(ObjectFields::from_pairs(&[])),
            ))
            .id();
        let first = app.world_mut().spawn_empty().id();
        let second = app.world_mut().spawn_empty().id();
        app.world_mut().resource_mut::<Channels>().active.push(
            super::super::channeling::ActiveChannel {
                caster,
                clock: survivors_core::channel::Channel::new([first, second], 0.0, 1.0, 3).unwrap(),
                damage: 25.0,
                range: 20.0,
                slow: 0.5,
                source: "mind_flay",
            },
        );
        app.world_mut().run_system_once(sync).unwrap();
        assert_eq!(
            app.world()
                .get::<ObjectStore>(caster)
                .unwrap()
                .0
                .unit_channel_spell(),
            15407
        );
        assert_eq!(
            app.world()
                .get::<ObjectStore>(caster)
                .unwrap()
                .0
                .unit_channel_object(),
            None
        );
        let plays: Vec<_> = app
            .world_mut()
            .resource_mut::<Messages<ChainProcPlay>>()
            .drain()
            .collect();
        assert_eq!(plays.len(), 2);
        for play in &plays {
            assert_eq!(play.spell_id, 15407);
            assert!(play.proc.flag);
            let anchor = app.world().get::<BeamAnchor>(play.entity).unwrap();
            assert_eq!(
                app.world()
                    .get::<crate::entities::ChainHops>(play.entity)
                    .unwrap()
                    .0,
                vec![anchor.target]
            );
            assert_eq!(
                app.world()
                    .get::<ObjectStore>(play.entity)
                    .unwrap()
                    .0
                    .unit_channel_spell(),
                0
            );
        }
        app.world_mut()
            .get_mut::<Transform>(caster)
            .unwrap()
            .translation
            .x = 2.0;
        app.world_mut().insert_resource(State::new(Phase::Paused));
        app.world_mut().run_system_once(sync).unwrap();
        assert_eq!(app.world().resource::<Messages<ChainProcPlay>>().len(), 0);
        for play in &plays {
            assert_eq!(
                app.world()
                    .get::<Transform>(play.entity)
                    .unwrap()
                    .translation,
                Vec3::new(2.0, 1.5, 0.0)
            );
        }
        app.world_mut().insert_resource(State::new(Phase::GameOver));
        app.world_mut().run_system_once(sync).unwrap();
        assert_eq!(
            app.world()
                .get::<ObjectStore>(caster)
                .unwrap()
                .0
                .unit_channel_spell(),
            0
        );
        assert_eq!(app.world().resource::<Messages<SpellKitFx>>().len(), 2);
        assert!(plays
            .iter()
            .all(|play| app.world().get_entity(play.entity).is_err()));
    }
}
