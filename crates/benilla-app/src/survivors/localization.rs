//! Runtime string keys shared by Survivors UI, copied from the engine's boot VM.
use benilla_ui::{
    script::UiScript,
    strings::{self, Arg},
};
use bevy::prelude::*;
use std::collections::HashMap;

const KEYS: &[&str] = &[
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

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Strings>()
        .add_systems(PostStartup, load);
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
