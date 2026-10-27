//! The mode's screens, in bevy_ui since FrameXML is off: the class and hero pick, the in-run HUD,
//! the level-up cards, pause, and the results.

pub(super) mod card_growth;
mod draft;
pub(super) mod inline;
mod settings;
pub(super) use settings::checkboxes::refresh as refresh_settings;
pub(super) use settings::keyboard::navigate as settings_keyboard;
pub(super) use settings::sliders::{
    input as settings_slider_input, refresh as refresh_settings_sliders,
};

use std::collections::HashMap;

use benilla_ui::script::nameplate;
use benilla_world::view::WorldCamera;
use bevy::prelude::*;
type MeterValueFilter = (Without<MeterHead>, Without<MeterName>);
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::window::PrimaryWindow;

use super::data::{self, AbilityDef, Effect, Kind, PassiveDef, Stat};
use super::hero::{self, Hero};
use super::minimap::{self, MiniMap};
use super::{Owned, Phase, Rng, Run, Selection, Stats};

const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);
/// The mobility button's edge, an action button's 36 units.
const MOBILITY_SIZE: f32 = 40.0;
const EDGE: Color = Color::srgb(0.55, 0.45, 0.25);
const BUTTON: Color = Color::srgba(0.18, 0.06, 0.04, 0.95);
const BUTTON_HOT: Color = Color::srgba(0.36, 0.12, 0.06, 0.98);
const DIM: Color = Color::srgb(0.6, 0.6, 0.6);
pub(super) const DIM_GREY: Color = DIM;
pub(super) const GOLD_TONE: Color = GOLD;

#[derive(Component)]
pub(super) struct MenuRoot;
#[derive(Component)]
pub(super) struct HudRoot;
#[derive(Component)]
pub(super) struct LevelUpRoot;
#[derive(Component)]
pub(super) struct PauseRoot;
#[derive(Component)]
pub(super) struct OverRoot;
#[derive(Component)]
pub(super) struct XpBar;
#[derive(Component)]
pub(super) struct XpFill;
#[derive(Component)]
pub(super) struct XpText;
#[derive(Component)]
pub(super) struct LevelText;
#[derive(Component)]
pub(super) struct TimerText;
#[derive(Component)]
pub(super) struct ReactiveBuffText;
#[derive(Component)]
pub(super) struct KillText;
#[derive(Component)]
pub(super) struct HpFrame;
#[derive(Component)]
pub(super) struct HpBar;
#[derive(Component)]
pub(super) struct HpLevelSeat;
#[derive(Component)]
pub(super) struct HpLevel;
#[derive(Component)]
pub(super) struct HpFill;
#[derive(Component)]
pub(super) struct ShieldFill;
#[derive(Component)]
pub(super) struct BannerText;
#[derive(Component)]
pub(super) struct AbilityRow;
#[derive(Component)]
pub(super) struct PassiveRow;
#[derive(Component)]
pub(super) struct OfferRow;

#[derive(Component)]
pub(super) struct LevelHint;
/// The line under LEVEL UP: which level this is.
#[derive(Component)]
pub(super) struct LevelUpSub;

#[derive(Component)]
pub(super) struct LevelTitle;

/// A radial cooldown sweep over one ability icon: `sweep` is the remaining fraction
/// (1.0 = just cast, 0.0 = ready), drawn by `ui_sweep.wgsl`.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(crate) struct SweepUiMaterial {
    #[uniform(0)]
    pub(crate) sweep: f32,
}

impl UiMaterial for SweepUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://benilla_app/shaders/ui_sweep.wgsl".into()
    }
}

/// The cooldown overlay on one ability slot, driven every frame.
#[derive(Component)]
pub(super) struct AbilitySweep {
    handle: Handle<SweepUiMaterial>,
    index: usize,
}

/// The class mobility's action button, bottom centre over the XP bar; shown once it is learned.
#[derive(Component)]
pub(super) struct MobilitySlot;
#[derive(Component)]
pub(super) struct MobilityIcon;
#[derive(Component)]
pub(super) struct MobilitySweep(Handle<SweepUiMaterial>);
#[derive(Component)]
pub(super) struct OverTitle;
#[derive(Component)]
pub(super) struct OverStats;
#[derive(Component)]
pub(super) struct OverLine;
#[derive(Component)]
pub(super) struct OverStat(usize);

/// An icon slot filled from the spell's `SpellIcon` once the client data is up.
#[derive(Component)]
pub(super) struct SpellIcon(pub(super) u32);
/// How bright [`fill_icons`] leaves the slot. Unseen codex entries stay drawn, darker.
#[derive(Component)]
pub(super) struct IconDim(pub(super) f32);
#[derive(Component)]
pub(super) struct IconDone;

/// A FrameXML `Texture` region: a client art file, its `TexCoords` (left, right, top, bottom) and
/// vertex colour, drawn once the archives are readable.
#[derive(Component)]
pub(super) struct UiArt {
    path: &'static str,
    coords: [f32; 4],
    tint: Color,
}

impl UiArt {
    pub(super) fn crop(path: &'static str, coords: [f32; 4]) -> Self {
        Self {
            path,
            coords,
            tint: Color::WHITE,
        }
    }

    fn tinted(path: &'static str, tint: Color) -> Self {
        Self {
            path,
            coords: [0.0, 1.0, 0.0, 1.0],
            tint,
        }
    }
}

#[derive(Component)]
pub(super) struct ArtDone;

/// What an icon slot's tooltip describes: an index into the run's abilities or passives.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TipFor {
    Ability(usize),
    Passive(usize),
    Mobility,
}

/// The one `GameTooltip`, and the column its lines go in.
#[derive(Component)]
pub(super) struct GameTooltip;
#[derive(Component)]
pub(super) struct TipBody;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Knob {
    CameraSpeed,
    ZoomSpeed,
    InvertX,
    InvertY,
    Master,
    Effects,
    Music,
    Ambience,
    Sound,
    MusicOn,
    AutoPush,
    Cutaway,
    View,
    Grass,
    Spells,
    Scale,
    InterfaceScale,
}

#[derive(Component, Clone, Copy, PartialEq)]
pub(super) enum UiAction {
    Resume,
    MainMenu,
    Quit,
    OpenSettings,
    CloseSettings,
    SettingsCategory(settings::Category),
    BuildNotes,
    RestoreSettings(settings::Category),
    Toggle(Knob),
}

/// Shared settings hide the starter or replace pause buttons until Back/Escape.
#[derive(Resource, Default)]
pub(super) struct PausePage {
    pub open: bool,
    category: settings::Category,
}

/// Knob clicks, applied by [`apply_settings`] so this screen does not share the cvar lock
/// with the rest of the client.
#[derive(Resource, Default)]
pub(super) struct SettingEdits {
    set: Vec<(Knob, f32)>,
    toggle: Vec<Knob>,
    restore: Option<settings::Category>,
}

#[derive(Component)]
pub(super) struct PauseMain;

#[derive(Component)]
pub(super) struct SettingsPanel;

#[derive(Component)]
pub(super) struct MeterRoot;

#[derive(Component)]
pub(super) struct MeterHead;

#[derive(Component)]
pub(super) struct MeterLine(u8);

#[derive(Component)]
pub(super) struct MeterName(u8);

#[derive(Component)]
pub(super) struct MeterValue(u8);

#[derive(Component)]
pub(super) struct MeterFill(u8);

const METER_ROWS: u8 = 6;

/// The colour a button returns to when not hovered.
#[derive(Component)]
pub(super) struct BaseColor(Color);

