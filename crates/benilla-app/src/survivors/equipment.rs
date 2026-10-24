//! Class equipment and cosmetic choices are independent of collection ownership.

use super::campaign::{self, BANDS, GRID_SLOTS, PIECES};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Look {
    #[default]
    Worn,
    Hidden,
    Piece(u8),
}

#[derive(Clone, Debug, Default)]
pub(super) struct Equipment {
    pub worn: [Option<u8>; GRID_SLOTS],
    pub looks: [Look; GRID_SLOTS],
}

impl Equipment {
    pub fn legacy(pieces: &[u8]) -> Self {
        Self {
            worn: std::array::from_fn(|slot| {
                campaign::best_piece(pieces, slot).map(|(band, _)| band as u8)
            }),
            ..Self::default()
        }
    }

    pub fn owns(pieces: &[u8], slot: usize, band: u8) -> bool {
        campaign::piece_index(usize::from(band), slot)
            .and_then(|index| pieces.get(index))
            .is_some_and(|&stacks| stacks > 0)
    }

    pub fn wear(&mut self, pieces: &[u8], slot: usize, band: Option<u8>) -> bool {
        if slot >= GRID_SLOTS || band.is_some_and(|band| !Self::owns(pieces, slot, band)) {
            return false;
        }
        self.worn[slot] = band;
        true
    }

    pub fn show(&mut self, pieces: &[u8], slot: usize, look: Look) -> bool {
        if slot >= GRID_SLOTS
            || matches!(look, Look::Piece(band) if !Self::owns(pieces, slot, band))
        {
            return false;
        }
        self.looks[slot] = look;
        true
    }

    /// Only the selected piece in each slot can contribute to combat bonuses.
    pub fn worn_pieces(&self, pieces: &[u8]) -> [u8; PIECES] {
        let mut selected = [0; PIECES];
        for (slot, band) in self.worn.iter().enumerate() {
            if let Some(band) = band {
                if let Some(index) = campaign::piece_index(usize::from(*band), slot) {
                    selected[index] = pieces.get(index).copied().unwrap_or(0);
                }
            }
        }
        selected
    }

    pub fn encode(&self) -> (String, String) {
        let worn = self
            .worn
            .iter()
            .map(|band| char::from(b'0' + band.map_or(0, |b| b + 1)))
            .collect();
        let looks = self
            .looks
            .iter()
            .map(|look| {
                char::from(
                    b'0' + match look {
                        Look::Worn => 0,
                        Look::Hidden => 1,
                        Look::Piece(band) => band + 2,
                    },
                )
            })
            .collect();
        (worn, looks)
    }

    pub fn decode(worn: &str, looks: &str, pieces: &[u8]) -> Self {
        let mut result = Self::default();
        for (slot, code) in worn.bytes().take(GRID_SLOTS).enumerate() {
            if (b'1'..b'1' + BANDS as u8).contains(&code) {
                result.wear(pieces, slot, Some(code - b'1'));
            }
        }
        for (slot, code) in looks.bytes().take(GRID_SLOTS).enumerate() {
            let look = match code {
                b'1' => Look::Hidden,
                b'2'..=b'7' => Look::Piece(code - b'2'),
                _ => Look::Worn,
            };
            result.show(pieces, slot, look);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_weaker_gear_and_appearance_are_independent() {
        let mut pieces = [0; PIECES];
        pieces[0] = 1;
        pieces[GRID_SLOTS] = 2;
        let mut gear = Equipment::legacy(&pieces);
        let strong = campaign::equipped_bonus(&gear.worn_pieces(&pieces), None).might;
        assert!(gear.wear(&pieces, 0, Some(0)));
        let weak = campaign::equipped_bonus(&gear.worn_pieces(&pieces), None).might;
        assert!(weak < strong);
        assert!(gear.show(&pieces, 0, Look::Piece(1)));
        assert_eq!(
            campaign::equipped_bonus(&gear.worn_pieces(&pieces), None).might,
            weak
        );
        assert!(gear.show(&pieces, 0, Look::Hidden));
        let (worn, looks) = gear.encode();
        let loaded = Equipment::decode(&worn, &looks, &pieces);
        assert_eq!(loaded.worn, gear.worn);
        assert_eq!(loaded.looks, gear.looks);
        assert!(gear.wear(&pieces, 0, None));
        assert_eq!(
            campaign::equipped_bonus(&gear.worn_pieces(&pieces), None).might,
            0.0
        );
    }

    #[test]
    fn unowned_and_invalid_choices_never_add_stats() {
        let pieces = [0; PIECES];
        let mut gear = Equipment::default();
        assert!(!gear.wear(&pieces, 0, Some(0)));
        assert!(!gear.wear(&pieces, GRID_SLOTS, None));
        assert!(!gear.wear(&pieces, 0, Some(u8::MAX)));
        assert!(!gear.show(&pieces, 0, Look::Piece(0)));
        let corrupt = Equipment::decode("999999999", "777777777", &pieces);
        assert_eq!(corrupt.worn_pieces(&pieces), pieces);
        assert_eq!(corrupt.looks, [Look::Worn; GRID_SLOTS]);
    }

    #[test]
    fn unequipping_breaks_the_worn_set_without_losing_collection() {
        let mut pieces = [0; PIECES];
        pieces[..GRID_SLOTS].fill(1);
        let mut gear = Equipment::legacy(&pieces);
        assert!(campaign::set_complete(&gear.worn_pieces(&pieces)));
        gear.wear(&pieces, 8, None);
        assert!(!campaign::set_complete(&gear.worn_pieces(&pieces)));
        assert!(campaign::set_complete(&pieces));
    }
}
