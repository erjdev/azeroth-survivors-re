//! Pure Survivors rules: test balance, progression and equipment without building the engine.

/// GameObjectDisplayInfo 10: the vanilla wooden chest.
pub const CHEST_DISPLAY: u32 = 10;

pub mod campaign;
pub mod data;
pub mod encounters;
pub mod equipment;
mod gear_stats;
mod gear_upgrades;
pub mod gear_visuals;
pub mod layout;
pub mod rewards;
pub mod selection;
pub mod sheet;
pub mod talents;
pub mod tuning;
pub mod visibility;
