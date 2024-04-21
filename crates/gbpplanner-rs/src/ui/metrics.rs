use std::time::Duration;

use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        LogDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
    },
    prelude::*,
};
use bevy_egui::egui;
use egui_plot::{Line, Plot, PlotPoints};

use super::UiState;
use crate::{config::Config, diagnostic::prelude::RobotDiagnosticsPlugin};

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

struct MetricWidget {}

// #[derive(Resource)]
// struct MetricsState {
//     pub timer: Timer
// }

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

        app.add_systems(PostUpdate, Self::render);
    }
}

impl MetricsPlugin {
    /// **Bevy** system to render the metrics window widget
    fn render(
        mut egui_ctx: bevy_egui::EguiContexts,
        diagnostics: Res<DiagnosticsStore>,
        config: Res<Config>,
        mut ui_state: ResMut<UiState>,
        mut current_pos: Local<egui::Pos2>,
    ) {
        if !ui_state.metrics_window_visible {
            return;
        }

        let window = egui::Window::new("Metrics")
            .collapsible(true)
            .interactable(true)
            .movable(true)
            .default_pos(*current_pos)
            .title_bar(true)
            .vscroll(true)
            .show(egui_ctx.ctx_mut(), |ui| {
                ui_state.mouse_over.floating_window = ui.rect_contains_pointer(ui.max_rect())
                    && config.interaction.ui_focus_cancels_inputs;

                // TODO: add diagnostic source for number of collisions
                for (name, diagnostic_path) in [
                    ("CPU", &SystemInformationDiagnosticsPlugin::CPU_USAGE),
                    ("MEM", &SystemInformationDiagnosticsPlugin::MEM_USAGE),
                    ("FPS", &FrameTimeDiagnosticsPlugin::FPS),
                    // ("frame_count", &FrameTimeDiagnosticsPlugin::FRAME_COUNT),
                    ("frame_time", &FrameTimeDiagnosticsPlugin::FRAME_TIME),
                    ("entities", &EntityCountDiagnosticsPlugin::ENTITY_COUNT),
                    ("robots", &RobotDiagnosticsPlugin::ROBOT_COUNT),
                    ("variables", &RobotDiagnosticsPlugin::VARIABLE_COUNT),
                    ("factors", &RobotDiagnosticsPlugin::FACTOR_COUNT),
                    ("collisions", &RobotDiagnosticsPlugin::ROBOT_COLLISION_COUNT),
                ] {
                    #[allow(clippy::cast_possible_truncation)]
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
                    #[allow(clippy::cast_precision_loss)]
                    let points: PlotPoints = messages_sent.values()
                        // .iter()
                        .enumerate()
                        .map(|(i, robot)| [i as f64, *robot])
                        .collect();
                    let line = Line::new(points);

                    let plot = Plot::new("messages sent")
                        .view_aspect(2.0)
                        .show_grid(true)
                        .x_axis_label("samples recorded")
                        .y_axis_label("messages sent");

                    plot.show(ui, |plot_ui| plot_ui.line(line));
                }

                ui.label(format!("{}", egui::special_emojis::GITHUB));

                // if ui.color_edit_button_rgb(&mut [0.1, 0.5, 0.6]).clicked() {
                //     info!("todo");
                // }

                if ui.button("export").clicked() {
                    info!("todo, not implemented");
                }

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

        // let messages_sent_window = egui::Window::new("Metrics")
        //     .collapsible(true)
        //     .interactable(true)
        //     .movable(true)
        //     // .default_pos(*current_pos)
        //     .title_bar(true)
        //     // .vscroll(true)
        //     .show(egui_ctx.ctx_mut(), |ui| {

        //         if let Some(messages_sent) =
        //
        // diagnostics.get(&RobotDiagnosticsPlugin::MESSAGES_SENT_COUNT)
        //         {
        //             #[allow(clippy::cast_precision_loss)]
        //             let points: PlotPoints = messages_sent.values()
        //                 // .iter()
        //                 .enumerate()
        //                 .map(|(i, robot)| [i as f64, *robot])
        //                 .collect();
        //             let line = Line::new(points);

        //             let plot = Plot::new("messages sent")
        //                 .view_aspect(2.0)
        //                 .show_grid(true)
        //                 .x_axis_label("samples recorded")
        //                 .y_axis_label("messages sent");

        //             plot.show(ui, |plot_ui| plot_ui.line(line));
        //         }
        //     });
    }
}
