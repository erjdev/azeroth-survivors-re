//! Survivors' illustrated specialization headings, separate from the stock talent feed.
use std::collections::HashMap;

use anyhow::Result;
use benilla_dbc::{FieldType, Schema, SchemaField};

use crate::dbc::{load_spell_icon_map, parse, u32_at};
use crate::Chain;

/// Resolve TalentTab.dbc's SpellIconID (column 10) through SpellIcon.dbc.
pub fn load_survivors_talent_icons(chain: &mut Chain) -> Result<HashMap<u32, String>> {
    let bytes = chain.read_file("DBFilesClient\\TalentTab.dbc")?;
    let icons = load_spell_icon_map(chain)?;
    tab_icons(&bytes, &icons)
}

fn tab_icons(bytes: &[u8], icons: &HashMap<u32, String>) -> Result<HashMap<u32, String>> {
    let mut schema = Schema::new("TalentTab");
    // Only numeric IDs are needed; localized names and background string offsets stay opaque.
    for column in 0..15 {
        schema.add_field(SchemaField::new(format!("F{column}"), FieldType::UInt32));
    }
    let rows = parse(bytes, schema, "TalentTab.dbc")?;
    Ok(rows
        .records()
        .iter()
        .filter_map(|row| Some((u32_at(row, 0)?, icons.get(&u32_at(row, 10)?)?.clone())))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_icon_uses_icon_id_not_tab_id_or_name_offset() {
        let mut bytes = b"WDBC".to_vec();
        for value in [2u32, 15, 60, 1] {
            bytes.extend(value.to_le_bytes());
        }
        for (tab, icon) in [(81u32, 122u32), (61, 999)] {
            let mut row = [0u32; 15];
            row[0] = tab;
            row[1] = 88;
            row[10] = icon;
            for value in row {
                bytes.extend(value.to_le_bytes());
            }
        }
        bytes.push(0);
        let icons = HashMap::from([
            (122, "Interface\\Icons\\Arcane".into()),
            (81, "wrong".into()),
        ]);
        assert_eq!(
            tab_icons(&bytes, &icons).unwrap(),
            HashMap::from([(81, "Interface\\Icons\\Arcane".into())])
        );
        assert!(tab_icons(&bytes[..30], &icons).is_err());
    }
}
