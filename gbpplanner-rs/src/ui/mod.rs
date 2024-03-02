use bevy::{prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{self, Color32, RichText, Visuals},
    EguiContexts, EguiPlugin,
};
use catppuccin::Flavour;
use color_eyre::owo_colors::OwoColorize;
use leafwing_input_manager::input_map::InputMap;
use strum::IntoEnumIterator;

use crate::input::{
    CameraAction, GeneralAction, InputAction, MoveableObjectAction, UiAction,
};
use crate::theme::{CatppuccinTheme, CatppuccinThemeExt};

//  _     _ _______ _______  ______      _____ __   _ _______ _______  ______ _______ _______ _______ _______
//  |     | |______ |______ |_____/        |   | \  |    |    |______ |_____/ |______ |_____| |       |______
//  |_____| ______| |______ |    \_      __|__ |  \_|    |    |______ |    \_ |       |     | |_____  |______
//

pub struct EguiInterfacePlugin;

impl Plugin for EguiInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OccupiedScreenSpace>()
            .init_resource::<UiState>()
            .add_plugins(EguiPlugin)
            .add_systems(Startup, configure_visuals_system)
            .add_systems(Update, (ui_example_system));
    }
}

/// Resource to store the occupied screen space by each `egui` panel
#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
}

/// UI state to represent which `equi` panels are open
#[derive(Default, Resource)]
pub struct UiState {
    pub left_panel: bool,
}

/// `Setup` **Bevy** sytem to initialise the `egui` visuals
/// This is where the **default** for `egui` is set
fn configure_visuals_system(
    mut contexts: EguiContexts,
    catppuccin: Res<CatppuccinTheme>,
    windows: Query<&Window>,
) {
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

/// `Update` **Bevy** system to render the `egui` UI
/// Uses the `UiState` to understand which panels are open and should be rendered
fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    ui_state: ResMut<UiState>,
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
) {
    let ctx = contexts.ctx_mut();

    let left_panel = egui::SidePanel::left("left_panel")
        .default_width(300.0)
        .resizable(true)
        .show_animated(ctx, ui_state.left_panel, |ui| {
            ui.label(RichText::new("Bindings").heading());
            // ui.label(RichText::new("Keyboard").raised());
            // ui.label("◀ ▲ ▼ ▶ - Move camera");

            // go through all InputAction variants, and make a title for each
            // then nested go through each inner variant and make a button for each
            for action in InputAction::iter() {
                ui.label(RichText::new(action.to_string()).strong());
                match action {
                    InputAction::MoveableObject(_) => {
                        let mut map = query_moveable_object_action.single_mut();
                        for inner_action in map.iter() {
                            // let _ = ui.button(RichText::new(inner_action.to_string()));
                            // put inner_action.0 as a label to the left, and inner_action.1 as a button to the right
                            // with space in between
                            ui.horizontal(|ui| {
                                ui.label(inner_action.0.to_string());
                                let _ = ui.button(RichText::new(
                                    inner_action
                                        .1
                                        .iter()
                                        .map(|x| x.to_string())
                                        .collect::<String>(),
                                ));
                            });
                        }
                    }
                    InputAction::General(_) => {
                        let mut map = query_general_action.single_mut();
                        for inner_action in map.iter() {
                            ui.horizontal(|ui| {
                                ui.label(inner_action.0.to_string());
                                let _ = ui.button(RichText::new(
                                    inner_action
                                        .1
                                        .iter()
                                        .map(|x| x.to_string())
                                        .collect::<String>(),
                                ));
                            });
                        }
                    }
                    InputAction::Camera(_) => {
                        let mut map = query_camera_action.iter_mut().next().unwrap();
                        for inner_action in map.iter() {
                            ui.horizontal(|ui| {
                                ui.label(inner_action.0.to_string());
                                let _ = ui.button(RichText::new(
                                    inner_action
                                        .1
                                        .iter()
                                        .map(|x| x.to_string())
                                        .collect::<String>(),
                                ));
                            });
                        }
                    }
                    InputAction::Ui(_) => {
                        let mut map = query_ui_action.single_mut();
                        for inner_action in map.iter() {
                            ui.horizontal(|ui| {
                                ui.label(inner_action.0.to_string());
                                let _ = ui.button(RichText::new(
                                    inner_action
                                        .1
                                        .iter()
                                        .map(|x| x.to_string())
                                        .collect::<String>(),
                                ));
                            });
                        }
                    }
                }
            }

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    occupied_screen_space.left = if left_panel.is_some() {
        left_panel.unwrap().response.rect.width()
    } else {
        0.0
    };
}
