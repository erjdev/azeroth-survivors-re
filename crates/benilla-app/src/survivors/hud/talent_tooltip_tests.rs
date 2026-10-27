use super::*;
use survivors_core::talents::Milestones;

#[test]
fn translated_tooltip_templates_preserve_green_upgrade_lines() {
    let script = benilla_ui::script::UiScript::new().unwrap();
    script
        .lua()
        .load(
            r#"
        SPELL_RANGE = "Portée : %s yd"
        SPELL_RECAST_TIME_SEC = "Recharge : %s sec"
        TOOLTIP_TALENT_NEXT_RANK = "Rang suivant :"
        HONOR_HIGHEST_RANK = "Rang maximal"
    "#,
        )
        .exec()
        .unwrap();
    let strings = Strings::from_script(&script);
    let owned = Owned {
        def: data::ability("fireball"),
        level: 1,
        cd: 0.,
    };
    let run = Run::default();
    let tip = ability_tip(&owned, run.milestones, &run.stats(), None, &strings);
    assert!(tip.iter().any(|line| line
        .right
        .as_ref()
        .is_some_and(|(s, _)| s.starts_with("Portée : "))));
    assert!(tip.iter().any(|line| line.left == "Rang suivant :"));
    assert!(tip
        .iter()
        .any(|line| line.left.contains("Recharge : ") && line.color == card_growth::GAIN));
    let maxed = Owned {
        level: data::MAX_ABILITY_LEVEL,
        ..owned
    };
    let tip = ability_tip(&maxed, run.milestones, &run.stats(), None, &strings);
    assert!(tip.iter().any(|line| line.left == "Rang maximal"));
    assert!(!tip.iter().any(|line| line.left == "Rang suivant :"));
}

#[test]
fn mage_range_tooltips_use_the_same_trained_range_as_targeting() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    }
    .with_ranks(&[(1, 28, 2), (2, 741, 2)]);
    let stats = Run::default().stats();
    for (key, expected) in [
        ("fireball", "31.2 yd range"),
        ("blizzard", "24 yd range"),
        ("arcane_missiles", "22 yd range"),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, mage, &stats, None, &Strings::default());
        assert!(
            tip.iter().any(|line| line
                .right
                .as_ref()
                .is_some_and(|(text, _)| text == expected)),
            "{key}"
        );
    }
}

#[test]
fn nova_tooltip_explains_trained_immunity_and_combined_barrier() {
    let trained = Milestones {
        class: 6,
        points: [0, 0, 30],
        ..Default::default()
    }
    .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
    let owned = Owned {
        def: data::ability("frost_nova"),
        level: 1,
        cd: 0.,
    };
    let tip = ability_tip(
        &owned,
        trained,
        &Run::default().stats(),
        None,
        &Strings::default(),
    );
    for text in [
        "Grants 0.5 sec immunity once per successful cast; preserves longer immunity.",
        "Absorbs 15% maximum health for 5 sec once per successful cast; does not stack.",
    ] {
        assert!(tip
            .iter()
            .any(|line| line.left == text && line.color == card_growth::GAIN));
    }
}

#[test]
fn shatter_tooltip_is_green_and_only_shown_for_trained_mage_spells() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = mage.with_ranks(&[(2, 67, 5)]);
    let stats = Run::default().stats();
    for (key, shown) in [
        ("fireball", true),
        ("frostbolt", true),
        ("blizzard", true),
        ("arcane_missiles", true),
        ("volley", false),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let line = tip.iter().find(|line| line.left.starts_with("Shatter:"));
        assert_eq!(line.is_some(), shown);
        if let Some(line) = line {
            assert!(line.left.contains("50%"));
            assert_eq!(line.color, card_growth::GAIN);
        }
        assert!(
            !ability_tip(&owned, mage, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left.starts_with("Shatter:"))
        );
    }
}

