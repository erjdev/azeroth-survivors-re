//! Per-hero features use the stock creation ranges and player compositor.
use super::{account::Account, Selection};
use benilla_formats::{CharCreateCatalog, DialRanges};

pub(super) struct Options {
    pub values: [u8; 5],
    pub counts: [u8; 5],
    pub labels: [&'static str; 5],
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
    let horns = catalog.hair_customization(race) == Some("HORNS");
    let facial = match catalog.facial_hair_customization(race, sex) {
        Some("BEARD") => "Facial hair",
        Some("PIERCINGS") => "Piercings",
        Some("EARRINGS") => "Earrings",
        Some("TUSKS") => "Tusks",
        Some("HORNS") => "Horns",
        _ => "Facial features",
    };
    Some(Options {
        values,
        counts,
        labels: [
            "Skin color",
            "Face",
            if horns { "Horn style" } else { "Hair style" },
            if horns { "Horn color" } else { "Hair color" },
            facial,
        ],
    })
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
    fn malformed_values_are_bounded_and_dials_wrap_both_ways() {
        let counts = [10, 4, 6, 8, 1];
        let options = Options {
            values: bounded([255; 5], counts),
            counts,
            labels: [""; 5],
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
