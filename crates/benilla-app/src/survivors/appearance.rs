//! Per-hero features use the stock creation ranges and player compositor.
use super::{account::Account, Selection};
use crate::glue_strings::GlueStrings;
use benilla_formats::{CharCreateCatalog, DialRanges};

pub(super) struct Options {
    pub values: [u8; 5],
    pub counts: [u8; 5],
    pub labels: [String; 5],
}

pub(super) fn counts(r: DialRanges) -> [u8; 5] {
    [r.skin, r.face, r.hair_style, r.hair_color, r.facial_hair]
}

pub(super) fn bounded(values: [u8; 5], counts: [u8; 5]) -> [u8; 5] {
    std::array::from_fn(|i| values[i].min(counts[i].saturating_sub(1)))
}

pub(super) fn options(
    selection: &Selection,
    account: &Account,
    creatures: &crate::entities::Creatures,
    catalog: &CharCreateCatalog,
    strings: Option<&GlueStrings>,
) -> Option<Options> {
    let hero = account.character(selection.character?)?;
    let &(preset, race, sex) = selection.looks.get(selection.look?)?;
    if selection.class != Some(hero.class) || hero.body != Some((race, sex)) {
        return None;
    }
    let counts = counts(catalog.ranges(race, sex)?);
    let look = creatures.survivors_appearance(preset)?;
    let values = bounded(
        hero.features.unwrap_or([
            look.skin,
            look.face,
            look.hair_style,
            look.hair_color,
            look.facial_hair,
        ]),
        counts,
    );
    let fallback = GlueStrings::default();
    Some(Options {
        values,
        counts,
        labels: labels(
            catalog.hair_customization(race).unwrap_or("NORMAL"),
            catalog
                .facial_hair_customization(race, sex)
                .unwrap_or("NORMAL"),
            strings.unwrap_or(&fallback),
        ),
    })
}

fn labels(hair: &str, facial: &str, strings: &GlueStrings) -> [String; 5] {
    let horns = hair == "HORNS";
    let facial_fallback = match facial {
        "BEARD" | "NORMAL" => "Facial hair",
        "PIERCINGS" => "Piercings",
        "EARRINGS" => "Earrings",
        "TUSKS" => "Tusks",
        "HORNS" => "Horns",
        _ => "Facial features",
    };
    [
        strings
            .text("CHAR_CUSTOMIZATION1_DESC", "Skin color")
            .to_owned(),
        strings.text("CHAR_CUSTOMIZATION2_DESC", "Face").to_owned(),
        strings
            .text(
                &format!("HAIR_{hair}_STYLE"),
                if horns { "Horn style" } else { "Hair style" },
            )
            .to_owned(),
        strings
            .text(
                &format!("HAIR_{hair}_COLOR"),
                if horns { "Horn color" } else { "Hair color" },
            )
            .to_owned(),
        strings
            .text(&format!("FACIAL_HAIR_{facial}"), facial_fallback)
            .to_owned(),
    ]
}

pub(super) fn step(options: &Options, dial: usize, direction: i8) -> Option<[u8; 5]> {
    let count = *options.counts.get(dial)?;
    if count < 2 || !matches!(direction, -1 | 1) {
        return None;
    }
    let mut values = options.values;
    values[dial] =
        (i16::from(values[dial]) + i16::from(direction)).rem_euclid(i16::from(count)) as u8;
    Some(values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appearance_labels_resolve_race_specific_keys_and_keep_missing_key_fallbacks() {
        let strings = GlueStrings::from_map(
            [
                ("CHAR_CUSTOMIZATION1_DESC".into(), "Peau".into()),
                ("CHAR_CUSTOMIZATION2_DESC".into(), "Visage".into()),
                ("HAIR_HORNS_STYLE".into(), "Cornes".into()),
                ("HAIR_HORNS_COLOR".into(), "Couleur des cornes".into()),
                ("FACIAL_HAIR_TUSKS".into(), "Défenses".into()),
            ]
            .into(),
        );
        assert_eq!(
            labels("HORNS", "TUSKS", &strings),
            ["Peau", "Visage", "Cornes", "Couleur des cornes", "Défenses"]
        );
        assert_eq!(
            labels("NORMAL", "BEARD", &strings),
            ["Peau", "Visage", "Hair style", "Hair color", "Facial hair"]
        );
        assert_eq!(
            labels("HORNS", "PIERCINGS", &GlueStrings::default()),
            [
                "Skin color",
                "Face",
                "Horn style",
                "Horn color",
                "Piercings"
            ]
        );
    }

    #[test]
    fn malformed_values_are_bounded_and_dials_wrap_both_ways() {
        let counts = [10, 4, 6, 8, 1];
        let options = Options {
            values: bounded([255; 5], counts),
            counts,
            labels: std::array::from_fn(|_| String::new()),
        };
        assert_eq!(options.values, [9, 3, 5, 7, 0]);
        assert_eq!(step(&options, 0, 1).unwrap()[0], 0);
        let options = Options {
            values: [0; 5],
            ..options
        };
        assert_eq!(step(&options, 0, -1).unwrap()[0], 9);
        assert!(step(&options, 4, 1).is_none());
        assert!(step(&options, 5, 1).is_none());
    }
}
