//! One paragraph of prose with typed numeric changes, shared with plain tooltips.
use super::*;

pub(in crate::survivors) const PROSE: Color = Color::srgb(0.92, 0.9, 0.84);
pub(in crate::survivors) type Spans = Vec<(String, Color)>;
#[derive(Default)]
struct Sentence(Spans);
trait Part {
    fn append(self, out: &mut Spans);
}
impl Part for &str {
    fn append(self, out: &mut Spans) {
        out.push((self.into(), PROSE));
    }
}
impl Part for (f32, f32, &str) {
    fn append(self, out: &mut Spans) {
        let (old, new, unit) = self;
        let changed = (new - old).abs() > 0.0001;
        let precise = changed && num(old) == num(new);
        let fmt = |v| if precise { format!("{v:.2}") } else { num(v) };
        let text = if changed {
            format!("{} » {}{unit}", fmt(old), fmt(new))
        } else {
            format!("{}{unit}", fmt(old))
        };
        out.push((text, if changed { card_growth::GAIN } else { PROSE }));
    }
}
impl Sentence {
    fn add(&mut self, part: impl Part) {
        part.append(&mut self.0);
    }
}

pub(in crate::survivors) fn ability(def: &AbilityDef, a: &Cast, b: &Cast) -> Spans {
    let mut out = Sentence::default();
    macro_rules! prose { ($($p:expr),* $(,)?) => { $(out.add($p);)* }; }
    let damage = (a.dmg, b.dmg, " damage");
    let radius = (a.radius, b.radius, " yd");
    let range = (a.range, b.range, " yd");
    let duration = (a.duration, b.duration, " sec");
    let count = (
        a.count as f32,
        b.count as f32,
        if b.count == 1 { " enemy" } else { " enemies" },
    );
    let cooldown = (a.cooldown, b.cooldown, " sec");
    let burst = def.duration <= 0.0;
    match def.kind {
        Kind::Bolt | Kind::Strike | Kind::Melee => {
            let toughest = matches!(
                def.key,
                "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
            );
            let melee = def.kind == Kind::Melee;
            prose!(if melee { "Strikes " } else { "Hits " });
            if a.count == 1 && b.count == 1 {
                prose!(if toughest {
                    "the toughest enemy"
                } else {
                    "the closest enemy"
                });
            } else {
                prose!(
                    "the ",
                    count,
                    if toughest {
                        " with the most health"
                    } else {
                        " nearest to you"
                    }
                );
            }
            prose!(" within ", range, " for ", damage);
            if a.radius > 0.0 && !melee {
                prose!(", and everything within ", radius, " of the hit");
            }
        }
        Kind::Nova => {
            prose!(
                "Deals ",
                damage,
                " to every enemy within ",
                radius,
                " of you"
            );
        }
        Kind::Cone => {
            prose!(
                "Deals ",
                damage,
                " to every enemy in a ",
                radius,
                " cone toward the closest"
            );
        }
        Kind::Ground | Kind::GroundSelf => {
            let at = if def.kind == Kind::GroundSelf {
                " around you"
            } else {
                " on the thickest pack"
            };
            if burst {
                prose!("Strikes a ", radius, " area", at, " for ", damage);
            } else {
                prose!(
                    "Covers a ",
                    radius,
                    " area",
                    at,
                    " for ",
                    duration,
                    ", dealing ",
                    damage,
                    " every second"
                );
            }
            if def.kind == Kind::Ground {
                prose!(" within ", range, " casting range");
            }
        }
        Kind::Chain => {
            prose!(
                "Strikes the nearest enemy within ",
                range,
                " for ",
                damage,
                ", then jumps to ",
                count,
                " within ",
                radius,
                " of each other, 15% weaker with every jump"
            );
        }
        Kind::Dot => {
            prose!(
                "Afflicts ",
                count,
                " within ",
                range,
                " with ",
                (a.dot, b.dot, " damage"),
                " every second for ",
                duration
            );
        }
        Kind::Aura => {
            prose!(
                "Every ",
                cooldown,
                ", deals ",
                damage,
                " to every enemy within ",
                radius,
                " of you"
            );
        }
        Kind::Orbit => {
            prose!(
                "Every ",
                cooldown,
                ", zaps ",
                count,
                " within ",
                radius,
                " of you for ",
                damage
            );
        }
        Kind::Shield => {
            prose!("Absorbs the next ", damage, " you take within ", duration);
        }
        Kind::Immune => {
            prose!(
                "When an enemy closes in, makes you immune to all damage for ",
                duration
            );
        }
        Kind::Thorns => {
            prose!(
                "When enemies close in, deals ",
                damage,
                " to every attacker for ",
                duration
            );
        }
        Kind::Heal => {
            prose!(
                "Below 85% health, heals you for ",
                (a.dmg, b.dmg, " health"),
                " over ",
                duration
            );
        }
        Kind::Totem if def.key == "magma_totem" => {
            prose!(
                "Drops a totem for ",
                duration,
                " that burns every enemy within ",
                radius,
                " for ",
                damage,
                " every 2 sec"
            );
        }
        Kind::Totem => {
            prose!(
                "Drops a totem for ",
                duration,
                " that hits the nearest enemy within ",
                (a.range, b.range, " yd"),
                " for ",
                damage,
                " every second"
            );
        }
    }
    let ground = matches!(def.kind, Kind::Ground | Kind::GroundSelf);
    if a.dot > 0.0 && def.kind != Kind::Dot && (!ground || burst) {
        let secs = if ground { (5., 5., " sec") } else { duration };
        prose!(
            ", then ",
            (a.dot, b.dot, " damage"),
            " every second for ",
            secs
        );
    }
    let effect_time = (a.effect.duration(), b.effect.duration(), " sec");
    match a.effect {
        Effect::None => {}
        Effect::Slow(m, _) => {
            let next = if let Effect::Slow(next, _) = b.effect {
                next
            } else {
                m
            };
            prose!(
                ", slowing them by ",
                ((1. - m) * 100., (1. - next) * 100., "%"),
                " for ",
                effect_time
            );
        }
        Effect::Root(_) => {
            prose!(", freezing them in place for ", effect_time);
        }
        Effect::Stun(_) => {
            prose!(", stunning them for ", effect_time);
        }
        Effect::Fear(_) => {
            prose!(", sending them fleeing for ", effect_time);
        }
        Effect::Weaken(m, _) => {
            prose!(
                ", cutting their damage by ",
                ((1. - m) * 100., (1. - m) * 100., "%"),
                " for ",
                effect_time
            );
        }
        Effect::Drain(f) => {
            prose!(
                ", healing you for ",
                (f * 100., f * 100., "%"),
                " of the damage"
            );
        }
        Effect::Execute(f) => {
            prose!(
                ", killing them outright below ",
                (f * 100., f * 100., "%"),
                " health"
            );
        }
    }
    if b.shield_fraction > 0.0 {
        prose!(
            ". Absorbs ",
            (
                a.shield_fraction * 100.,
                b.shield_fraction * 100.,
                "% maximum health"
            ),
            " for ",
            (a.shield_seconds, b.shield_seconds, " sec"),
            " once per successful cast; does not stack"
        );
    }
    if b.immune_seconds > 0.0 {
        prose!(
            ". Grants ",
            (a.immune_seconds, b.immune_seconds, " sec"),
            " immunity once per successful cast; preserves longer immunity"
        );
    }
    prose!(".");
    if !matches!(def.kind, Kind::Aura | Kind::Orbit) && (a.cooldown - b.cooldown).abs() > 0.0001 {
        prose!(" Recharges in ", cooldown, ".");
    }
    out.0
}

