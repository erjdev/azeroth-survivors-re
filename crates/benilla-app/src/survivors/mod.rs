//! Warcraft Survivors: a top-down survival mode on the benilla world, started by
//! [`crate::run_survivors`]. There is no server: the hero, the enemies and the spells are local
//! entities dressed by the same attach, animation and spell-visual systems a streamed unit gets,
//! on a battleground picked at the menu ([`data::MAPS`]), with the player controller and FrameXML
//! off ([`crate::run_mode::CaptureMode`]).

mod abilities;
mod account;
#[cfg(feature = "survivors-visibility")]
mod actor_mask;
mod appearance;
mod armory;
mod auto;
mod backdrop;
mod camera;
mod cards;
mod channel_visuals;
mod channeling;
mod companions;
mod controller_input;
mod curse;
mod defense;
#[cfg(not(target_arch = "wasm32"))]
mod demon_preview;
mod disarm;
mod earthbind;
mod enemies;
mod enemy_casting;
#[cfg(not(target_arch = "wasm32"))]
mod equipment_preview;
mod floor;
mod gear_body;
mod gear_icons;
mod gear_template;
mod grade;
mod hero;
mod hit_timing;
mod hud;
mod hunter_pet;
mod jewel_icons;
#[path = "../ui_survivors_strings.rs"]
mod localization;
mod look_catalog;
mod marking;
mod menu;
mod minimap;
mod minimap_indoor;
mod mobility;
mod navigation;
mod notes;
mod perception;
mod renown;
mod spawn;
mod stash;
mod stoneclaw;
mod talent_validation;
#[cfg(feature = "survivors-text")]
pub(crate) mod text_placement;
mod tuning;
mod units;
#[cfg(feature = "survivors-visibility")]
mod visibility;
mod zone_art;

pub(crate) use floor::Floor;
pub(crate) use hero::HERO_RADIUS;
pub(crate) use hud::SweepUiMaterial;
pub(crate) use tuning::UiLayout;

pub(crate) use survivors_core::{campaign, data, encounters, equipment, rewards, sheet};

use benilla_world::schedule::WorldStage;
use bevy::prelude::*;

use data::{AbilityDef, ClassDef, PassiveDef, Stat};

/// Set by [`crate::run_survivors`] before the app builds. wasm32 has no process environment, so
/// the flag lives here; `WOW_SURVIVORS` (`?survivors=1` / `__wenilla_env.survivors` on the web)
/// still turns it on from outside.
pub(crate) static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn active() -> bool {
    ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
        || crate::webenv::var("WOW_SURVIVORS").is_some()
}

pub(crate) struct SurvivorsPlugin;

