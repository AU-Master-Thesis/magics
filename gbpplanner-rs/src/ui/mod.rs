mod controls;
mod custom;
mod decoration;
mod settings;

pub use controls::ChangingBinding;
pub use decoration::ToDisplayString;
pub use settings::DrawSettingsEvent;
pub use settings::ExportGraphEvent;

use bevy::{prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{self, Visuals},
    EguiContexts, EguiPlugin,
};
use strum_macros::EnumIter;

use crate::theme::CatppuccinThemeVisualsExt;

use self::{controls::ControlsPanelPlugin, settings::SettingsPanelPlugin};

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
        app.init_resource::<OccupiedScreenSpace>()
            .init_resource::<UiState>()
            .add_plugins(EguiPlugin)
            .add_systems(Startup, configure_visuals_system)
            .add_plugins((ControlsPanelPlugin, SettingsPanelPlugin));
    }
}

/// Resource to store the occupied screen space by each `egui` panel
#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
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

/// UI state to represent state of `egui` stateful widgets
#[derive(Resource)]
pub struct UiState {
    /// Whether the left panel is open
    pub left_panel: bool,
    /// Whether the right panel is open
    pub right_panel: bool,
    /// The type of UI scaling to use
    pub scale_type: UiScaleType,
    /// When `scale_type` is `Custom`, the percentage to scale by
    pub scale_percent: usize,
    // /// Whether the environment SDF is visible
    // pub environment_sdf: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel: false,
            right_panel: false,
            scale_type: UiScaleType::default(),
            scale_percent: 100, // start at default factor 1.0 = 100%
                                // environment_sdf: false,
        }
    }
}

/// `Setup` **Bevy** sytem to initialise the `egui` visuals
/// This is where the **default** for `egui` is set
fn configure_visuals_system(mut contexts: EguiContexts, windows: Query<&Window>) {
    let window = windows.single();
    contexts.ctx_mut().set_visuals(match window.window_theme {
        Some(WindowTheme::Dark) => Visuals::catppuccin_dark(),
        _ => Visuals::catppuccin_light(),
    });

    let mut fonts = egui::FontDefinitions::default();

    // TODO: somehow use the **Bevy** asset loader through `scene_assets` to load the font
    // instead of a relative path
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
