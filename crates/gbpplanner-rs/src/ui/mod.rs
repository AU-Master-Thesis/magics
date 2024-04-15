pub mod controls;
mod custom;
mod data;
mod decoration;
mod metrics;
mod scale;
// mod selected_entity;
mod settings;

use std::ops::RangeInclusive;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{self, Visuals},
    EguiContexts,
};
use bevy_touchpad::TwoFingerSwipe;
pub use decoration::ToDisplayString;
use strum_macros::EnumIter;

use self::{
    controls::ControlsPanelPlugin, data::DataPanelPlugin, metrics::MetricsPlugin,
    scale::ScaleUiPlugin, settings::SettingsPanelPlugin,
};
use crate::{theme::CatppuccinThemeVisualsExt, AppState, SimulationState};

//  _     _ _______ _______  ______
//  |     | |______ |______ |_____/
//  |_____| ______| |______ |    \_
//
//  _____ __   _ _______ _______  ______ _______ _______ _______ _______
//    |   | \  |    |    |______ |_____/ |______ |_____| |       |______
//  __|__ |  \_|    |    |______ |    \_ |       |     | |_____  |______
//

pub struct EguiInterfacePlugin;

pub struct UiPlugins;

impl PluginGroup for UiPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        bevy::app::PluginGroupBuilder::start::<Self>()
            .add(ControlsPanelPlugin)
            .add(SettingsPanelPlugin)
            .add(DataPanelPlugin)
            .add(MetricsPlugin::default())
            .add(ScaleUiPlugin::default())
    }
}

impl Plugin for EguiInterfacePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_touchpad::BevyTouchpadPlugin>() {
            app.add_plugins(bevy_touchpad::BevyTouchpadPlugin::default());
        }
        app.init_resource::<ActionBlock>()
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<UiState>()
            // .init_resource::<PreviousUiState>()
            .add_plugins(( ControlsPanelPlugin, SettingsPanelPlugin, DataPanelPlugin,
                ScaleUiPlugin::default(),


                MetricsPlugin::default()            ))
            // .add_systems(OnEnter(SimulationState::Loading), load_fonts)
            // .add_systems(Startup, load_fonts)
            // .add_systems(OnEnter(AppState::Loading), load_fonts)
            .add_systems(Startup, configure_visuals)
            .add_systems(Update, action_block)
            .add_systems(
                Update,
                (
                    hide_panels.run_if(input_just_pressed(KeyCode::Escape)),
                    // toggle_visibility_of_panels.run_if(input_just_pressed(KeyCode::Escape)),
                ),
            );
        // .add_systems(Update,
        // toggle_visibility_of_side_panels_when_two_finger_swiping);

        // .add_systems(
        //     Update,
        //     snapshot_
        // )
    }
}

// fn load_fonts() {}

// fn toggle_visibility_of_panels(mut ui_state: ResMut<UiState>) {}

// fn toggle_visibility_of_side_panels_when_two_finger_swiping(
//     mut ui_state: ResMut<UiState>,
//     mut evr_two_finger_swipe: EventReader<TwoFingerSwipe>,
//     // mut timeout: Local<Timer>,
// ) {
//     for event in evr_two_finger_swipe.read() {
//         match event.direction {
//             bevy_touchpad::TwoFingerSwipeDirection::Up
//             | bevy_touchpad::TwoFingerSwipeDirection::Down => {}
//             bevy_touchpad::TwoFingerSwipeDirection::Left => {
//                 ui_state.left_panel_visible = !ui_state.left_panel_visible;
//             }
//             bevy_touchpad::TwoFingerSwipeDirection::Right => {
//                 ui_state.right_panel_visible = !ui_state.right_panel_visible;
//             }
//         }
//     }
// }

/// **Bevy** system that hides both the left and right ui panels, if any of them
/// are visible.
fn hide_panels(mut ui_state: ResMut<UiState>) {
    if ui_state.left_panel_visible {
        ui_state.left_panel_visible = false;
    }

    if ui_state.right_panel_visible {
        ui_state.right_panel_visible = false;
    }

    if ui_state.top_panel_visible {
        ui_state.top_panel_visible = false;
    }

    if ui_state.bottom_panel_visible {
        ui_state.bottom_panel_visible = false;
    }

    if ui_state.metrics_window_visible {
        ui_state.metrics_window_visible = false;
    }
}

/// **Bevy** [`Resource`] to block actions from being performed
/// Blocks actions (except for UI actions) while hovering a UI element
#[derive(Debug, Default, Resource)]
pub struct ActionBlock(bool);

impl ActionBlock {
    #[inline]
    pub fn block(&mut self) {
        self.0 = true;
    }

    #[inline]
    pub fn unblock(&mut self) {
        self.0 = false;
    }

    #[inline]
    pub const fn is_blocked(&self) -> bool {
        self.0
    }
}

/// Resource to store the occupied screen space by each `egui` panel
#[derive(Debug, Default, Resource)]
struct OccupiedScreenSpace {
    left:   f32,
    right:  f32,
    top:    f32,
    bottom: f32,
}

