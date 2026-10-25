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
            let old_unit = if matches!(unit, " yd" | " sec" | "%") {
                unit
            } else {
                ""
            };
            format!("{}{old_unit} » {}{unit}", fmt(old), fmt(new))
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
                    "the toughest enemy in range"
                } else {
                    "the closest enemy"
                });
            } else {
                prose!(
                    "the ",
                    count,
                    if toughest {
                        " with the most health in range"
                    } else {
                        " nearest to you"
                    }
                );
            }
            prose!(" for ", damage);
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
        }
        Kind::Chain => {
            prose!(
                "Strikes the nearest enemy for ",
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
                " in range with ",
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
                (def.range, def.range, " yd"),
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
            prose!(
                ", slowing them by ",
                ((1. - m) * 100., (1. - m) * 100., "%"),
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
            prose!(
                "Increases your chance to critically strike for double damage by ",
                percent
            );
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
            let mut spans = vec![("Next rank: ".into(), PROSE)];
            spans.extend(ability(def, &a, &b));
            spans
        }
        Choice::Upgrade(i) => {
            let o = &run.abilities[i];
            ability(
                o.def,
                &cast_at(o.def, o.level, &stats, run.milestones),
                &cast_at(o.def, o.level + 1, &stats, run.milestones),
            )
        }
        Choice::NewPassive(p) => {
            let mut spans = vec![("Next rank: ".into(), PROSE)];
            spans.extend(passive(p.stat, p.per_level, p.per_level * 2.));
            spans
        }
        Choice::UpPassive(i) => {
            let (p, rank) = run.passives[i];
            passive(
                p.stat,
                p.per_level * f32::from(rank),
                p.per_level * f32::from(rank + 1),
            )
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
        assert!(plain(&spans)
            .starts_with("Deals 20 » 43 damage to every enemy within 5 yd » 7 yd of you"));
        for value in ["20 » 43 damage", "5 yd » 7 yd"] {
            assert!(spans.contains(&(value.into(), card_growth::GAIN)));
        }
        assert!(spans.contains(&(" to every enemy within ".into(), PROSE)));
        assert!(!plain(&spans).contains("Targets"));
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
        assert!(spans.contains(&("7.5 sec » 8.2 sec".into(), card_growth::GAIN)));
        assert!(spans.contains(&("50%".into(), PROSE)));
        let def = data::ability("sweeping_strikes");
        let spans = ability(
            def,
            &cast_at(def, 1, &stats, talents),
            &cast_at(def, 2, &stats, talents),
        );
        assert!(spans.contains(&("0.60 sec » 0.56 sec".into(), card_growth::GAIN)));
    }
    #[test]
    fn passive_and_defensive_prose_use_their_actual_units() {
        let spans = passive(Stat::Regen, 0.5, 1.);
        assert_eq!(plain(&spans), "Restores 0.5 » 1 health every second.");
        assert!(spans.contains(&("0.5 » 1 health".into(), card_growth::GAIN)));
        let run = Run::default();
        for (key, prefix) in [
            ("power_word_shield", "Next rank: Absorbs"),
            ("rejuvenation", "Next rank: Below 85% health, heals"),
        ] {
            let spans = describe(Choice::NewAbility(data::ability(key)), &run);
            assert!(plain(&spans).starts_with(prefix), "{}", plain(&spans));
        }
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
