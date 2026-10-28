//! Base spells must be inspected separately from talent modifiers and teaching rows.
use anyhow::{Context, Result};

pub(super) fn report(chain: &mut benilla_formats::Chain, id: u32) -> Result<()> {
    let spells = benilla_formats::load_spell_catalog(chain)?;
    let durations = benilla_formats::load_spell_durations(chain)?;
    let ranges = benilla_formats::load_spell_ranges(chain)?;
    let row = spells
        .get(id)
        .context("spell ID is absent from this install")?;
    println!("Install evidence only; not proof of implemented Survivors behavior.");
    println!("Spell {}: {} {:?}", row.id, row.name, row.rank);
    println!("description_template={:?}", row.description);
    println!(
        "levels: base={} spell={} maximum={}",
        row.base_level, row.spell_level, row.max_level
    );
    println!("duration={:?}", durations.get(row.duration_index));
    if let Some(range) = ranges.get(row.range_index) {
        println!(
            "range: minimum={} maximum={} flags={}",
            range.min, range.max, range.flags
        );
    }
    println!(
        "recovery_ms={} category={} category_recovery_ms={} start_recovery_ms={}",
        row.recovery_ms, row.category, row.category_recovery_ms, row.start_recovery_ms
    );
    println!(
        "family={} family_flags={:#x} school={} ranged_attack={} speed={}",
        row.spell_family,
        row.spell_family_flags,
        row.school,
        row.ranged_attack(),
        row.speed
    );
    println!(
        "targets={:#x} maximum_targets={} caster_aura_state={} target_aura_state={}",
        row.targets, row.max_affected_targets, row.caster_aura_state, row.target_aura_state
    );
    for slot in 0..3 {
        if row.effects[slot] == 0 {
            continue;
        }
        println!("effect[{slot}]: type={} aura={} base_points={} die_sides={} points_per_level={} misc={} trigger={} amplitude={} target_a={} target_b={} chain_targets={}",
            row.effects[slot], row.effect_apply_aura[slot], row.effect_base_points[slot],
            row.effect_die_sides[slot], row.effect_real_points_per_level[slot], row.effect_misc_value[slot],
            row.effect_trigger_spell[slot], row.effect_amplitude[slot], row.effect_implicit_target_a[slot],
            row.effect_implicit_target_b[slot], row.effect_chain_targets[slot]);
    }
    println!("Fields are raw DBC values. Server-side formulas and Survivors scaling require separate evidence or explicit authored rules.");
    Ok(())
}