#[derive(Debug, EnumIter, Default, derive_more::Display)]
pub enum UiScaleType {
    #[display(fmt = "None")]
    None,
    #[display(fmt = "Custom")]
    #[default]
    Custom,
    #[display(fmt = "Window")]
    Window,
}

impl ToDisplayString for UiScaleType {
    fn to_display_string(&self) -> String {
        match self {
            Self::None => "None".to_string(),
            Self::Custom => "Custom".to_string(),
            Self::Window => "Window".to_string(),
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct MouseOverPanel {
    pub left_panel:      bool,
    pub right_panel:     bool,
    pub top_panel:       bool,
    pub bottom_panel:    bool,
    pub metrics_window:  bool,
    pub floating_window: bool,
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum PanelDirection {
//     Left,
//     Right,
//     Top,
//     Bottom,
// }

// impl PanelDirection {
//     /// Get a vector containing all panel directions.
//     #[must_use]
//     pub fn all() -> impl ExactSizeIterator<Item = Self> {
//         [Self::Left, Self::Right, Self::Top, Self::Bottom]
//             .iter()
//             .copied()
//     }
// }

// struct PanelState;

/// UI state to represent state of `egui` stateful widgets
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Resource)]
pub struct UiState {
    /// Whether the left panel is open
    pub left_panel_visible: bool,
    /// Whether the right panel is open
    pub right_panel_visible: bool,
    /// Whether the top panel is open
    pub top_panel_visible: bool,
    /// Whether the bottom panel is open
    pub bottom_panel_visible: bool,
    /// Wheter the metrics window is open
    pub metrics_window_visible: bool,
    /// The type of UI scaling to use
    pub scale_type: UiScaleType,
    /// When `scale_type` is `Custom`, the percentage to scale by
    scale_percent: usize,
    // /// Whether the environment SDF is visible
    // pub environment_sdf: bool,
    pub mouse_over: MouseOverPanel,
}

impl UiState {
    pub const DEFAULT_SCALE_PERCENTAGE: usize = 100;
    pub const MAX_SCALE_PERCENTAGE: usize = 200;
    pub const MIN_SCALE_PERCENTAGE: usize = 50;
    pub const VALID_SCALE_INTERVAL: RangeInclusive<usize> =
        Self::MIN_SCALE_PERCENTAGE..=Self::MAX_SCALE_PERCENTAGE;

    pub fn set_scale(&mut self, percentage: usize) {
        if Self::VALID_SCALE_INTERVAL.contains(&percentage) {
            // if (Self::MIN_SCALE_PERCENTAGE..=Self::MAX_SCALE_PERCENTAGE).contains(&
            // percentage) {
            self.scale_percent = percentage;
        }
    }

    // #[inline(always)]
    // pub const fn scale(&self) -> usize {
    //     self.scale_percent
    // }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel_visible: false,
            right_panel_visible: false,
            top_panel_visible: false,
            bottom_panel_visible: false,
            metrics_window_visible: false,
            scale_type: UiScaleType::default(),
            scale_percent: Self::DEFAULT_SCALE_PERCENTAGE,
            // scale_percent: 100, // start at default factor 1.0 = 100%
            // environment_sdf: false,
            mouse_over: MouseOverPanel::default(),
        }
    }
}

#[derive(Debug, Resource, Deref, DerefMut, Default)]
struct PreviousUiState(UiState);

/// `Setup` **Bevy** system to initialise the `egui` visuals
/// This is where the **default** for `egui` is set
fn configure_visuals(
    mut contexts: EguiContexts,
    windows: Query<&Window>,
    // scene_assets: Res<SceneAssets>,
) {
    let window = windows.single();
    contexts.ctx_mut().set_visuals(match window.window_theme {
        Some(WindowTheme::Dark) => Visuals::catppuccin_dark(),
        _ => Visuals::catppuccin_light(),
    });

    let mut fonts = egui::FontDefinitions::default();

    // egui::font_loader

    // TODO: somehow use the **Bevy** asset loader through `scene_assets` to load
    // the font instead of a relative path
    fonts.font_data.insert(
        "JetBrainsMonoNerdFont-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf"
        )),
    );

    // fonts.font_data.insert(
    //     "JetBrainsMonoNerdFont-Regular".to_owned(),
    //     egui::FontData::from_owned(),
    // );

    // Put JetBrainsMono first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "JetBrainsMonoNerdFont-Regular".to_owned());

    // Put JetBrainsMono first (highest priority) for proportional text:
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMonoNerdFont-Regular".to_owned());

    contexts.ctx_mut().set_fonts(fonts);
}

fn action_block(mut action_block: ResMut<ActionBlock>, ui_state: Res<UiState>) {
    // TODO: add top and bottom
    if (ui_state.left_panel_visible && ui_state.mouse_over.left_panel)
        || (ui_state.right_panel_visible && ui_state.mouse_over.right_panel)
        || (ui_state.top_panel_visible && ui_state.mouse_over.top_panel)
        || (ui_state.bottom_panel_visible && ui_state.mouse_over.bottom_panel)
        || (ui_state.mouse_over.floating_window)
    {
        action_block.block();
    } else {
        action_block.unblock();
    }
}
