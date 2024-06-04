use bevy::prelude::*;
use bevy_egui::EguiContexts;
use bevy_inspector_egui::egui::{self};
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView};
use petgraph::stable_graph::StableGraph;

use super::{OccupiedScreenSpace, UiState};
// use crate::config::Config;

/// **Bevy** `Plugin` to add the data panel to the UI
pub struct DataPanelPlugin;

impl Plugin for DataPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            // .add_systems(Update, update_graph)
            .add_systems(Update, render_data_panel.run_if(data_panel_enabled));
    }
}

// fn create_plotting_data(mut commands: Commands) {
//     commands.insert_resource(plotpoints);
// }

// #[derive(Resource, Deref, DerefMut)]
// struct PlotData(egui_plot::PlotPoints);

#[inline]
fn data_panel_enabled(ui_state: Res<UiState>) -> bool {
    ui_state.bottom_panel_visible
}

fn render_data_panel(
    mut contexts: EguiContexts,
    // config: Res<Config>,
    ui_state: Res<UiState>,
    mut q_graph: Query<&mut BasicGraph>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();
    let mut graph = q_graph.single_mut();

    // egui::containers::was_tooltip_open_last_frame(, )

    // let floating_window = egui::containers::Window::new("Floating Window")
    //     .min_size((500.0, 400.0))
    //     .movable(true)
    //     .title_bar(true)
    //     .show(ctx, |ui| {
    //         let sin: PlotPoints = (0..1000)
    //             .map(|i| {
    //                 let x = i as f64 * 0.01;
    //                 [x, x.sin()]
    //             })
    //             .collect();
    //         let line = Line::new(sin);

    //         Plot::new("my_plot")
    //             .view_aspect(2.0)
    //             .show(ui, |plot_ui| plot_ui.line(line));
    //         // let plot = Plot::new("my_plot").view_aspect(2.0).show_grid(true);
    //         // plot.show(ui, |plot_ui| plot_ui.line(line));
    //     });

    let bottom_panel = egui::TopBottomPanel::bottom("Bottom Panel")
        .default_height(100.0)
        .resizable(false)
        .show_animated(ctx, ui_state.bottom_panel_visible, |ui| {
            // let progressbar = egui::widgets::ProgressBar::new(0.5).fill(Color32::RED);
            // ui.add(progressbar);
            // ui.code("bottom panel");

            // let sin: PlotPoints = (0..50)
            //     .map(|i| {
            //         let x = i as f64 * 0.01;
            //         [x, x.sin()]
            //     })
            //     .collect();
            // let line = Line::new(sin);

            // // ui.add(plot.try_into());
            // Plot::new("my_plot")
            //     .view_aspect(2.0)
            //     .show(ui, |plot_ui| plot_ui.line(line));

            // egui::CentralPanel::default().show(ctx, |ui| {
            ui.add(&mut GraphView::<
                _,
                _,
                _,
                _,
                DefaultNodeShape,
                DefaultEdgeShape,
            >::new(&mut graph.0));
            // });
        });

    occupied_screen_space.bottom =
        bottom_panel.map_or(0.0, |ref inner| inner.response.rect.width());
}

#[derive(Component)]
pub struct BasicGraph(pub Graph<(), ()>);

impl BasicGraph {
    fn new() -> Self {
        let g = generate_graph();
        Self(Graph::from(&g))
    }
}

fn generate_graph() -> StableGraph<(), ()> {
    let mut g = StableGraph::new();

    let a = g.add_node(());
    let b = g.add_node(());
    let c = g.add_node(());

    g.add_edge(a, b, ());
    g.add_edge(b, c, ());
    g.add_edge(c, a, ());

    g
}

fn setup(mut commands: Commands) {
    // add an entity with an egui_graphs::Graph component
    commands.spawn(BasicGraph::new());
}

// fn update_graph(mut contexts: EguiContexts, mut q_graph: Query<&mut
// BasicGraph>) {     let ctx = contexts.ctx_mut();
//     let mut graph = q_graph.single_mut();

//     egui::CentralPanel::default().show(ctx, |ui| {
//         ui.add(&mut GraphView::<
//             _,
//             _,
//             _,
//             _,
//             DefaultNodeShape,
//             DefaultEdgeShape,
//         >::new(&mut graph.0));
//     });
// }
