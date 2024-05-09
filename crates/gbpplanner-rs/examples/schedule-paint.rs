use std::{cell, time::Duration};

use bevy::{
    app::AppExit, input::common_conditions::*, prelude::*, time::common_conditions::on_timer,
};
use bevy_egui::{
    egui::{self, Color32, Pos2, Rect, Stroke},
    EguiContexts, EguiPlugin,
};
use rand::{thread_rng, Rng};

const NAME: &str = env!("CARGO_PKG_NAME");

// bevy v0.13.0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(Schedules::default())
        .add_systems(
            Update,
            (
                quit_application.run_if(input_just_pressed(KeyCode::KeyQ)),
                render,
                pick_random_schedule.run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .run();

    Ok(())
}

#[derive(Resource)]
struct Schedules {
    internal: Vec<bool>,
    external: Vec<bool>,
}

impl Default for Schedules {
    fn default() -> Self {
        let n = 5;
        Self {
            internal: vec![true; n],
            external: vec![true; n],
        }
    }
}

fn pick_random_schedule(mut schedules: ResMut<Schedules>) {
    let mut rng = thread_rng();
    let n = rng.gen_range(5..10);

    schedules.internal = (0..n).map(|_| rng.gen_bool(0.5)).collect();
    schedules.external = (0..n).map(|_| rng.gen_bool(0.5)).collect();
}

/// Quit the running bevy application
fn quit_application(mut app_exit_event: EventWriter<AppExit>) {
    info!("quitting application");
    app_exit_event.send(AppExit);
}

fn render(mut contexts: EguiContexts, schedules: Res<Schedules>) {
    let ctx = contexts.ctx_mut();
    // let internal = [true, false, true, false, true, true, true, true];
    // let external = [true, false, true, false, true, true, false, false];

    let width = 400.0;
    let right_panel = egui::SidePanel::right("right_panel")
        .resizable(false)
        .max_width(width)
        .show(ctx, |ui| {
            // ui.label("Right resizeable panel");

            // ui.allocate_exact_size(egui::Vec2::new(width, 0.0), egui::Sense::hover());

            let available_size = ui.available_size_before_wrap();
            let margin = 10.0;
            let line_height = 5.0; // height of each line segment
            let line_gap = 5.0; // gap between lines

            let start_x = margin;
            let cell_width = 40.0;
            // let end_x = available_size.x - margin;
            let mut current_y = margin;

            let max_x = 200.0;
            let inbetween_padding_percentage = 0.2;
            let cell_width =
                max_x * (1.0 - inbetween_padding_percentage) / schedules.internal.len() as f32;
            let inbetween_width =
                max_x * inbetween_padding_percentage / (schedules.internal.len() - 1) as f32;

            // Get the painter from the UI
            let painter = ui.painter();

            let mut x = start_x;

            // Draw lines for the 'internal' array
            for (i, &is_true) in schedules.internal.iter().enumerate() {
                let color = if is_true { Color32::RED } else { Color32::GRAY };
                let start_x = x;
                // let start_x: f32 = i as f32 * cell_width + 30.0;
                let start_pos = Pos2::new(start_x, current_y);
                let end_x = start_x + cell_width;
                let end_pos = Pos2::new(end_x, current_y);
                painter.line_segment(
                    [start_pos, end_pos],    // points
                    Stroke::new(2.0, color), // stroke (width and color)
                );

                x += cell_width + inbetween_width;
            }

            current_y += line_height + line_gap;

            x = start_x;

            // Draw lines for the 'internal' array
            for (i, &is_true) in schedules.external.iter().enumerate() {
                let color = if is_true {
                    Color32::BLUE
                } else {
                    Color32::GRAY
                };
                let start_x = x;
                // let start_x: f32 = i as f32 * cell_width + 30.0;
                let start_pos = Pos2::new(start_x, current_y);
                let end_x = start_x + cell_width;
                let end_pos = Pos2::new(end_x, current_y);
                painter.line_segment(
                    [start_pos, end_pos],    // points
                    Stroke::new(2.0, color), // stroke (width and color)
                );

                x += cell_width + inbetween_width;
            }

            // Draw lines for the 'external' array
            // for (i, &is_true) in external.iter().enumerate() {
            //     let color = if is_true {
            //         Color32::BLUE
            //     } else {
            //         Color32::GRAY
            //     };
            //     let start_pos = Pos2::new(start_x, current_y);
            //     let end_pos = Pos2::new(end_x, current_y);
            //     painter.line_segment(
            //         [start_pos, end_pos],    // points
            //         Stroke::new(2.0, color), // stroke (width and color)
            //     );
            //
            //     current_y += line_height + line_gap;
            // }

            // Optionally, allocate the space taken by the painter explicitly
            ui.allocate_rect(
                Rect::from_min_size(Pos2::new(0.0, 0.0), available_size),
                egui::Sense::hover(),
            );
        });
}
