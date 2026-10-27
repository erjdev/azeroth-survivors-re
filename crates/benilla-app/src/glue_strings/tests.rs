use super::*;

#[test]
fn parses_assignments_and_skips_the_rest() {
    let src = r#"
MALE = "Male";
FACTION_INFO_HORDE = "Four races\ncomprise the Horde";
QUOTED = "a \"b\" c";
-- a comment = "not a string";
CODE = getglobal("nope");
BROKEN = "no close
"#;
    let t = parse_glue_strings(src);
    assert_eq!(t.get("MALE").map(String::as_str), Some("Male"));
    assert_eq!(
        t.get("FACTION_INFO_HORDE").map(String::as_str),
        Some("Four races\ncomprise the Horde")
    );
    assert_eq!(t.get("QUOTED").map(String::as_str), Some(r#"a "b" c"#));
    assert!(!t.contains_key("CODE"));
    assert!(!t.contains_key("BROKEN"));
    assert_eq!(t.len(), 3);
}

#[test]
fn the_locale_patch_reads_localize_and_stops_at_its_end() {
    let src = r#"function Localize()
	-- Put all locale specific string adjustments here
	AUTH_REJECT = "Login unavailable - contact support";
	PVP_PARENTHESES = "PVP";
	--SHOW_CONTEST_AGREEMENT = 1;
end

function LocalizeFrames()
	NOT_A_STRING_TABLE = "must not leak";
	WorldOfWarcraftRating:SetTexture("Interface\\Glues\\Login\\Glues-FrenchRating");
end
"#;
    let t = parse_localize_overrides(src);
    assert_eq!(
        t.get("AUTH_REJECT").map(String::as_str),
        Some("Login unavailable - contact support")
    );
    assert_eq!(t.get("PVP_PARENTHESES").map(String::as_str), Some("PVP"));
    assert!(
        !t.contains_key("NOT_A_STRING_TABLE"),
        "LocalizeFrames leaked"
    );
    assert!(!t.contains_key("SHOW_CONTEST_AGREEMENT"), "comment parsed");
    assert_eq!(t.len(), 2);
}

#[test]
fn an_unrecognised_file_patches_nothing() {
    assert!(parse_localize_overrides("KEY = \"value\";\n").is_empty());
    assert!(parse_localize_overrides("function Localize()\n\tK = \"v\";\n").is_empty());
}

/// Compare the parser with Lua execution, so both empty enUS patches and enGB
/// overrides are checked against the installed scripts rather than one locale's text.
#[test]
fn the_real_chain_matches_locale_script_execution() {
    let data = benilla_formats::wow_data_or_skip!();
    let mut chain = benilla_formats::open_chain(&data).expect("open chain");
    let base = chain.read_file(GLUE_STRINGS).expect("GlueStrings.lua");
    let patch = chain
        .read_file(GLUE_LOCALIZATION)
        .expect("GlueLocalization.lua");
    let script = benilla_ui::script::UiScript::new().unwrap();
    script.run(&String::from_utf8_lossy(&base)).unwrap();
    script.run(&String::from_utf8_lossy(&patch)).unwrap();
    script.run("Localize()").unwrap();
    let expected = script
        .eval::<std::collections::HashMap<String, String>>(
            "local strings = {} for k, v in pairs(getfenv(0)) do \
         if type(k) == 'string' and type(v) == 'string' then strings[k] = v end end return strings",
        )
        .unwrap();
    let table = table_from_chain(&mut chain);
    let base = parse_glue_strings(&String::from_utf8_lossy(&base));
    assert!(base.len() > 100 && table.0.len() >= base.len());
    for key in base.keys().chain(table.0.keys()) {
        assert_eq!(
            table.get(key),
            expected.get(key).map(String::as_str),
            "{key}"
        );
    }
    assert!(table.get("FACTION_INFO_HORDE").is_some(), "base lost");
}
