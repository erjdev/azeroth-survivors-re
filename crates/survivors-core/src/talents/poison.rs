//! Acquired Rogue weapon coatings and target-local Deadly Poison timing.
use super::{Milestones, TalentStat};
use crate::gear_loadout::{MeleeKind, MeleeWeapon};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coating {
    Instant,
    Deadly,
}
impl Coating {
    pub fn key(self) -> &'static str {
        match self {
            Self::Instant => "instant_poison",
            Self::Deadly => "deadly_poison",
        }
    }
    pub fn chance(self, talents: Milestones) -> f32 {
        let base = match self {
            Self::Instant => 0.20,
            Self::Deadly => 0.30,
        };
        base + if talents.class == 3 {
            talents.trained_value(TalentStat::PoisonChance)
        } else {
            0.0
        }
    }
    pub fn weapon_allowed(weapon: Option<MeleeWeapon>) -> bool {
        weapon.is_some_and(|w| {
            !w.two_handed
                && matches!(
                    w.kind,
                    MeleeKind::Dagger | MeleeKind::Sword | MeleeKind::Mace | MeleeKind::Fist
                )
        })
    }
    pub fn eligible(talents: Milestones, source: &str, removed: f32, original: bool) -> bool {
        original
            && talents.class == 3
            && Self::weapon_allowed(talents.weapon)
            && removed.is_finite()
            && removed > 0.0
            && matches!(
                source,
                "backstab"
                    | "hemorrhage"
                    | "ghostly_strike"
                    | "sinister_strike"
                    | "eviscerate"
                    | "blade_flurry"
                    | "gouge"
                    | "kidney_shot"
            )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DeadlyPoison {
    stacks: u8,
    remaining: f64,
    next_tick: f64,
    damage_per_stack: f32,
}
impl DeadlyPoison {
    pub fn active(self) -> bool {
        self.stacks > 0
    }
    pub fn apply(&mut self, damage_per_stack: f32) {
        if !damage_per_stack.is_finite() || damage_per_stack <= 0.0 {
            return;
        }
        if self.stacks == 0 {
            self.next_tick = 3.0;
        }
        self.stacks = self.stacks.saturating_add(1).min(5);
        self.remaining = 12.0;
        self.damage_per_stack = damage_per_stack;
    }
    /// Returns due tick count and damage per tick, including the tick at expiry.
    /// Caller settles health/credit once per tick and stops after a lethal tick.
    pub fn advance(&mut self, dt: f32) -> (u32, f32) {
        if !dt.is_finite() || dt <= 0.0 || self.stacks == 0 {
            return (0, 0.0);
        }
        let elapsed = f64::from(dt).min(self.remaining);
        self.remaining = (self.remaining - elapsed).max(0.0);
        self.next_tick -= elapsed;
        let ticks = if self.next_tick <= 0.00000001 {
            ((-self.next_tick + 0.00000001) / 3.0).floor() as u32 + 1
        } else {
            0
        };
        self.next_tick += f64::from(ticks) * 3.0;
        let damage = self.damage_per_stack * f32::from(self.stacks);
        if self.remaining == 0.0 {
            *self = Self::default();
        }
        (ticks, damage)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn rogue(rank: u8) -> Milestones {
        Milestones {
            class: 3,
            weapon: Some(MeleeWeapon {
                kind: MeleeKind::Dagger,
                two_handed: false,
            }),
            ..Default::default()
        }
        .with_ranks(&[(0, 268, rank), (0, 682, rank)])
    }
    #[test]
    fn talent_ranks_only_scale_poison_chance_and_damage() {
        for rank in 0..=7 {
            let t = rogue(rank);
            let rank = rank.min(5) as f32;
            for (coating, base) in [(Coating::Instant, 0.2), (Coating::Deadly, 0.3)] {
                assert!((coating.chance(t) - (base + rank * 0.02)).abs() < 0.00001);
                assert!((t.cast(coating.key()).damage - (1.0 + rank * 0.04)).abs() < 0.00001);
            }
            for key in ["sinister_strike", "backstab", "rupture", "fireball"] {
                assert_eq!(t.cast(key).damage, 1.0);
            }
        }
        for class in (0..9).filter(|c| *c != 3) {
            let t = Milestones {
                class,
                ..Default::default()
            }
            .with_ranks(&[(0, 268, 5), (0, 682, 5)]);
            assert_eq!(Coating::Instant.chance(t), 0.2);
            assert_eq!(t.cast("deadly_poison").damage, 1.0);
        }
    }
    #[test]
    fn eligible_hits_require_original_damage_and_a_worn_rogue_weapon() {
        let t = rogue(5);
        for key in [
            "sinister_strike",
            "backstab",
            "eviscerate",
            "blade_flurry",
            "gouge",
            "kidney_shot",
        ] {
            assert!(Coating::eligible(t, key, 1.0, true));
            assert!(!Coating::eligible(t, key, 1.0, false));
            for removed in [0.0, -1.0, f32::NAN, f32::INFINITY] {
                assert!(!Coating::eligible(t, key, removed, true));
            }
            assert!(!Coating::eligible(
                Milestones { weapon: None, ..t },
                key,
                1.0,
                true
            ));
            assert!(!Coating::eligible(
                Milestones { class: 0, ..t },
                key,
                1.0,
                true
            ));
        }
        for key in [
            "rupture",
            "sword_specialization",
            "deadly_poison",
            "instant_poison",
            "fireball",
        ] {
            assert!(!Coating::eligible(t, key, 1.0, true));
        }
        for kind in [
            MeleeKind::Dagger,
            MeleeKind::Sword,
            MeleeKind::Mace,
            MeleeKind::Fist,
            MeleeKind::Axe,
            MeleeKind::Staff,
            MeleeKind::Polearm,
        ] {
            assert_eq!(
                Coating::weapon_allowed(Some(MeleeWeapon {
                    kind,
                    two_handed: false
                })),
                matches!(
                    kind,
                    MeleeKind::Dagger | MeleeKind::Sword | MeleeKind::Mace | MeleeKind::Fist
                )
            );
            assert!(!Coating::weapon_allowed(Some(MeleeWeapon {
                kind,
                two_handed: true
            })));
        }
    }
    #[test]
    fn expiry_includes_last_tick_and_reapplication_starts_fresh() {
        let mut poison = DeadlyPoison::default();
        poison.apply(9.0);
        assert_eq!(poison.advance(12.0), (4, 9.0));
        assert_eq!(poison, DeadlyPoison::default());
        assert_eq!(poison.advance(30.0), (0, 0.0));
        poison.apply(10.0);
        assert_eq!(poison.stacks, 1);
        assert_eq!(poison.advance(2.0), (0, 10.0));
        assert_eq!(poison.advance(1.0), (1, 10.0));
    }
    #[test]
    fn refresh_preserves_tick_phase_and_caps_stacks() {
        let mut poison = DeadlyPoison::default();
        poison.apply(9.0);
        assert_eq!(poison.advance(2.0), (0, 9.0));
        for _ in 0..9 {
            poison.apply(9.0);
        }
        assert_eq!(poison.stacks, 5);
        assert_eq!(poison.advance(1.0), (1, 45.0));
        assert_eq!(poison.advance(11.0), (3, 45.0));
        assert_eq!(poison, DeadlyPoison::default());
    }
    #[test]
    fn frame_partitioning_preserves_total_damage_and_expiry() {
        for frame in [0.01, 0.1, 0.25, 0.5, 1.0, 3.0, 12.0, 30.0] {
            let mut poison = DeadlyPoison::default();
            poison.apply(9.0);
            let mut ticks = 0;
            for _ in 0..4000 {
                ticks += poison.advance(frame).0;
            }
            assert_eq!(ticks, 4, "frame {frame}");
            assert_eq!(poison, DeadlyPoison::default());
        }
    }
    #[test]
    fn invalid_time_and_damage_cannot_change_live_state() {
        let mut poison = DeadlyPoison::default();
        poison.apply(9.0);
        let before = poison;
        for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            poison.apply(invalid);
            assert_eq!(poison, before);
            assert_eq!(poison.advance(invalid), (0, 0.0));
            assert_eq!(poison, before);
        }
    }
}
