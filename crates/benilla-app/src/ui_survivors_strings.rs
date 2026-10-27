//! Runtime string keys shared by Survivors UI, copied from the engine's boot VM.
use benilla_ui::{
    script::UiScript,
    strings::{self, Arg},
};
use bevy::prelude::*;
use std::collections::HashMap;

const KEYS: &[&str] = &[
    "ALREADY_LEARNED",
    "HP_TEMPLATE",
    "ARMOR_TEMPLATE",
    "SINGLE_DAMAGE_TEMPLATE",
    "TOOLTIP_TALENT_RANK",
    "UNIT_LEVEL_TEMPLATE",
    "ENABLE_SOUND",
    "ENABLE_MUSIC",
    "BINDING_NAME_TURNLEFT",
    "BINDING_NAME_TURNRIGHT",
    "SPELL_DURATION_SEC",
    "SPELL_RECAST_TIME_SEC",
    "SPELL_RANGE",
    "TOOLTIP_TALENT_NEXT_RANK",
    "HONOR_HIGHEST_RANK",
];

#[derive(Resource, Default)]
pub(super) struct Strings(HashMap<String, String>);

impl Strings {
    pub(super) fn text<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.0
            .get(key)
            .filter(|s| !s.is_empty())
            .map_or(fallback, String::as_str)
    }

    pub(super) fn by_key(&self, key: &str, fallback: &str, args: &[Arg<'_>]) -> String {
        strings::fill(self.text(key, fallback), args)
    }

    pub(super) fn from_script(script: &UiScript) -> Self {
        Self(
            KEYS.iter()
                .filter_map(|&key| {
                    script
                        .lua()
                        .globals()
                        .get::<String>(key)
                        .ok()
                        .map(|value| (key.to_owned(), value))
                })
                .collect(),
        )
    }
}

/// A static label that may be created before the boot VM's strings are copied.
#[derive(Component)]
pub(super) struct Label {
    key: &'static str,
    fallback: &'static str,
}

impl Label {
    pub(super) fn by_key(key: &'static str, fallback: &'static str) -> Self {
        Self { key, fallback }
    }
}

fn refresh_labels(strings: Res<Strings>, mut labels: Query<(Ref<Label>, &mut Text)>) {
    for (label, mut text) in &mut labels {
        if strings.is_changed() || label.is_added() {
            let value = strings.text(label.key, label.fallback);
            if text.0 != value {
                text.0 = value.to_owned();
            }
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Strings>()
        .add_systems(PostStartup, load)
        .add_systems(Update, refresh_labels);
}

fn load(script: Option<NonSend<UiScript>>, mut strings: ResMut<Strings>) {
    if let Some(script) = script {
        *strings = Strings::from_script(&script);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_created_before_and_after_startup_resolve_and_refresh_with_fallbacks() {
        let script = UiScript::new().unwrap();
        script
            .lua()
            .load(r#"ENABLE_SOUND = "Activer le son""#)
            .exec()
            .unwrap();
        let mut app = App::new();
        app.insert_non_send_resource(script).add_plugins(plugin);
        let early = app
            .world_mut()
            .spawn((
                Text::new("Enable sound"),
                Label::by_key("ENABLE_SOUND", "Enable sound"),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Text>(early).unwrap().0, "Activer le son");
        let late = app
            .world_mut()
            .spawn((
                Text::new("Enable sound"),
                Label::by_key("ENABLE_SOUND", "Enable sound"),
            ))
            .id();
        app.update();
        assert_eq!(app.world().get::<Text>(late).unwrap().0, "Activer le son");
        app.world_mut().resource_mut::<Strings>().0.clear();
        app.update();
        for entity in [early, late] {
            assert_eq!(app.world().get::<Text>(entity).unwrap().0, "Enable sound");
        }
    }

    #[test]
    fn boot_vm_keys_keep_translated_order_and_fractional_gameplay_values() {
        let script = UiScript::new().unwrap();
        script
            .lua()
            .load(
                r#"
            SPELL_RECAST_TIME_SEC = "Recharge : %1$s secondes"
            SPELL_RANGE = "Portée : %s mètres"
            TOOLTIP_TALENT_NEXT_RANK = "Rang suivant :"
            HONOR_HIGHEST_RANK = ""
        "#,
            )
            .exec()
            .unwrap();
        let mut app = App::new();
        app.insert_non_send_resource(script).add_plugins(plugin);
        app.update();
        let strings = app.world().resource::<Strings>();
        assert_eq!(
            strings.by_key("SPELL_RECAST_TIME_SEC", "%s sec cooldown", &["1.25".into()]),
            "Recharge : 1.25 secondes"
        );
        assert_eq!(
            strings.text("TOOLTIP_TALENT_NEXT_RANK", "Next rank:"),
            "Rang suivant :"
        );
        assert_eq!(
            strings.text("HONOR_HIGHEST_RANK", "Highest rank"),
            "Highest rank"
        );
        assert_eq!(
            strings.by_key("SPELL_DURATION_SEC", "%s sec", &["0.5".into()]),
            "0.5 sec"
        );
    }
}
