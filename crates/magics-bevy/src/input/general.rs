use std::collections::HashMap;

use bevy::{app::AppExit, prelude::*, tasks::IoTaskPool};
use bevy_notify::prelude::*;
use gbp_config::{Config, DrawSetting};
use leafwing_input_manager::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    input::screenshot::{ScreenshotPlugin, TakeScreenshot},
    input::ChangingBinding,
    pause_play::{PausePlay, SimulationSpeed},
    theme::{CatppuccinTheme, CycleTheme},
    SimulationState,
};

/// Component marker for general input handling
#[derive(Component)]
pub struct GeneralInputs;

/// Plugin for general input handling
pub struct GeneralInputPlugin;

impl Plugin for GeneralInputPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ScreenshotPlugin>() {
            app.add_plugins(ScreenshotPlugin::default());
        }

        app.add_event::<EnvironmentEvent>()
            .add_event::<ExportFactorGraphAsGraphviz>()
            .add_event::<DrawSettingsEvent>()
            .add_event::<QuitApplication>()
            .add_event::<ExportFactorGraphAsGraphvizFinished>()
            .add_plugins(InputManagerPlugin::<GeneralAction>::default())
            .add_systems(PostStartup, bind_general_input)
            .add_systems(
                Update,
                (
                    general_actions_system,
                    pause_play_simulation,
                    export_graph_finished_system,
                    screenshot,
                    quit_application_system,
                ),
            );
    }
}

/// Event to toggle the environment visualization
#[derive(Event, Debug, Copy, Clone)]
pub struct EnvironmentEvent;

/// Event to export the factor graph
#[derive(Event, Debug, Copy, Clone)]
pub struct ExportFactorGraphAsGraphviz;

/// Event for when draw settings change
#[derive(Event, Debug, Clone)]
pub struct DrawSettingsEvent {
    /// The draw setting that was toggled
    pub setting: DrawSetting,
    /// The new value of the draw setting
    pub draw: bool,
}

/// Event for when the export graph operation is finished
#[derive(Event, Debug)]
pub enum ExportFactorGraphAsGraphvizFinished {
    /// The export was successful with a message
    Success(String),
    /// The export failed with a message
    Failure(String),
}

/// General actions that can be triggered
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect, EnumIter, Default)]
pub enum GeneralAction {
    #[default]
    /// Cycle between catppuccin themes
    CycleTheme,
    /// Export all factorgraphs as graphviz format
    ExportGraph,
    /// Take a screenshot of the primary window
    ScreenShot,
    /// Save current settings
    SaveSettings,
    /// Quit the application
    QuitApplication,
    /// Toggle the simulation time between paused and playing
    PausePlaySimulation,
}

impl std::fmt::Display for GeneralAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::CycleTheme => "Cycle Theme",
            Self::ExportGraph => "Export Graph",
            Self::ScreenShot => "Take ScreenShot",
            Self::SaveSettings => "Save Settings",
            Self::QuitApplication => "Quit Application",
            Self::PausePlaySimulation => "Pause/Play Simulation",
        })
    }
}

impl GeneralAction {
    /// Get the default keyboard input for an action
    fn default_keyboard_input(action: Self) -> UserInput {
        match action {
            Self::CycleTheme => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyT)),
            Self::ExportGraph => UserInput::Single(InputKind::PhysicalKey(KeyCode::KeyG)),
            Self::SaveSettings => {
                UserInput::modified(Modifier::Control, InputKind::PhysicalKey(KeyCode::KeyS))
            }
            Self::ScreenShot => {
                UserInput::modified(Modifier::Control, InputKind::PhysicalKey(KeyCode::KeyP))
            }
            Self::QuitApplication => {
                UserInput::modified(Modifier::Control, InputKind::PhysicalKey(KeyCode::KeyQ))
            }
            Self::PausePlaySimulation => UserInput::Single(InputKind::PhysicalKey(KeyCode::Space)),
        }
    }
}

/// System to bind general input actions
fn bind_general_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    for action in GeneralAction::iter() {
        let input = GeneralAction::default_keyboard_input(action);
        input_map.insert(action, input);
    }

    commands.spawn((
        InputManagerBundle::<GeneralAction> {
            input_map,
            ..Default::default()
        },
        GeneralInputs,
    ));
}

