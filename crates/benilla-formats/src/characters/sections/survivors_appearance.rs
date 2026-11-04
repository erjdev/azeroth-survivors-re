//! Opt-in Survivors race-footwear composition; the ordinary compositor is unchanged.
use super::{CompositePlan, EQUIP_TILES};

impl CompositePlan {
    /// Keep the base skin at the foot tile for a race with ChrRaces flag 0x02.
    /// Lower-leg overlays and boot geometry remain selected by the worn equipment.
    pub fn preserve_bare_feet(mut self) -> Self {
        self.layers.retain(|(_, tile)| *tile != EQUIP_TILES[7]);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn barefoot_plan_preserves_base_and_every_nonfoot_layer_in_order() {
        let plan = CompositePlan {
            base: "skin".into(),
            layers: (0..8)
                .map(|i| (vec![format!("layer{i}")], EQUIP_TILES[i]))
                .collect(),
        };
        let original = plan.clone();
        let bare = plan.preserve_bare_feet();
        assert_eq!(bare.base, original.base);
        assert_eq!(bare.layers, original.layers[..7]);
        assert_eq!(bare.clone().preserve_bare_feet(), bare);
        assert_eq!(original.layers.len(), 8);
    }
}