#[derive(Clone, Copy)]
pub(super) enum Choice {
    NewAbility(&'static AbilityDef),
    Upgrade(usize),
    NewPassive(&'static PassiveDef),
    UpPassive(usize),
    Heal,
    Jewel(super::campaign::JewelKind),
}

/// The level-up cards on offer.
#[derive(Resource, Default)]
pub(super) struct Offer {
    pub(super) choices: Vec<Choice>,
}

impl Offer {
    pub(super) fn len(&self) -> usize {
        self.choices.len()
    }

    pub(super) fn choices(&self) -> &[Choice] {
        &self.choices
    }
}

#[derive(Resource)]
pub(super) struct HudFont(pub(super) Handle<Font>);

pub(super) fn rgb(hex: u32) -> Color {
    Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn text(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::splat(1.5),
            color: Color::BLACK,
        },
    )
}

pub(super) fn text_bundle(
    font: &Handle<Font>,
    s: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Bundle {
    text(font, s, size, color)
}

pub(super) fn centered_bundle(
    font: &Handle<Font>,
    s: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Bundle {
    centered(font, s, size, color)
}

fn centered(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        text(font, s, size, color),
        TextLayout::new_with_justify(Justify::Center),
    )
}

fn button(action: UiAction, base: Color) -> impl Bundle {
    edged_button(action, base, EDGE)
}

fn edged_button(action: UiAction, base: Color, edge: Color) -> impl Bundle {
    (
        Button,
        action,
        BaseColor(base),
        BackgroundColor(base),
        BorderColor::all(edge),
    )
}

fn icon(spell: u32, size: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        ImageNode {
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
            ..default()
        },
        SpellIcon(spell),
    )
}

fn spell_name(spells: Option<&crate::ui_action::Spells>, spell: u32, fallback: &str) -> String {
    spells
        .and_then(|s| s.catalog.get(spell))
        .map(|d| d.name.clone())
        .unwrap_or_else(|| fallback.replace('_', " "))
}

pub(super) fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut sweeps_init: ResMut<Assets<SweepUiMaterial>>,
) {
    let font = crate::char_select::wow_font(&asset_server);
    let number_font: Handle<Font> = asset_server.load("mpq://Fonts/ARIALN.ttf");
    commands.insert_resource(HudFont(font.clone()));
    let f = &font;
    let map = MiniMap::new(&mut images);

    // --- The in-run HUD. ---
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(400),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            // `MainMenuExpBar`, 1024×13 at the bottom centre. The reference stacks it on the
            // 40-unit action bar; with no action bar here it rests on the screen's edge.
            root.spawn((
                XpBar,
                Interaction::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(0.0),
                    margin: UiRect::left(Val::Px(-512.0)),
                    width: Val::Px(1024.0),
                    height: Val::Px(13.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    XpFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode {
                        color: Color::NONE,
                        ..default()
                    },
                    UiArt::tinted(
                        "Interface\\TargetingFrame\\UI-StatusBar",
                        Color::srgb(0.58, 0.0, 0.55),
                    ),
                ));
                // `MainMenuXPBarTexture0`..`3`: the segment rules, 256×10 on the bar's top.
                for (i, top) in [0.792_968_75, 0.542_968_75, 0.292_968_75, 0.042_968_75]
                    .into_iter()
                    .enumerate()
                {
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(256.0 * i as f32),
                            top: Val::Px(0.0),
                            width: Val::Px(256.0),
                            height: Val::Px(10.0),
                            ..default()
                        },
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
                        UiArt::crop(
                            "Interface\\MainMenuBar\\UI-MainMenuBar-Dwarf",
                            [0.0, 1.0, top, top + 0.039_062_5],
                        ),
                    ));
                }
                // `MainMenuBarExpText`, `TextStatusBarText` centred one unit up, shown on hover.
                bar.spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    top: Val::Px(-1.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|n| {
                    n.spawn((
                        XpText,
                        Text::new(""),
                        TextFont {
                            font: number_font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextShadow {
                            offset: Vec2::new(1.0, 1.0),
                            color: Color::BLACK,
                        },
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        Visibility::Hidden,
                    ));
                });
            });
            minimap::spawn_cluster(root, f, &map);
            // The mobility button: a 36-unit action slot in `UI-Quickslot2`, its key over it.
            root.spawn((
                MobilitySlot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(24.0),
                    margin: UiRect::left(Val::Px(-MOBILITY_SIZE / 2.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|col| {
                col.spawn(centered(f, "F", 18.0, Color::WHITE));
                col.spawn((
                    Node {
                        width: Val::Px(MOBILITY_SIZE),
                        height: Val::Px(MOBILITY_SIZE),
                        ..default()
                    },
                    Interaction::default(),
                    TipFor::Mobility,
                ))
                .with_children(|slot| {
                    // `UI-Quickslot2` is the empty slot's 64×64 frame, the button 36 inside it.
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-MOBILITY_SIZE * 0.42),
                            top: Val::Px(-MOBILITY_SIZE * 0.42),
                            width: Val::Px(MOBILITY_SIZE * 1.84),
                            height: Val::Px(MOBILITY_SIZE * 1.84),
                            ..default()
                        },
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
                        UiArt::crop("Interface\\Buttons\\UI-Quickslot2", [0.0, 1.0, 0.0, 1.0]),
                    ));
                    slot.spawn((
                        MobilityIcon,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ImageNode {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                            ..default()
                        },
                    ));
                    let handle = sweeps_init.add(SweepUiMaterial { sweep: 0.0 });
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        MaterialNode(handle.clone()),
                        MobilitySweep(handle),
                    ));
                });
            });
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|row| {
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(300.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        AbilityRow,
                        Node {
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                    ));
                    col.spawn((
                        PassiveRow,
                        Node {
                            column_gap: Val::Px(4.0),
                            ..default()
                        },
                    ));
                });
                // The clock, with the level and the kill count on one line under it.
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(0.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((TimerText, centered(f, "00:00", 34.0, Color::WHITE)));
                    col.spawn(Node {
                        column_gap: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((LevelText, text(f, "Level 1", 14.0, GOLD)));
                        line.spawn((KillText, text(f, "Kills 0", 14.0, Color::WHITE)));
                    });
                    col.spawn((ReactiveBuffText, centered(f, "", 14.0, card_growth::GAIN)));
                    col.spawn((
                        super::hunter_pet::PetStatusText,
                        centered(f, "", 14.0, card_growth::GAIN),
                    ));
                });
                // The minimap cluster's column.
                row.spawn(Node {
                    width: Val::Px(300.0),
                    ..default()
                });
            });
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(22.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|n| {
                n.spawn((BannerText, centered(f, "", 30.0, GOLD)));
            });
            // The hero's health as a nameplate (`CGNamePlateFrame`): the bar under the border that
            // caps it, the level in the border's seat; [`hero_plate`] sizes and seats it.
            let absolute = || Node {
                position_type: PositionType::Absolute,
                ..default()
            };
            let unpainted = || ImageNode {
                color: Color::NONE,
                ..default()
            };
            let [r, g, b, _] = crate::vplates::PLATE_FRIENDLY;
            root.spawn((HpFrame, absolute(), Visibility::Hidden))
                .with_children(|plate| {
                    plate.spawn((HpBar, absolute())).with_children(|bar| {
                        bar.spawn((
                            HpFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            unpainted(),
                            UiArt::tinted(nameplate::BAR_FILL_TEXTURE, Color::srgb(r, g, b)),
                        ));
                        bar.spawn((
                            ShieldFill,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..absolute()
                            },
                            unpainted(),
                            UiArt::tinted(
                                nameplate::BAR_FILL_TEXTURE,
                                Color::srgba(0.85, 0.9, 1.0, 0.75),
                            ),
                        ));
                    });
                    plate.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..absolute()
                        },
                        unpainted(),
                        UiArt::crop(nameplate::BORDER_TEXTURE, [0.0, 1.0, 0.0, 1.0]),
                    ));
                    let [r, g, b, _] = crate::vplates::CON_YELLOW;
                    plate
                        .spawn((
                            HpLevelSeat,
                            Node {
                                width: Val::Px(0.0),
                                height: Val::Px(0.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::visible(),
                                ..absolute()
                            },
                        ))
                        .with_children(|seat| {
                            seat.spawn((
                                HpLevel,
                                Text::new("1"),
                                TextFont {
                                    font: f.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(r, g, b)),
                                TextShadow {
                                    offset: Vec2::new(1.0, 1.0),
                                    color: Color::BLACK,
                                },
                                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            ));
                        });
                });
            root.spawn((
                MeterRoot,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(28.0),
                    width: Val::Px(236.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.03, 0.02, 0.01, 0.78)),
                BorderColor::all(EDGE),
            ))
            .with_children(|m| {
                m.spawn((MeterHead, text(f, "Damage", 13.0, GOLD)));
                for i in 0..METER_ROWS {
                    m.spawn((
                        MeterLine(i),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(15.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Visibility::Hidden,
                    ))
                    .with_children(|line| {
                        line.spawn((
                            MeterFill(i),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                height: Val::Percent(100.0),
                                width: Val::Percent(0.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.85, 0.55, 0.12, 0.4)),
                        ));
                        line.spawn((MeterName(i), text(f, "", 12.0, Color::WHITE)));
                        line.spawn((MeterValue(i), text(f, "", 12.0, GOLD)));
                    });
                }
            });
        });
    commands.insert_resource(map);

    // --- The run's pick screen; [`super::menu`] builds into it once the glue art is in. ---
    commands.spawn((
        MenuRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(410),
        bevy::ui::FocusPolicy::Pass,
    ));

    // --- Level up. ---
    commands
        .spawn((
            LevelUpRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            // A dark vignette closing in on the cards.
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::percent(Color::srgba(0.04, 0.025, 0.0, 0.72), 0.0),
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.9), 100.0),
                ],
            )),
            GlobalZIndex(420),
            Visibility::Hidden,
            // The build's icons stay hoverable under the cards.
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((LevelTitle, centered(f, "LEVEL UP", 52.0, GOLD)));
            root.spawn((
                LevelUpSub,
                centered(f, "Choose your power", 16.0, Color::srgb(0.85, 0.78, 0.6)),
            ));
            root.spawn((
                OfferRow,
                ScrollPosition::default(),
                Node {
                    width: Val::Percent(94.0),
                    max_width: Val::Px(1280.0),
                    max_height: Val::Percent(70.0),
                    flex_shrink: 1.0,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(22.0),
                    row_gap: Val::Px(36.0),
                    overflow: Overflow::scroll_y(),
                    // Reserve space inside the scrollport for hover, tilt and pick motion.
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(40.0)),
                    ..default()
                },
            ));
            root.spawn((
                LevelHint,
                centered(
                    f,
                    "Click a card or press its number · Scroll to see more",
                    14.0,
                    DIM,
                ),
            ));
        });

    // --- Pause. ---
    commands
        .spawn((
            PauseRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(430),
            Visibility::Hidden,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn((
                PauseMain,
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
            ))
            .with_children(|col| {
                col.spawn(centered(f, "Paused", 46.0, GOLD));
                for (label, action) in [
                    ("Resume", UiAction::Resume),
                    ("Settings", UiAction::OpenSettings),
                    ("Abandon run", UiAction::MainMenu),
                    ("Quit game", UiAction::Quit),
                ] {
                    col.spawn((
                        button(action, BUTTON),
                        Node {
                            width: Val::Px(240.0),
                            justify_content: JustifyContent::Center,
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                    ))
                    .with_children(|b| {
                        b.spawn(text(f, label, 20.0, Color::WHITE));
                    });
                }
            });
            root.spawn((
                SettingsPanel,
                Node {
                    display: Display::None,
                    width: Val::Percent(90.0),
                    max_width: Val::Px(800.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.035, 0.025, 0.015)),
                BorderColor::all(EDGE),
            ))
            .with_children(|col| {
                settings::build(col, f);
            });
        });

    // --- Results. ---
    commands
        .spawn((
            OverRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            BackgroundGradient::from(RadialGradient::new(
                UiPosition::CENTER,
                RadialGradientShape::FarthestCorner,
                vec![
                    ColorStop::percent(Color::srgba(0.07, 0.02, 0.0, 0.8), 0.0),
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.94), 100.0),
                ],
            )),
            GlobalZIndex(440),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                OverTitle,
                centered(f, "", 64.0, GOLD),
                Node {
                    max_width: Val::Percent(90.0),
                    ..default()
                },
            ));
            root.spawn((
                OverLine,
                centered(f, "", 18.0, Color::srgb(0.85, 0.78, 0.6)),
                Node {
                    max_width: Val::Percent(90.0),
                    ..default()
                },
            ));
            // The run in numbers, one tile each.
            root.spawn(Node {
                width: Val::Percent(80.0),
                max_width: Val::Px(900.0),
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Percent(1.5),
                row_gap: Val::Px(12.0),
                margin: UiRect::vertical(Val::Px(14.0)),
                ..default()
            })
            .with_children(|row| {
                for (i, label) in ["Survived", "Level", "Kills", "Damage", "Bosses"]
                    .into_iter()
                    .enumerate()
                {
                    row.spawn((
                        Node {
                            width: Val::Percent(18.0),
                            min_width: Val::Px(100.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundGradient::from(LinearGradient::to_bottom(vec![
                            ColorStop::percent(Color::srgba(0.2, 0.14, 0.05, 0.92), 0.0),
                            ColorStop::percent(Color::srgba(0.04, 0.03, 0.02, 0.94), 100.0),
                        ])),
                        BorderColor::all(EDGE),
                    ))
                    .with_children(|tile| {
                        tile.spawn((OverStat(i), centered(f, "", 26.0, Color::WHITE)));
                        tile.spawn(centered(f, label, 12.0, DIM));
                    });
                }
            });
            root.spawn((
                OverStats,
                centered(f, "", 15.0, DIM),
                Node {
                    max_width: Val::Percent(85.0),
                    ..default()
                },
            ));
            root.spawn((
                edged_button(UiAction::MainMenu, BUTTON, GOLD),
                Node {
                    width: Val::Percent(30.0),
                    min_width: Val::Px(180.0),
                    max_width: Val::Px(320.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(11.0)),
                    margin: UiRect::top(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BoxShadow::new(
                    Color::srgba(1.0, 0.7, 0.2, 0.35),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(16.0),
                ),
            ))
            .with_children(|b| {
                b.spawn(text(f, "Play again", 22.0, GOLD));
            });
            root.spawn((
                button(UiAction::Quit, Color::NONE),
                Node {
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn(text(f, "Quit game", 15.0, DIM));
            });
        });
}

/// The menu's hero is the run's hero: respawned whenever the pick changes.
pub(super) fn menu_preview(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    mut run: ResMut<Run>,
    heroes: Query<Entity, With<Hero>>,
    world: super::Floor,
    mut shown: Local<Option<(usize, u32, usize)>>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    let want = selection
        .class
        .zip(selection.look.and_then(|i| selection.looks.get(i)).copied());
    let key = want.map(|(c, l)| (c, l.0, selection.stage));
    if key == *shown && (key.is_none() || !heroes.is_empty()) {
        return;
    }
    for e in &heroes {
        commands.entity(e).despawn();
    }
    *shown = key;
    let Some((class, (display, race, sex))) = want else {
        return;
    };
    let mut pos = hero::arena(selection.stage_def());
    if let Some(h) = world.height(pos) {
        pos.y = h;
    }
    let serial = run.guid();
    hero::spawn_hero(
        &mut commands,
        &data::CLASSES[class],
        display,
        race,
        sex,
        serial,
        pos,
    );
}

pub(super) fn start_run(
    run: &mut Run,
    selection: &mut Selection,
    ranks: [u8; 3],
    gear: [Option<(u8, u8)>; 3],
    bonus: super::campaign::Bonus,
) -> bool {
    let (Some(class), Some(&(_, race, _))) = (
        selection.class_def(),
        selection.look.and_then(|i| selection.looks.get(i)),
    ) else {
        return false;
    };
    // The world goes to the battleground picked; until now it stood in the last one.
    selection.stage = selection.map;
    let next_guid = run.next_guid;
    *run = Run {
        class,
        race,
        next_guid,
        renown_might: ranks[0],
        renown_fortitude: ranks[1],
        renown_celerity: ranks[2],
        gear,
        bonus,
        ..Run::default()
    };
    let mut notes = Vec::new();
    if ranks.iter().any(|r| *r > 0) {
        notes.push(format!(
            "Renown — Might {}, Fortitude {}, Quickness {}",
            ranks[0], ranks[1], ranks[2]
        ));
    }
    let relics = super::stash::blurb(gear);
    if !relics.is_empty() {
        notes.push(relics);
    }
    if !notes.is_empty() {
        run.banner = Some((notes.join("   "), 4.0));
    }
    run.abilities.push(Owned {
        def: data::ability(class.start),
        level: 1,
        cd: 0.5,
    });
    run.hp = run.stats().max_hp;
    run.version += 1;
    true
}

fn choices_for(run: &Run, rng: &mut Rng, n: usize) -> Vec<Choice> {
    choices_excluding(run, rng, n, &[])
}

fn choices_excluding(run: &Run, rng: &mut Rng, n: usize, excluded: &[Choice]) -> Vec<Choice> {
    let mut pool: Vec<(Choice, f32)> = Vec::new();
    for (i, o) in run.abilities.iter().enumerate() {
        if o.level < data::MAX_ABILITY_LEVEL {
            pool.push((Choice::Upgrade(i), 1.2));
        }
    }
    if run.abilities.len() < data::MAX_ABILITIES {
        for key in run.class.pool {
            if !run.abilities.iter().any(|o| o.def.key == *key) {
                pool.push((Choice::NewAbility(data::ability(key)), 1.0));
            }
        }
    }
    for (i, (_, level)) in run.passives.iter().enumerate() {
        if *level < data::MAX_PASSIVE_LEVEL {
            pool.push((Choice::UpPassive(i), 0.8));
        }
    }
    if run.passives.len() < data::MAX_PASSIVES {
        for p in data::PASSIVES {
            if !run.passives.iter().any(|(q, _)| q.key == p.key) {
                pool.push((Choice::NewPassive(p), 0.5));
            }
        }
    }
    pool.retain(|(card, _)| !excluded.iter().any(|other| draft::same(*card, *other)));
    let mut out = Vec::with_capacity(n);
    while out.len() < n && !pool.is_empty() {
        let total: f32 = pool.iter().map(|(_, w)| w).sum();
        let mut pick = rng.f32() * total;
        let mut idx = pool.len() - 1;
        for (i, (_, w)) in pool.iter().enumerate() {
            if pick < *w {
                idx = i;
                break;
            }
            pick -= w;
        }
        out.push(pool.swap_remove(idx).0);
    }
    if out.is_empty() && excluded.is_empty() {
        out.push(Choice::Heal);
    }
    out
}

pub(super) fn roll_offer(mut run: ResMut<Run>, mut rng: ResMut<Rng>, mut offer: ResMut<Offer>) {
    offer.choices = if run.trading {
        deal_trade(&mut run, &mut rng)
    } else {
        deal_level(&mut run, &mut rng)
    };
}

fn deal_level(run: &mut Run, rng: &mut Rng) -> Vec<Choice> {
    let n = super::rewards::hand_size(run.opening_hand, run.drafts);
    run.drafts += 1;
    choices_for(run, rng, n)
}

fn deal_trade(run: &mut Run, rng: &mut Rng) -> Vec<Choice> {
    if run.boss_trades > 0 {
        run.boss_trades -= 1;
    }
    trade_choices(run, rng, 0.0)
}

/// Redraw a level-up hand. Chest trades are not redrawn.
pub(super) fn reroll(run: &mut Run, offer: &mut Offer, rng: &mut Rng) -> bool {
    if run.trading || run.rerolls == 0 {
        return false;
    }
    run.rerolls -= 1;
    let n = offer.choices.len().clamp(3, 4);
    offer.choices = choices_for(run, rng, n);
    true
}

/// Pass the hand and take gold instead.
pub(super) fn skip_hand(
    run: &mut Run,
    offer: &mut Offer,
    rng: &mut Rng,
    stash: &mut super::stash::Stash,
    next: &mut NextState<Phase>,
) -> bool {
    if !run.can_skip {
        return false;
    }
    let gold =
        super::sheet::gold_amount(super::rewards::skip_gold(run.level), run.stats().gold_find);
    run.gold_found += gold;
    stash.add_gold(gold);
    run.banner = Some((format!("Skipped the hand for {gold} gold"), 2.5));
    if run.trading {
        run.trades = run.trades.saturating_sub(1);
        if run.trades == 0 {
            run.trading = false;
        }
    } else {
        run.pending_levels = run.pending_levels.saturating_sub(1);
    }
    advance(run, offer, rng, next);
    true
}

/// Throw out one card and draw a replacement.
pub(super) fn banish_card(run: &mut Run, offer: &mut Offer, rng: &mut Rng, index: usize) -> bool {
    if run.banishes == 0 || index >= offer.choices.len() {
        return false;
    }
    let original_rng = rng.0;
    let replacement = if run.trading {
        trade_choices_excluding(run, rng, 0.0, &offer.choices)
            .into_iter()
            .next()
    } else {
        choices_excluding(run, rng, 1, &offer.choices)
            .into_iter()
            .next()
    };
    // An exhausted roll leaves the hand and charge intact, including a one-card chest.
    let Some(card) = replacement else {
        // A free failed attempt must not become repeated free jewelry rolls.
        rng.0 = original_rng;
        run.banish_armed = false;
        run.banner = Some((
            "No distinct replacement rolled; banish charge kept".into(),
            2.5,
        ));
        return false;
    };
    offer.choices[index] = card;
    run.banishes -= 1;
    run.banish_armed = false;
    true
}

pub(super) fn pick(
    i: usize,
    run: &mut Run,
    offer: &mut Offer,
    rng: &mut Rng,
    next: &mut NextState<Phase>,
) {
    if !run.alive() {
        return;
    }
    let Some(&choice) = offer.choices.get(i) else {
        return;
    };
    match choice {
        Choice::NewAbility(def) => run.abilities.push(Owned {
            def,
            level: 1,
            cd: 0.3,
        }),
        Choice::Upgrade(k) => run.abilities[k].level += 1,
        Choice::NewPassive(p) => run.passives.push((p, 1)),
        Choice::UpPassive(k) => run.passives[k].1 += 1,
        Choice::Heal => {
            let max_hp = run.stats().max_hp;
            run.heal(max_hp, max_hp);
        }
        Choice::Jewel(kind) => run.worn.equip(kind),
    }
    run.version += 1;
    if run.trading {
        run.trades = run.trades.saturating_sub(1);
        if run.trades == 0 {
            run.trading = false;
        }
    } else {
        run.pending_levels = run.pending_levels.saturating_sub(1);
    }
    advance(run, offer, rng, next);
}

fn advance(run: &mut Run, offer: &mut Offer, rng: &mut Rng, next: &mut NextState<Phase>) {
    if run.pending_levels > 0 {
        offer.choices = deal_level(run, rng);
    } else if run.trades > 0 {
        run.trading = true;
        offer.choices = deal_trade(run, rng);
    } else {
        offer.choices.clear();
        next.set(Phase::Playing);
    }
}

fn trade_choices(run: &mut Run, rng: &mut Rng, extra: f32) -> Vec<Choice> {
    trade_choices_excluding(run, rng, extra, &[])
}

fn trade_choices_excluding(
    run: &mut Run,
    rng: &mut Rng,
    extra: f32,
    excluded: &[Choice],
) -> Vec<Choice> {
    let excluded: Vec<_> = excluded
        .iter()
        .filter_map(|card| draft::trade(*card, run))
        .collect();
    let upgradeable: Vec<usize> = run
        .abilities
        .iter()
        .enumerate()
        .filter(|(_, o)| o.level < data::MAX_ABILITY_LEVEL)
        .map(|(i, _)| i)
        .collect();
    let luck = super::sheet::luck_percent(run.stats().luck);
    let rolls = [rng.f32(), rng.f32(), rng.f32()];
    let picks = [
        rng.next_u64() as u32,
        rng.next_u64() as u32,
        rng.next_u64() as u32,
    ];
    super::campaign::trade_cards_excluding(&upgradeable, luck, rolls, picks, extra, &excluded)
        .into_iter()
        .map(|card| match card {
            super::campaign::TradeCard::Upgrade(i) => Choice::Upgrade(i),
            super::campaign::TradeCard::Ring(i) => {
                Choice::Jewel(super::campaign::JewelKind::Ring(i as u8))
            }
            super::campaign::TradeCard::Trinket(i) => {
                Choice::Jewel(super::campaign::JewelKind::Trinket(i as u8))
            }
            super::campaign::TradeCard::Amulet(i) => {
                Choice::Jewel(super::campaign::JewelKind::Amulet(i as u8))
            }
            super::campaign::TradeCard::Legend(i) => {
                let key = run.abilities[i].def.key;
                Choice::Jewel(super::campaign::JewelKind::Legend(key))
            }
        })
        .collect()
}

pub(super) fn looks_for(
    class: &data::ClassDef,
    creatures: &crate::entities::Creatures,
) -> Vec<(u32, u8, u8)> {
    super::look_catalog::resolve(class, |display| creatures.display_race_sex(display))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn buttons(
    mut presses: Query<
        (&Interaction, &UiAction, &BaseColor, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next: ResMut<NextState<Phase>>,
    mut exit: MessageWriter<AppExit>,
    mut edits: ResMut<SettingEdits>,
    mut page: ResMut<PausePage>,
    phase: Res<State<Phase>>,
    mut armory: ResMut<super::armory::Armory>,
) {
    for (interaction, action, base, mut bg) in &mut presses {
        match interaction {
            Interaction::Hovered => bg.0 = BUTTON_HOT,
            Interaction::None => bg.0 = base.0,
            Interaction::Pressed => {
                bg.0 = BUTTON_HOT;
                activate_action(
                    *action,
                    *phase.get(),
                    &mut next,
                    &mut exit,
                    &mut edits,
                    &mut page,
                    &mut armory,
                );
            }
        }
    }
}

/// Shared by pointer and keyboard activation; phase/modal guards stay in one place.
#[allow(clippy::too_many_arguments)]
fn activate_action(
    action: UiAction,
    phase: Phase,
    next: &mut NextState<Phase>,
    exit: &mut MessageWriter<AppExit>,
    edits: &mut SettingEdits,
    page: &mut PausePage,
    armory: &mut super::armory::Armory,
) {
    match (action, phase) {
        (UiAction::Resume, Phase::Paused) => next.set(Phase::Playing),
        (UiAction::MainMenu, Phase::Paused | Phase::GameOver) => next.set(Phase::Menu),
        (UiAction::Quit, Phase::Paused | Phase::GameOver | Phase::Menu) => {
            exit.write(AppExit::Success);
        }
        (UiAction::OpenSettings, Phase::Paused) => page.open = true,
        (UiAction::CloseSettings, Phase::Paused | Phase::Menu) => page.open = false,
        (UiAction::SettingsCategory(category), Phase::Paused | Phase::Menu) if page.open => {
            page.category = category;
        }
        (UiAction::RestoreSettings(category), Phase::Menu | Phase::Paused)
            if page.open && category == page.category =>
        {
            edits.restore = Some(category);
        }
        (UiAction::BuildNotes, Phase::Menu) if page.open => {
            page.open = false;
            armory.open_page(super::armory::Page::Notes);
        }
        (UiAction::Toggle(knob), Phase::Paused | Phase::Menu) if page.open => {
            edits.toggle.push(knob);
        }
        _ => {}
    }
}

pub(super) fn close_settings(mut page: ResMut<PausePage>) {
    page.open = false;
}

pub(super) fn apply_settings(
    mut edits: ResMut<SettingEdits>,
    mut camera: ResMut<super::camera::CameraSettings>,
    mut cvars: ResMut<crate::cvars::Cvars>,
) {
    if edits.set.is_empty() && edits.toggle.is_empty() && edits.restore.is_none() {
        return;
    }
    for (knob, position) in edits.set.drain(..) {
        if let Some(value) = settings::sliders::speed_value(knob, position) {
            camera.set_speed(knob == Knob::ZoomSpeed, value);
        } else if !matches!(knob, Knob::CameraSpeed | Knob::ZoomSpeed) {
            settings::sliders::apply(&mut cvars, knob, position);
        }
    }
    for knob in edits.toggle.drain(..) {
        if knob == Knob::AutoPush {
            camera.toggle();
        } else if knob == Knob::Cutaway {
            camera.toggle_cutaway();
        } else if matches!(knob, Knob::InvertX | Knob::InvertY) {
            camera.toggle_inversion(knob == Knob::InvertX);
        } else {
            toggle_knob(&mut cvars, knob);
        }
    }
    if let Some(category) = edits.restore.take() {
        settings::restore_cvars(&mut cvars, category);
        if category == settings::Category::Graphics {
            camera.restore_defaults();
        } else if category == settings::Category::Controls {
            camera.restore_controls();
        }
    }
}

pub(super) fn sync_pause_page(
    page: Res<PausePage>,
    mut main: Query<&mut Node, With<PauseMain>>,
    mut settings: Query<&mut Node, (With<SettingsPanel>, Without<PauseMain>)>,
    categories: settings::Categories,
) {
    categories.sync(&page);
    let main_display = if page.open {
        Display::None
    } else {
        Display::Flex
    };
    let settings_display = if page.open {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut main {
        if node.display != main_display {
            node.display = main_display;
        }
    }
    for mut node in &mut settings {
        if node.display != settings_display {
            node.display = settings_display;
        }
    }
}

fn toggle_knob(cvars: &mut crate::cvars::Cvars, knob: Knob) {
    let name = match knob {
        Knob::Sound => "MasterSoundEffects",
        Knob::MusicOn => "EnableMusic",
        _ => return,
    };
    let on = cvars.num(name).unwrap_or(1.0) != 0.0;
    cvars.set(name, if on { "0" } else { "1" });
}

pub(super) fn refresh_meter(
    phase: Res<State<Phase>>,
    run: Res<Run>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut root: Query<&mut Visibility, With<MeterRoot>>,
    mut head: Query<&mut Text, With<MeterHead>>,
    mut lines: Query<(&MeterLine, &mut Visibility), Without<MeterRoot>>,
    mut names: Query<(&MeterName, &mut Text), Without<MeterHead>>,
    mut values: Query<(&MeterValue, &mut Text), MeterValueFilter>,
    mut fills: Query<(&MeterFill, &mut Node)>,
) {
    let show = *phase.get() == Phase::Playing || *phase.get() == Phase::Paused;
    for mut vis in &mut root {
        let want = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
    if !show {
        return;
    }
    let mut rows: Vec<(&str, f64)> = run.damage_sources.clone();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let top = rows.first().map(|r| r.1).unwrap_or(1.0).max(1.0);
    let dps = run.damage_done / f64::from(run.elapsed.max(1.0));
    if let Ok(mut t) = head.single_mut() {
        let want = format!("Damage  {}   {}/s", compact(run.damage_done), compact(dps));
        if **t != want {
            **t = want;
        }
    }
    let spells = spells.as_deref();
    for (line, mut vis) in &mut lines {
        let on = (line.0 as usize) < rows.len();
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
    for (name, mut text) in &mut names {
        let Some((key, _)) = rows.get(name.0 as usize) else {
            continue;
        };
        let want = source_name(key, spells);
        if **text != want {
            **text = want;
        }
    }
    for (value, mut text) in &mut values {
        let Some((_, amount)) = rows.get(value.0 as usize) else {
            continue;
        };
        let share = if run.damage_done > 0.0 {
            amount / run.damage_done * 100.0
        } else {
            0.0
        };
        let want = format!("{}  {}%", compact(*amount), share.round() as u32);
        if **text != want {
            **text = want;
        }
    }
    for (fill, mut node) in &mut fills {
        let pct = rows
            .get(fill.0 as usize)
            .map(|(_, amount)| (*amount / top * 100.0) as f32)
            .unwrap_or(0.0);
        node.width = Val::Percent(pct.clamp(0.0, 100.0));
    }
}

fn source_name(key: &str, spells: Option<&crate::ui_action::Spells>) -> String {
    data::ABILITIES
        .iter()
        .find(|a| a.key == key)
        .map(|a| spell_name(spells, a.spell, a.key))
        .unwrap_or_else(|| key.replace('_', " "))
}

pub(super) fn keys(
    focus: Option<Res<super::armory::controller::ControllerFocus>>,
    mut settings: ResMut<PausePage>,
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<State<Phase>>,
    renown: Res<super::renown::Renown>,
    stash: Res<super::stash::Stash>,
    account: Res<super::account::Account>,
    typing: Res<super::Typing>,
    mut armory: ResMut<super::armory::Armory>,
    mut selection: ResMut<Selection>,
    mut run: ResMut<Run>,
    mut next: ResMut<NextState<Phase>>,
) {
    if focus.is_some_and(|focus| focus.keyboard_selected) {
        return;
    }
    if settings.open {
        if keys.just_pressed(KeyCode::Escape) {
            settings.open = false;
        }
        return;
    }
    match phase.get() {
        Phase::Menu => {
            let ranks = selection.class.map(|i| renown.ranks(i)).unwrap_or([0; 3]);
            let class = selection.class.unwrap_or(0);
            let hero = selection.character;
            if !typing.0 && keys.just_pressed(KeyCode::Enter) {
                if let Some(reason) =
                    super::armory::enter_refused(&account, &selection, armory.entered_zone())
                {
                    armory.note = reason;
                } else if start_run(
                    &mut run,
                    &mut selection,
                    ranks,
                    stash.gear(),
                    account.bonus_picked(class, hero),
                ) {
                    run.finish_start(account.perks, account.milestones_picked(class, hero));
                    next.set(Phase::Playing);
                }
            }
        }
        Phase::Playing => {
            if keys.just_pressed(KeyCode::Escape) {
                next.set(Phase::Paused);
            }
        }
        Phase::Paused => {
            if keys.just_pressed(KeyCode::Escape) {
                next.set(Phase::Playing);
            }
        }
        Phase::LevelUp => {}
        Phase::GameOver => {
            if keys.just_pressed(KeyCode::Enter) {
                next.set(Phase::Menu);
            }
        }
    }
}

/// The class's looks once the creature data is in, defaulting to the first. The auto-runner picks
/// a class with no look; the menu itself always sets both.
pub(super) fn refresh_menu(
    mut selection: ResMut<Selection>,
    creatures: Option<Res<crate::entities::Creatures>>,
) {
    if let (Some(class), Some(creatures)) = (selection.class_def(), creatures.as_deref()) {
        if selection.looks.is_empty() {
            let looks = looks_for(class, creatures);
            if !looks.is_empty() {
                selection.looks = looks;
                selection.look = Some(0);
                selection.version += 1;
            }
        }
    }
}

/// Art sources stay typed: item display IDs must never be resolved as spell IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CardIcon {
    Spell(u32),
    Item(u32),
    Missing,
}

/// What a level-up card shows.
pub(super) struct CardText {
    pub icon: CardIcon,
    pub name: String,
    pub tag: String,
    pub tag_color: Color,
    pub intro: String,
    /// `(rank now, top rank)`, for the pips; `None` for the healthstone.
    pub ranks: Option<(u8, u8)>,
}

pub(super) fn describe_card(
    choice: Choice,
    run: &Run,
    spells: Option<&crate::ui_action::Spells>,
) -> CardText {
    let new = Color::srgb(0.3, 1.0, 0.3);
    match choice {
        Choice::NewAbility(def) => CardText {
            icon: CardIcon::Spell(def.spell),
            name: spell_name(spells, def.spell, def.key),
            tag: "New spell".into(),
            tag_color: new,
            intro: String::new(),
            ranks: Some((0, data::MAX_ABILITY_LEVEL)),
        },
        Choice::Upgrade(k) => {
            let o = &run.abilities[k];
            let next = o.level + 1;
            CardText {
                icon: CardIcon::Spell(o.def.spell),
                name: spell_name(spells, o.def.spell, o.def.key),
                tag: format!("Rank {} » {next}", o.level),
                tag_color: Color::WHITE,
                intro: String::new(),
                ranks: Some((o.level, data::MAX_ABILITY_LEVEL)),
            }
        }
        Choice::NewPassive(p) => CardText {
            icon: CardIcon::Spell(p.spell),
            name: p.name.into(),
            tag: "New talent".into(),
            tag_color: new,
            intro: String::new(),
            ranks: Some((0, data::MAX_PASSIVE_LEVEL)),
        },
        Choice::UpPassive(k) => {
            let (p, level) = run.passives[k];
            CardText {
                icon: CardIcon::Spell(p.spell),
                name: p.name.into(),
                tag: format!("Rank {level} » {}", level + 1),
                tag_color: Color::WHITE,
                intro: String::new(),
                ranks: Some((level, data::MAX_PASSIVE_LEVEL)),
            }
        }
        Choice::Heal => CardText {
            icon: CardIcon::Spell(5720),
            name: "Healthstone".into(),
            tag: "Restore".into(),
            tag_color: Color::WHITE,
            intro: String::new(),
            ranks: None,
        },
        Choice::Jewel(kind) => jewel_card(kind, spells),
    }
}

fn jewel_card(
    kind: super::campaign::JewelKind,
    spells: Option<&crate::ui_action::Spells>,
) -> CardText {
    use super::campaign::{JewelKind, AMULET_NAMES, RING_NAMES, TRINKET_NAMES};
    let (icon, name, tag, desc) = match kind {
        JewelKind::Ring(i) => (
            CardIcon::Missing,
            RING_NAMES
                .get(i as usize)
                .copied()
                .unwrap_or("Ring")
                .to_string(),
            "Ring",
            "Worn for this run. A second ring fills the other hand.",
        ),
        JewelKind::Trinket(i) => (
            CardIcon::Missing,
            TRINKET_NAMES
                .get(i as usize)
                .copied()
                .unwrap_or("Trinket")
                .to_string(),
            "Trinket",
            "Worn for this run. Two trinkets can be carried.",
        ),
        JewelKind::Amulet(i) => (
            CardIcon::Missing,
            AMULET_NAMES
                .get(i as usize)
                .copied()
                .unwrap_or("Amulet")
                .to_string(),
            "Amulet",
            "Worn for this run. One amulet.",
        ),
        JewelKind::Legend(key) => {
            let def = data::ABILITIES.iter().find(|def| def.key == key);
            (
                def.map_or(CardIcon::Missing, |def| CardIcon::Spell(def.spell)),
                def.map_or_else(
                    || super::campaign::legendary_amulet(key),
                    |def| format!("Amulet of {}", spell_name(spells, def.spell, def.key)),
                ),
                "Legendary amulet",
                "Seen once dealt, whether or not you take it. One legendary amulet for this run.",
            )
        }
    };
    let icon = if matches!(kind, JewelKind::Legend(_)) {
        icon
    } else {
        super::jewel_icons::display(&name).map_or(CardIcon::Missing, CardIcon::Item)
    };
    CardText {
        icon,
        name,
        tag: tag.into(),
        tag_color: GOLD,
        intro: desc.into(),
        ranks: None,
    }
}

fn reactive_buff_status(run: &Run) -> String {
    if run.dead_t.is_some() || run.hp <= 0.0 {
        return String::new();
    }
    let mut active = Vec::new();
    let deterrence_remaining = run.milestones.deterrence_remaining(run.elapsed);
    if deterrence_remaining > 0.0 {
        active.push(format!(
            "Deterrence: 50% less incoming damage ({:.0}s)",
            deterrence_remaining.ceil()
        ));
    }
    let rush_remaining = run.adrenaline_rush.remaining(run.milestones, run.elapsed);
    if rush_remaining > 0.0 {
        active.push(format!(
            "Adrenaline Rush: 50% shorter Rogue cast cooldown ({:.0}s)",
            rush_remaining.ceil()
        ));
    }
    if let Some((bonus, criticals)) = run.combustion.progress(run.milestones, run.elapsed) {
        active.push(format!(
            "Combustion +{:.0}% Fire critical chance ({criticals}/3 critical hits)",
            bonus * 100.0
        ));
    }
    let power_remaining = run.arcane_power.remaining(run.milestones, run.elapsed);
    if power_remaining > 0.0 {
        active.push(format!(
            "Arcane Power +30% Mage damage ({:.0}s)",
            power_remaining.ceil()
        ));
    }
    let infusion_remaining = run.power_infusion.remaining(run.milestones, run.elapsed);
    if infusion_remaining > 0.0 {
        active.push(format!(
            "Power Infusion +20% Priest damage/healing ({:.0}s)",
            infusion_remaining.ceil()
        ));
    }
    let inspiration = run
        .milestones
        .inspiration_multiplier(run.inspiration_t > 0.)
        - 1.;
    if inspiration > 0. {
        active.push(format!(
            "Inspiration +{:.0}% armor ({:.0}s)",
            inspiration * 100.,
            run.inspiration_t.ceil()
        ));
    }
    let ancestral = run
        .milestones
        .ancestral_healing_armor(run.ancestral_healing_t > 0.);
    if ancestral > 0. {
        active.push(format!(
            "Ancestral Healing +{ancestral:.0} armor ({:.0}s)",
            run.ancestral_healing_t.ceil()
        ));
    }
    let enrage = run.milestones.enrage_damage(run.enrage_t > 0.0) - 1.0;
    if enrage > 0.0 {
        active.push(format!(
            "Enrage +{:.0}% damage ({:.0}s)",
            enrage * 100.0,
            run.enrage_t.ceil()
        ));
    }
    let flurry = run.milestones.flurry_haste(run.flurry_t > 0.0);
    if flurry > 0.0 {
        active.push(format!(
            "Flurry +{:.0}% haste ({:.0}s)",
            flurry * 100.0,
            run.flurry_t.ceil()
        ));
    }
    let vengeance = run
        .milestones
        .vengeance_bonus(run.alive() && run.vengeance_t > 0.0);
    if vengeance > 0.0 {
        active.push(format!(
            "Vengeance +{:.0}% Paladin Holy damage ({:.0}s)",
            vengeance * 100.0,
            run.vengeance_t.ceil()
        ));
    }
    let recovery = run.blessed_recovery.remaining(run.milestones);
    if recovery > 0. {
        active.push(format!("Blessed Recovery ({:.0}s)", recovery.ceil()));
    }
    let healing = run.milestones.blood_craze_fraction();
    if healing > 0.0 && run.blood_craze_t > 0.0 {
        active.push(format!(
            "Blood Craze +{:.2}% health/sec ({:.0}s)",
            healing * 100.0 / 6.0,
            run.blood_craze_t.ceil()
        ));
    }
    let regen = run.milestones.spirit_tap_regen();
    if regen > 0.0 && run.spirit_tap_t > 0.0 {
        active.push(format!(
            "Spirit Tap +{:.0}% health/sec ({:.0}s)",
            regen * 100.0,
            run.spirit_tap_t.ceil()
        ));
    }
    active.join("\n")
}

#[cfg(test)]
mod reactive_buff_tests {
    use super::*;

    #[test]
    fn inspiration_status_expires_on_refund_and_death() {
        let mut run = Run {
            inspiration_t: 14.2,
            milestones: survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(1, 361, 3)]),
            ..Default::default()
        };
        assert_eq!(reactive_buff_status(&run), "Inspiration +24% armor (15s)");
        run.inspiration_t = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.inspiration_t = 8.;
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 10.;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
    }

    #[test]
    fn spirit_tap_status_reports_regeneration_and_expiry() {
        let mut run = Run {
            spirit_tap_t: 14.2,
            milestones: survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(2, 465, 1)]),
            ..Default::default()
        };
        assert_eq!(
            reactive_buff_status(&run),
            "Spirit Tap +1% health/sec (15s)"
        );
        run.spirit_tap_t = 0.0;
        assert!(reactive_buff_status(&run).is_empty());
        run.spirit_tap_t = 10.0;
        run.hp = 0.0;
        assert!(reactive_buff_status(&run).is_empty());
    }

    #[test]
    fn active_talents_show_effects_and_remaining_seconds_only_while_alive() {
        let mut run = Run {
            milestones: survivors_core::talents::Milestones::default().with_ranks(&[
                (1, 155, 5),
                (1, 156, 5),
                (1, 661, 3),
            ]),
            ..Default::default()
        };
        assert!(reactive_buff_status(&run).is_empty());
        run.enrage_t = 3.2;
        run.flurry_t = 2.1;
        run.blood_craze_t = 5.1;
        assert_eq!(
            reactive_buff_status(&run),
            "Enrage +20% damage (4s)\nFlurry +25% haste (3s)\nBlood Craze +0.50% health/sec (6s)"
        );
        run.flurry_t = 0.;
        assert!(!reactive_buff_status(&run).contains("Flurry"));
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 10.;
        run.dead_t = Some(0.);
        assert!(reactive_buff_status(&run).is_empty());
        run.dead_t = None;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
        run.milestones = survivors_core::talents::Milestones {
            class: 5,
            ..Default::default()
        }
        .with_ranks(&[(1, 602, 5)]);
        run.flurry_t = 2.1;
        assert_eq!(reactive_buff_status(&run), "Flurry +25% haste (3s)");
        run.flurry_t = 0.;
        assert!(reactive_buff_status(&run).is_empty());
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn refresh_hud(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<State<Phase>>,
    mut run: ResMut<Run>,
    selection: Res<Selection>,
    renown: Res<super::renown::Renown>,
    stash: Res<super::stash::Stash>,
    font: Res<HudFont>,
    mut texts: ParamSet<(
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<TimerText>>,
        Query<&mut Text, With<KillText>>,
        Query<&mut Text, With<BannerText>>,
        Query<&mut Text, With<OverTitle>>,
        Query<&mut Text, With<OverStats>>,
        Query<&mut Text, With<XpText>>,
        Query<&mut Text, With<ReactiveBuffText>>,
    )>,
    mut xp_fill: Query<&mut Node, With<XpFill>>,
    rows: Query<(Entity, Has<AbilityRow>), Or<(With<AbilityRow>, With<PassiveRow>)>>,
    mut sweeps: ResMut<Assets<SweepUiMaterial>>,
    mut last_version: Local<Option<u32>>,
) {
    let need = data::xp_to_next(run.level);
    if let Ok(mut n) = xp_fill.single_mut() {
        n.width = Val::Percent((run.xp / need * 100.0).clamp(0.0, 100.0));
    }
    if let Ok(mut t) = texts.p6().single_mut() {
        let xp = format!("XP {} / {}", run.xp as u32, need as u32);
        if **t != xp {
            **t = xp;
        }
    }
    if let Ok(mut t) = texts.p0().single_mut() {
        **t = format!("Level {}", run.level);
    }
    let secs = run.elapsed as u32;
    if let Ok(mut t) = texts.p1().single_mut() {
        **t = format!("{:02}:{:02}", secs / 60, secs % 60);
    }
    if let Ok(mut text) = texts.p7().single_mut() {
        let next = reactive_buff_status(&run);
        if text.0 != next {
            text.0 = next;
        }
    }
    let kills = run.kills;
    if let Ok(mut t) = texts.p2().single_mut() {
        **t = format!("Kills {kills}    Gold {}", stash.gold);
    }
    let banner = match run.banner.as_mut() {
        Some((msg, t)) => {
            *t -= time.delta_secs();
            if *t > 0.0 {
                msg.clone()
            } else {
                String::new()
            }
        }
        None => String::new(),
    };
    if banner.is_empty() {
        run.banner = None;
    }
    // The picker owns the screen: an announcement waits until it closes.
    let banner = if *phase.get() == Phase::LevelUp {
        String::new()
    } else {
        banner
    };
    if let Ok(mut t) = texts.p3().single_mut() {
        if **t != banner {
            **t = banner;
        }
    }
    let title = if run.won {
        "Victory!"
    } else {
        "You have fallen"
    }
    .to_string();
    if let Ok(mut t) = texts.p4().single_mut() {
        **t = title;
    }
    let summary = format!(
        "{} {} in {}   —   {} {}   —   {} gold   —   press Enter to play again",
        crate::char_select::race_name(run.race),
        run.class.name,
        selection.map_def().name,
        renown.last_earned,
        if renown.last_earned == 1 {
            "mark"
        } else {
            "marks"
        },
        run.gold_found,
    );
    if let Ok(mut t) = texts.p5().single_mut() {
        if **t != summary {
            **t = summary;
        }
    }

    // The ability and passive icons, rebuilt when the build changes.
    if *last_version == Some(run.version) {
        return;
    }
    *last_version = Some(run.version);
    let f = &font.0;
    for (row, is_abilities) in &rows {
        commands.entity(row).despawn_related::<Children>();
        commands.entity(row).with_children(|row| {
            let entries: Vec<(u32, u8, f32, TipFor)> = if is_abilities {
                run.abilities
                    .iter()
                    .enumerate()
                    .map(|(i, o)| (o.def.spell, o.level, 40.0, TipFor::Ability(i)))
                    .collect()
            } else {
                run.passives
                    .iter()
                    .enumerate()
                    .map(|(i, (p, l))| (p.spell, *l, 30.0, TipFor::Passive(i)))
                    .collect()
            };
            for (spell, level, size, tip) in entries {
                let ability = match &tip {
                    TipFor::Ability(i) => Some(*i),
                    _ => None,
                };
                row.spawn((
                    Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                    Interaction::default(),
                    tip,
                ))
                .with_children(|slot| {
                    slot.spawn(icon(spell, size - 2.0));
                    // The cooldown sweep sits over the icon, under the level text.
                    if let Some(i) = ability {
                        let handle = sweeps.add(SweepUiMaterial { sweep: 0.0 });
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            MaterialNode(handle.clone()),
                            AbilitySweep { handle, index: i },
                        ));
                    }
                    slot.spawn(Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(2.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|n| {
                        n.spawn(text(f, level.to_string(), 12.0, Color::WHITE));
                    });
                });
            }
        });
    }
}

/// Fill icon slots from `SpellIcon.dbc` once the client data and the asset reader are up.
pub(super) fn fill_icons(
    mut commands: Commands,
    mut slots: Query<(Entity, &mut ImageNode, &SpellIcon, Option<&IconDim>), Without<IconDone>>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<u32, Handle<Image>>>,
) {
    let (Some(spells), Some(assets)) = (spells.as_deref(), assets.as_deref_mut()) else {
        return;
    };
    for (e, mut img, icon, dim) in &mut slots {
        let handle = match cache.get(&icon.0) {
            Some(h) => Some(h.clone()),
            None => {
                let path = spells
                    .catalog
                    .get(icon.0)
                    .and_then(|d| d.icon.clone())
                    .unwrap_or_else(|| "Interface\\Icons\\INV_Misc_QuestionMark".into());
                let h = assets.sprite_texture(&format!("{path}.blp"), &mut images);
                if let Some(h) = &h {
                    cache.insert(icon.0, h.clone());
                }
                h
            }
        };
        if let Some(h) = handle {
            img.image = h;
            let alpha = dim.map(|d| d.0).unwrap_or(1.0).clamp(0.0, 1.0);
            img.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        }
        commands.entity(e).insert(IconDone);
    }
}

/// The mobility button: shown once learned, its icon the class's spell, its sweep the cooldown.
#[allow(clippy::type_complexity)]
pub(super) fn refresh_mobility(
    run: Res<Run>,
    mut commands: Commands,
    mut slot: Query<&mut Node, With<MobilitySlot>>,
    icon: Query<(Entity, Option<&SpellIcon>), With<MobilityIcon>>,
    sweep: Query<&MobilitySweep>,
    mut mats: ResMut<Assets<SweepUiMaterial>>,
) {
    let def = run.mobility();
    if let Ok(mut node) = slot.single_mut() {
        let display = if def.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let Some(def) = def else { return };
    // A new spell re-arms the icon fill ([`fill_icons`]).
    if let Ok((e, have)) = icon.single() {
        if have.is_none_or(|s| s.0 != def.spell) {
            commands
                .entity(e)
                .remove::<IconDone>()
                .insert(SpellIcon(def.spell));
        }
    }
    if let Ok(s) = sweep.single() {
        let f = (run.mobile.cd / def.cooldown.max(0.01)).clamp(0.0, 1.0);
        if let Some(m) = mats.get_mut(&s.0) {
            if m.sweep != f {
                m.sweep = f;
            }
        }
    }
}

/// Drive each ability icon's cooldown sweep from its remaining cooldown.
pub(super) fn update_cooldown_sweeps(
    run: Res<Run>,
    mut mats: ResMut<Assets<SweepUiMaterial>>,
    sweeps: Query<&AbilitySweep>,
) {
    let stats = run.stats();
    for s in &sweeps {
        let Some(o) = run.abilities.get(s.index) else {
            continue;
        };
        let total = o.cooldown(&stats).max(0.01);
        let f = (o.cd / total).clamp(0.0, 1.0);
        if let Some(m) = mats.get_mut(&s.handle) {
            m.sweep = f;
        }
    }
}

/// Draw each [`UiArt`] region once its file can be read.
pub(super) fn fill_art(
    mut commands: Commands,
    mut regions: Query<(Entity, &mut ImageNode, &UiArt), Without<ArtDone>>,
    assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut assets) = assets else { return };
    for (e, mut img, art) in &mut regions {
        commands.entity(e).insert(ArtDone);
        let Some(h) = assets.sprite_texture(art.path, &mut images) else {
            warn!("survivors: no UI art {}", art.path);
            continue;
        };
        let [l, r, t, b] = art.coords;
        if let Some(size) = images.get(&h).map(Image::size_f32) {
            img.rect = Some(Rect::new(l * size.x, t * size.y, r * size.x, b * size.y));
        }
        img.image = h;
        img.color = art.tint;
    }
}

/// Height-relative sizing, with an optional persistent readability multiplier.
pub(super) fn apply_ui_scale(
    cvars: Res<crate::cvars::Cvars>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let multiplier = if cvars.num("useUiScale").unwrap_or(0.) != 0. {
        cvars
            .num("uiScale")
            .filter(|v| v.is_finite())
            .unwrap_or(1.)
            .clamp(0.75, 1.25)
    } else {
        1.
    };
    let want = window.height() / 768.0 * multiplier;
    if want > 0.0 && (scale.0 - want).abs() > 1e-4 {
        scale.0 = want;
    }
}

/// Between the hero's feet and the top of its plate's bar, in gx units.
const PLATE_FEET_GAP: f32 = 0.004;
/// A couple of screen pixels lower than the gap alone sets it.
const PLATE_NUDGE: f32 = 2.0;

/// The hero's nameplate under its feet, sized from the screen diagonal as [`crate::vplates`]
/// sizes a plate, outside the UI scale. The border's top half seats a name the hero has none of,
/// so the plate rises until its bar sits just under the feet.
#[allow(clippy::type_complexity)]
pub(super) fn hero_plate(
    run: Res<Run>,
    ui_scale: Res<UiScale>,
    hero: Query<&Transform, With<Hero>>,
    cams: Query<(&Camera, &Transform), With<WorldCamera>>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut Visibility), With<HpFrame>>,
        Query<&mut Node, With<HpBar>>,
        Query<&mut Node, With<HpFill>>,
        Query<&mut Node, With<ShieldFill>>,
        Query<&mut Node, With<HpLevelSeat>>,
    )>,
    mut level: Query<(&mut Text, &mut TextFont), With<HpLevel>>,
) {
    use crate::vplates::{
        gx_px, plate_basis, text_px, BAR_H, BAR_OFF_X, BAR_OFF_Y, BAR_W, LEVEL_H, LEVEL_OFF_X,
        LEVEL_OFF_Y, PLATE_H, PLATE_W,
    };
    let seat = hero.single().ok().and_then(|h| {
        let (cam, tf) = cams.single().ok()?;
        let viewport = cam.logical_viewport_size()?;
        let feet = cam
            .world_to_viewport(&GlobalTransform::from(*tf), h.translation)
            .ok()?;
        Some((feet, viewport))
    });
    let mut frames = nodes.p0();
    let Ok((mut frame, mut vis)) = frames.single_mut() else {
        return;
    };
    let Some((feet, viewport)) = seat else {
        *vis = Visibility::Hidden;
        return;
    };
    *vis = Visibility::Inherited;
    let s = ui_scale.0;
    let basis = plate_basis(viewport);
    let gx = |v: f32| gx_px(v, basis) / s;
    let (pw, ph) = (gx(PLATE_W), gx(PLATE_H));
    let bar_top = ph - gx(BAR_OFF_Y) - gx(BAR_H);
    frame.width = Val::Px(pw);
    frame.height = Val::Px(ph);
    frame.left = Val::Px(feet.x / s - pw / 2.0);
    frame.top = Val::Px(feet.y / s + gx(PLATE_FEET_GAP) - bar_top + PLATE_NUDGE / s);
    if let Ok(mut bar) = nodes.p1().single_mut() {
        bar.left = Val::Px(gx(BAR_OFF_X));
        bar.top = Val::Px(bar_top);
        bar.width = Val::Px(gx(BAR_W));
        bar.height = Val::Px(gx(BAR_H));
    }
    let stats = run.stats();
    let hp = (run.hp / stats.max_hp).clamp(0.0, 1.0);
    let shield = if run.shield_t > 0.0 {
        (run.shield / stats.max_hp).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if let Ok(mut n) = nodes.p2().single_mut() {
        n.width = Val::Percent(hp * 100.0);
    }
    if let Ok(mut n) = nodes.p3().single_mut() {
        n.width = Val::Percent(shield * 100.0);
    }
    if let Ok(mut seat) = nodes.p4().single_mut() {
        seat.left = Val::Px(pw - gx(LEVEL_OFF_X));
        seat.top = Val::Px(ph - gx(LEVEL_OFF_Y));
    }
    if let Ok((mut text, mut font)) = level.single_mut() {
        let n = run.level.to_string();
        if text.0 != n {
            text.0 = n;
        }
        let size = text_px(LEVEL_H, basis) / s;
        if font.font_size != size {
            font.font_size = size;
        }
    }
}

/// `MainMenuExpBar`'s `OnEnter` and `OnLeave`: the numbers show while the cursor is on the bar.
pub(super) fn xp_hover(
    bars: Query<&Interaction, (With<XpBar>, Changed<Interaction>)>,
    mut texts: Query<&mut Visibility, With<XpText>>,
) {
    for interaction in &bars {
        let want = if *interaction == Interaction::None {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for mut v in &mut texts {
            *v = want;
        }
    }
}

// ── The build icons' `GameTooltip` ──────────────────────────────────────────────────────────────

/// `TOOLTIP_DEFAULT_BACKGROUND_COLOR`, the `GameTooltip` backdrop tint (`GameTooltip.lua`).
const TIP_FILL: Color = Color::srgb(0.09, 0.09, 0.19);
/// The grey a spell tooltip prints its rank in, right of the name.
const TIP_GREY: Color = Color::srgb(0.5, 0.5, 0.5);

/// One tooltip line: the left text, an optional right-aligned one, the colour and font size
/// (`GameTooltipHeaderText` 14, `GameTooltipText` 12).
struct TipLine {
    left: String,
    right: Option<(String, Color)>,
    color: Color,
    size: f32,
}

impl TipLine {
    fn new(left: impl Into<String>, color: Color) -> Self {
        Self {
            left: left.into(),
            right: None,
            color,
            size: 12.0,
        }
    }

    fn right(mut self, text: impl Into<String>, color: Color) -> Self {
        self.right = Some((text.into(), color));
        self
    }

    fn header(mut self) -> Self {
        self.size = 14.0;
        self
    }
}

/// The results screen's tiles and its line under the title.
#[allow(clippy::type_complexity)]
pub(super) fn refresh_results(
    run: Res<Run>,
    selection: Res<Selection>,
    mut over_texts: ParamSet<(
        Query<(&OverStat, &mut Text)>,
        Query<&mut Text, With<OverLine>>,
    )>,
) {
    let secs = run.elapsed as u32;
    let stats_line = if run.won {
        selection.map_def().victory
    } else {
        "The horde was too much this time."
    };
    let values = [
        format!("{:02}:{:02}", secs / 60, secs % 60),
        run.level.to_string(),
        run.kills.to_string(),
        compact(run.damage_done),
        run.bosses.to_string(),
    ];
    for (stat, mut t) in &mut over_texts.p0() {
        if let Some(v) = values.get(stat.0) {
            if **t != *v {
                **t = v.clone();
            }
        }
    }
    if let Ok(mut t) = over_texts.p1().single_mut() {
        if **t != stats_line {
            **t = stats_line.to_string();
        }
    }
}

/// 12.3k, 4.5M.
fn compact(v: f64) -> String {
    if v >= 1e6 {
        format!("{:.1}M", v / 1e6)
    } else if v >= 1e4 {
        format!("{:.1}k", v / 1e3)
    } else {
        format!("{}", v.round() as u64)
    }
}

fn num(v: f32) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

// Small regeneration talents must remain visible rather than rounding back to baseline.
fn regen_num(v: f32) -> String {
    format!("{v:.3}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

fn pct(v: f32) -> String {
    format!("{}%", num(v * 100.0))
}

/// The numbers an ability casts with at a level, as [`super::abilities::cast_abilities`] takes
/// them from [`Owned`].
type Cast = super::abilities::Params;

fn cast_at(
    def: &'static AbilityDef,
    level: u8,
    stats: &Stats,
    milestones: survivors_core::talents::Milestones,
) -> Cast {
    Cast::new(
        &Owned {
            def,
            level,
            cd: 0.0,
        },
        stats,
        milestones,
    )
}

/// Plain tooltips share the card's prose and effective values.
fn ability_body(def: &AbilityDef, c: &Cast) -> String {
    inline::ability(def, c, c)
        .into_iter()
        .map(|(text, _)| text)
        .collect()
}

/// Catalog prose uses rank one without a hero's equipment, training or run upgrades.
pub(super) fn base_ability_body(def: &'static AbilityDef) -> String {
    let base = Run::default();
    let cast = cast_at(def, 1, &base.stats(), base.milestones);
    let body = ability_body(def, &cast);
    if matches!(def.kind, Kind::Aura | Kind::Orbit) {
        body
    } else {
        format!("{body} Recharges in {} sec.", num(cast.cooldown))
    }
}

/// What the next rank changes, at the current stats.
fn next_rank(def: &AbilityDef, now: &Cast, next: &Cast) -> String {
    let what = match def.kind {
        Kind::Shield => "absorbed",
        Kind::Heal => "healing",
        _ => "damage",
    };
    let mut gains = Vec::new();
    if next.dmg > 0.0 && def.kind != Kind::Dot && def.kind != Kind::Immune {
        gains.push(format!("{} {what}", num(next.dmg)));
    }
    if next.dot > 0.0 {
        gains.push(format!("{} damage per second", num(next.dot)));
    }
    if next.count != now.count
        && matches!(
            def.kind,
            Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Dot | Kind::Chain | Kind::Orbit
        )
    {
        gains.push(match def.kind {
            Kind::Chain => format!("{} jumps", next.count),
            _ => format!("{} targets", next.count),
        });
    }
    if next.radius > 0.0 {
        gains.push(format!("{} yd", num(next.radius)));
    }
    if next.duration > 0.0 {
        gains.push(format!("{} sec", num(next.duration)));
    }
    gains.push(format!("{} sec cooldown", num(next.cooldown)));
    gains.join(", ")
}

fn ability_tip(
    o: &Owned,
    milestones: survivors_core::talents::Milestones,
    stats: &Stats,
    spells: Option<&crate::ui_action::Spells>,
) -> Vec<TipLine> {
    let def = o.def;
    let now = cast_at(def, o.level, stats, milestones);
    let mut lines = vec![
        TipLine::new(spell_name(spells, def.spell, def.key), Color::WHITE)
            .header()
            .right(format!("Rank {}", o.level), TIP_GREY),
    ];
    let range = match def.kind {
        Kind::Melee => Some("Melee Range".to_string()),
        Kind::Bolt | Kind::Strike | Kind::Chain | Kind::Dot | Kind::Ground | Kind::Totem => {
            Some(format!("{} yd range", num(now.range)))
        }
        _ => None,
    };
    let mut cast = TipLine::new("Casts itself", Color::WHITE);
    if let Some(range) = range {
        cast = cast.right(range, Color::WHITE);
    }
    lines.push(cast);
    lines.push(
        TipLine::new("Instant", Color::WHITE)
            .right(format!("{} sec cooldown", num(now.cooldown)), Color::WHITE),
    );
    lines.push(TipLine::new(ability_body(def, &now), GOLD));
    for (verb, duration, boss_duration) in [
        (
            "stuns",
            milestones.secondary_stun(def.key, false),
            milestones.secondary_stun(def.key, true),
        ),
        (
            "roots",
            milestones.secondary_root(def.key, false),
            milestones.secondary_root(def.key, true),
        ),
    ] {
        if duration > 0. {
            lines.push(TipLine::new(
                format!(
                    "Also {verb} for {} sec (bosses: {boss_duration:.2} sec).",
                    num(duration)
                ),
                card_growth::GAIN,
            ));
        }
    }
    let remorseless = milestones.remorseless_crit(def.key);
    if remorseless > 0.0 {
        lines.push(TipLine::new(
            format!("Remorseless Attacks: after a kill, the next successful Rogue melee cast gains +{} critical chance; expires after 20 sec.", pct(remorseless)),
            card_growth::GAIN,
        ));
    }
    let shatter = milestones.shatter(def.key, true);
    if shatter > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Shatter: +{} critical chance against frozen enemies.",
                pct(shatter)
            ),
            card_growth::GAIN,
        ));
    }
    let chill = milestones.winters_chill_chance(def.key, now.dmg);
    if chill > 0.0 {
        lines.push(TipLine::new(
            format!("Winter's Chill: {} chance on frost damage to add a 15 sec stack; +2% Frost critical chance against that enemy per stack, up to 10%.", pct(chill)),
            card_growth::GAIN,
        ));
    }
    let impact = milestones.impact_chance(def.key, now.dmg);
    if impact > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Impact: {} chance on fire damage to stun for 1 sec (bosses: 0.3 sec).",
                pct(impact)
            ),
            card_growth::GAIN,
        ));
    }
    let frostbite = milestones.frostbite_chance(def.key, now.dmg);
    if frostbite > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Frostbite: {} chance on frost damage to freeze for 5 sec (bosses: 1.5 sec).",
                pct(frostbite)
            ),
            card_growth::GAIN,
        ));
    }
    let ignite = milestones.ignite(def.key, 1.0, true);
    if ignite > 0.0 {
        lines.push(TipLine::new(
            format!("Ignite: critical hits add {} of their damage to a four-tick burn; unpaid damage carries forward.", pct(ignite)),
            card_growth::GAIN,
        ));
    }
    let effect = milestones.cast(def.key);
    if effect.immune_seconds > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Grants {} sec immunity once per successful cast; preserves longer immunity.",
                num(effect.immune_seconds)
            ),
            card_growth::GAIN,
        ));
    }
    if effect.shield_fraction > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Absorbs {} maximum health for {} sec once per successful cast; does not stack.",
                pct(effect.shield_fraction),
                num(effect.shield_seconds)
            ),
            card_growth::GAIN,
        ));
    }
    if let Some((key, seconds)) = effect.cooldown_advance {
        let target = super::data::ability(key);
        lines.push(TipLine::new(
            format!(
                "Advances {} cooldown by {} sec once per successful cast.",
                spell_name(spells, target.spell, target.key),
                num(seconds)
            ),
            card_growth::GAIN,
        ));
    }
    if effect.storm_fraction > 0.0 {
        lines.push(TipLine::new(
            format!(
                "Divine Storm: {} damage within {} yd of you once per cast.",
                num(now.dmg * effect.storm_fraction),
                num(6.0 * stats.area)
            ),
            GOLD,
        ));
    }
    if !matches!(
        def.kind,
        Kind::Shield | Kind::Immune | Kind::Heal | Kind::Dot
    ) {
        lines.push(TipLine::new(
            format!(
                "{} chance to critically strike for {} damage",
                pct(now.crit),
                pct(now.crit_damage.max(1.0))
            ),
            DIM,
        ));
    }
    if o.level < data::MAX_ABILITY_LEVEL {
        let next = cast_at(def, o.level + 1, stats, milestones);
        lines.push(TipLine::new("Next rank:", Color::WHITE));
        lines.push(TipLine::new(next_rank(def, &now, &next), card_growth::GAIN));
    } else {
        lines.push(TipLine::new("Highest rank", DIM));
    }
    lines
}