impl Plugin for SurvivorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<controller_input::ControllerInput>()
            .add_systems(
                PreUpdate,
                controller_input::sample.after(bevy::input::InputSystems),
            );
        // Stock 1.12 boot disables controllers; Survivors opts back into native
        // device discovery and Bevy's normalized gamepad buttons.
        if !app.is_plugin_added::<bevy::gilrs::GilrsPlugin>() {
            app.add_plugins(bevy::gilrs::GilrsPlugin);
        }
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(armory::preview::plugin);
        #[cfg(feature = "survivors-visibility")]
        app.add_plugins((visibility::plugin, actor_mask::plugin));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins((equipment_preview::plugin, demon_preview::plugin));
        // Present before `OnEnter(InWorld)`, where the UI lifecycle decides not to load FrameXML.
        app.insert_resource(crate::run_mode::CaptureMode)
            .add_plugins((
                backdrop::plugin,
                localization::plugin,
                grade::plugin,
                tuning::plugin,
                crate::entities::survivors_look::plugin,
            ))
            .init_state::<Phase>()
            .init_resource::<Rng>()
            .init_resource::<Selection>()
            .init_resource::<Run>()
            .init_resource::<armory::controller::ControllerFocus>()
            .init_resource::<hero::CamRig>()
            .init_resource::<camera::CameraSettings>()
            .init_resource::<hero::AimPoint>()
            .init_resource::<abilities::PendingHits>()
            .init_resource::<channeling::Channels>()
            .init_resource::<enemies::Spawner>()
            .init_resource::<hud::Offer>()
            .init_resource::<renown::Renown>()
            .init_resource::<stash::Stash>()
            .init_resource::<account::Account>()
            .add_systems(PreUpdate, talent_validation::load_rules)
            .add_systems(
                Update,
                channel_visuals::sync
                    .after(abilities::cast_abilities)
                    .after(channeling::tick),
            )
            .init_resource::<armory::Armory>()
            .init_resource::<armory::TalentScrollMemory>()
            .add_systems(
                PreUpdate,
                (
                    armory::remember_talent_scroll,
                    armory::remember_reward_scroll,
                    armory::remember_achievement_scroll,
                    armory::remember_collection_scroll,
                )
                    .run_if(in_state(Phase::Menu)),
            )
            .add_plugins(crate::portrait::survivors_pane::SurvivorsPanePlugin)
            .add_systems(Update, armory::character_subject.after(hud::menu_preview))
            .add_systems(
                Update,
                (
                    armory::layout_achievements,
                    armory::layout_achievement_grids,
                    armory::layout_companions,
                    armory::scrollbar_input,
                    armory::update_scrollbars,
                )
                    .chain()
                    .run_if(in_state(Phase::Menu)),
            )
            .add_systems(PreUpdate, gear_body::sync)
            .add_systems(PreUpdate, disarm::sync)
            .init_resource::<Typing>()
            .init_resource::<hud::PausePage>()
            .init_resource::<hud::SettingEdits>()
            .init_resource::<cards::PendingPick>()
            .init_resource::<cards::PreviewHand>()
            .add_message::<abilities::Hit>()
            .add_message::<enemies::Loot>()
            .add_plugins(bevy::ui_render::UiMaterialPlugin::<hud::SweepUiMaterial>::default())
            .add_systems(Startup, hud::setup_hud)
            .add_systems(Update, hero::daylight)
            .add_plugins(perception::plugin)
            .add_systems(Update, hunter_pet::update_status.after(hunter_pet::fight))
            .add_systems(
                PostUpdate,
                zone_art::crop.after(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                Update,
                (
                    hero::aim_cursor,
                    mobility::cast_mobility,
                    hero::drive_hero,
                    hero::place_camera,
                )
                    .chain()
                    .in_set(WorldStage::Input),
            )
            .add_systems(
                Update,
                (
                    enemies::spawn_enemies,
                    channeling::tick,
                    enemies::move_enemies,
                    marking::prune,
                    abilities::cast_abilities,
                    earthbind::tick,
                    stoneclaw::tick,
                    abilities::tick_areas,
                    hunter_pet::fight,
                    abilities::apply_hits,
                    enemies::tick_marks,
                    enemies::drop_loot,
                    companions::herd,
                    enemies::collect_gems,
                    enemies::collect_chests,
                    hero::hero_vitals,
                )
                    .chain()
                    .after(WorldStage::Input)
                    .run_if(in_state(Phase::Playing))
                    // Entering a battleground raises its loading screen: the run waits behind it.
                    .run_if(|l: Res<crate::loading_screen::LoadingScreen>| !l.covering()),
            )
            .add_systems(
                Update,
                (enemies::reap_dead, enemies::still_gems).after(WorldStage::Input),
            )
            .add_systems(
                Update,
                (
                    hero::follow_map,
                    hud::menu_preview,
                    hud::buttons.run_if(not(cards::preview_active)),
                    (
                        armory::controller::navigate,
                        armory::input.run_if(not(cards::preview_active)),
                    )
                        .chain(),
                    hud::keys.run_if(not(cards::preview_active)),
                    hud::refresh_menu,
                    menu::restore_pick,
                    menu::build_menu,
                    (
                        armory::refresh,
                        armory::style_headings,
                        armory::style_controls,
                        armory::scroll_collection,
                        armory::codex_details,
                        armory::starter_equipment_details,
                        armory::talent_details,
                        armory::keep_talent_inspection_visible,
                        armory::keep_reward_actions_visible,
                        armory::fill_talent_art,
                        armory::fill_talent_background,
                        armory::fill_talent_headings,
                    )
                        .chain(),
                    (
                        menu::menu_clicks.run_if(not(cards::preview_active)),
                        menu::sync_hero_look,
                    )
                        .chain(),
                    menu::refresh_menu_ui.before(crate::glue::GlueVisuals),
                    cards::deal_cards,
                    cards::choose_card,
                    cards::animate_cards,
                    hud::refresh_hud,
                    hud::refresh_results,
                    (hud::fill_icons, gear_icons::fill, zone_art::fill).chain(),
                    hud::update_cooldown_sweeps,
                    hud::refresh_mobility,
                    hud::show_panels,
                )
                    .chain()
                    .after(WorldStage::Input)
                    .after(enemies::collect_chests),
            )
            .add_systems(
                Update,
                (
                    hud::refresh_meter,
                    hud::sync_pause_page,
                    hud::refresh_settings,
                    hud::refresh_settings_sliders,
                    stash::flush,
                )
                    .chain()
                    .after(hud::show_panels),
            )
            .add_systems(
                Update,
                (
                    hud::apply_ui_scale.after(hud::apply_settings),
                    cards::scroll_hand,
                    hud::fill_art,
                    hud::xp_hover,
                    hud::icon_tooltip,
                    hud::hero_plate,
                    minimap::map_buttons,
                    minimap::zone_text,
                    minimap::compose,
                )
                    .chain()
                    .after(hud::show_panels),
            )
            .add_systems(OnEnter(Phase::Playing), unpause)
            .add_systems(OnEnter(Phase::LevelUp), (pause, hud::roll_offer))
            .add_systems(OnEnter(Phase::Paused), (pause, hud::close_settings))
            .add_systems(
                OnEnter(Phase::GameOver),
                (unpause, renown::grant, account::settle, stash::flush_now).chain(),
            )
            .add_systems(
                OnEnter(Phase::Menu),
                (
                    unpause,
                    account::flush_discoveries,
                    reset_run,
                    stash::flush_now,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    hud::settings_keyboard.after(hud::keys),
                    hud::settings_slider_input,
                    hud::apply_settings.after(crate::cvars::sync_cvars),
                )
                    .chain(),
            );
        // After the saved config, so a default install gets the browser budget and a value the
        // player already chose (or passed on the URL) is left alone.
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Startup, browser_budget.after(crate::cvars::CvarLoad));
        if let Some(auto) = auto::from_env() {
            app.insert_resource(auto)
                .add_systems(
                    Update,
                    auto::drive
                        .after(hud::keys)
                        .before(hud::refresh_menu)
                        .run_if(not(cards::preview_active)),
                )
                .add_systems(Update, auto::plan.before(WorldStage::Input));
        }
    }
}

