//! Stable hero selection, separate from account ownership and equipment.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SavedPick {
    pub class: usize,
    /// Zero means no appearance has been chosen yet.
    pub race: u8,
    pub sex: u8,
    pub map: usize,
    pub hero: Option<u32>,
}

impl SavedPick {
    pub fn encode(self) -> String {
        format!(
            "{} {} {} {} {}",
            self.class,
            self.race,
            self.sex,
            self.map,
            self.hero.unwrap_or(0)
        )
    }

    /// Four-field saves predate persistent hero IDs. Reject malformed fields without shifting them.
    pub fn parse(text: &str) -> Option<Self> {
        let fields: Vec<_> = text.split_whitespace().collect();
        if !(4..=5).contains(&fields.len()) {
            return None;
        }
        let class = fields[0].parse().ok()?;
        let race = fields[1].parse().ok()?;
        let sex = fields[2].parse().ok()?;
        let map = fields[3].parse().ok()?;
        let hero = fields
            .get(4)
            .map(|value| value.parse::<u32>())
            .transpose()
            .ok()?
            .filter(|id| *id > 0);
        (class < super::data::CLASSES.len()
            && map < super::data::MAPS.len()
            && race <= 8
            && sex <= 1)
            .then_some(Self {
                class,
                race,
                sex,
                map,
                hero,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_picks_load_and_new_picks_keep_the_specific_hero() {
        let old = SavedPick::parse("0 1 0 0").unwrap();
        assert_eq!(old.hero, None);
        let selected = SavedPick {
            hero: Some(42),
            ..old
        };
        assert_eq!(SavedPick::parse(&selected.encode()), Some(selected));
        let without_look = SavedPick {
            race: 0,
            ..selected
        };
        assert_eq!(SavedPick::parse(&without_look.encode()), Some(without_look));
    }
    #[test]
    fn broken_fields_do_not_shift_or_wrap_into_valid_selections() {
        for text in [
            "0 nope 1 0 0",
            "0 257 0 0",
            "0 1 2 0",
            "99 1 0 0",
            "0 1 0 9999",
            "0 1 0 0 -1",
            "0 1 0 0 4294967296",
            "0 1 0",
            "0 1 0 0 1 2",
        ] {
            assert_eq!(SavedPick::parse(text), None, "{text}");
        }
    }
}
