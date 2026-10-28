//! Choice benefits use the same cast parameters and jewelry rules as applying the choice.
use super::*;
use crate::survivors::campaign;

pub(in crate::survivors) const GAIN: Color = Color::srgb(0.3, 1.0, 0.3);
const LOSS: Color = Color::srgb(1.0, 0.4, 0.35);

pub(in crate::survivors) fn describe(choice: Choice, run: &Run) -> Vec<(String, Color)> {
    let stats = run.stats();
    match choice {
        Choice::NewAbility(def) => {
            let a = cast_at(def, 1, &stats, run.milestones);
            let b = cast_at(def, 2, &stats, run.milestones);
            vec![(format!("Next rank\n{}", ability_changes(def, &a, &b)), GAIN)]
        }
        Choice::Upgrade(i) => {
            let o = &run.abilities[i];
            let a = cast_at(o.def, o.level, &stats, run.milestones);
            let b = cast_at(o.def, o.level + 1, &stats, run.milestones);
            vec![(ability_changes(o.def, &a, &b), GAIN)]
        }
        Choice::NewPassive(p) => vec![(
            format!(
                "{}\nNext rank: {} » {}",
                passive_text(p.stat, run.passive_value(p, 1)),
                passive_value(p.stat, run.passive_value(p, 1)),
                passive_value(p.stat, run.passive_value(p, 2))
            ),
            GAIN,
        )],
        Choice::UpPassive(i) => {
            let (p, rank) = run.passives[i];
            vec![(
                format!(
                    "{}\n{} » {}",
                    stat_label(p.stat),
                    passive_value(p.stat, run.passive_value(p, rank)),
                    passive_value(p.stat, run.passive_value(p, rank + 1))
                ),
                GAIN,
            )]
        }
        Choice::Heal => vec![(
            format!("Health {} » {}", num(run.hp), num(stats.max_hp)),
            GAIN,
        )],
        Choice::Jewel(campaign::JewelKind::Legend(key)) => {
            let mut after = run.worn;
            after.equip(campaign::JewelKind::Legend(key));
            let next = legend_effect(after).unwrap();
            if run
                .worn
                .legendary_bonus()
                .is_some_and(|(old, _)| old == key)
            {
                return legendary_changes(key, next.1, next.1);
            }
            let mut lines = Vec::new();
            if let Some((old, bonus)) = legend_effect(run.worn) {
                lines.push((
                    format!(
                        "{} only (replaced)",
                        crate::survivors::armory::spell_title(old)
                    ),
                    LOSS,
                ));
                lines.extend(legendary_changes(old, bonus, campaign::Bonus::default()));
            }
            lines.push((
                format!("{} only", crate::survivors::armory::spell_title(key)),
                GAIN,
            ));
            lines.extend(legendary_changes(key, campaign::Bonus::default(), next.1));
            lines
        }
        Choice::Jewel(kind) => {
            let before = run.worn.bonus();
            let mut after = run.worn;
            after.equip(kind);
            jewelry_changes(before, after.bonus())
        }
    }
}

pub(in crate::survivors) fn legend_effect(
    worn: campaign::Worn,
) -> Option<(&'static str, campaign::Bonus)> {
    worn.legendary_bonus().map(|(key, mut bonus)| {
        let def = data::ability(key);
        bonus.targets = super::super::sheet::extra_targets(def.kind, def.count, bonus.targets);
        if matches!(
            def.kind,
            Kind::Heal
                | Kind::Shield
                | Kind::Immune
                | Kind::Thorns
                | Kind::Channel
                | Kind::Dot
                | Kind::WeaponBuff
                | Kind::Aspect
                | Kind::Mark
        ) {
            bonus.crit_damage = 0.0;
        }
        if matches!(def.kind, Kind::Immune | Kind::WeaponBuff) {
            bonus.might = 0.0;
        }
        (key, bonus)
    })
}

/// The same power multiplier heals or absorbs for defensive spells; it is not damage.
pub(in crate::survivors) fn legendary_changes(
    key: &str,
    before: campaign::Bonus,
    after: campaign::Bonus,
) -> Vec<(String, Color)> {
    let label = match data::ability(key).kind {
        Kind::Aspect => "Dodge chance multiplier",
        Kind::Mark => "Mark power multiplier",
        Kind::Heal => "Healing bonus",
        Kind::Shield => "Absorb bonus",
        _ => "Damage bonus",
    };
    bonus_changes(before, after, "No change to this spell's bonuses")
        .into_iter()
        .map(|(text, color)| (text.replacen("Damage bonus", label, 1), color))
        .collect()
}

fn passive_value(stat: Stat, value: f32) -> String {
    match stat {
        Stat::Regen => regen_num(value),
        Stat::Armor => num(value),
        _ => pct(value),
    }
}

