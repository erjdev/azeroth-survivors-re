//! Installed race appearance flags used by the Survivors compositor.
use crate::{
    dbc::{parse, u32_at},
    Chain,
};
use anyhow::{Context, Result};
use benilla_dbc::{FieldType, Schema, SchemaField};
use std::collections::HashSet;

/// ChrRaces flag 0x02 means "Do Not Component Feet". The installed 1.12.1
/// table sets it for Tauren and Troll; load the flag rather than hard-code races.
/// Field meaning: https://trinitycore.info/files/DBC/335/chrraces
pub fn bare_foot_races(chain: &mut Chain) -> Result<HashSet<u8>> {
    let bytes = chain
        .read_file("DBFilesClient\\ChrRaces.dbc")
        .context("reading race appearance flags")?;
    let mut schema = Schema::new("ChrRaces");
    for i in 0..29 {
        schema.add_field(SchemaField::new(format!("f{i}"), FieldType::UInt32));
    }
    let rows = parse(&bytes, schema, "ChrRaces")?;
    Ok(rows
        .records()
        .iter()
        .filter_map(|r| {
            let race = u8::try_from(u32_at(r, 0)?).ok()?;
            (u32_at(r, 1)? & 2 != 0).then_some(race)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_vanilla_flags_mark_only_tauren_and_troll() {
        let data = crate::wow_data_or_skip!();
        let mut chain = crate::open_chain(&data).expect("open chain");
        assert_eq!(bare_foot_races(&mut chain).unwrap(), HashSet::from([6, 8]));
    }
}