/// The browser runs the whole client on the tab thread, and the stock video defaults are what
/// make Chrome itself hitch: a full-DPR world, 350 yards of terrain, medium grass, and full
/// spell shards. Applied only while those knobs are still at their defaults, so a saved choice
/// or a URL override (`?renderscale=`, `?farclip=`, `?worlddetail=`, `?spelleffectlevel=`,
/// and `?render_scale=` for the render scale) is left alone.
#[cfg(target_arch = "wasm32")]
fn browser_budget(mut cvars: ResMut<crate::cvars::Cvars>) {
    let pinned = |name: &str| {
        crate::webenv::var(&format!("WOW_{name}")).is_some()
            || (name.eq_ignore_ascii_case("renderScale")
                && crate::webenv::var("WOW_RENDER_SCALE").is_some())
    };
    let ease = |cvars: &mut crate::cvars::Cvars, name: &str, default: &str, budget: &str| {
        if !pinned(name) && cvars.get(name) == Some(default) {
            cvars.set(name, budget);
        }
    };
    ease(&mut cvars, "renderScale", "1", "0.6");
    ease(&mut cvars, "farclip", "350", "177");
    ease(&mut cvars, "WorldDetail", "1", "0");
    ease(&mut cvars, "spellEffectLevel", "2", "1");
}

/// The screen the mode is on.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum Phase {
    #[default]
    Menu,
    Playing,
    LevelUp,
    Paused,
    GameOver,
}

/// Everything spawned for a run, despawned when the next begins.
#[derive(Component)]
pub(super) struct RunEntity;

/// True while a campaign name field has the keyboard, so Enter does not start the run.
#[derive(Resource, Default)]
pub(super) struct Typing(pub bool);

/// The class, look and battleground picked at the menu; the battleground holds for the run.
#[derive(Resource, Default, Clone)]
pub(super) struct Selection {
    pub class: Option<usize>,
    /// The named character whose talents, gear and zone gates this run uses.
    pub character: Option<u32>,
    /// `(display, race, sex)`, one per race and sex the class's trainers come in.
    pub looks: Vec<(u32, u8, u8)>,
    pub look: Option<usize>,
    /// An index into [`data::MAPS`].
    pub map: usize,
    /// The battleground the world is standing in: the last one entered (or the saved pick at
    /// boot). Picking another at the menu changes only `map`, so the menu hero stays put and no
    /// loading screen rises until Enter World.
    pub stage: usize,
    /// Bumped on every change, so the menu rebuilds once.
    pub version: u32,
}

impl Selection {
    pub fn class_def(&self) -> Option<&'static ClassDef> {
        self.class.map(|i| &data::CLASSES[i])
    }

    pub fn map_def(&self) -> &'static data::MapDef {
        &data::MAPS[self.map.min(data::MAPS.len() - 1)]
    }

    pub fn stage_def(&self) -> &'static data::MapDef {
        &data::MAPS[self.stage.min(data::MAPS.len() - 1)]
    }
}

/// An ability the hero has, with its own cooldown.
pub(super) struct Owned {
    pub def: &'static AbilityDef,
    pub level: u8,
    pub cd: f32,
}

impl Owned {
    pub fn damage_mult(&self) -> f32 {
        1.0 + 0.3 * f32::from(self.level - 1)
    }
    pub fn cooldown(&self, stats: &Stats) -> f32 {
        let base = self.def.cooldown * 0.93f32.powi(i32::from(self.level) - 1);
        sheet::cooldown_time(base, stats.haste, stats.cooldown)
    }
    pub fn count(&self, stats: &Stats) -> u32 {
        let extra = [3u8, 5, 7].iter().filter(|&&l| self.level >= l).count() as u32;
        self.def.count + extra + sheet::extra_targets(self.def.kind, self.def.count, stats.targets)
    }
    pub fn radius(&self, stats: &Stats) -> f32 {
        self.def.radius * (1.0 + 0.08 * f32::from(self.level - 1)) * stats.area
    }
    pub fn duration(&self) -> f32 {
        self.def.duration * (1.0 + 0.1 * f32::from(self.level - 1))
    }
}