/// What a passive's bonus `v` does, in the voice of a buff.
/// A stat's short name, for the card's before » after line.
fn stat_label(stat: Stat) -> &'static str {
    match stat {
        Stat::Might => "All damage",
        Stat::MaxHp => "Maximum health",
        Stat::Speed => "Movement speed",
        Stat::Regen => "Health per second",
        Stat::Haste => "Recharge speed",
        Stat::Area => "Area size",
        Stat::Magnet => "Pickup range",
        Stat::Growth => "Experience gained",
        Stat::Armor => "Damage blocked per hit",
        Stat::Crit => "Critical strike chance",
    }
}

fn passive_text(stat: Stat, v: f32) -> String {
    match stat {
        Stat::Might => format!("Increases all damage you deal by {}.", pct(v)),
        Stat::MaxHp => format!("Increases your maximum health by {}.", pct(v)),
        Stat::Speed => format!("Increases your movement speed by {}.", pct(v)),
        Stat::Regen => format!("Restores {} health every second.", regen_num(v)),
        Stat::Haste => format!("Your abilities recharge {} faster.", pct(v)),
        Stat::Area => format!("Increases the size of your areas of effect by {}.", pct(v)),
        Stat::Magnet => format!("Increases your pickup range by {}.", pct(v)),
        Stat::Growth => format!("Increases the experience you gain by {}.", pct(v)),
        Stat::Armor => format!("Reduces the damage of every hit you take by {}.", num(v)),
        Stat::Crit => format!("Increases your critical strike chance by {}.", pct(v)),
    }
}

