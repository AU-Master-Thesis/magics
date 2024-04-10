use std::time::Duration;

use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticPath, Diagnostics, DiagnosticsStore, EntityCountDiagnosticsPlugin,
        FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin, RegisterDiagnostic,
        SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_egui::egui;
use egui_plot::{Line, Plot, PlotPoints};

use super::UiState;
use crate::{config::Config, diagnostic::prelude::RobotDiagnosticsPlugin, SimulationState};

pub struct MetricsPlugin {
    wait_duration: Duration,
}

impl Default for MetricsPlugin {
    fn default() -> Self {
        Self {
            wait_duration: Duration::from_millis(500),
        }
    }
}

// #[derive(Resource)]
// struct MetricsState {
//     pub timer: Timer
// }

// All diagnostics should have a unique DiagnosticPath.
const SYSTEM_ITERATION_COUNT: DiagnosticPath = DiagnosticPath::const_new("system_iteration_count");

impl Plugin for MetricsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }

        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }

        if !app.is_plugin_added::<EntityCountDiagnosticsPlugin>() {
            app.add_plugins(EntityCountDiagnosticsPlugin::default());
        }

        if !app.is_plugin_added::<SystemInformationDiagnosticsPlugin>() {
            app.add_plugins(SystemInformationDiagnosticsPlugin::default());
        }

        if !app.is_plugin_added::<RobotDiagnosticsPlugin>() {
            app.add_plugins(RobotDiagnosticsPlugin::default());
        }

        if !app.is_plugin_added::<LogDiagnosticsPlugin>() {
            app.add_plugins(LogDiagnosticsPlugin {
                debug: true,
                wait_duration: Duration::from_secs(1),
                ..Default::default()
            });
        }

        // Diagnostics must be initialized before measurements can be added.
        // app.register_diagnostic(Diagnostic::new(SYSTEM_ITERATION_COUNT).with_suffix("
        // iterations")); app.add_systems(Update, Self::system_iteration_count);

        // app.add_system(Startup, setup);
        app.add_systems(PostUpdate, Self::render);
        // app.add_systems(OnEnter(SimulationState::Running), render);
    }
}

// fn setup(mut commands: Commands, mut egui_ctx: bevy_egui::EguiContexts) {
//     // let window = egui::
//     let window = egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
//         ui.label("world");
//     });

// }

impl MetricsPlugin {
    fn render(
        mut egui_ctx: bevy_egui::EguiContexts,
        time: Res<Time<Real>>,
        diagnostics: Res<DiagnosticsStore>,
        timer: Local<Timer>,
        config: Res<Config>,
        mut ui_state: ResMut<UiState>,
        mut current_pos: Local<egui::Pos2>,
    ) {
        let window = egui::Window::new("Metrics")
            .collapsible(true)
            .interactable(true)
            .movable(true)
            .default_pos(*current_pos)
            .show(egui_ctx.ctx_mut(), |ui| {
                if ui.rect_contains_pointer(ui.max_rect())
                    && config.interaction.ui_focus_cancels_inputs
                {
                    ui_state.mouse_over.floating_window = true;
                } else {
                    ui_state.mouse_over.floating_window = false;
                }

                // for diagnostic in &[EntityCountDiagnosticsPlugin::ENTITY_COUNT]

                // TODO: add diagnostic source for number of collisions
                for (name, diagnostic_path) in [
                    ("entities", &EntityCountDiagnosticsPlugin::ENTITY_COUNT),
                    ("robots", &RobotDiagnosticsPlugin::ROBOT_COUNT),
                    ("variables", &RobotDiagnosticsPlugin::VARIABLE_COUNT),
                    ("factors", &RobotDiagnosticsPlugin::FACTOR_COUNT),
                ] {
                    if let Some(value) = diagnostics
                        .get_measurement(diagnostic_path)
                        .map(|d| d.value as i64)
                    {
                        ui.label(format!("{}: {}", name, value));
                    }
                }

                if let Some(messages_sent) =
                    diagnostics.get(&RobotDiagnosticsPlugin::MESSAGES_SENT_COUNT)
                {
                    let points: PlotPoints = messages_sent.values()
                        // .iter()
                        .enumerate()
                        .map(|(i, robot)| [i as f64, *robot])
                        .collect();
                    let line = Line::new(points);

                    let plot = Plot::new("my_plot")
                        .view_aspect(2.0)
                        .show_grid(true)
                        .x_axis_label("samples recorded")
                        .y_axis_label("messages sent");

                    plot.show(ui, |plot_ui| plot_ui.line(line));
                }

                ui.label(format!("{}", egui::special_emojis::GITHUB));
                ui.allocate_space(ui.available_size()); // put this LAST in your
                                                        // panel/window code
                                                        // ui.allocate_space(ui.
                                                        // available_size()); //
                                                        // put this LAST in your
                                                        // panel/window code
            });

        // occupied_screen_space.left = left_panel
        //     .map(|ref inner| inner.response.rect.width())
        //     .unwrap_or(0.0);

        *current_pos = window
            .map(|ref inner| inner.response.rect.min)
            .unwrap_or_default();
    }

    fn system_iteration_count(mut diagnostics: Diagnostics, time: Res<Time<Real>>) {
        // Add a measurement of 10.0 for our diagnostic each time this system runs.
        diagnostics.add_measurement(&SYSTEM_ITERATION_COUNT, || time.delta_seconds_f64());
    }
}
