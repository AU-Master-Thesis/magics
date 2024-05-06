#![allow(missing_docs, unused_variables, dead_code)]
use bevy::{app::AppExit, input::common_conditions::input_just_pressed, prelude::*};
use bevy_egui::{
    egui::{self, Color32, RichText, Ui},
    EguiContexts, EguiPlugin,
};
use ndarray::prelude::*;

const NAME: &str = env!("CARGO_PKG_NAME");

// bevy v0.13.0
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .insert_resource(VariableData::default())
        .add_systems(Update, quit_application.run_if(input_just_pressed(KeyCode::KeyQ)))
        .add_systems(Update, render)
        .run();
}

/// Quit the running bevy application
fn quit_application(mut app_exit_event: EventWriter<AppExit>) {
    info!("quitting application");
    app_exit_event.send(AppExit);
}

#[derive(Debug, Resource)]
pub struct VariableData {
    pub information_vector: Array1<f32>,
    pub precision_matrix: Array2<f32>,
    pub mean: Array1<f32>,
}

impl Default for VariableData {
    fn default() -> Self {
        Self {
            information_vector: array![-1., 2., -3., 4.],
            precision_matrix: array![[1., 2., 3., 4.], [5., 6., 7., 8.], [9., 10., 11., 12.], [
                13., 14., 15., 16.
            ]],
            mean: array![1., 2., 3., 4.],
        }
    }
}

fn render(mut egui_ctx: EguiContexts, data: Res<VariableData>) {
    let window = egui::Window::new("window").show(egui_ctx.ctx_mut(), |ui| {
        // let grid = egui::Grid::new("some_unique_id").show(ui, |ui| {
        //     ui.label("First row, first column");
        //     ui.label("First row, second column");
        //     ui.end_row();

        //     ui.label("Second row, first column");
        //     ui.label("Second row, second column");
        //     ui.label("Second row, third column");
        //     ui.end_row();

        //     ui.horizontal(|ui| {
        //         ui.label("Same");
        //         ui.label("cell");
        //     });
        //     ui.label("Third row, second column");
        //     ui.end_row();
        // });

        subheading(ui, "Information Vector", Some(Color32::LIGHT_RED));
        let information_vector = egui::Grid::new("information_vector")
            .with_row_color(|row, style| None)
            .num_columns(data.information_vector.len())
            .show(ui, |ui| {
                for elem in &data.information_vector {
                    ui.label(float_cell(*elem));
                    // ui.label(elem.to_string()).highlight();
                }
                ui.end_row();
            });

        ui.add_space(10.0);
        subheading(ui, "Precision Matrix", Some(Color32::LIGHT_GREEN));
        let precision_matrix = egui::Grid::new("precision_matrix")
            .striped(true)
            // .with_row_color(|row, style| {
            //     if row % 2 == 0 {
            //         Some(Color32::LIGHT_YELLOW)
            //     } else {
            //         None
            //     }
            // })
            .num_columns(data.information_vector.len())
            .show(ui, |ui| {
                for r in 0..data.precision_matrix.nrows() {
                    for c in 0..data.precision_matrix.ncols() {
                        ui.label(float_cell(data.precision_matrix[(r, c)]));
                        // ui.label(data.precision_matrix[(r, c)].to_string());
                    }
                    ui.end_row();
                }
            });

        ui.add_space(10.0);

        subheading(ui, "Mean", Some(Color32::LIGHT_BLUE));
        let mean = egui::Grid::new("mean").show(ui, |ui| {
            for elem in &data.mean {
                ui.label(float_cell(*elem));
                // ui.label(elem.to_string());
            }
            ui.end_row();
        });

        // ui.end_row();
        // ui.end_row();
        // ui.end_row();
    });
}

#[must_use]
pub fn float_cell(f: f32) -> egui::RichText {
    if f < 0.0 {
        RichText::new(f.to_string()).color(Color32::LIGHT_RED)
    } else {
        // Default::default()
        RichText::new(f.to_string()).color(Color32::WHITE)
    }
}

/// A separator with color and space before and after
pub fn separator(ui: &mut Ui, color: Option<Color32>, space_before: Option<f32>, space_after: Option<f32>) {
    ui.add_space(space_before.unwrap_or(0.0));
    let before = ui.visuals().widgets.noninteractive.bg_stroke.color;
    ui.visuals_mut().widgets.noninteractive.bg_stroke.color = color.unwrap_or(before);
    ui.separator();
    ui.visuals_mut().widgets.noninteractive.bg_stroke.color = before;
    ui.add_space(space_after.unwrap_or(0.0));
}

/// A heading with color and a separator after
pub fn heading(ui: &mut Ui, text: &str, color: Option<Color32>) {
    ui.add_space(10.0);
    ui.heading(RichText::new(text).color(color.unwrap_or_else(|| ui.visuals().text_color())));
    separator(ui, color, Some(5.0), Some(0.0));
}

/// A subheading with color and a separator after
pub fn subheading(ui: &mut Ui, text: &str, color: Option<Color32>) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(text)
            .size(16.0)
            .color(color.unwrap_or_else(|| ui.visuals().text_color())),
    );
    separator(ui, color, None, Some(5.0));
}