#[test]
fn impact_tooltip_is_green_and_only_shown_for_trained_fire_spells() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = mage.with_ranks(&[(1, 30, 5)]);
    let stats = Run::default().stats();
    for (key, shown) in [
        ("fireball", true),
        ("frostbolt", false),
        ("flamestrike", true),
        ("arcane_missiles", false),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let line = tip.iter().find(|line| line.left.starts_with("Impact:"));
        assert_eq!(line.is_some(), shown);
        if let Some(line) = line {
            assert!(line.left.contains("10%"));
            assert_eq!(line.color, card_growth::GAIN);
        }
        assert!(
            !ability_tip(&owned, mage, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left.starts_with("Impact:"))
        );
    }
}

#[test]
fn winters_chill_tooltip_is_green_and_only_shown_for_trained_frost_spells() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = mage.with_ranks(&[(2, 68, 5)]);
    let stats = Run::default().stats();
    for (key, shown) in [
        ("fireball", false),
        ("frostbolt", true),
        ("blizzard", true),
        ("arcane_missiles", false),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let line = tip
            .iter()
            .find(|line| line.left.starts_with("Winter's Chill:"));
        assert_eq!(line.is_some(), shown);
        if let Some(line) = line {
            assert!(line.left.contains("100%"));
            assert_eq!(line.color, card_growth::GAIN);
        }
        assert!(
            !ability_tip(&owned, mage, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left.starts_with("Winter's Chill:"))
        );
    }
}

#[test]
fn frostbite_tooltip_is_green_and_only_shown_for_trained_frost_spells() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = mage.with_ranks(&[(2, 38, 3)]);
    let stats = Run::default().stats();
    for (key, shown) in [
        ("fireball", false),
        ("frostbolt", true),
        ("blizzard", true),
        ("arcane_missiles", false),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let line = tip.iter().find(|line| line.left.starts_with("Frostbite:"));
        assert_eq!(line.is_some(), shown);
        if let Some(line) = line {
            assert!(line.left.contains("15%"));
            assert_eq!(line.color, card_growth::GAIN);
        }
        assert!(
            !ability_tip(&owned, mage, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left.starts_with("Frostbite:"))
        );
    }
}

#[test]
fn ignite_tooltip_is_green_and_only_shown_for_trained_fire_spells() {
    let mage = Milestones {
        class: 6,
        ..Default::default()
    };
    let trained = mage.with_ranks(&[(1, 34, 5)]);
    let stats = Run::default().stats();
    for (key, shown) in [
        ("fireball", true),
        ("flamestrike", true),
        ("blizzard", false),
        ("arcane_missiles", false),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let line = tip.iter().find(|line| line.left.starts_with("Ignite:"));
        assert_eq!(line.is_some(), shown);
        if let Some(line) = line {
            assert!(line.left.contains("40%"));
            assert_eq!(line.color, card_growth::GAIN);
        }
        assert!(
            !ability_tip(&owned, mage, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left.starts_with("Ignite:"))
        );
    }
}

#[test]
fn hunter_control_tooltips_show_ordinary_and_boss_durations() {
    let trained = Milestones {
        class: 2,
        ..Default::default()
    }
    .with_ranks(&[(1, 1353, 1), (2, 1304, 5), (2, 1325, 1)]);
    let stats = Run::default().stats();
    for (key, text) in [
        (
            "concussive_shot",
            "Also stuns for 1 sec (bosses: 0.30 sec).",
        ),
        (
            "serpent_sting",
            "Also stuns for 1.5 sec (bosses: 0.45 sec).",
        ),
        ("explosive_trap", "Also roots for 2 sec (bosses: 0.60 sec)."),
    ] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.,
        };
        assert!(
            ability_tip(&owned, trained, &stats, None, &Strings::default())
                .iter()
                .any(|line| line.left == text)
        );
        assert!(!ability_tip(
            &owned,
            trained.with_ranks(&[]),
            &stats,
            None,
            &Strings::default()
        )
        .iter()
        .any(|line| line.left == text));
    }
}

