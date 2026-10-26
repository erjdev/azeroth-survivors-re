//! Compare authored effects with the explicit install's selectable talent catalog.
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use survivors_core::{campaign, data, talents};

pub(super) fn report(chain: &mut benilla_formats::Chain, list_gaps: bool) -> Result<()> {
    let catalog = benilla_formats::load_talent_catalog(chain)?;
    let spells = benilla_formats::load_spell_catalog(chain)?;
    let mut invalid = 0;
    let mut total = 0;
    let mut supported = 0;
    let mut milestones = 0;
    println!("Definition coverage only: not proof of combat, balance, saves or visual acceptance.");
    println!("Class | Matching node definitions / selectable nodes | Milestone descriptions / 9");
    for (class_index, class) in data::CLASSES.iter().enumerate() {
        let tabs = (1..12)
            .map(|race| catalog.tabs_for_class(race, class.id))
            .find(|tabs| tabs.len() == 3)
            .context("class has no three-tree talent catalog")?;
        let mut seen = HashSet::new();
        let mut matched = 0;
        let mut pending = Vec::new();
        for (tree, tab) in tabs.iter().enumerate() {
            if tab.name != campaign::TREES[class_index][tree] {
                println!("MISMATCH: {} tree {tree}: {}", class.name, tab.name);
                invalid += 1;
            }
            for node in catalog.talents_in_tab(tab.id) {
                seen.insert(node.id);
                let name = spells
                    .get(node.ranks[0])
                    .map(|spell| spell.name.as_str())
                    .unwrap_or("MISSING SPELL");
                match talents::talent_definition(class_index, node.id) {
                    Some(effect)
                        if effect.name == name && u32::from(effect.max_rank) == node.max_rank() =>
                    {
                        matched += 1
                    }
                    Some(_) => {
                        println!("MISMATCH: {} node {} ({name})", class.name, node.id);
                        invalid += 1;
                    }
                    None => pending.push(format!("{} {name}", node.id)),
                }
            }
        }
        for effect in talents::definitions(class_index) {
            if !seen.contains(&effect.id) {
                println!("MISSING NODE: {} {} {}", class.name, effect.id, effect.name);
                invalid += 1;
            }
        }
        let described = (0..3)
            .flat_map(|tree| [10, 20, 30].map(move |at| (tree, at)))
            .filter(|&(tree, at)| talents::description(class_index, tree, at).is_some())
            .count();
        println!("{} | {matched}/{} | {described}/9", class.name, seen.len());
        if list_gaps && !pending.is_empty() {
            println!("  PENDING: {}", pending.join("; "));
        }
        total += seen.len();
        supported += matched;
        milestones += described;
    }
    println!(
        "TOTAL | {supported}/{total} | {milestones}/{}",
        data::CLASSES.len() * 9
    );
    if invalid > 0 {
        bail!("{invalid} authored talent definitions or tree identities disagree with the install");
    }
    Ok(())
}
