//! Pure Survivors rules: test balance, progression and equipment without building the engine.

/// GameObjectDisplayInfo 10: the vanilla wooden chest.
pub const CHEST_DISPLAY: u32 = 10;

pub mod campaign;
pub mod channel;
pub mod controller;
pub mod creature;
pub mod data;
pub mod encounters;
pub mod equipment;
pub mod gear_loadout;
mod gear_stats;
pub mod gear_tier;
mod gear_upgrades;
pub mod gear_visuals;
pub mod layout;
pub mod marked_target;
pub mod racials;
pub mod raid_sets;
pub mod rewards;
pub mod scorch;
pub mod selection;
pub mod sheet;
pub mod silhouette;
pub mod stormstrike;
pub mod supplemental_gear;
pub mod talents;
pub mod tuning;
pub mod visibility;
pub mod weapon_alternatives;

pub mod battle_shout;

pub mod disarm;
pub mod enemy_equipment;
#[cfg(test)]
mod enemy_equipment_tests;
pub mod slows;
