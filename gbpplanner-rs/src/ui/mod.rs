mod controls;
mod custom;
mod decoration;
mod settings;

use bevy::{prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{self, Visuals},
    EguiContexts, EguiPlugin,
};
pub use controls::ChangingBinding;
pub use decoration::ToDisplayString;
pub use settings::{DrawSettingsEvent, ExportGraphEvent};
use strum_macros::EnumIter;

use self::{controls::ControlsPanelPlugin, settings::SettingsPanelPlugin};
use crate::theme::CatppuccinThemeVisualsExt;

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
            .add_plugins((EguiPlugin, ControlsPanelPlugin, SettingsPanelPlugin))
            .add_systems(Startup, configure_visuals)
            .add_systems(Update, action_block);
    }
}

/// **Bevy** [`Resource`] to block actions from being performed
/// Blocks actions (except for UI actions) while hovering a UI element
#[derive(Debug, Default, Resource)]
pub struct ActionBlock(bool);

impl ActionBlock {
    pub fn block(&mut self) {
        self.0 = true;
    }

    pub fn unblock(&mut self) {
        self.0 = false;
    }

    pub fn is_blocked(&self) -> bool {
        self.0
    }
}

/// Resource to store the occupied screen space by each `egui` panel
#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left:  f32,
    right: f32,
}

#[derive(EnumIter)]
pub enum UiScaleType {
    None,
    Custom,
    Window,
}

impl Default for UiScaleType {
    fn default() -> Self {
        Self::Custom
    }
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

#[derive(Default)]
pub struct MouseOverPanel {
    pub left_panel:  bool,
    pub right_panel: bool,
}

/// UI state to represent state of `egui` stateful widgets
#[derive(Resource)]
pub struct UiState {
    /// Whether the left panel is open
    pub left_panel:    bool,
    /// Whether the right panel is open
    pub right_panel:   bool,
    /// The type of UI scaling to use
    pub scale_type:    UiScaleType,
    /// When `scale_type` is `Custom`, the percentage to scale by
    pub scale_percent: usize,
    // /// Whether the environment SDF is visible
    // pub environment_sdf: bool,
    pub mouse_over:    MouseOverPanel,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel:    false,
            right_panel:   false,
            scale_type:    UiScaleType::default(),
            scale_percent: 100, // start at default factor 1.0 = 100%
            // environment_sdf: false,
            mouse_over:    MouseOverPanel::default(),
        }
    }
}

/// `Setup` **Bevy** system to initialise the `egui` visuals
/// This is where the **default** for `egui` is set
fn configure_visuals(mut contexts: EguiContexts, windows: Query<&Window>) {
    let window = windows.single();
    contexts.ctx_mut().set_visuals(match window.window_theme {
        Some(WindowTheme::Dark) => Visuals::catppuccin_dark(),
        _ => Visuals::catppuccin_light(),
    });

    let mut fonts = egui::FontDefinitions::default();

    // TODO: somehow use the **Bevy** asset loader through `scene_assets` to load
    // the font instead of a relative path
    fonts.font_data.insert(
        "JetBrainsMonoNerdFont-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../assets/fonts/JetBrainsMonoNerdFont-Regular.ttf"
        )),
    );

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
    if (ui_state.left_panel && ui_state.mouse_over.left_panel)
        || (ui_state.right_panel && ui_state.mouse_over.right_panel)
    {
        action_block.block();
    } else {
        action_block.unblock();
    }
}