/// The hero's total for the stat a passive raises, every source counted.
fn stat_total(stat: Stat, s: &Stats) -> String {
    match stat {
        Stat::Might => format!("Damage dealt: {}", pct(s.might)),
        Stat::MaxHp => format!("Maximum health: {}", num(s.max_hp)),
        Stat::Speed => format!("Movement speed: {} yd per second", num(s.speed)),
        Stat::Regen => format!("Health restored: {} per second", regen_num(s.regen)),
        Stat::Haste => format!("Cooldowns: {} shorter", pct(s.haste.min(0.65))),
        Stat::Area => format!("Area of effect: {}", pct(s.area)),
        Stat::Magnet => format!("Pickup range: {} yd", num(s.magnet)),
        Stat::Growth => format!("Experience gained: {}", pct(s.growth)),
        Stat::Armor => format!("Damage blocked per hit: {}", num(s.armor)),
        Stat::Crit => format!("Critical strike chance: {}", pct(s.crit)),
    }
}

fn passive_tip(p: &PassiveDef, level: u8, stats: &Stats, run: &Run) -> Vec<TipLine> {
    let mut lines = vec![
        TipLine::new(p.name, Color::WHITE)
            .header()
            .right(format!("Rank {level}"), TIP_GREY),
        TipLine::new(passive_text(p.stat, run.passive_value(p, level)), GOLD),
        TipLine::new(stat_total(p.stat, stats), Color::WHITE),
    ];
    if level < data::MAX_PASSIVE_LEVEL {
        lines.push(TipLine::new("Next rank:", Color::WHITE));
        lines.push(TipLine::new(
            passive_text(p.stat, run.passive_value(p, level + 1)),
            card_growth::GAIN,
        ));
    } else {
        lines.push(TipLine::new("Highest rank", DIM));
    }
    lines
}