/// The hero's derived numbers, from the base and the passives.
#[derive(Clone, Copy)]
pub(super) struct Stats {
    legendary: Option<(&'static str, campaign::Bonus)>,
    vengeance: f32,
    battle_shout: f32,
    pub might: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub regen: f32,
    pub haste: f32,
    pub area: f32,
    pub magnet: f32,
    pub growth: f32,
    pub armor: f32,
    pub crit: f32,
    /// Fraction of damage dealt returned as health.
    pub leech: f32,
    /// Percent of damage ignored after armor, capped when it is applied.
    pub reduction: f32,
    /// Crit multiplier. 2 is a double.
    pub crit_damage: f32,
    /// Cooldown reduction, separate from haste.
    pub cooldown: f32,
    /// Gold multiplier. 1 leaves a drop unchanged.
    pub gold_find: f32,
    /// Luck rating. One point is one percent.
    pub luck: f32,
    /// Echo rating. 1 is 100% multistrike.
    pub multistrike: f32,
    /// Extra targets for a spell that starts as single-target.
    pub targets: u32,
}

impl Stats {
    fn for_spell(&self, key: &str) -> Self {
        let mut scoped = *self;
        scoped.legendary = None;
        scoped.vengeance = 0.0;
        scoped.battle_shout = 0.0;
        if let Some((spell, bonus)) = self.legendary {
            if spell == key {
                apply_sheet(&mut scoped, bonus);
            }
        }
        if key == "Weapon Attack" || survivors_core::talents::PALADIN_HOLY.contains(&key) {
            scoped.might *= 1.0 + self.vengeance;
        }
        if survivors_core::battle_shout::BattleShout::affects(key) {
            scoped.might *= 1.0 + self.battle_shout;
        }
        scoped
    }
}

fn apply_sheet(s: &mut Stats, b: campaign::Bonus) {
    s.max_hp += b.health;
    s.magnet *= 1.0 + b.pickup;
    s.armor += b.armor;
    s.might += b.might;
    s.crit += b.crit;
    s.multistrike += b.multistrike;
    s.speed *= 1.0 + b.speed;
    s.leech += b.leech;
    s.area += b.area;
    s.crit_damage += b.crit_damage;
    s.reduction += b.reduction;
    s.cooldown += b.cooldown;
    s.gold_find += b.gold;
    s.growth += b.growth;
    s.luck += b.luck;
    s.targets += b.targets;
}

pub(super) const BASE_HP: f32 = 120.0;
pub(super) const BASE_SPEED: f32 = 7.0;
pub(super) const BASE_MAGNET: f32 = 4.5;

/// One run's state.
#[derive(Resource)]
pub(super) struct Run {
    pub(crate) tuning: survivors_core::tuning::Tuning,
    pub class: &'static ClassDef,
    pub race: u8,
    pub elapsed: f32,
    pub kills: u32,
    pub level: u32,
    pub xp: f32,
    pub hp: f32,
    pub abilities: Vec<Owned>,
    pub passives: Vec<(&'static PassiveDef, u8)>,
    pub shield: f32,
    pub shield_t: f32,
    pub immune_t: f32,
    pub incoming_slows: survivors_core::slows::Slows,
    pub enrage_t: f32,
    pub battle_shout: survivors_core::battle_shout::BattleShout,
    pub ancestral_healing_t: f32,
    pub inspiration_t: f32,
    pub flurry_t: f32,
    pub vengeance_t: f32,
    pub remorseless_until: f32,
    pub blood_craze_t: f32,
    pub blessed_recovery: survivors_core::talents::BlessedRecovery,
    pub spirit_tap_t: f32,
    pub thorns: f32,
    pub thorns_t: f32,
    pub pet_intimidation_ready_at: f32,
    pub counterattack_ready_at: f32,
    /// Independent of the six drafted ability slots. Active simulation seconds.
    pub weapon_attack_cd: f32,
    pub thorns_spell: u32,
    pub healing: survivors_core::healing::HealingOverTime,
    /// The class mobility's cooldown and running effect.
    mobile: mobility::MobilityState,
    pub pending_levels: u32,
    pub bosses: u32,
    /// Bosses killed, separately from the number already spawned.
    pub bosses_defeated: u32,
    pub rings: u32,
    pub won: bool,
    /// Chest trades waiting behind the level-up draft.
    pub trades: u32,
    /// The cards on screen are a chest trade.
    pub trading: bool,
    /// The three area-start chests have been placed.
    pub stocked: bool,
    /// Special elite loot is paced independently of how many elites a wave contains.
    pub elite_chests: u8,
    pub last_elite_chest: f32,
    cold_snap: survivors_core::talents::ColdSnapClock,
    preparation: survivors_core::talents::PreparationClock,
    critical_cast: survivors_core::talents::CriticalCastClock,
    ghostly_strike: survivors_core::ghostly_strike::GhostlyStrike,
    natures_grace: survivors_core::talents::NaturesGrace,
    healing_way: survivors_core::talents::HealingWay,
    power_infusion: survivors_core::talents::PowerInfusionClock,
    arcane_power: survivors_core::talents::ArcanePowerClock,
    combustion: survivors_core::talents::CombustionClock,
    hawk: survivors_core::talents::HawkClock,
    adrenaline_rush: survivors_core::talents::AdrenalineRushClock,
    blade_flurry: survivors_core::talents::BladeFlurryClock,
    /// Rings, trinkets and amulets worn this run.
    worn: campaign::Worn,
    /// Seconds since the hero fell, while the death plays out.
    pub dead_t: Option<f32>,
    pub next_guid: u64,
    pub damage_done: f64,
    /// Damage per ability key, for the meter. "Thorns" is the contact reflect.
    pub damage_sources: Vec<(&'static str, f64)>,
    /// Gold picked up this run. The purse itself lives on [`stash::Stash`].
    pub gold_found: u32,
    /// Relics worn at the start: catalog index and rank.
    pub gear: [Option<(u8, u8)>; 3],
    /// Ranks bought with renown, copied on at the start of the run.
    pub renown_might: u8,
    pub renown_fortitude: u8,
    pub renown_celerity: u8,
    /// The end-of-run grant has already been paid.
    pub renown_paid: bool,
    /// Account rewards have been applied; a save retry must not pay them again.
    pub account_paid: bool,
    /// Permanent account bonuses copied on at the start of the run.
    bonus: campaign::Bonus,
    milestones: survivors_core::talents::Milestones,
    /// Bumped whenever abilities or passives change, so the HUD rebuilds the icon rows once.
    pub version: u32,
    /// A centre-screen announcement and its seconds left.
    pub banner: Option<(String, f32)>,
    /// Free level-up redraws left this run (Second Thoughts).
    pub rerolls: u8,
    /// Cards that can still be banished this run.
    pub banishes: u8,
    pub banish_armed: bool,
    /// Opening Hand: the first level-up deals four cards.
    pub opening_hand: bool,
    /// Skip: pass a hand for gold.
    pub can_skip: bool,
    /// Keen Eye rank, copied on at the start.
    pub keen_eye: u8,
    /// Boss-chest trades that still receive Keen Eye.
    pub boss_trades: u32,
    /// Level-up hands dealt this run.
    pub drafts: u32,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            tuning: survivors_core::tuning::Tuning::default(),
            class: &data::CLASSES[0],
            race: 0,
            elapsed: 0.0,
            kills: 0,
            level: 1,
            xp: 0.0,
            hp: BASE_HP,
            abilities: Vec::new(),
            passives: Vec::new(),
            shield: 0.0,
            shield_t: 0.0,
            immune_t: 0.0,
            incoming_slows: Default::default(),
            enrage_t: 0.0,
            battle_shout: default(),
            ancestral_healing_t: 0.0,
            inspiration_t: 0.0,
            flurry_t: 0.0,
            vengeance_t: 0.0,
            remorseless_until: 0.0,
            blood_craze_t: 0.0,
            blessed_recovery: Default::default(),
            spirit_tap_t: 0.0,
            thorns: 0.0,
            thorns_t: 0.0,
            pet_intimidation_ready_at: 0.0,
            counterattack_ready_at: 0.0,
            weapon_attack_cd: 0.0,
            thorns_spell: 0,
            healing: Default::default(),
            mobile: mobility::MobilityState::default(),
            pending_levels: 0,
            bosses: 0,
            bosses_defeated: 0,
            rings: 0,
            won: false,
            trades: 0,
            trading: false,
            stocked: false,
            elite_chests: 0,
            last_elite_chest: 0.0,
            cold_snap: default(),
            preparation: default(),
            critical_cast: default(),
            ghostly_strike: default(),
            natures_grace: default(),
            healing_way: default(),
            arcane_power: default(),
            combustion: default(),
            hawk: default(),
            adrenaline_rush: default(),
            blade_flurry: default(),
            power_infusion: default(),
            worn: campaign::Worn::default(),
            dead_t: None,
            next_guid: 1,
            damage_done: 0.0,
            damage_sources: Vec::new(),
            gold_found: 0,
            gear: [None; 3],
            renown_might: 0,
            renown_fortitude: 0,
            renown_celerity: 0,
            renown_paid: false,
            account_paid: false,
            bonus: campaign::Bonus::default(),
            milestones: survivors_core::talents::Milestones::default(),
            version: 0,
            banner: None,
            rerolls: 0,
            banishes: 0,
            banish_armed: false,
            opening_hand: false,
            can_skip: false,
            keen_eye: 0,
            boss_trades: 0,
            drafts: 0,
        }
    }
}

impl Run {
    pub(super) fn alive(&self) -> bool {
        self.hp > 0.0 && self.dead_t.is_none()
    }