#[test]
fn critical_tooltip_matches_effective_chance_and_damage() {
    let mut stats = Run::default().stats();
    stats.crit = 0.9;
    let trained = Milestones::default().with_ranks(&[(0, 131, 2)]);
    let owned = Owned {
        def: data::ability("heroic_strike"),
        level: 1,
        cd: 0.0,
    };
    for (multiplier, expected) in [(2.0, "200%"), (2.2, "220%"), (0.5, "100%")] {
        stats.crit_damage = multiplier;
        let lines = ability_tip(&owned, trained, &stats, None, &Strings::default());
        assert!(lines
            .iter()
            .any(|line| line.left
                == format!("100% chance to critically strike for {expected} damage")));
    }
    assert_eq!(
        passive_text(Stat::Crit, 0.05),
        "Increases your critical strike chance by 5%."
    );
}

#[test]
fn shaman_deployment_tooltip_shows_healing_and_counterpart_cooldown() {
    let stats = Run::default().stats();
    let owned = super::super::Owned {
        def: data::ability("searing_totem"),
        level: 1,
        cd: 0.,
    };
    let trained = Milestones {
        class: 5,
        points: [0, 0, 30],
        ..Default::default()
    };
    let lines = ability_tip(&owned, trained, &stats, None, &Strings::default());
    assert!(lines.iter().any(|line| line
        .left
        .contains("Heals 3% maximum health once per successful cast.")));
    assert!(lines.iter().any(|line| line.left.contains("Advances")
        && line.left.contains("2 sec once per successful cast")
        && line.color == card_growth::GAIN));
}

#[test]
fn presence_of_mind_tooltip_explains_the_shared_cast_benefit() {
    let stats = Run::default().stats();
    let owned = super::super::Owned {
        def: data::ability("arcane_missiles"),
        level: 1,
        cd: 0.,
    };
    for points in [19, 20, 30] {
        let trained = Milestones {
            class: 6,
            points: [points, 0, 0],
            ..Default::default()
        };
        let lines = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let benefit = lines.iter().find(|line| {
            line.left.contains("Advances") && line.left.contains("2 sec once per successful cast")
        });
        assert_eq!(benefit.is_some(), points >= 20);
        if let Some(line) = benefit {
            assert_eq!(line.color, card_growth::GAIN);
        }
    }
}

#[test]
fn effect_description_uses_rank_and_talent_scaled_duration() {
    let stats = Run::default().stats();
    let def = data::ability("demoralizing_shout");
    let trained = Milestones::default().with_ranks(&[(1, 161, 5)]);
    for (rank, talents, expected) in [
        (1, Milestones::default(), "for 5 sec"),
        (1, trained, "for 7.5 sec"),
        (2, trained, "for 8.2 sec"),
    ] {
        let cast = cast_at(def, rank, &stats, talents);
        let body = ability_body(def, &cast);
        assert!(body.contains(expected), "{body}");
    }
}

#[test]
fn healing_tooltip_uses_combined_cast_effects_for_each_spell() {
    let stats = Run::default().stats();
    let trained = Milestones {
        class: 1,
        points: [20, 0, 0],
        ..Default::default()
    }
    .with_ranks(&[(0, 1444, 3), (0, 1443, 2)]);
    for (key, expected) in [("holy_shock", "6%"), ("divine_shield", "10%")] {
        let owned = Owned {
            def: data::ability(key),
            level: 1,
            cd: 0.0,
        };
        let lines = ability_tip(&owned, trained, &stats, None, &Strings::default());
        let healing: Vec<_> = lines
            .iter()
            .filter(|line| line.left.contains("Heals "))
            .collect();
        assert_eq!(healing.len(), 1);
        let expected = format!("Heals {expected} maximum health once per successful cast.");
        assert!(healing[0].left.contains(&expected));
        assert_eq!(healing[0].left.matches("Heals ").count(), 1);
        assert!(!ability_tip(
            &owned,
            Milestones::default(),
            &stats,
            None,
            &Strings::default()
        )
        .iter()
        .any(|line| line.left.contains("Heals ")));
    }
}
