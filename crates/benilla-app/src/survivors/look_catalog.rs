//! Class presets define allowed races, not which sexes a player may choose.
use super::data::{ClassDef, CLASSES};

pub(super) fn resolve(
    class: &ClassDef,
    race_sex: impl Fn(u32) -> Option<(u8, u8)>,
) -> Vec<(u32, u8, u8)> {
    let mut looks = Vec::new();
    for &display in class.looks {
        if let Some((race, sex)) = race_sex(display) {
            if (1..=8).contains(&race)
                && sex <= 1
                && !looks.iter().any(|&(_, r, s)| (r, s) == (race, sex))
            {
                looks.push((display, race, sex));
            }
        }
    }
    let races: Vec<_> = looks.iter().map(|&(_, race, _)| race).collect();
    // Equipment and combat class are applied separately. A missing body may use a
    // trainer of another class, without borrowing their gear or granting their class.
    for &display in CLASSES.iter().flat_map(|c| c.looks) {
        if let Some((race, sex)) = race_sex(display) {
            if races.contains(&race)
                && sex <= 1
                && !looks.iter().any(|&(_, r, s)| (r, s) == (race, sex))
            {
                looks.push((display, race, sex));
            }
        }
    }
    looks.sort_by_key(|&(_, race, sex)| (race, sex));
    looks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_class_race_has_both_verified_bodies_without_replacing_existing_presets() {
        let data = benilla_formats::wow_data_or_skip!();
        let mut chain = benilla_formats::Chain::open(&data).unwrap();
        let catalog = benilla_formats::load_creature_catalog(&mut chain).unwrap();
        for class in CLASSES {
            let looks = resolve(class, |d| catalog.display_race_sex(d));
            assert!(!looks.is_empty(), "{}", class.name);
            let mut original = Vec::new();
            for &display in class.looks {
                let Some((race, sex)) = catalog.display_race_sex(display) else {
                    continue;
                };
                if !original.iter().any(|&(_, r, s)| (r, s) == (race, sex)) {
                    original.push((display, race, sex));
                }
            }
            for &(display, race, sex) in &original {
                assert!(
                    looks.contains(&(display, race, sex)),
                    "{} lost preferred preset",
                    class.name
                );
                for wanted in 0..=1 {
                    assert!(
                        looks.iter().any(|&(_, r, s)| r == race && s == wanted),
                        "{} missing race {race} sex {wanted}",
                        class.name
                    );
                }
            }
            for &(display, race, sex) in &looks {
                assert!(original.iter().any(|&(_, r, _)| r == race));
                let model = catalog.model(display).expect("verified model");
                let appearance = model.npc_appearance.expect("player appearance fields");
                assert_eq!((appearance.race, appearance.sex), (race, sex));
                let url = benilla_assets::m2_url(&model.model_path);
                assert!(!chain
                    .read(url.strip_prefix("mpq://").unwrap())
                    .expect("readable model")
                    .is_empty());
            }
        }
    }
}
