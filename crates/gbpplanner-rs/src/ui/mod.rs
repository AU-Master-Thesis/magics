mod controls;
mod custom;
mod data;
mod decoration;
mod selected_entity;
mod settings;

use bevy::{input::common_conditions::*, prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{self, Visuals},
    EguiContexts, EguiPlugin,
};
pub use controls::ChangingBinding;
pub use decoration::ToDisplayString;
pub use settings::{DrawSettingsEvent, ExportGraphEvent};
use strum_macros::EnumIter;

use self::{controls::ControlsPanelPlugin, data::DataPanelPlugin, settings::SettingsPanelPlugin};
use crate::{theme::CatppuccinThemeVisualsExt, SimulationState};

//  _     _ _______ _______  ______
//  |     | |______ |______ |_____/
//  |_____| ______| |______ |    \_
//
//  _____ __   _ _______ _______  ______ _______ _______ _______ _______
//    |   | \  |    |    |______ |_____/ |______ |_____| |       |______
//  __|__ |  \_|    |    |______ |    \_ |       |     | |_____  |______
//

pub struct EguiInterfacePlugin;

impl Plugin for EguiInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionBlock>()
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<UiState>()
            // .init_resource::<PreviousUiState>()
            .add_plugins((EguiPlugin, ControlsPanelPlugin, SettingsPanelPlugin, DataPanelPlugin))
            .add_systems(OnEnter(SimulationState::Loading), load_fonts)
            .add_systems(Startup, configure_visuals)
            .add_systems(Update, action_block)
            .add_systems(
                Update,
                (
                    hide_panels.run_if(input_just_pressed(KeyCode::Escape)),
                    // toggle_visibility_of_panels.run_if(input_just_pressed(KeyCode::Escape)),
                ),
            );
        // .add_systems(
        //     Update,
        //     snapshot_
        // )
    }
}

fn load_fonts() {}

fn toggle_visibility_of_panels(mut ui_state: ResMut<UiState>) {}

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
    pub fn is_blocked(&self) -> bool {
        self.0
    }
}

/// Resource to store the occupied screen space by each `egui` panel
#[derive(Debug, Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    right: f32,
    top: f32,
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

#[derive(Debug, Default)]
pub struct MouseOverPanel {
    pub left_panel: bool,
    pub right_panel: bool,
    pub top_panel: bool,
    pub bottom_panel: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelDirection {
    Left,
    Right,
    Top,
    Bottom,
}

impl PanelDirection {
    /// Get a vector containing all panel directions.
    #[must_use]
    pub fn all() -> impl ExactSizeIterator<Item = Self> {
        [Self::Left, Self::Right, Self::Top, Self::Bottom]
            .iter()
            .copied()
    }
}

struct PanelState;

/// UI state to represent state of `egui` stateful widgets
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
    /// The type of UI scaling to use
    pub scale_type: UiScaleType,
    /// When `scale_type` is `Custom`, the percentage to scale by
    pub scale_percent: usize,
    // /// Whether the environment SDF is visible
    // pub environment_sdf: bool,
    pub mouse_over: MouseOverPanel,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel_visible: false,
            right_panel_visible: false,
            top_panel_visible: false,
            bottom_panel_visible: true,
            scale_type: UiScaleType::default(),
            scale_percent: 100, // start at default factor 1.0 = 100%
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
    {
        action_block.block();
    } else {
        action_block.unblock();
    }
}