/// The `GameTooltip` frame: `UI-Tooltip-Background` tiled under `UI-Tooltip-Border`, 16-unit edges
/// and 5-unit insets, its lines 10 units in. Returns the frame and its line column.
fn spawn_tooltip(
    commands: &mut Commands,
    assets: &mut benilla_assets::WorldAssets,
    images: &mut Assets<Image>,
    ui_scale: f32,
) -> (Entity, Entity) {
    use crate::glue::backdrop::{backdrop_border, backdrop_edges, tiled_bg_node};
    let bg = assets.sprite_texture("Interface\\Tooltips\\UI-Tooltip-Background", images);
    let edges = backdrop_edges(assets, "Interface\\Tooltips\\UI-Tooltip-Border", images);
    let mut body = Entity::PLACEHOLDER;
    let mut root = commands.spawn((
        GameTooltip,
        Node {
            position_type: PositionType::Absolute,
            max_width: Val::Px(300.0),
            ..default()
        },
        GlobalZIndex(1000),
        Visibility::Hidden,
        bevy::ui::FocusPolicy::Pass,
    ));
    if bg.is_none() || edges.is_none() {
        root.insert(BackgroundColor(TIP_FILL.with_alpha(0.95)));
    }
    root.with_children(|t| {
        if let Some(bg) = bg {
            t.spawn((
                tiled_bg_node(bg, 16.0, ui_scale, TIP_FILL),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(5.0),
                    bottom: Val::Px(5.0),
                    ..default()
                },
            ));
        }
        if let Some(edges) = &edges {
            backdrop_border(t, edges, 16.0, Color::WHITE);
        }
        body = t
            .spawn((
                TipBody,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
            ))
            .id();
    });
    (root.id(), body)
}

