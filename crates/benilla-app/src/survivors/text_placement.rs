//! Reuse exact combat-text placement prefixes, including while a choice pauses the game.
//! A changed request invalidates the remaining prefix; the stock solver decides every new seat.
use bevy::math::{Rect, Vec2};

use crate::smart_rect::SmartBucket;

#[derive(Clone, Copy)]
struct Seat {
    desired: Rect,
    viewport: Vec2,
    solved: Rect,
    claimed: Rect,
}

#[derive(Default)]
pub(crate) struct Bucket {
    inner: SmartBucket,
    seats: Vec<Seat>,
    cursor: usize,
    matching: bool,
}

impl Bucket {
    pub(crate) fn clear(&mut self) {
        self.inner.clear();
        self.seats.truncate(self.cursor);
        self.cursor = 0;
        self.matching = true;
    }

    pub(crate) fn resolve(&mut self, desired: Rect, viewport: Vec2) -> Rect {
        if self.matching {
            if let Some(seat) = self.seats.get(self.cursor) {
                if seat.desired == desired && seat.viewport == viewport {
                    self.cursor += 1;
                    return seat.solved;
                }
            }
        }
        self.matching = false;
        let solved = self.inner.resolve(desired, viewport);
        let seat = Seat {
            desired,
            viewport,
            solved,
            claimed: solved,
        };
        if let Some(previous) = self.seats.get_mut(self.cursor) {
            *previous = seat;
        } else {
            self.seats.push(seat);
        }
        self.cursor += 1;
        solved
    }

    pub(crate) fn claim(&mut self, rect: Rect) {
        // A different caller-supplied claim also breaks the cached prefix.
        if self.cursor == 0 {
            self.matching = false;
        } else {
            let seat = &mut self.seats[self.cursor - 1];
            self.matching &= seat.claimed == rect;
            seat.claimed = rect;
        }
        self.inner.claim(rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_seats_match_stock_through_pause_motion_resize_and_population_changes() {
        let mut cached = Bucket::default();
        let mut stock = SmartBucket::default();
        let base: Vec<_> = (0..32)
            .map(|i| {
                Rect::from_center_size(
                    Vec2::new(300.0 + (i % 4) as f32, 200.0 + (i / 4) as f32),
                    Vec2::new(40.0, 25.0),
                )
            })
            .collect();
        for frame in 0..8 {
            let viewport = if frame >= 5 {
                Vec2::new(800.0, 600.0)
            } else {
                Vec2::new(640.0, 480.0)
            };
            let mut rects = base.clone();
            match frame {
                2 => {
                    rects[7].min.x += 8.0;
                    rects[7].max.x += 8.0;
                }
                3 => rects.truncate(12),
                4 => rects.reverse(),
                6 => rects.clear(),
                _ => {}
            }
            cached.clear();
            stock.clear();
            for desired in rects {
                let actual = cached.resolve(desired, viewport);
                let expected = stock.resolve(desired, viewport);
                assert_eq!(actual, expected, "frame {frame}");
                cached.claim(actual);
                stock.claim(expected);
            }
        }
    }

    #[test]
    fn caller_claim_changes_invalidate_the_remaining_prefix() {
        let mut cached = Bucket::default();
        let mut stock = SmartBucket::default();
        let viewport = Vec2::new(640.0, 480.0);
        let desired = Rect::from_center_size(Vec2::splat(200.0), Vec2::splat(30.0));
        cached.clear();
        for _ in 0..3 {
            let seat = cached.resolve(desired, viewport);
            cached.claim(seat);
        }
        cached.clear();
        let first = cached.resolve(desired, viewport);
        let changed = Rect::from_center_size(first.center(), Vec2::splat(60.0));
        cached.claim(changed);
        stock.claim(changed);
        let next = cached.resolve(desired, viewport);
        assert_eq!(next, stock.resolve(desired, viewport));
        cached.claim(next);
        // Restoring the original claim on a later frame must not reuse seats that
        // were solved against the enlarged rectangle.
        cached.clear();
        stock.clear();
        for _ in 0..3 {
            let actual = cached.resolve(desired, viewport);
            let expected = stock.resolve(desired, viewport);
            assert_eq!(actual, expected);
            cached.claim(actual);
            stock.claim(expected);
        }
    }
}