fn passive(stat: Stat, a: f32, b: f32) -> Spans {
    let mut out = Sentence::default();
    macro_rules! prose { ($($p:expr),* $(,)?) => { $(out.add($p);)* }; }
    let percent = (a * 100., b * 100., "%");
    match stat {
        Stat::Might => {
            prose!("Increases all damage you deal by ", percent);
        }
        Stat::MaxHp => {
            prose!("Increases your maximum health by ", percent);
        }
        Stat::Speed => {
            prose!("Increases your movement speed by ", percent);
        }
        Stat::Regen => {
            prose!("Restores ", (a, b, " health"), " every second");
        }
        Stat::Haste => {
            prose!("Your abilities recharge ", percent, " faster");
        }
        Stat::Area => {
            prose!("Increases the size of your areas of effect by ", percent);
        }
        Stat::Magnet => {
            prose!("Increases your pickup range by ", percent);
        }
        Stat::Growth => {
            prose!("Increases the experience you gain by ", percent);
        }
        Stat::Armor => {
            prose!("Reduces every hit you take by ", (a, b, " damage"));
        }
        Stat::Crit => {
            prose!("Increases your critical strike chance by ", percent);
        }
    }
    prose!(".");
    out.0
}

pub(in crate::survivors) fn describe(choice: Choice, run: &Run) -> Spans {
    let stats = run.stats();
    match choice {
        Choice::NewAbility(def) => {
            let a = cast_at(def, 1, &stats, run.milestones);
            let b = cast_at(def, 2, &stats, run.milestones);
            ability(def, &a, &b)
        }
        Choice::Upgrade(i) => {
            let o = &run.abilities[i];
            ability(
                o.def,
                &cast_at(o.def, o.level, &stats, run.milestones),
                &cast_at(o.def, o.level + 1, &stats, run.milestones),
            )
        }
        Choice::NewPassive(p) => passive(p.stat, p.per_level, p.per_level * 2.),
        Choice::UpPassive(i) => {
            let (p, rank) = run.passives[i];
            passive(
                p.stat,
                p.per_level * f32::from(rank),
                p.per_level * f32::from(rank + 1),
            )
        }
        Choice::Heal if run.hp >= stats.max_hp => {
            vec![("Already at full health.".into(), PROSE)]
        }
        Choice::Heal => {
            let mut out = Sentence::default();
            out.add("Restores your health from ");
            out.add((run.hp, stats.max_hp, " health"));
            out.add(".");
            out.0
        }
        Choice::Jewel(_) => {
            let mut out = Vec::new();
            for (line, color) in card_growth::describe(choice, run) {
                if !out.is_empty() {
                    out.push((" ".into(), PROSE));
                }
                if let Some((label, values)) = line.split_once(" bonus ") {
                    out.push(("Changes your ".into(), PROSE));
                    out.push((format!("{} bonus {values}", label.to_lowercase()), color));
                    out.push((".".into(), PROSE));
                } else {
                    out.push((format!("{line}."), color));
                }
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn plain(spans: &Spans) -> String {
        spans.iter().map(|(s, _)| s.as_str()).collect()
    }
    #[test]
    fn targeting_range_is_inline_and_distinct_from_splash_radius() {
        let stats = Run::default().stats();
        let base = survivors_core::talents::Milestones {
            class: 4,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(1, 1635, 2), (2, 881, 3)]);
        for key in ["smite", "holy_fire", "mind_blast", "shadow_word_pain"] {
            let def = data::ability(key);
            let a = cast_at(def, 1, &stats, base);
            let b = cast_at(def, 1, &stats, trained);
            let spans = ability(def, &a, &b);
            let delta = format!("{} » {} yd", num(a.range), num(b.range));
            assert!(
                spans.contains(&(delta.clone(), card_growth::GAIN)),
                "{key}: {}",
                plain(&spans)
            );
            assert!(plain(&spans).contains(&format!("within {delta}")));
            assert!(!plain(&spans).contains("yd »"));
            let current = ability(def, &b, &b);
            assert!(current.contains(&(format!("{} yd", num(b.range)), PROSE)));
            assert!(!plain(&current).contains('»'));
        }
        let def = data::ability("holy_nova");
        let a = cast_at(def, 1, &stats, base);
        let b = cast_at(def, 1, &stats, trained);
        let text = plain(&ability(def, &a, &b));
        assert!(text.contains("of you"));
        assert!(!text.contains("casting range"));
    }

    #[test]
    fn damage_and_radius_changes_are_inline_colored_with_units() {
        let def = data::ability("frost_nova");
        let stats = Run::default().stats();
        let talents = survivors_core::talents::Milestones::default();
        let mut a = cast_at(def, 1, &stats, talents);
        let mut b = cast_at(def, 2, &stats, talents);
        a.dmg = 20.;
        b.dmg = 43.;
        a.radius = 5.;
        b.radius = 7.;
        let spans = ability(def, &a, &b);
        assert!(
            plain(&spans).starts_with("Deals 20 » 43 damage to every enemy within 5 » 7 yd of you")
        );
        for value in ["20 » 43 damage", "5 » 7 yd"] {
            assert!(spans.contains(&(value.into(), card_growth::GAIN)));
        }
        assert!(spans.contains(&(" to every enemy within ".into(), PROSE)));
        assert!(!plain(&spans).contains("Targets"));
    }
    #[test]
    fn nova_defense_changes_are_inline_with_single_units() {
        let def = data::ability("frost_nova");
        let stats = Run::default().stats();
        let base = survivors_core::talents::Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = survivors_core::talents::Milestones {
            points: [0, 0, 30],
            ..base
        }
        .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
        let spans = ability(
            def,
            &cast_at(def, 1, &stats, base),
            &cast_at(def, 1, &stats, trained),
        );
        for text in ["0 » 15% maximum health", "0 » 5 sec", "0 » 0.5 sec"] {
            assert!(spans.contains(&(text.into(), card_growth::GAIN)), "{text}");
        }
        assert!(plain(&spans).contains("once per successful cast; does not stack"));
    }

    #[test]
    fn frost_slow_strength_and_duration_compare_inline_with_single_units() {
        let def = data::ability("blizzard");
        let stats = Run::default().stats();
        let base = survivors_core::talents::Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = base.with_ranks(&[(2, 65, 3), (2, 63, 3)]);
        let spans = ability(
            def,
            &cast_at(def, 1, &stats, base),
            &cast_at(def, 1, &stats, trained),
        );
        assert!(spans.contains(&("40 » 79%".into(), card_growth::GAIN)));
        assert!(spans.contains(&("1.5 » 4.5 sec".into(), card_growth::GAIN)));
    }

    #[test]
    fn trained_duration_and_fast_cooldowns_preserve_effective_values() {
        let stats = Run::default().stats();
        let talents = survivors_core::talents::Milestones::default().with_ranks(&[(1, 161, 5)]);
        let def = data::ability("demoralizing_shout");
        let spans = ability(
            def,
            &cast_at(def, 1, &stats, talents),
            &cast_at(def, 2, &stats, talents),
        );
        assert!(spans.contains(&("7.5 » 8.2 sec".into(), card_growth::GAIN)));
        assert!(spans.contains(&("50%".into(), PROSE)));
        let def = data::ability("sweeping_strikes");
        let spans = ability(
            def,
            &cast_at(def, 1, &stats, talents),
            &cast_at(def, 2, &stats, talents),
        );
        assert!(spans.contains(&("0.60 » 0.56 sec".into(), card_growth::GAIN)));
    }
    #[test]
    fn passive_and_defensive_prose_use_their_actual_units() {
        let crit = passive(Stat::Crit, 0.05, 0.1);
        assert_eq!(
            plain(&crit),
            "Increases your critical strike chance by 5 » 10%."
        );
        assert!(crit.contains(&("5 » 10%".into(), card_growth::GAIN)));
        let spans = passive(Stat::Regen, 0.5, 1.);
        assert_eq!(plain(&spans), "Restores 0.5 » 1 health every second.");
        assert!(spans.contains(&("0.5 » 1 health".into(), card_growth::GAIN)));
        let run = Run::default();
        for (key, prefix) in [
            ("power_word_shield", "Absorbs"),
            ("rejuvenation", "Below 85% health, heals"),
        ] {
            let spans = describe(Choice::NewAbility(data::ability(key)), &run);
            assert!(plain(&spans).starts_with(prefix), "{}", plain(&spans));
        }
    }
    #[test]
    fn healthstone_shows_a_gain_only_when_health_can_be_restored() {
        let mut run = Run::default();
        let max_hp = run.stats().max_hp;
        run.hp = max_hp;
        assert_eq!(
            describe(Choice::Heal, &run),
            vec![("Already at full health.".into(), PROSE)]
        );
        run.hp = max_hp - 20.;
        let spans = describe(Choice::Heal, &run);
        assert!(spans.contains(&(
            format!("{} » {} health", num(run.hp), num(max_hp)),
            card_growth::GAIN
        )));
        run.hp = max_hp + 1.;
        assert_eq!(
            plain(&describe(Choice::Heal, &run)),
            "Already at full health."
        );
    }

    #[test]
    fn jewelry_replacement_keeps_losses_and_neutral_duplicates_in_prose() {
        use crate::survivors::campaign::JewelKind;
        let mut run = Run::default();
        run.worn.equip(JewelKind::Legend("fireball"));
        let spans = describe(Choice::Jewel(JewelKind::Legend("consecration")), &run);
        assert!(plain(&spans).contains("Fireball only (replaced)"));
        assert!(spans.iter().any(|(_, c)| *c == Color::srgb(1., 0.4, 0.35)));
        let same = describe(Choice::Jewel(JewelKind::Legend("fireball")), &run);
        assert_eq!(
            same,
            vec![("No change to current jewelry bonuses.".into(), DIM)]
        );
    }
}