/// Hovering a build icon raises its `GameTooltip` under the icon, every number from the current
/// rank and stats, and what the next rank brings.
#[allow(clippy::too_many_arguments)]
pub(super) fn icon_tooltip(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    run: Res<Run>,
    spells: Option<Res<crate::ui_action::Spells>>,
    font: Res<HudFont>,
    ui_scale: Res<UiScale>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    slots: Query<(
        &Interaction,
        &TipFor,
        &ComputedNode,
        &bevy::ui::UiGlobalTransform,
    )>,
    mut frames: Query<(&mut Node, &mut Visibility), With<GameTooltip>>,
    mut built: Local<Option<(Entity, Entity)>>,
    mut shown: Local<Option<(TipFor, u32)>>,
) {
    let Some((frame, body)) = *built else {
        if let Some(assets) = assets.as_deref_mut() {
            *built = Some(spawn_tooltip(
                &mut commands,
                assets,
                &mut images,
                ui_scale.0,
            ));
        }
        return;
    };
    let Ok((mut node, mut vis)) = frames.get_mut(frame) else {
        return;
    };
    let hovered = slots
        .iter()
        .find(|(i, ..)| **i != Interaction::None)
        .filter(|_| *phase.get() != Phase::Menu);
    let Some((_, &tip, computed, at)) = hovered else {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        *shown = None;
        return;
    };
    // Physical pixels to the 768-unit canvas the `Val`s are on.
    let inv = computed.inverse_scale_factor;
    let (left, top) = (
        Val::Px((at.translation.x - computed.size().x / 2.0) * inv),
        Val::Px((at.translation.y + computed.size().y / 2.0) * inv + 4.0),
    );
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    if *shown == Some((tip, run.version)) {
        if *vis != Visibility::Inherited {
            *vis = Visibility::Inherited;
        }
        return;
    }
    let stats = run.stats();
    let lines = match tip {
        TipFor::Ability(i) => run
            .abilities
            .get(i)
            .map(|o| ability_tip(o, run.milestones, &stats, spells.as_deref())),
        TipFor::Passive(i) => run
            .passives
            .get(i)
            .map(|&(p, level)| passive_tip(p, level, &stats, &run)),
        TipFor::Mobility => run.mobility().map(|m| {
            vec![
                TipLine::new(spell_name(spells.as_deref(), m.spell, m.name), Color::WHITE).header(),
                TipLine::new("Press F", Color::WHITE)
                    .right(format!("{} sec cooldown", num(m.cooldown)), Color::WHITE),
                TipLine::new(m.description(), GOLD),
            ]
        }),
    };
    let Some(lines) = lines else { return };
    *shown = Some((tip, run.version));
    // Shown from the next frame, once the new lines have laid out.
    *vis = Visibility::Hidden;
    let f = &font.0;
    commands.entity(body).despawn_related::<Children>();
    commands.entity(body).with_children(|b| {
        for line in lines {
            let face = |s: String, size: f32, color: Color| {
                (
                    Text::new(s),
                    TextFont {
                        font: f.clone(),
                        font_size: size,
                        ..default()
                    },
                    TextColor(color),
                    TextShadow {
                        offset: Vec2::splat(1.0),
                        color: Color::BLACK,
                    },
                )
            };
            b.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn(face(line.left, line.size, line.color));
                if let Some((text, color)) = line.right {
                    row.spawn((
                        face(text, line.size, color),
                        TextLayout::new(Justify::Right, LineBreak::NoWrap),
                    ));
                }
            });
        }
    });
}

