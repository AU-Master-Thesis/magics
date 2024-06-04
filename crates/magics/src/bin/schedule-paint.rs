use bevy::{app::AppExit, input::common_conditions::*, prelude::*};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

const NAME: &str = env!("CARGO_PKG_NAME");

// bevy v0.13.0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("huuh");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        // .add_plugins(EguiPlugin)
        .add_systems(
            Update,
            (render,
            quit_application.run_if(input_just_pressed(KeyCode::KeyQ)),
            )
        )
        .run();

    Ok(())
}

fn render(mut egui_ctx: EguiContexts) {
    let ctx = egui_ctx.ctx_mut();

    error!("hello");
    let right_side_panel = egui::SidePanel::right("right_side_panel")
        .default_width(800.0)
        .resizable(false)
        .show(ctx, |ui| {
            // let painter = ui.painter();

            // // Define the line start and end points
            // let start_point = egui::pos2(10.0, 10.0);
            // let end_point = egui::pos2(100.0, 100.0);

            // // Define the stroke (color and width of the line)
            // let stroke = egui::Stroke::new(2.0, egui::Color32::RED);

            // // Draw the line
            // painter.line_segment([start_point, end_point], stroke);
            // let painter = egui::Painter::new(, , )
        });
}

/// Quit the running bevy application
fn quit_application(mut app_exit_event: EventWriter<AppExit>) {
    info!("quitting application");
    app_exit_event.send(AppExit);
}
