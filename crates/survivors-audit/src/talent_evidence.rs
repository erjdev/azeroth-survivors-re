//! Read exact rank spell data before choosing a Survivors mechanic or conversion.
use anyhow::{bail, Context, Result};

pub(super) fn report(chain: &mut benilla_formats::Chain, id: u32) -> Result<()> {
    let catalog = benilla_formats::load_talent_catalog(chain)?;
    let spells = benilla_formats::load_spell_catalog(chain)?;
    let durations = benilla_formats::load_spell_durations(chain)?;
    for (class_index, class) in survivors_core::data::CLASSES.iter().enumerate() {
        let tabs = (1..12)
            .map(|race| catalog.tabs_for_class(race, class.id))
            .find(|tabs| tabs.len() == 3)
            .context("class has no three-tree talent catalog")?;
        for tab in tabs {
            let Some(node) = catalog
                .talents_in_tab(tab.id)
                .iter()
                .find(|node| node.id == id)
            else {
                continue;
            };
            println!("Install evidence only; not proof of implemented Survivors behavior.");
            println!("{} / {} / node {}", class.name, tab.name, id);
            println!(
                "row={} column={} prerequisite={} prerequisite_rank_zero_based={} required_spell={} exceptional={}",
                node.row, node.col, node.prereq_talent, node.prereq_rank,
                node.required_spell, node.exceptional
            );
            if let Some(effect) = survivors_core::talents::talent_definition(class_index, id) {
                println!("Survivors definition: {}", effect.description);
            } else {
                println!("Survivors definition: PENDING (generic provisional bonus only)");
            }
            for (rank, spell) in node
                .ranks
                .iter()
                .copied()
                .take(node.max_rank() as usize)
                .enumerate()
            {
                let row = spells.get(spell).context("talent rank spell missing")?;
                println!("Rank {}: spell {} {}", rank + 1, spell, row.name);
                println!("  description_template={:?}", row.description);
                println!(
                    "  duration={:?} proc_chance={} proc_flags={} charges={} stacks={}",
                    durations.get(row.duration_index),
                    row.proc_chance,
                    row.proc_flags,
                    row.proc_charges,
                    row.stack_amount
                );
                for slot in 0..3 {
                    if row.effects[slot] == 0 {
                        continue;
                    }
                    println!("  effect[{slot}]: type={} aura={} base_points={} die_sides={} misc={} trigger={} amplitude={}",
                        row.effects[slot], row.effect_apply_aura[slot], row.effect_base_points[slot],
                        row.effect_die_sides[slot], row.effect_misc_value[slot],
                        row.effect_trigger_spell[slot], row.effect_amplitude[slot]);
                }
            }
            println!("Effect fields are raw DBC values; interpret their units and targeting before implementation.");
            return Ok(());
        }
    }
    bail!("talent node {id} is not selectable by any Survivors class in this install")
}