#[allow(clippy::type_complexity)]
pub(super) fn show_panels(
    settings: Res<PausePage>,
    phase: Res<State<Phase>>,
    preview: Res<super::cards::PreviewHand>,
    mut hints: Query<&mut Text, With<LevelHint>>,
    mut roots: ParamSet<(
        Query<(&mut Visibility, &mut Node), With<MenuRoot>>,
        Query<(&mut Visibility, &mut Node), With<HudRoot>>,
        Query<(&mut Visibility, &mut Node), With<LevelUpRoot>>,
        Query<(&mut Visibility, &mut Node), With<PauseRoot>>,
        Query<(&mut Visibility, &mut Node), With<OverRoot>>,
    )>,
) {
    let p = *phase.get();
    let hint = if preview.0 {
        "Preview only · Picks disabled · Scroll to see more"
    } else {
        "Click a card or press its number · Scroll to see more"
    };
    for mut text in &mut hints {
        if text.0 != hint {
            text.0 = hint.into();
        }
    }
    // A hidden panel leaves the layout too: a merely invisible button still takes clicks.
    let set = |(mut v, mut node): (Mut<Visibility>, Mut<Node>), on: bool| {
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    };
    for root in &mut roots.p0() {
        set(root, p == Phase::Menu && !preview.0 && !settings.open);
    }
    for root in &mut roots.p1() {
        set(root, p != Phase::Menu && p != Phase::GameOver);
    }
    for root in &mut roots.p2() {
        set(root, p == Phase::LevelUp || preview.0);
    }
    for root in &mut roots.p3() {
        set(
            root,
            p == Phase::Paused || (p == Phase::Menu && settings.open),
        );
    }
    for root in &mut roots.p4() {
        set(root, p == Phase::GameOver);
    }
}

#[cfg(test)]
mod talent_tooltip_tests {
    use super::*;
    use survivors_core::talents::Milestones;

    #[test]
    fn mage_range_tooltips_use_the_same_trained_range_as_targeting() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        }
        .with_ranks(&[(1, 28, 2), (2, 741, 2)]);
        let stats = Run::default().stats();
        for (key, expected) in [
            ("fireball", "31.2 yd range"),
            ("blizzard", "24 yd range"),
            ("arcane_missiles", "22 yd range"),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, mage, &stats, None);
            assert!(
                tip.iter().any(|line| line
                    .right
                    .as_ref()
                    .is_some_and(|(text, _)| text == expected)),
                "{key}"
            );
        }
    }

    #[test]
    fn nova_tooltip_explains_trained_immunity_and_combined_barrier() {
        let trained = Milestones {
            class: 6,
            points: [0, 0, 30],
            ..Default::default()
        }
        .with_ranks(&[(2, 72, 1), (2, 71, 1)]);
        let owned = Owned {
            def: data::ability("frost_nova"),
            level: 1,
            cd: 0.,
        };
        let tip = ability_tip(&owned, trained, &Run::default().stats(), None);
        for text in [
            "Grants 0.5 sec immunity once per successful cast; preserves longer immunity.",
            "Absorbs 15% maximum health for 5 sec once per successful cast; does not stack.",
        ] {
            assert!(tip
                .iter()
                .any(|line| line.left == text && line.color == card_growth::GAIN));
        }
    }

    #[test]
    fn shatter_tooltip_is_green_and_only_shown_for_trained_mage_spells() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = mage.with_ranks(&[(2, 67, 5)]);
        let stats = Run::default().stats();
        for (key, shown) in [
            ("fireball", true),
            ("frostbolt", true),
            ("blizzard", true),
            ("arcane_missiles", true),
            ("volley", false),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, trained, &stats, None);
            let line = tip.iter().find(|line| line.left.starts_with("Shatter:"));
            assert_eq!(line.is_some(), shown);
            if let Some(line) = line {
                assert!(line.left.contains("50%"));
                assert_eq!(line.color, card_growth::GAIN);
            }
            assert!(!ability_tip(&owned, mage, &stats, None)
                .iter()
                .any(|line| line.left.starts_with("Shatter:")));
        }
    }

    #[test]
    fn impact_tooltip_is_green_and_only_shown_for_trained_fire_spells() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = mage.with_ranks(&[(1, 30, 5)]);
        let stats = Run::default().stats();
        for (key, shown) in [
            ("fireball", true),
            ("frostbolt", false),
            ("flamestrike", true),
            ("arcane_missiles", false),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, trained, &stats, None);
            let line = tip.iter().find(|line| line.left.starts_with("Impact:"));
            assert_eq!(line.is_some(), shown);
            if let Some(line) = line {
                assert!(line.left.contains("10%"));
                assert_eq!(line.color, card_growth::GAIN);
            }
            assert!(!ability_tip(&owned, mage, &stats, None)
                .iter()
                .any(|line| line.left.starts_with("Impact:")));
        }
    }

    #[test]
    fn winters_chill_tooltip_is_green_and_only_shown_for_trained_frost_spells() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = mage.with_ranks(&[(2, 68, 5)]);
        let stats = Run::default().stats();
        for (key, shown) in [
            ("fireball", false),
            ("frostbolt", true),
            ("blizzard", true),
            ("arcane_missiles", false),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, trained, &stats, None);
            let line = tip
                .iter()
                .find(|line| line.left.starts_with("Winter's Chill:"));
            assert_eq!(line.is_some(), shown);
            if let Some(line) = line {
                assert!(line.left.contains("100%"));
                assert_eq!(line.color, card_growth::GAIN);
            }
            assert!(!ability_tip(&owned, mage, &stats, None)
                .iter()
                .any(|line| line.left.starts_with("Winter's Chill:")));
        }
    }

    #[test]
    fn frostbite_tooltip_is_green_and_only_shown_for_trained_frost_spells() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = mage.with_ranks(&[(2, 38, 3)]);
        let stats = Run::default().stats();
        for (key, shown) in [
            ("fireball", false),
            ("frostbolt", true),
            ("blizzard", true),
            ("arcane_missiles", false),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, trained, &stats, None);
            let line = tip.iter().find(|line| line.left.starts_with("Frostbite:"));
            assert_eq!(line.is_some(), shown);
            if let Some(line) = line {
                assert!(line.left.contains("15%"));
                assert_eq!(line.color, card_growth::GAIN);
            }
            assert!(!ability_tip(&owned, mage, &stats, None)
                .iter()
                .any(|line| line.left.starts_with("Frostbite:")));
        }
    }

    #[test]
    fn ignite_tooltip_is_green_and_only_shown_for_trained_fire_spells() {
        let mage = Milestones {
            class: 6,
            ..Default::default()
        };
        let trained = mage.with_ranks(&[(1, 34, 5)]);
        let stats = Run::default().stats();
        for (key, shown) in [
            ("fireball", true),
            ("flamestrike", true),
            ("blizzard", false),
            ("arcane_missiles", false),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            let tip = ability_tip(&owned, trained, &stats, None);
            let line = tip.iter().find(|line| line.left.starts_with("Ignite:"));
            assert_eq!(line.is_some(), shown);
            if let Some(line) = line {
                assert!(line.left.contains("40%"));
                assert_eq!(line.color, card_growth::GAIN);
            }
            assert!(!ability_tip(&owned, mage, &stats, None)
                .iter()
                .any(|line| line.left.starts_with("Ignite:")));
        }
    }

    #[test]
    fn hunter_control_tooltips_show_ordinary_and_boss_durations() {
        let trained = Milestones {
            class: 2,
            ..Default::default()
        }
        .with_ranks(&[(1, 1353, 1), (2, 1304, 5), (2, 1325, 1)]);
        let stats = Run::default().stats();
        for (key, text) in [
            (
                "concussive_shot",
                "Also stuns for 1 sec (bosses: 0.30 sec).",
            ),
            (
                "serpent_sting",
                "Also stuns for 1.5 sec (bosses: 0.45 sec).",
            ),
            ("explosive_trap", "Also roots for 2 sec (bosses: 0.60 sec)."),
        ] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.,
            };
            assert!(ability_tip(&owned, trained, &stats, None)
                .iter()
                .any(|line| line.left == text));
            assert!(!ability_tip(&owned, trained.with_ranks(&[]), &stats, None)
                .iter()
                .any(|line| line.left == text));
        }
    }

    #[test]
    fn critical_tooltip_matches_effective_chance_and_damage() {
        let mut stats = Run::default().stats();
        stats.crit = 0.9;
        let trained = Milestones::default().with_ranks(&[(0, 131, 2)]);
        let owned = Owned {
            def: data::ability("heroic_strike"),
            level: 1,
            cd: 0.0,
        };
        for (multiplier, expected) in [(2.0, "200%"), (2.2, "220%"), (0.5, "100%")] {
            stats.crit_damage = multiplier;
            let lines = ability_tip(&owned, trained, &stats, None);
            assert!(lines.iter().any(|line| line.left
                == format!("100% chance to critically strike for {expected} damage")));
        }
        assert_eq!(
            passive_text(Stat::Crit, 0.05),
            "Increases your critical strike chance by 5%."
        );
    }

    #[test]
    fn shaman_deployment_tooltip_shows_healing_and_counterpart_cooldown() {
        let stats = Run::default().stats();
        let owned = super::super::Owned {
            def: data::ability("searing_totem"),
            level: 1,
            cd: 0.,
        };
        let trained = Milestones {
            class: 5,
            points: [0, 0, 30],
            ..Default::default()
        };
        let lines = ability_tip(&owned, trained, &stats, None);
        assert!(lines.iter().any(|line| line
            .left
            .contains("Heals 3% maximum health once per successful cast.")));
        assert!(lines.iter().any(|line| line.left.contains("Advances")
            && line.left.contains("2 sec once per successful cast")
            && line.color == card_growth::GAIN));
    }

    #[test]
    fn presence_of_mind_tooltip_explains_the_shared_cast_benefit() {
        let stats = Run::default().stats();
        let owned = super::super::Owned {
            def: data::ability("arcane_missiles"),
            level: 1,
            cd: 0.,
        };
        for points in [19, 20, 30] {
            let trained = Milestones {
                class: 6,
                points: [points, 0, 0],
                ..Default::default()
            };
            let lines = ability_tip(&owned, trained, &stats, None);
            let benefit = lines.iter().find(|line| {
                line.left.contains("Advances")
                    && line.left.contains("2 sec once per successful cast")
            });
            assert_eq!(benefit.is_some(), points >= 20);
            if let Some(line) = benefit {
                assert_eq!(line.color, card_growth::GAIN);
            }
        }
    }

    #[test]
    fn effect_description_uses_rank_and_talent_scaled_duration() {
        let stats = Run::default().stats();
        let def = data::ability("demoralizing_shout");
        let trained = Milestones::default().with_ranks(&[(1, 161, 5)]);
        for (rank, talents, expected) in [
            (1, Milestones::default(), "for 5 sec"),
            (1, trained, "for 7.5 sec"),
            (2, trained, "for 8.2 sec"),
        ] {
            let cast = cast_at(def, rank, &stats, talents);
            let body = ability_body(def, &cast);
            assert!(body.contains(expected), "{body}");
        }
    }

    #[test]
    fn healing_tooltip_uses_combined_cast_effects_for_each_spell() {
        let stats = Run::default().stats();
        let trained = Milestones {
            class: 1,
            points: [20, 0, 0],
            ..Default::default()
        }
        .with_ranks(&[(0, 1444, 3), (0, 1443, 2)]);
        for (key, expected) in [("holy_shock", "6%"), ("divine_shield", "10%")] {
            let owned = Owned {
                def: data::ability(key),
                level: 1,
                cd: 0.0,
            };
            let lines = ability_tip(&owned, trained, &stats, None);
            let healing: Vec<_> = lines
                .iter()
                .filter(|line| line.left.contains("Heals "))
                .collect();
            assert_eq!(healing.len(), 1);
            let expected = format!("Heals {expected} maximum health once per successful cast.");
            assert!(healing[0].left.contains(&expected));
            assert_eq!(healing[0].left.matches("Heals ").count(), 1);
            assert!(!ability_tip(&owned, Milestones::default(), &stats, None)
                .iter()
                .any(|line| line.left.contains("Heals ")));
        }
    }
}