    /// All combat healing shares the lethal-hit boundary, including before the death animation starts.
    pub(super) fn heal(&mut self, amount: f32, max_hp: f32) -> f32 {
        if !self.alive() || !amount.is_finite() || amount <= 0.0 || !max_hp.is_finite() {
            return 0.0;
        }
        let healed = amount.min((max_hp - self.hp).max(0.0));
        self.hp += healed;
        healed
    }

    /// Complete a newly created run after its saved talents and perks are known.
    /// Never use this to refresh an active run: filling health is startup-only.
    pub(in crate::survivors) fn finish_start(
        &mut self,
        perks: rewards::PerkRanks,
        milestones: survivors_core::talents::Milestones,
    ) {
        self.apply_perks(perks);
        self.milestones = milestones;
        self.hp = self.stats().max_hp;
    }

    pub(in crate::survivors) fn apply_perks(&mut self, perks: rewards::PerkRanks) {
        self.rerolls = perks.second_thoughts;
        self.banishes = perks.banish;
        self.opening_hand = perks.opening_hand > 0;
        self.can_skip = perks.skip > 0;
        self.keen_eye = perks.keen_eye;
    }

    pub(super) fn record_spirit_tap_kill(&mut self, rng: &mut Rng) {
        let chance = self.milestones.spirit_tap_chance();
        if self.alive() && chance > 0.0 && (chance >= 1.0 || rng.f32() < chance) {
            self.spirit_tap_t = 15.0;
        }
    }

    pub(super) fn record_remorseless_kill(&mut self) {
        if self.alive() && self.milestones.remorseless_crit("sinister_strike") > 0.0 {
            self.remorseless_until = self.elapsed + 20.0;
        }
    }

    pub(super) fn remorseless_crit(&self, key: &str) -> f32 {
        if self.alive() && self.elapsed < self.remorseless_until {
            self.milestones.remorseless_crit(key)
        } else {
            0.0
        }
    }

    pub(super) fn consume_remorseless(&mut self, key: &str) {
        if self.milestones.remorseless_crit(key) > 0.0 {
            self.remorseless_until = 0.0;
        }
    }

    pub(super) fn record_vengeance_hit(&mut self, source: &str, damage: f32, critical: bool) {
        if self.alive() && self.milestones.triggers_vengeance(source, damage, critical) {
            self.vengeance_t = 8.0;
        }
    }

    pub(super) fn record_direct_healing(&mut self, source: &str, healed: f32, critical: bool) {
        if source == "greater_heal" && critical {
            self.apply_inspiration(healed);
        }
    }

    pub(super) fn record_inspiration_healing(&mut self, source: &str, healed: f32) {
        if source == "holy_nova" {
            self.apply_inspiration(healed);
        }
    }

    fn apply_inspiration(&mut self, healed: f32) {
        if self.alive()
            && healed.is_finite()
            && healed > 0.
            && self.milestones.inspiration_multiplier(true) > 1.
        {
            self.inspiration_t = 15.;
        }
    }

    pub(super) fn record_totem_healing(&mut self, source: &str, healed: f32) {
        if self.alive()
            && healed.is_finite()
            && healed > 0.
            && matches!(source, "searing_totem" | "magma_totem")
            && self.milestones.ancestral_healing_armor(true) > 0.
        {
            self.ancestral_healing_t = 10.;
        }
    }

    pub(super) fn record_flurry_hit(&mut self, source: &str, damage: f32, critical: bool) {
        if self.hp > 0.0
            && self.dead_t.is_none()
            && self.milestones.triggers_flurry(source, damage, critical)
        {
            self.flurry_t = 3.0;
        }
    }