fn ability_changes(def: &AbilityDef, a: &Cast, b: &Cast) -> String {
    let mut lines = Vec::new();
    let mut change = |label: &str, old: f32, new: f32, unit: &str| {
        if (new - old).abs() > 0.001 {
            let (old, new) = if num(old) == num(new) {
                (format!("{old:.2}"), format!("{new:.2}"))
            } else {
                (num(old), num(new))
            };
            lines.push(format!("{label} {old} » {new}{unit}"));
        }
    };
    if def.kind != Kind::Immune && def.kind != Kind::Dot {
        let label = match def.kind {
            Kind::Heal => "Healing",
            Kind::Shield => "Absorb",
            _ => "Damage",
        };
        change(label, a.dmg, b.dmg, "");
    }
    if a.dot > 0.0 || b.dot > 0.0 {
        change("Damage / sec", a.dot, b.dot, "");
    }
    change(
        "Weapon damage bonus",
        a.weapon_bonus * 100.0,
        b.weapon_bonus * 100.0,
        "%",
    );
    change(
        "Mark power",
        a.mark_power,
        b.mark_power,
        &inline::mark_power_unit(&crate::survivors::localization::Strings::default()),
    );
    change(
        "Dodge chance",
        a.dodge_chance * 100.0,
        b.dodge_chance * 100.0,
        "%",
    );
    change("Cooldown", a.cooldown, b.cooldown, "s");
    if matches!(
        def.kind,
        Kind::Channel
            | Kind::Mark
            | Kind::Bolt
            | Kind::Strike
            | Kind::Melee
            | Kind::Dot
            | Kind::Chain
            | Kind::Orbit
    ) {
        change(
            if def.kind == Kind::Chain {
                "Jumps"
            } else {
                "Targets"
            },
            a.count as f32,
            b.count as f32,
            "",
        );
    }
    change("Radius", a.radius, b.radius, " yd");
    change("Duration", a.duration, b.duration, "s");
    change(
        "Effect duration",
        a.effect.duration(),
        b.effect.duration(),
        "s",
    );
    if lines.is_empty() {
        "No numerical change at current bonuses".into()
    } else {
        lines.join("\n")
    }
}

pub(in crate::survivors) fn jewelry_changes(
    a: super::super::campaign::Bonus,
    b: super::super::campaign::Bonus,
) -> Vec<(String, Color)> {
    bonus_changes(a, b, "No change to current jewelry bonuses")
}

