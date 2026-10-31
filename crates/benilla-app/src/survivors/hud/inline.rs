//! One paragraph of prose with typed numeric changes, shared with plain tooltips.
use super::*;
use crate::survivors::localization::Strings;

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
    ability_localized(def, a, b, &Strings::default())
}

pub(in crate::survivors) fn mark_power_unit(strings: &Strings) -> String {
    format!(
        " {}",
        strings
            .text("RANGED_ATTACK_POWER", "ranged attack power")
            .to_lowercase()
    )
}

fn ability_localized(def: &AbilityDef, a: &Cast, b: &Cast, strings: &Strings) -> Spans {
    let mark_unit = mark_power_unit(strings);
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
    if def.key == "kick" {
        prose!("Deal ", damage, " to up to ", count, " within ", range,
            ". Waits for an ordinary visible caster with clear line of sight. An original damaging hit locks its interrupted spell school for 5 sec");
        if a.silence_chance > 0. || b.silence_chance > 0. {
            prose!(
                " and has ",
                (a.silence_chance * 100., b.silence_chance * 100., "% chance"),
                " to silence it for 2 sec"
            );
        }
        prose!(". Recharges in ", cooldown, ".");
        return out.0;
    }
    match def.kind {
        Kind::Silence => {
            prose!(
                "Silence up to ",
                (a.count as f32, b.count as f32, if b.count == 1 { " ordinary caster" } else { " ordinary casters" }),
                " within ", range, " for ", duration,
                ", preventing all spellcasting. Waits for a visible cast with clear line of sight; does not damage enemies or prevent melee attacks"
            );
        }
        Kind::Interrupt => {
            prose!(
                "Interrupt up to ",
                (
                    a.count as f32,
                    b.count as f32,
                    if b.count == 1 {
                        " ordinary caster"
                    } else {
                        " ordinary casters"
                    }
                ),
                " within ",
                range,
                ", locking their interrupted spell school for ",
                duration,
                ". Waits for a visible cast with clear line of sight; does not damage enemies"
            );
            if a.silence_chance > 0. || b.silence_chance > 0. {
                prose!(
                    ". Each target also has ",
                    (a.silence_chance * 100., b.silence_chance * 100., "% chance"),
                    " to be silenced for 4 sec"
                );
            }
        }

        Kind::Channel => {
            prose!("Channel ", damage, " every second for ", duration, " into ", count,
                " within ", range, ", slowing them by 50% while connected. You can move and use other abilities. Loses targets that die, become concealed, leave range or move behind solid scenery. Does not retarget or overlap itself");
        }

        Kind::Mark => {
            prose!("Mark ", count, " with the most health within ", range,
                ", adding ", (a.mark_power, b.mark_power, mark_unit.as_str()),
                " against them for ", duration,
                ". Aimed Shot and Multi-Shot gain 0.2 damage per point before their cast modifiers. Reveals concealed marked enemies and tracks them on the minimap. Replaces your previous selection");
        }
        Kind::ContactAura => {
            prose!("Deal ", (a.dmg, b.dmg, " Holy damage"),
                " to each enemy that makes contact with you, including absorbed or immune hits. Dodged and parried attacks do not trigger it. Active for the run; protects only you.");
            return out.0;
        }
        Kind::Aspect => {
            prose!("Gain ", (a.dodge_chance * 100.0, b.dodge_chance * 100.0, "% chance"),
                " to dodge enemy contact attacks before armor and shields. Dodged attacks deal no damage and do not trigger reflection or damage-taken talents. Protects you, not your wolf. One aspect per run; lasts for the run.");
            return out.0;
        }
        Kind::Coating(coating) => {
            prose!(
                "Your original weapon hits have ",
                (a.poison_chance * 100.0, b.poison_chance * 100.0, "% chance"),
                " to deal ",
                damage,
                " as Nature"
            );
            if coating == survivors_core::talents::Coating::Deadly {
                prose!(" every 3 sec for 12 sec. Stacks up to 5 times; refreshes duration without delaying ticks");
            }
            prose!(". Requires a worn melee weapon. One coating per run; lasts for the run.");
            return out.0;
        }
        Kind::WeaponBuff => {
            prose!(
                "Increases your weapon damage by ",
                (a.weapon_bonus * 100.0, b.weapon_bonus * 100.0, "%"),
                " for ",
                duration,
                ". Only weapon attacks benefit; refreshes without stacking"
            );
        }
        Kind::Strike if matches!(a.effect, Effect::Incapacitate(_)) => {
            prose!("Incapacitate the nearest ", count, " within ", range);
        }
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
            if def.key == "ghostly_strike" {
                prose!(". A landed original hit grants 15 percentage points melee dodge for 7 sec; repeated hits refresh without stacking");
            }
            if def.key == "hemorrhage" {
                prose!(". Surviving targets take +3 damage from each subsequent physical hit or bleed tick for 30 charges or 15 sec; recasting refreshes the effect");
            }
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
        Kind::Curse => {
            prose!("Curse ", count, " within ", range);
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
        Kind::DirectHeal => {
            prose!(
                "Below 85% health, heals you immediately for ",
                (a.dmg, b.dmg, " health"),
                ". Has ",
                (a.crit * 100., b.crit * 100., "% chance"),
                " to critically heal for 150%"
            );
        }
        Kind::Regrowth => {
            prose!("Below 85% health, heals you immediately for ", (a.dmg, b.dmg, " health"),
                ", then restores ", (a.dot, b.dot, " health per second"), " for ", duration,
                ". The immediate heal has ", (a.crit * 100., b.crit * 100., "% chance"),
                " to critically heal for 150%. Periodic healing cannot crit; refreshes only Regrowth");
        }
        Kind::Heal => {
            prose!(
                "Below 85% health, heals you for ",
                (a.dmg, b.dmg, " health"),
                " over ",
                duration
            );
        }
        Kind::Totem if def.key == "stoneclaw_totem" => {
            prose!(
                "Drops a totem with ",
                (a.totem_health, b.totem_health, " health"),
                " for ",
                duration,
                ". Forces visible ordinary enemies within ",
                radius,
                " to attack it until it dies or expires. Bosses are immune. Replaces your previous Stoneclaw Totem"
            );
        }
        Kind::Totem if def.key == "earthbind_totem" => {
            prose!(
                "Drops a 5-health totem for ",
                duration,
                ". Every 3 sec it slows every enemy within ",
                radius,
                " by 50% (bosses: 25%) for 5 sec. Replaces your previous Earthbind Totem"
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
    if a.dot > 0.0 && !matches!(def.kind, Kind::Dot | Kind::Regrowth) && (!ground || burst) {
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
        Effect::Incapacitate(_) => {
            prose!(
                " for ",
                effect_time,
                ". Only non-boss humanoids qualify. Any damage awakens them"
            );
        }
        Effect::Stun(_) => {
            prose!(", stunning them for ", effect_time);
        }
        Effect::Fear(_) => {
            prose!(", sending them fleeing for ", effect_time);
        }
        Effect::Weakness(amount, _) => {
            let next = if let Effect::Weakness(next, _) = b.effect {
                next
            } else {
                amount
            };
            prose!(
                ", reducing each physical hit by ",
                (amount, next, " damage"),
                " for ",
                effect_time,
                ". Replaces your other curse on each affected enemy; preserves unrelated debuffs"
            );
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
    if b.shadow_weaving_chance > 0.0 {
        prose!(
            ". Shadow damage has ",
            (
                a.shadow_weaving_chance * 100.,
                b.shadow_weaving_chance * 100.,
                "% chance"
            ),
            " to add 3% Shadow vulnerability for 15 sec, up to 15%"
        );
    }
    if b.vampiric_fraction > 0.0 {
        prose!(
            ". Heals you for ",
            (
                a.vampiric_fraction * 100.,
                b.vampiric_fraction * 100.,
                "% of enemy health removed"
            ),
            " by this spell, including periodic ticks"
        );
    }
    if b.wing_clip_root_chance > 0.0 {
        prose!(
            ". Each direct hit has ",
            (
                a.wing_clip_root_chance * 100.,
                b.wing_clip_root_chance * 100.,
                "% chance"
            ),
            " to root for 5 sec (bosses: 1.5 sec); this does not freeze the target"
        );
    }
    if b.blackout_chance > 0.0 {
        prose!(
            ". Shadow damage has ",
            (
                a.blackout_chance * 100.,
                b.blackout_chance * 100.,
                "% chance"
            ),
            " to stun for 3 sec (bosses: 0.9 sec), including periodic ticks"
        );
    }
    if b.heal_fraction > 0.0 {
        prose!(
            ". Heals ",
            (
                a.heal_fraction * 100.,
                b.heal_fraction * 100.,
                "% maximum health"
            ),
            " once per successful cast"
        );
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
    if def.requires_rear {
        prose!(" Requires an equipped dagger. Only targets enemies you are behind.");
    }
    out.0
}

fn passive(stat: Stat, a: f32, b: f32, strings: &Strings) -> Spans {
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
            prose!("Restores ");
            let changed = (b - a).abs() > 0.0001;
            let value = if changed {
                format!("{} » {}", regen_num(a), regen_num(b))
            } else {
                regen_num(a)
            };
            let value = strings.by_key("HP_TEMPLATE", "%s health", &[value.as_str().into()]);
            out.0
                .push((value, if changed { card_growth::GAIN } else { PROSE }));
            prose!(" every second");
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

pub(in crate::survivors) fn describe(choice: Choice, run: &Run, strings: &Strings) -> Spans {
    let stats = run.stats();
    match choice {
        Choice::NewAbility(def) => {
            let a = cast_at(def, 1, &stats, run.milestones);
            let b = cast_at(def, 2, &stats, run.milestones);
            ability_localized(def, &a, &b, strings)
        }
        Choice::Upgrade(i) => {
            let o = &run.abilities[i];
            ability_localized(
                o.def,
                &cast_at(o.def, o.level, &stats, run.milestones),
                &cast_at(o.def, o.level + 1, &stats, run.milestones),
                strings,
            )
        }
        Choice::NewPassive(p) => passive(
            p.stat,
            run.passive_value(p, 1),
            run.passive_value(p, 2),
            strings,
        ),
        Choice::UpPassive(i) => {
            let (p, rank) = run.passives[i];
            passive(
                p.stat,
                run.passive_value(p, rank),
                run.passive_value(p, rank + 1),
                strings,
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
mod tests;
