use super::*;

#[test]
fn equipment_templates_preserve_signed_and_percentage_values() {
    let script = benilla_ui::script::UiScript::new().unwrap();
    script.lua().load("HP_TEMPLATE = '%s points de vie'; ARMOR_TEMPLATE = 'Armure : %s'; SINGLE_DAMAGE_TEMPLATE = '%s dégâts'").exec().unwrap();
    let strings = crate::survivors::localization::Strings::from_script(&script);
    let bonus = campaign::Bonus {
        health: 8.0,
        armor: 3.0,
        might: 0.195,
        ..Default::default()
    };
    assert_eq!(
        bonus_line(&strings, bonus),
        "+8 points de vie   Armure : +3   +19.5% dégâts"
    );
    assert_eq!(
        bonus_line(&crate::survivors::localization::Strings::default(), bonus),
        "+8 health   +3 armor   +19.5% damage"
    );
}