    pub fn passive_value(&self, passive: &PassiveDef, rank: u8) -> f32 {
        let infusion = if passive.key == "renew" && self.alive() {
            self.power_infusion
                .active_bonus(self.milestones, self.elapsed)
        } else {
            0.
        };
        passive.per_level
            * f32::from(rank)
            * self.milestones.passive_multiplier(passive.key)
            * (1. + infusion)
    }

    pub fn stats(&self) -> Stats {
        let mut s = Stats {
            battle_shout: if self.alive() {
                self.battle_shout.bonus("cleave", self.elapsed)
            } else {
                0.0
            },
            vengeance: self
                .milestones
                .vengeance_bonus(self.alive() && self.vengeance_t > 0.0),
            legendary: self.worn.legendary_bonus().map(|(key, mut bonus)| {
                bonus.might *= self.tuning.damage;
                (key, bonus)
            }),
            might: 1.0,
            max_hp: BASE_HP,
            speed: BASE_SPEED,
            regen: 0.0,
            haste: 0.0,
            area: 1.0,
            magnet: BASE_MAGNET,
            growth: 1.0,
            armor: 0.0,
            crit: 0.05,
            leech: 0.0,
            reduction: 0.0,
            crit_damage: 2.0,
            cooldown: 0.0,
            gold_find: 1.0,
            luck: 0.0,
            multistrike: 0.0,
            targets: 0,
        };
        // Tanky classes start tougher, cloth starts quicker.
        match self.class.id {
            1 | 2 => s.max_hp += 40.0,
            7 | 11 | 3 | 4 => s.max_hp += 15.0,
            _ => s.speed += 0.5,
        }
        s.might += 0.08 * f32::from(self.renown_might);
        s.max_hp *= 1.0 + 0.10 * f32::from(self.renown_fortitude);
        s.speed *= 1.0 + 0.05 * f32::from(self.renown_celerity);
        s.haste += 0.03 * f32::from(self.renown_celerity);
        stash::apply_gear(&mut s, self.gear);
        for &(p, level) in &self.passives {
            let v = self.passive_value(p, level);
            match p.stat {
                Stat::Might => s.might += v,
                Stat::MaxHp => s.max_hp *= 1.0 + v,
                Stat::Speed => s.speed *= 1.0 + v,
                Stat::Regen => s.regen += v,
                Stat::Haste => s.haste += v,
                Stat::Area => s.area += v,
                Stat::Magnet => s.magnet *= 1.0 + v,
                Stat::Growth => s.growth += v,
                Stat::Armor => s.armor += v,
                Stat::Crit => s.crit += v,
            }
        }
        apply_sheet(&mut s, self.bonus);
        apply_sheet(&mut s, self.worn.bonus());
        s.haste += self.milestones.flurry_haste(self.flurry_t > 0.0);
        s.armor += self
            .milestones
            .ancestral_healing_armor(self.alive() && self.ancestral_healing_t > 0.);
        let racial = survivors_core::racials::effects(self.race, self.elapsed);
        apply_sheet(&mut s, racial.bonus);
        s.armor *= self
            .milestones
            .inspiration_multiplier(self.alive() && self.inspiration_t > 0.);
        s.max_hp *= racial.health_multiplier;
        s.max_hp *= self.milestones.health_multiplier();
        s.regen += racial.regen;
        s.haste += racial.haste;
        let damage_buff =
            self.milestones.enrage_damage(self.enrage_t > 0.0) * racial.damage_multiplier;
        s.might *= self.tuning.damage * damage_buff;
        if let Some((_, bonus)) = s.legendary.as_mut() {
            bonus.might *= damage_buff;
        }
        s.speed *= self.tuning.movement;
        s.growth *= self.tuning.experience;
        s.magnet *= self.tuning.pickup;
        s
    }

    pub fn guid(&mut self) -> u64 {
        self.next_guid += 1;
        self.next_guid
    }

    pub fn add_damage(&mut self, source: &'static str, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        let n = f64::from(amount);
        self.damage_done += n;
        if let Some(row) = self.damage_sources.iter_mut().find(|r| r.0 == source) {
            row.1 += n;
        } else {
            self.damage_sources.push((source, n));
        }
    }
}

/// xorshift64*: the mode's dice, seeded from the clock.
#[derive(Resource)]
pub(super) struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self(seed | 1)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }
    pub fn index(&mut self, len: usize) -> usize {
        (self.next_u64() % len.max(1) as u64) as usize
    }
}