pub(in crate::survivors) fn bonus_changes(
    a: campaign::Bonus,
    b: campaign::Bonus,
    unchanged: &str,
) -> Vec<(String, Color)> {
    let mut lines = Vec::new();
    // Compare the complete contribution, including replacement losses.
    for (label, old, new, percent) in [
        ("Health", a.health, b.health, false),
        ("Pickup range", a.pickup, b.pickup, true),
        ("Armor", a.armor, b.armor, false),
        ("Damage", a.might, b.might, true),
        ("Critical chance", a.crit, b.crit, true),
        ("Multistrike", a.multistrike, b.multistrike, true),
        ("Movement speed", a.speed, b.speed, true),
        ("Leech", a.leech, b.leech, true),
        ("Area", a.area, b.area, true),
        ("Critical damage", a.crit_damage, b.crit_damage, true),
        ("Damage reduction", a.reduction, b.reduction, true),
        ("Cooldown reduction", a.cooldown, b.cooldown, true),
        ("Gold", a.gold, b.gold, true),
        ("Experience", a.growth, b.growth, true),
        ("Luck", a.luck, b.luck, false),
        (
            "Direct-attack targets",
            a.targets as f32,
            b.targets as f32,
            false,
        ),
    ] {
        if (new - old).abs() > 0.0001 {
            let fmt = |v| if percent { pct(v) } else { num(v) };
            lines.push((
                format!(
                    "{label} bonus {} » {}",
                    num(if percent { old * 100.0 } else { old }),
                    fmt(new)
                ),
                if new > old { GAIN } else { LOSS },
            ));
        }
    }
    if lines.is_empty() {
        lines.push((unchanged.into(), DIM));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periodic_legendaries_show_effective_damage_without_critical_or_target_bonuses() {
        let pure_dots: Vec<_> = data::ABILITIES
            .iter()
            .filter(|def| def.kind == Kind::Dot)
            .collect();
        assert!(!pure_dots.is_empty());
        for def in pure_dots {
            let choice = Choice::Jewel(campaign::JewelKind::Legend(def.key));
            let run = Run::default();
            let lines = describe(choice, &run);
            assert!(
                lines.iter().any(|(text, _)| text == "Damage bonus 0 » 5%"),
                "{}: {lines:?}",
                def.key
            );
            assert!(
                !lines
                    .iter()
                    .any(|(text, _)| text.contains("Critical") || text.contains("targets")),
                "{}: {lines:?}",
                def.key
            );
            let mut replaced = Run::default();
            replaced.worn.equip(campaign::JewelKind::Legend(def.key));
            let lines = describe(
                Choice::Jewel(campaign::JewelKind::Legend("fireball")),
                &replaced,
            );
            assert!(lines.iter().any(|(text, _)| text == "Damage bonus 5 » 0%"));
            assert!(!lines
                .iter()
                .any(|(text, _)| text == "Critical damage bonus 15 » 0%"));
            assert!(lines
                .iter()
                .any(|(text, _)| text == "Critical damage bonus 0 » 15%"));
            let identical = describe(choice, &replaced);
            assert!(identical
                .iter()
                .any(|(text, _)| text == "No change to this spell's bonuses"));
        }
    }

    #[test]
    fn defensive_legendaries_report_only_effective_power_and_replacement_losses() {
        for (key, label) in [
            ("rejuvenation", "Healing"),
            ("power_word_shield", "Absorb"),
            ("thorns", "Damage"),
        ] {
            let run = Run::default();
            let lines = describe(Choice::Jewel(campaign::JewelKind::Legend(key)), &run);
            assert!(
                lines
                    .iter()
                    .any(|(text, _)| text == &format!("{label} bonus 0 » 5%")),
                "{key}: {lines:?}"
            );
            assert!(!lines
                .iter()
                .any(|(text, _)| text.contains("Critical") || text.contains("targets")));
            let mut worn = run;
            worn.worn.equip(campaign::JewelKind::Legend(key));
            let lines = describe(
                Choice::Jewel(campaign::JewelKind::Legend("fireball")),
                &worn,
            );
            assert!(lines
                .iter()
                .any(|(text, color)| text == &format!("{label} bonus 5 » 0%") && *color == LOSS));
        }
        let run = Run::default();
        let lines = describe(
            Choice::Jewel(campaign::JewelKind::Legend("divine_shield")),
            &run,
        );
        assert!(lines
            .iter()
            .any(|(text, color)| text == "Cooldown reduction bonus 0 » 15%" && *color == GAIN));
        assert!(!lines
            .iter()
            .any(|(text, _)| text.contains("Damage bonus") || text.contains("Critical")));
        let mut worn = run;
        worn.worn
            .equip(campaign::JewelKind::Legend("divine_shield"));
        let duplicate = describe(
            Choice::Jewel(campaign::JewelKind::Legend("divine_shield")),
            &worn,
        );
        assert!(duplicate
            .iter()
            .any(|(text, _)| text == "No change to this spell's bonuses"));
        let replacement = describe(
            Choice::Jewel(campaign::JewelKind::Legend("fireball")),
            &worn,
        );
        assert!(replacement
            .iter()
            .any(|(text, color)| text == "Cooldown reduction bonus 15 » 0%" && *color == LOSS));
    }

    #[test]
    fn legendary_replacement_reports_scoped_losses_and_gains() {
        let mut run = Run::default();
        run.worn.equip(campaign::JewelKind::Legend("fireball"));
        let lines = describe(
            Choice::Jewel(campaign::JewelKind::Legend("consecration")),
            &run,
        );
        assert!(lines
            .iter()
            .any(|(s, c)| s.contains("Fireball only (replaced)") && *c == LOSS));
        assert!(lines
            .iter()
            .any(|(s, c)| s.contains("Consecration only") && *c == GAIN));
        assert!(!lines
            .iter()
            .any(|(s, c)| s.contains("targets") && *c == GAIN));
        let same = describe(Choice::Jewel(campaign::JewelKind::Legend("fireball")), &run);
        assert_eq!(same.len(), 1);
        assert_eq!(same[0].1, DIM);
    }

    #[test]
    fn fast_ability_cooldown_changes_keep_enough_precision() {
        let def = data::ability("sweeping_strikes");
        let stats = Run::default().stats();
        let milestones = survivors_core::talents::Milestones::default();
        let a = cast_at(def, 1, &stats, milestones);
        let b = cast_at(def, 2, &stats, milestones);
        assert!(ability_changes(def, &a, &b).contains("Cooldown 0.60 » 0.56s"));
    }

    #[test]
    fn full_jewelry_slots_report_losses_as_well_as_gains() {
        use crate::survivors::campaign::{JewelKind, Worn};
        let mut worn = Worn::default();
        worn.equip(JewelKind::Ring(0));
        worn.equip(JewelKind::Ring(2));
        let old = worn.bonus();
        worn.equip(JewelKind::Ring(1));
        let lines = jewelry_changes(old, worn.bonus());
        assert!(lines
            .iter()
            .any(|(s, c)| s.contains("Critical chance") && *c == LOSS));
        assert!(lines
            .iter()
            .any(|(s, c)| s.contains("Movement speed") && *c == GAIN));
    }
    #[test]
    fn area_upgrades_never_claim_extra_locations_and_defenses_use_their_own_labels() {
        let stats = Run::default().stats();
        let milestones = survivors_core::talents::Milestones::default();
        for key in ["frost_nova", "power_word_shield"] {
            let def = data::ability(key);
            let a = cast_at(def, 2, &stats, milestones);
            let b = cast_at(def, 3, &stats, milestones);
            let text = ability_changes(def, &a, &b);
            assert!(!text.contains("Targets"), "{key}: {text}");
            if def.kind == Kind::Shield {
                assert!(text.contains("Absorb"));
            }
            if def.kind == Kind::Heal {
                assert!(text.contains("Healing"));
            }
        }
    }
}
