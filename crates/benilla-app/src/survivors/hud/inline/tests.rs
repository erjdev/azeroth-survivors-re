use super::*;

fn describe(choice: Choice, run: &Run) -> Spans {
    super::describe(choice, run, &Strings::default())
}

fn passive(stat: Stat, a: f32, b: f32) -> Spans {
    super::passive(stat, a, b, &Strings::default())
}

#[test]
fn translated_regeneration_keeps_fractional_values_and_one_green_unit() {
    let script = benilla_ui::script::UiScript::new().unwrap();
    script
        .lua()
        .load("HP_TEMPLATE = '%s points de vie'")
        .exec()
        .unwrap();
    let strings = Strings::from_script(&script);
    let changed = super::passive(Stat::Regen, 0.525, 1.05, &strings);
    assert_eq!(
        plain(&changed),
        "Restores 0.525 » 1.05 points de vie every second."
    );
    assert!(changed.contains(&("0.525 » 1.05 points de vie".into(), card_growth::GAIN)));
    let current = super::passive(Stat::Regen, 0.525, 0.525, &strings);
    assert!(current.contains(&("0.525 points de vie".into(), PROSE)));
    assert!(!current.iter().any(|(_, color)| *color == card_growth::GAIN));
}

#[test]
fn renew_preview_matches_trained_regeneration_without_rounding_away_ranks() {
    let renew = data::PASSIVES.iter().find(|p| p.key == "renew").unwrap();
    let mut run = Run {
        milestones: survivors_core::talents::Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(&[(1, 406, 1)]),
        ..Default::default()
    };
    let spans = describe(Choice::NewPassive(renew), &run);
    assert_eq!(plain(&spans), "Restores 0.525 » 1.05 health every second.");
    assert!(spans
        .iter()
        .any(|(text, color)| text == "0.525 » 1.05 health" && *color == card_growth::GAIN));
    run.passives.push((renew, 2));
    assert_eq!(
        plain(&describe(Choice::UpPassive(0), &run)),
        "Restores 1.05 » 1.575 health every second."
    );
}

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
fn cast_healing_is_visible_inline_without_duplicate_tooltip_rewards() {
    let stats = Run::default().stats();
    for (class, points, key, expected) in [
        (4, [0, 20, 0], "holy_fire", "3"),
        (4, [0, 30, 0], "holy_nova", "5"),
        (1, [20, 0, 0], "holy_shock", "3"),
    ] {
        let base = survivors_core::talents::Milestones {
            class,
            ..Default::default()
        };
        let trained = survivors_core::talents::Milestones { points, ..base };
        let def = data::ability(key);
        let a = cast_at(def, 1, &stats, base);
        let b = cast_at(def, 1, &stats, trained);
        let changed = ability(def, &a, &b);
        assert!(changed.contains(&(format!("0 » {expected}% maximum health"), card_growth::GAIN)));
        let current = ability(def, &b, &cast_at(def, 2, &stats, trained));
        assert!(current.contains(&(format!("{expected}% maximum health"), PROSE)));
        assert_eq!(
            plain(&current).matches("once per successful cast").count(),
            1
        );
        assert!(!plain(&ability(def, &a, &a)).contains("maximum health"));
    }
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
    assert!(plain(&spans).starts_with("Deals 20 » 43 damage to every enemy within 5 » 7 yd of you"));
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
        vec![("No change to this spell's bonuses.".into(), DIM)]
    );
}
#[test]
fn wing_clip_preview_shows_trained_root_as_fixed_and_slow_growth_inline() {
    let run = Run {
        milestones: survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(2, 1305, 3)]),
        ..Default::default()
    };
    let spans = describe(Choice::NewAbility(data::ability("wing_clip")), &run);
    let text = plain(&spans);
    assert!(
        text.contains("12% chance to root for 5 sec (bosses: 1.5 sec)"),
        "{text}"
    );
    assert!(spans.contains(&("12% chance".into(), PROSE)));
    assert!(spans.contains(&("5 » 5.5 sec".into(), card_growth::GAIN)));
    let other = plain(&describe(
        Choice::NewAbility(data::ability("raptor_strike")),
        &run,
    ));
    assert!(!other.contains("chance to root"));
}

#[test]
fn battle_shout_cards_show_percentage_strength_and_duration_without_damage_hits() {
    let mut run = Run::default();
    let def = data::ability("battle_shout");
    let spans = describe(Choice::NewAbility(def), &run);
    assert!(spans.contains(&("10 » 13%".into(), card_growth::GAIN)));
    assert!(spans.contains(&("8 » 8.8 sec".into(), card_growth::GAIN)));
    let text = plain(&spans);
    assert!(text.starts_with("Increases your weapon damage by 10 » 13% for 8 » 8.8 sec."));
    assert!(!text.contains("0 damage"));
    assert!(!text.contains("targets"));
    run.milestones = survivors_core::talents::Milestones::default().with_ranks(&[(1, 154, 5)]);
    let trained = describe(Choice::NewAbility(def), &run);
    assert!(plain(&trained).starts_with("Increases your weapon damage by 12.5 » 16.2%"));
    let before = trained.clone();
    run.bonus.might = 10.0;
    run.bonus.crit = 0.5;
    assert_eq!(describe(Choice::NewAbility(def), &run), before);
}