fn pause(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

/// Back at the menu: everything the last run spawned goes, and the run starts over.
fn reset_run(
    mut commands: Commands,
    spawned: Query<Entity, With<RunEntity>>,
    mut run: ResMut<Run>,
    mut pending: ResMut<abilities::PendingHits>,
    mut channels: ResMut<channeling::Channels>,
    mut spawner: ResMut<enemies::Spawner>,
    mut selection: ResMut<Selection>,
) {
    for e in &spawned {
        commands.entity(e).despawn();
    }
    // Guids stay unique across runs: caches keyed by guid may outlive a despawn by a frame.
    let next_guid = run.next_guid;
    *run = Run {
        next_guid,
        ..Run::default()
    };
    pending.0.clear();
    *channels = channeling::Channels::default();
    *spawner = enemies::Spawner::default();
    // The preview hero is respawned for the look still picked.
    selection.version += 1;
}

#[cfg(test)]
mod stat_tests {
    use super::*;

    #[test]
    fn improved_renew_scales_owned_renew_only_and_resets_with_spec() {
        let renew = data::PASSIVES.iter().find(|p| p.key == "renew").unwrap();
        let mut run = Run {
            race: 8,
            ..Default::default()
        }; // Separate Troll regeneration.
        let base = run.stats().regen;
        run.milestones = survivors_core::talents::Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(&[(1, 406, 3)]);
        assert_eq!(run.stats().regen, base); // Training alone does not acquire Renew.
        for rank in 1..=data::MAX_PASSIVE_LEVEL {
            run.passives = vec![(renew, rank)];
            let value = 0.5 * f32::from(rank) * 1.15;
            assert!((run.stats().regen - base - value).abs() < 0.00001);
            assert!((run.passive_value(renew, rank) - value).abs() < 0.00001);
        }
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(
            (run.stats().regen - base - 0.5 * f32::from(data::MAX_PASSIVE_LEVEL)).abs() < 0.00001
        );
    }

    #[test]
    fn spiritual_healing_multiplies_renew_without_other_regeneration() {
        let renew = data::PASSIVES.iter().find(|p| p.key == "renew").unwrap();
        let mut run = Run {
            race: 8,
            ..Default::default()
        };
        let baseline = run.stats().regen;
        run.milestones = survivors_core::talents::Milestones {
            class: 4,
            ..Default::default()
        }
        .with_ranks(&[(1, 406, 3), (1, 404, 5)]);
        assert_eq!(run.stats().regen, baseline);
        for rank in 1..=data::MAX_PASSIVE_LEVEL {
            run.passives = vec![(renew, rank)];
            let expected = 0.5 * f32::from(rank) * 1.15 * 1.1;
            assert!((run.stats().regen - baseline - expected).abs() < 0.00001);
        }
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(
            (run.stats().regen - baseline - 0.5 * f32::from(data::MAX_PASSIVE_LEVEL)).abs()
                < 0.00001
        );
    }

    #[test]
    fn survivalist_scales_final_equipped_health_without_healing_current_hp() {
        let fortitude = data::PASSIVES
            .iter()
            .find(|p| matches!(p.stat, Stat::MaxHp))
            .unwrap();
        let mut run = Run {
            class: &data::CLASSES[2],
            race: 6,
            renown_fortitude: 2,
            hp: 30.,
            passives: vec![(fortitude, 3)],
            bonus: campaign::Bonus {
                health: 25.,
                ..Default::default()
            },
            ..Default::default()
        };
        let base = run.stats();
        run.milestones = survivors_core::talents::Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(2, 1622, 5)]);
        assert!((run.stats().max_hp - base.max_hp * 1.1).abs() < 0.00001);
        assert_eq!(run.hp, 30.);
        let mut starting = Run {
            bonus: run.bonus,
            ..Default::default()
        };
        starting.finish_start(Default::default(), run.milestones);
        assert_eq!(starting.hp, starting.stats().max_hp);
        assert_eq!(run.stats().might, base.might);
        run.bonus.health += 35.;
        assert!(
            (run.stats().max_hp
                - (base.max_hp
                    + 35. * survivors_core::racials::effects(run.race, 0.).health_multiplier)
                    * 1.1)
                .abs()
                < 0.0001
        );
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(
            (run.stats().max_hp
                - base.max_hp
                - 35. * survivors_core::racials::effects(run.race, 0.).health_multiplier)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn heart_of_the_wild_scales_final_health_without_healing_current_hp() {
        let fortitude = data::PASSIVES
            .iter()
            .find(|p| matches!(p.stat, Stat::MaxHp))
            .unwrap();
        let mut run = Run {
            class: &data::CLASSES[8],
            race: 6,
            renown_fortitude: 2,
            hp: 30.,
            passives: vec![(fortitude, 3)],
            bonus: campaign::Bonus {
                health: 25.,
                ..Default::default()
            },
            ..Default::default()
        };
        let base = run.stats();
        run.milestones = survivors_core::talents::Milestones {
            class: 8,
            ..Default::default()
        }
        .with_ranks(&[(1, 808, 5)]);
        assert!((run.stats().max_hp - base.max_hp * 1.2).abs() < 0.00001);
        assert_eq!(run.hp, 30.);
        let mut starting = Run {
            bonus: run.bonus,
            ..Default::default()
        };
        starting.finish_start(Default::default(), run.milestones);
        assert_eq!(starting.hp, starting.stats().max_hp);
        assert_eq!(run.stats().might, base.might);
        run.bonus.health += 35.;
        assert!(
            (run.stats().max_hp
                - (base.max_hp
                    + 35. * survivors_core::racials::effects(run.race, 0.).health_multiplier)
                    * 1.2)
                .abs()
                < 0.0001
        );
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(
            (run.stats().max_hp
                - base.max_hp
                - 35. * survivors_core::racials::effects(run.race, 0.).health_multiplier)
                .abs()
                < 0.0001
        );
    }

    #[test]
    fn all_racial_bonuses_reach_combat_and_race_changes_remove_them() {
        let mut run = Run {
            race: 0,
            ..Run::default()
        };
        let base = run.stats();
        for race in 1..=8 {
            run.race = race;
            let rules = survivors_core::racials::effects(race, 30.0);
            run.elapsed = 30.0;
            let actual = run.stats();
            assert!(
                (actual.might - (base.might + rules.bonus.might) * rules.damage_multiplier).abs()
                    < 0.00001
            );
            assert!((actual.max_hp - base.max_hp * rules.health_multiplier).abs() < 0.00001);
            assert!((actual.regen - base.regen - rules.regen).abs() < 0.00001);
            assert!((actual.haste - base.haste - rules.haste).abs() < 0.00001);
            assert!((actual.reduction - base.reduction - rules.bonus.reduction).abs() < 0.00001);
            assert!((actual.speed - base.speed * (1.0 + rules.bonus.speed)).abs() < 0.00001);
            assert!((actual.magnet - base.magnet * (1.0 + rules.bonus.pickup)).abs() < 0.00001);
            assert!((actual.area - base.area - rules.bonus.area).abs() < 0.00001);
            assert!((actual.armor - base.armor - rules.bonus.armor).abs() < 0.00001);
            assert!(
                (actual.crit_damage - base.crit_damage - rules.bonus.crit_damage).abs() < 0.00001
            );
            assert!((actual.growth - base.growth - rules.bonus.growth).abs() < 0.00001);
            assert!((actual.gold_find - base.gold_find - rules.bonus.gold).abs() < 0.00001);
            assert_eq!(run.stats().might, actual.might);
        }
        run.race = 0;
        assert_eq!(run.stats().might, base.might);
        assert_eq!(run.stats().regen, base.regen);
        assert_eq!(run.stats().max_hp, base.max_hp);
    }

    #[test]
    fn human_racials_reach_the_run_sheet_without_accumulating() {
        let mut run = Run {
            race: 1,
            ..Run::default()
        };
        run.bonus.growth = 0.2;
        run.bonus.gold = 0.3;
        run.bonus.might = 0.4;
        run.worn.equip(campaign::JewelKind::Legend("fireball"));
        let normal = run.stats();
        assert!((normal.growth - 1.3).abs() < 0.001);
        assert!((normal.gold_find - 1.5).abs() < 0.001);
        run.elapsed = 30.0;
        assert!((run.stats().might - normal.might * 1.15).abs() < 0.001);
        let active = run.stats();
        assert!(
            (active.for_spell("fireball").might - normal.for_spell("fireball").might * 1.15).abs()
                < 0.001
        );
        assert_eq!(run.stats().might, active.might);
        run.elapsed = 40.0;
        assert_eq!(run.stats().might, normal.might);
        run.race = 2;
        assert!((run.stats().growth - 1.2).abs() < 0.001);
        assert!((run.stats().gold_find - 1.3).abs() < 0.001);
    }

    #[test]
    fn chosen_slot_profiles_reach_combat_stats_and_downgrade_cleanly() {
        let mut run = Run::default();
        let base = run.stats();
        let mut owned = [0; campaign::PIECES];
        owned[2] = 1; // Common chest, kept for an explicit downgrade.
        owned[3 * campaign::GRID_SLOTS + 2] = 20; // Epic chest duplicates.
        owned[5] = 1; // Common boots.
        owned[6] = 1; // Common wrists.
        let mut equipment = survivors_core::equipment::Equipment::legacy(&owned);
        run.bonus = campaign::equipped_bonus(&equipment.worn_pieces(&owned), None);
        let epic = run.stats();
        assert!((epic.max_hp - base.max_hp - 25.0).abs() < 0.001);
        assert!((epic.armor - base.armor - 14.0).abs() < 0.001);
        assert!((epic.magnet - base.magnet * 1.26).abs() < 0.001);
        assert!((epic.speed - base.speed * 1.01).abs() < 0.001);
        assert!((epic.crit - base.crit - 0.01).abs() < 0.001);
        assert!((epic.cooldown - base.cooldown - 0.005).abs() < 0.001);
        assert!((epic.reduction - base.reduction - 0.02).abs() < 0.001);
        equipment.wear(&owned, 2, Some(0));
        run.bonus = campaign::equipped_bonus(&equipment.worn_pieces(&owned), None);
        let common = run.stats();
        assert!((common.max_hp - base.max_hp - 10.0).abs() < 0.001);
        assert!((common.armor - base.armor - 6.0).abs() < 0.001);
        assert!((common.magnet - base.magnet * 1.08).abs() < 0.001);
        assert_eq!(common.reduction, base.reduction);
        assert_eq!(common.might, base.might);
        assert_eq!(common.speed, epic.speed);
        assert_eq!(common.cooldown, epic.cooldown);
    }

    #[test]
    fn equipment_health_and_pickup_reach_the_run_sheet() {
        let mut run = Run::default();
        let base = run.stats();
        run.bonus = campaign::piece_bonus(0, 7);
        let equipped = run.stats();
        assert!((equipped.max_hp - base.max_hp - 10.0).abs() < 0.001);
        assert!((equipped.magnet - base.magnet * 1.12).abs() < 0.001);
        assert_eq!(equipped.might, base.might);
        run.bonus = campaign::Bonus::default();
        assert_eq!(run.stats().max_hp, base.max_hp);
        assert_eq!(run.stats().magnet, base.magnet);
    }
}

#[cfg(test)]
mod power_infusion_tests {
    use super::*;
    #[test]
    fn infusion_scales_only_owned_renew_during_the_active_window() {
        let renew = data::PASSIVES.iter().find(|p| p.key == "renew").unwrap();
        let mut run = Run {
            class: &data::CLASSES[4],
            milestones: survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(0, 322, 1), (1, 406, 3), (1, 404, 5)]),
            ..Default::default()
        };
        run.race = 8; // Troll regeneration must remain outside the healing multiplier.
        let without_renew = run.stats().regen;
        run.passives = vec![(renew, 3)];
        let normal = run.passive_value(renew, 3);
        assert!((run.stats().regen - without_renew - normal).abs() < 0.0001);
        run.power_infusion.consume(run.milestones, "smite", 0.);
        for elapsed in [0., 14.99, 15., 180.] {
            run.elapsed = elapsed;
            let factor = if elapsed < 15. { 1.2 } else { 1. };
            assert!((run.passive_value(renew, 3) - normal * factor).abs() < 0.0001);
            assert!((run.stats().regen - without_renew - normal * factor).abs() < 0.0001);
        }
        run.elapsed = 1.;
        run.passives.clear();
        assert_eq!(run.stats().regen, without_renew);
        run.milestones = run.milestones.with_ranks(&[(1, 406, 3), (1, 404, 5)]);
        assert_eq!(run.passive_value(renew, 3), normal);
    }
}
