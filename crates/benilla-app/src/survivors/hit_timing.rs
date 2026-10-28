//! Preserve occurrence order across channel pulses, missiles and current-frame hits.
use super::abilities::{Hit, PendingHits};

impl PendingHits {
    /// Stable ties keep already queued hits before new arrivals, and preserve each
    /// producer's order. Expiry/refresh of target modifiers remains a separate timeline.
    pub(super) fn drain_due(
        &mut self,
        now: f32,
        incoming: impl IntoIterator<Item = Hit>,
    ) -> Vec<Hit> {
        self.0.extend(incoming.into_iter().map(|hit| {
            let at = hit.occurred.unwrap_or(now);
            (at, hit)
        }));
        if !now.is_finite() || now < 0.0 {
            return Vec::new();
        }
        let mut due = Vec::new();
        self.0.retain(|(at, hit)| {
            if !at.is_finite() || *at < 0.0 {
                return false;
            }
            if *at > now {
                return true;
            }
            let mut hit = hit.clone();
            hit.occurred = Some(*at);
            due.push(hit);
            false
        });
        due.sort_by(|a, b| a.occurred.unwrap().total_cmp(&b.occurred.unwrap()));
        due
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn interleaved_batches_keep_times_ties_and_future_hits() {
        let target = World::new().spawn_empty().id();
        let hit = |at, damage| Hit {
            occurred: Some(at),
            ..Hit::direct(target, damage, false)
        };
        let mut pending = PendingHits(vec![(2.0, hit(99.0, 20.0))]);
        let due = pending.drain_due(
            3.0,
            [
                hit(3.0, 30.0),
                hit(1.0, 10.0),
                hit(2.0, 21.0),
                hit(4.0, 40.0),
                hit(f32::NAN, 99.0),
                hit(-1.0, 99.0),
            ],
        );
        assert_eq!(
            due.iter()
                .map(|h| (h.occurred.unwrap(), h.damage))
                .collect::<Vec<_>>(),
            [(1.0, 10.0), (2.0, 20.0), (2.0, 21.0), (3.0, 30.0)]
        );
        assert!(pending.drain_due(3.0, []).is_empty());
        assert!(pending.drain_due(f32::NAN, []).is_empty());
        let later = pending.drain_due(4.0, [Hit::direct(target, 41.0, false)]);
        assert_eq!(
            later.iter().map(|h| h.damage).collect::<Vec<_>>(),
            [40.0, 41.0]
        );
        assert!(later.iter().all(|h| h.occurred == Some(4.0)));
        assert!(pending.0.is_empty());
    }

    #[test]
    fn earlier_missile_kills_before_later_channel_tick_even_when_batched() {
        for batched in [false, true] {
            let (mut app, target, mut settle) =
                super::super::abilities::poison_tests::fixture("mind_flay", 30.0);
            app.world_mut().resource_mut::<PendingHits>().0.push((
                1.0,
                Hit {
                    source: "mind_blast",
                    ..Hit::direct(target, 50.0, false)
                },
            ));
            if !batched {
                app.world_mut().resource_mut::<super::super::Run>().elapsed = 1.0;
                settle.run(app.world_mut());
            }
            app.world_mut().write_message(Hit {
                occurred: Some(2.0),
                source: "mind_flay",
                ..Hit::direct(target, 25.0, false)
            });
            app.world_mut().resource_mut::<super::super::Run>().elapsed = 3.0;
            settle.run(app.world_mut());
            let run = app.world().resource::<super::super::Run>();
            assert_eq!(
                run.damage_sources,
                [("mind_blast", 50.0)],
                "batched={batched}"
            );
            assert_eq!(run.hp, 53.0); // Leech uses the victim's 30 health, once.
            assert!(
                app.world()
                    .get::<super::super::enemies::Enemy>(target)
                    .unwrap()
                    .dead
            );
        }
    }
}