#[cfg(test)]
mod card_icon_tests {
    use super::super::campaign::{JewelKind, AMULET_NAMES, RING_NAMES, TRINKET_NAMES};
    use super::*;

    #[test]
    fn mage_can_acquire_and_upgrade_mana_shield_through_normal_offers() {
        let mut run = Run {
            class: &data::CLASSES[6],
            hp: 100.,
            ..Default::default()
        };
        let mut rng = Rng(42);
        let mut offer = Offer {
            choices: choices_for(&run, &mut rng, 100),
            ..Default::default()
        };
        let slot = offer
            .choices
            .iter()
            .position(
                |choice| matches!(choice, Choice::NewAbility(def) if def.key == "mana_shield"),
            )
            .unwrap();
        pick(
            slot,
            &mut run,
            &mut offer,
            &mut rng,
            &mut NextState::default(),
        );
        assert_eq!(run.abilities.len(), 1);
        assert_eq!(
            (run.abilities[0].def.key, run.abilities[0].level),
            ("mana_shield", 1)
        );
        offer.choices = choices_for(&run, &mut rng, 100);
        assert!(!offer
            .choices
            .iter()
            .any(|choice| matches!(choice, Choice::NewAbility(def) if def.key == "mana_shield")));
        let slot = offer
            .choices
            .iter()
            .position(|choice| matches!(choice, Choice::Upgrade(0)))
            .unwrap();
        pick(
            slot,
            &mut run,
            &mut offer,
            &mut rng,
            &mut NextState::default(),
        );
        assert_eq!(run.abilities[0].level, 2);
    }

    #[test]
    fn hunter_can_acquire_and_upgrade_scorpid_sting_through_normal_offers() {
        let mut run = Run {
            class: &data::CLASSES[2],
            hp: 100.,
            ..Default::default()
        };
        let mut rng = Rng(42);
        let mut offer = Offer {
            choices: choices_for(&run, &mut rng, 100),
            ..Default::default()
        };
        let slot = offer
            .choices
            .iter()
            .position(
                |choice| matches!(choice, Choice::NewAbility(def) if def.key == "scorpid_sting"),
            )
            .unwrap();
        pick(
            slot,
            &mut run,
            &mut offer,
            &mut rng,
            &mut NextState::default(),
        );
        assert_eq!(run.abilities.len(), 1);
        assert_eq!(
            (run.abilities[0].def.key, run.abilities[0].level),
            ("scorpid_sting", 1)
        );
        offer.choices = choices_for(&run, &mut rng, 100);
        assert!(!offer
            .choices
            .iter()
            .any(|choice| matches!(choice, Choice::NewAbility(def) if def.key == "scorpid_sting")));
        let slot = offer
            .choices
            .iter()
            .position(|choice| matches!(choice, Choice::Upgrade(0)))
            .unwrap();
        pick(
            slot,
            &mut run,
            &mut offer,
            &mut rng,
            &mut NextState::default(),
        );
        assert_eq!(run.abilities[0].level, 2);
    }

    #[test]
    fn chest_jewelry_resolves_item_art_and_keeps_legend_spell_art() {
        let mut run = Run::default();
        for (names, kind) in [
            (
                RING_NAMES.as_slice(),
                JewelKind::Ring as fn(u8) -> JewelKind,
            ),
            (
                TRINKET_NAMES.as_slice(),
                JewelKind::Trinket as fn(u8) -> JewelKind,
            ),
            (
                AMULET_NAMES.as_slice(),
                JewelKind::Amulet as fn(u8) -> JewelKind,
            ),
        ] {
            for (i, name) in names.iter().enumerate() {
                let card = describe_card(Choice::Jewel(kind(i as u8)), &run, None);
                assert_eq!(card.name, *name);
                assert_eq!(
                    card.icon,
                    CardIcon::Item(super::super::jewel_icons::display(name).unwrap())
                );
            }
        }
        for kind in [
            JewelKind::Ring(255),
            JewelKind::Trinket(255),
            JewelKind::Amulet(255),
        ] {
            assert_eq!(
                describe_card(Choice::Jewel(kind), &run, None).icon,
                CardIcon::Missing
            );
        }
        for def in data::ABILITIES {
            let card = describe_card(
                Choice::Jewel(JewelKind::Legend(def.key)),
                &Run::default(),
                None,
            );
            assert_eq!(card.icon, CardIcon::Spell(def.spell), "{}", def.key);
            assert_eq!(
                card.name,
                format!("Amulet of {}", def.key.replace('_', " "))
            );
        }
        let def = data::ability("cleave");
        run.abilities.push(super::super::Owned {
            def,
            level: 1,
            cd: 0.,
        });
        assert_eq!(
            describe_card(Choice::Jewel(JewelKind::Legend("cleave")), &run, None).icon,
            CardIcon::Spell(def.spell)
        );
        assert_eq!(
            describe_card(Choice::NewAbility(def), &run, None).icon,
            CardIcon::Spell(def.spell)
        );
        assert_eq!(
            describe_card(Choice::Heal, &run, None).icon,
            CardIcon::Spell(5720)
        );
    }
}

#[cfg(test)]
mod settings_navigation_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn settings_consume_enter_and_escape_without_starting_or_resuming_a_run() {
        for phase in [Phase::Menu, Phase::Paused] {
            let mut world = World::new();
            world.insert_resource(State::new(phase));
            world.init_resource::<NextState<Phase>>();
            world.init_resource::<ButtonInput<KeyCode>>();
            world.init_resource::<super::super::renown::Renown>();
            world.init_resource::<super::super::stash::Stash>();
            world.insert_resource(super::super::account::empty());
            world.init_resource::<super::super::Typing>();
            world.init_resource::<super::super::armory::Armory>();
            world.insert_resource(Selection {
                class: Some(0),
                looks: vec![(49, 1, 0)],
                look: Some(0),
                map: data::MAPS
                    .iter()
                    .position(|map| map.name == "Elwynn Forest")
                    .unwrap(),
                ..default()
            });
            world.init_resource::<Run>();
            world.insert_resource(PausePage {
                open: true,
                ..default()
            });
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Enter);
            world.run_system_once(keys).unwrap();
            assert!(world.resource::<PausePage>().open);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::Escape);
            world.run_system_once(keys).unwrap();
            assert!(!world.resource::<PausePage>().open);
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Unchanged
            ));
            world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
            world
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(if phase == Phase::Menu {
                    KeyCode::Enter
                } else {
                    KeyCode::Escape
                });
            world.run_system_once(keys).unwrap();
            assert!(matches!(
                world.resource::<NextState<Phase>>(),
                NextState::Pending(Phase::Playing)
            ));
        }
    }

    #[test]
    fn starter_settings_remove_menu_from_layout_and_back_restores_it() {
        let mut world = World::new();
        world.insert_resource(State::new(Phase::Menu));
        world.insert_resource(PausePage {
            open: true,
            ..default()
        });
        world.init_resource::<super::super::cards::PreviewHand>();
        let menu = world
            .spawn((MenuRoot, Visibility::Inherited, Node::default()))
            .id();
        let pause = world
            .spawn((PauseRoot, Visibility::Hidden, Node::default()))
            .id();
        world.run_system_once(show_panels).unwrap();
        assert_eq!(world.get::<Node>(menu).unwrap().display, Display::None);
        assert_eq!(world.get::<Visibility>(menu), Some(&Visibility::Hidden));
        assert_eq!(world.get::<Node>(pause).unwrap().display, Display::Flex);
        world.resource_mut::<PausePage>().open = false;
        world.run_system_once(show_panels).unwrap();
        assert_eq!(world.get::<Node>(menu).unwrap().display, Display::Flex);
        assert_eq!(world.get::<Node>(pause).unwrap().display, Display::None);
    }
}

#[cfg(test)]
mod ancestral_status_tests {
    use super::*;
    #[test]
    fn ancestral_status_requires_a_living_hero_and_active_training() {
        let mut run = Run {
            hp: 50.,
            ancestral_healing_t: 9.2,
            milestones: survivors_core::talents::Milestones {
                class: 5,
                ..Default::default()
            }
            .with_ranks(&[(2, 581, 2)]),
            ..Default::default()
        };
        assert_eq!(
            reactive_buff_status(&run),
            "Ancestral Healing +24 armor (10s)"
        );
        run.ancestral_healing_t = 0.;
        assert_eq!(reactive_buff_status(&run), "");
        run.ancestral_healing_t = 5.;
        run.hp = 0.;
        assert_eq!(reactive_buff_status(&run), "");
    }
}

#[cfg(test)]
mod arcane_power_status_tests {
    use super::*;
    #[test]
    fn power_status_only_shows_a_trained_living_hero_during_the_burst() {
        let mut run = Run {
            milestones: survivors_core::talents::Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(0, 87, 1)]),
            ..Default::default()
        };
        assert!(reactive_buff_status(&run).is_empty());
        run.arcane_power.consume(run.milestones, "fireball", 0.0);
        run.elapsed = 1.2;
        assert_eq!(
            reactive_buff_status(&run),
            "Arcane Power +30% Mage damage (14s)"
        );
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 100.;
        run.elapsed = 15.;
        assert!(reactive_buff_status(&run).is_empty());
        run.elapsed = 1.;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
    }
}

#[cfg(test)]
mod power_infusion_status_tests {
    use super::*;
    #[test]
    fn infusion_status_expires_and_hides_on_death_or_refund() {
        let mut run = Run {
            milestones: survivors_core::talents::Milestones {
                class: 4,
                ..Default::default()
            }
            .with_ranks(&[(0, 322, 1)]),
            ..Default::default()
        };
        assert!(reactive_buff_status(&run).is_empty());
        run.power_infusion.consume(run.milestones, "smite", 0.);
        run.elapsed = 1.2;
        assert_eq!(
            reactive_buff_status(&run),
            "Power Infusion +20% Priest damage/healing (14s)"
        );
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 100.;
        run.elapsed = 15.;
        assert!(reactive_buff_status(&run).is_empty());
        run.elapsed = 1.;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
    }
}

#[cfg(test)]
mod adrenaline_rush_status_tests {
    use super::*;
    #[test]
    fn active_rush_status_expires_and_hides_on_death_or_refund() {
        let mut run = Run {
            class: &super::super::data::CLASSES[3],
            milestones: survivors_core::talents::Milestones {
                class: 3,
                ..Default::default()
            }
            .with_ranks(&[(1, 205, 1)]),
            ..Default::default()
        };
        assert!(reactive_buff_status(&run).is_empty());
        run.adrenaline_rush
            .consume(run.milestones, "sinister_strike", 0.);
        run.elapsed = 1.2;
        assert_eq!(
            reactive_buff_status(&run),
            "Adrenaline Rush: 50% shorter Rogue cast cooldown (14s)"
        );
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 100.;
        run.elapsed = 15.;
        assert!(reactive_buff_status(&run).is_empty());
        run.elapsed = 1.;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
    }
}

#[cfg(test)]
mod combustion_status_tests {
    use super::*;
    #[test]
    fn combustion_status_tracks_burst_and_hides_on_completion_refund_or_death() {
        let mut run = Run {
            milestones: survivors_core::talents::Milestones {
                class: 6,
                ..Default::default()
            }
            .with_ranks(&[(1, 36, 1)]),
            ..Default::default()
        };
        assert!(reactive_buff_status(&run).is_empty());
        run.combustion
            .hit(run.milestones, "fireball", 10., true, 0.);
        assert_eq!(
            reactive_buff_status(&run),
            "Combustion +20% Fire critical chance (1/3 critical hits)"
        );
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 100.;
        let trained = run.milestones;
        run.milestones = trained.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
        run.milestones = trained;
        run.combustion.hit(trained, "fireball", 10., true, 0.);
        run.combustion.hit(trained, "fireball", 10., true, 0.);
        assert!(reactive_buff_status(&run).is_empty());
    }
}

#[cfg(test)]
mod deterrence_status_tests {
    use super::*;
    #[test]
    fn deterrence_status_tracks_windows_and_hides_on_death_or_refund() {
        let mut run = Run {
            class: &super::super::data::CLASSES[2],
            milestones: survivors_core::talents::Milestones {
                class: 2,
                ..Default::default()
            }
            .with_ranks(&[(2, 1308, 1)]),
            elapsed: 1.2,
            ..Default::default()
        };
        assert_eq!(
            reactive_buff_status(&run),
            "Deterrence: 50% less incoming damage (9s)"
        );
        run.elapsed = 10.;
        assert!(reactive_buff_status(&run).is_empty());
        run.elapsed = 180.;
        assert_eq!(
            reactive_buff_status(&run),
            "Deterrence: 50% less incoming damage (10s)"
        );
        run.hp = 0.;
        assert!(reactive_buff_status(&run).is_empty());
        run.hp = 100.;
        run.milestones = run.milestones.with_ranks(&[]);
        assert!(reactive_buff_status(&run).is_empty());
    }
}
