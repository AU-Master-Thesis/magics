use bevy::prelude::*;
use bevy_egui::{
    egui::{self, RichText},
    EguiContexts, EguiPlugin,
};

pub struct EguiInterfacePlugin;

impl Plugin for EguiInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OccupiedScreenSpace>()
            .init_resource::<UiState>()
            .add_plugins(EguiPlugin)
            .add_systems(Update, ui_example_system);
    }
}

// fn ui_example_system(mut contexts: EguiContexts) {
//     egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
//         ui.label("world");
//     });
// }

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    // top: f32,
    // right: f32,
    // bottom: f32,
}

#[derive(Default, Resource)]
pub struct UiState {
    pub left_panel: bool,
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut ui_state: ResMut<UiState>,
) {
    let ctx = contexts.ctx_mut();

    let side_panel = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show_animated(ctx, ui_state.left_panel, |ui| {
            ui.label(RichText::new("Bindings").heading());
            ui.label(RichText::new("Keyboard").raised());
            ui.label("◀ ▲ ▼ ▶ - Move camera");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    // occupied_screen_space.left =
    // .show(ctx, |ui| {
    //     ui.label(RichText::new("Bindings").heading());
    //     ui.label(RichText::new("Keyboard").raised());
    //     ui.label("◀ ▲ ▼ ▶ - Move camera");
    //     ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    // })
    // .unwrap_or_default()
    // .response
    // .rect
    // .width();
    // occupied_screen_space.right = egui::SidePanel::right("right_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Right resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .width();
    // occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Top resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .height();
    // occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Bottom resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .height();
}
