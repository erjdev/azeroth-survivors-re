//! Appearance access for the local Survivors body compositor.
impl super::Creatures {
    pub(crate) fn survivors_appearance(
        &self,
        display: u32,
    ) -> Option<benilla_formats::NpcAppearance> {
        self.catalog.model(display)?.npc_appearance
    }
}

use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, refresh_features.before(super::EntityVisualsSet));
}

// The stock equipment comparison does not include skin/face/hair. Invalidate its
// applied stamp when local customization changes so the existing compositor updates
// materials and geosets. Portraits then observe those changed material/mesh handles.
fn refresh_features(
    mut units: Query<(
        Entity,
        &crate::net::ObjectStore,
        &super::Equipment,
        &mut super::equipment::AppliedEquipment,
    )>,
    mut previous: Local<std::collections::HashMap<Entity, (Option<u32>, Option<u32>)>>,
) {
    previous.retain(|entity, _| units.contains(*entity));
    for (entity, store, live, mut applied) in &mut units {
        let key = (store.0.player_bytes(), store.0.player_bytes_2());
        if previous.insert(entity, key) != Some(key) && live.settled {
            applied.0.settled = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::equipment::AppliedEquipment;
    use crate::net::ObjectStore;
    use benilla_protocol::ObjectFields;

    #[test]
    fn survivors_features_redress_only_when_descriptor_changes() {
        let mut app = App::new();
        app.add_systems(Update, refresh_features);
        let live = super::super::Equipment {
            settled: true,
            ..Default::default()
        };
        let entity = app
            .world_mut()
            .spawn((
                ObjectStore(ObjectFields::from_pairs(&[(193, 1), (194, 2)])),
                live,
                AppliedEquipment(live),
            ))
            .id();
        app.update();
        assert!(
            !app.world()
                .get::<AppliedEquipment>(entity)
                .unwrap()
                .0
                .settled
        );
        app.world_mut()
            .get_mut::<AppliedEquipment>(entity)
            .unwrap()
            .0 = live;
        app.update();
        assert!(
            app.world()
                .get::<AppliedEquipment>(entity)
                .unwrap()
                .0
                .settled
        );
        app.world_mut()
            .get_mut::<ObjectStore>(entity)
            .unwrap()
            .0
            .merge(ObjectFields::from_pairs(&[(193, 3)]));
        app.update();
        assert!(
            !app.world()
                .get::<AppliedEquipment>(entity)
                .unwrap()
                .0
                .settled
        );
        app.world_mut()
            .get_mut::<AppliedEquipment>(entity)
            .unwrap()
            .0 = live;
        app.world_mut()
            .get_mut::<ObjectStore>(entity)
            .unwrap()
            .0
            .merge(ObjectFields::from_pairs(&[(194, 4)]));
        app.update();
        assert!(
            !app.world()
                .get::<AppliedEquipment>(entity)
                .unwrap()
                .0
                .settled
        );
    }
}