/// System to cycle through catppuccin themes
fn cycle_theme(
    theme_event_writer: &mut EventWriter<CycleTheme>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    info!("Toggling application theme");

    let next_theme = match catppuccin_theme.flavour {
        catppuccin::Flavour::Latte => catppuccin::Flavour::Frappe,
        catppuccin::Flavour::Frappe => catppuccin::Flavour::Macchiato,
        catppuccin::Flavour::Macchiato => catppuccin::Flavour::Mocha,
        catppuccin::Flavour::Mocha => catppuccin::Flavour::Latte,
    };

    theme_event_writer.send(CycleTheme(next_theme));
}

/// System to handle export graph finished events
fn export_graph_finished_system(
    mut export_graph_finished_reader: EventReader<ExportFactorGraphAsGraphvizFinished>,
    mut toast_event: EventWriter<ToastEvent>,
) {
    for event in export_graph_finished_reader.read() {
        match event {
            ExportFactorGraphAsGraphvizFinished::Success(path) => {
                toast_event.send(ToastEvent::info(format!(
                    "Successfully exported factorgraphs to ./{path}"
                )));
            }
            ExportFactorGraphAsGraphvizFinished::Failure(message) => {
                toast_event.send(ToastEvent::error(format!(
                    "Failed to export factorgraphs: {message}"
                )));
            }
        }
    }
}

/// Event to quit the application
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct QuitApplication;

/// System to handle quit application events
fn quit_application_system(
    mut quit_application_reader: EventReader<QuitApplication>,
    mut app_exit_event: EventWriter<AppExit>,
) {
    for _ in quit_application_reader.read() {
        info!("Quitting application");
        app_exit_event.send(AppExit);
    }
}

/// Event to save settings
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct SaveSettings;

/// System to handle general action inputs
#[allow(clippy::too_many_arguments)]
fn general_actions_system(
    mut theme_event: EventWriter<CycleTheme>,
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    currently_changing: Res<ChangingBinding>,
    catppuccin_theme: Res<CatppuccinTheme>,
    mut quit_application_event: EventWriter<QuitApplication>,
    mut export_graph_event: EventWriter<ExportFactorGraphAsGraphviz>,
    mut save_settings_event: EventWriter<SaveSettings>,
    mut toast_event: EventWriter<ToastEvent>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }
    
    let Ok(action_state) = query.get_single() else {
        warn!("general_actions_system was called without an action state!");
        return;
    };

    if action_state.just_pressed(&GeneralAction::CycleTheme) {
        cycle_theme(&mut theme_event, catppuccin_theme);
    } else if action_state.just_pressed(&GeneralAction::ExportGraph) {
        export_graph_event.send(ExportFactorGraphAsGraphviz);
    }

    if action_state.just_pressed(&GeneralAction::QuitApplication) {
        quit_application_event.send(QuitApplication);
    }

    if action_state.just_pressed(&GeneralAction::SaveSettings) {
        save_settings_event.send(SaveSettings);
        toast_event.send(ToastEvent {
            caption: "Saved settings to config.toml".to_string(),
            options: ToastOptions {
                duration: Some(std::time::Duration::from_millis(500)),
                level: ToastLevel::Success,
                show_progress_bar: false,
                closable: false,
            },
        });
    }
}

/// System to handle pause/play simulation inputs
fn pause_play_simulation(
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    currently_changing: Res<ChangingBinding>,
    mut pause_play_event: EventWriter<PausePlay>,
    mut simulation_state: Option<ResMut<SimulationState>>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }

    let Ok(action_state) = query.get_single() else {
        warn!("pause_play_simulation was called without an action state!");
        return;
    };

    if action_state.just_pressed(&GeneralAction::PausePlaySimulation) {
        pause_play_event.send(PausePlay::Toggle);
        
        // Also update simulation state if available
        if let Some(mut state) = simulation_state {
            state.paused = !state.paused;
        }
    }
}

/// System to handle screenshot inputs
fn screenshot(
    query: Query<&ActionState<GeneralAction>, With<GeneralInputs>>,
    currently_changing: Res<ChangingBinding>,
    mut screen_shot_event: EventWriter<TakeScreenshot>,
) {
    if currently_changing.on_cooldown() || currently_changing.is_changing() {
        return;
    }

    let Ok(action_state) = query.get_single() else {
        warn!("screenshot was called without an action state!");
        return;
    };

    if action_state.just_pressed(&GeneralAction::ScreenShot) {
        info!("Taking screenshot");
        screen_shot_event.send(TakeScreenshot::default());
    }
}
