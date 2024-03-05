use bevy::{
    input::{gamepad::GamepadButtonInput, keyboard::KeyboardInput},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts,
};
use leafwing_input_manager::{
    input_map::InputMap,
    user_input::{InputKind, UserInput},
};
use strum::IntoEnumIterator;

use crate::{
    input::{CameraAction, GeneralAction, InputAction, MoveableObjectAction, UiAction},
    theme::{CatppuccinTheme, FromCatppuccinColourExt},
};

use super::{OccupiedScreenSpace, ToDisplayString, UiState};

pub struct BindingsPanelPlugin;

impl Plugin for BindingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangingBinding>().add_systems(
            Update,
            (
                ui_binding_panel,
                change_binding_keyboard,
                change_binding_gamepad,
                binding_cooldown_system,
            ),
        );
    }
}

/// **Bevy** `Resource` to store the currently changing binding
/// If this is not `default`, then all input will be captured and the binding will be updated
#[derive(Debug, Default, Resource)]
pub struct ChangingBinding {
    pub action: InputAction,
    pub binding: usize,
    cooldown: f32,
}

impl ChangingBinding {
    pub fn new(action: InputAction, binding: usize) -> Self {
        Self {
            action,
            binding,
            cooldown: 0.0,
        }
    }
    pub fn is_changing(&self) -> bool {
        !matches!(self.action, InputAction::Undefined)
    }

    pub fn on_cooldown(&self) -> bool {
        self.cooldown > 0.0
    }

    pub fn with_cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown;
        self
    }

    // Decrease the cooldown by `delta`, ensuring that it does not go below 0
    pub fn decrease_cooldown(&mut self, delta: f32) {
        self.cooldown -= delta;
        if self.cooldown < 0.0 {
            self.cooldown = 0.0;
        }
    }
}

fn binding_cooldown_system(time: Res<Time>, mut currently_changing: ResMut<ChangingBinding>) {
    if currently_changing.on_cooldown() {
        currently_changing.decrease_cooldown(time.delta_seconds());
    }
}

/// `Update` **Bevy** system to render the `egui` UI
/// Uses the `UiState` to understand which panels are open and should be rendered
fn ui_binding_panel(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    ui_state: ResMut<UiState>,
    query_camera_action: Query<&InputMap<CameraAction>>,
    query_general_action: Query<&InputMap<GeneralAction>>,
    query_moveable_object_action: Query<&InputMap<MoveableObjectAction>>,
    query_ui_action: Query<&InputMap<UiAction>>,
    catppuccin: Res<CatppuccinTheme>,
    mut currently_changing: ResMut<ChangingBinding>,
) {
    let ctx = contexts.ctx_mut();

    let grid_row_color = catppuccin.flavour.mantle();
    let grid_title_colors = [
        catppuccin.flavour.green(),
        catppuccin.flavour.blue(),
        catppuccin.flavour.mauve(),
        catppuccin.flavour.maroon(),
        catppuccin.flavour.lavender(),
    ];

    let mut counter = 1; // offset by 1 to account for header row
    let mut grid_title_rows = Vec::with_capacity(InputAction::iter().count());
    let grid_map_ranges = InputAction::iter()
        .map(|variant| {
            grid_title_rows.push(counter);
            counter += 1;
            let start = counter;
            match variant {
                InputAction::Camera(_) => {
                    counter += CameraAction::iter().count();
                }
                InputAction::General(_) => {
                    counter += GeneralAction::iter().count();
                }
                InputAction::MoveableObject(_) => {
                    counter += MoveableObjectAction::iter().count();
                }
                InputAction::Ui(_) => {
                    counter += UiAction::iter().count();
                }
                _ => { /* do nothing */ }
            }
            let end = counter;

            (start..end).step_by(2)
        })
        .flatten()
        .collect::<Vec<usize>>();

    let left_panel = egui::SidePanel::left("left_panel")
        .default_width(300.0)
        .resizable(false)
        .show_animated(ctx, ui_state.left_panel, |ui| {
            ui.add_space(10.0);
            ui.heading("Binding Panel");
            ui.add_space(5.0);
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(10.0);
                egui::Grid::new("cool_grid")
                    .num_columns(3)
                    .min_col_width(100.0)
                    // .striped(true)
                    .spacing((10.0, 10.0))
                    .with_row_color(move |r, _| {
                        if grid_map_ranges.iter().any(|x| *x == r) {
                            Some(Color32::from_catppuccin_colour(grid_row_color))
                        } else if grid_title_rows.iter().any(|x| *x == r) {
                            // TODO: Do this better
                            Some(Color32::from_catppuccin_colour_with_alpha(grid_title_colors[grid_title_rows.iter().position(|&e| e == r).expect("In a conditional branch that ensures this")], 0.5))
                        } else {
                            None
                        }
                    })
                    .show(ui, |ui| {
                        let size = 15.0; // pt
                        ui.label(RichText::new("Binding").size(size).color(
                            // Color32::from_catppuccin_colour_with_alpha(catppuccin.flavour.lavender(), 0.5),
                            Color32::from_catppuccin_colour(catppuccin.flavour.lavender()),
                        ));
                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("󰌌").size(size + 5.0).color(
                                // Color32::from_catppuccin_colour_with_alpha(catppuccin.flavour.lavender(), 0.5),
                                Color32::from_catppuccin_colour(catppuccin.flavour.lavender()),
                            ));
                        });

                        ui.centered_and_justified(|ui| {
                            ui.label(RichText::new("󰊗").size(size + 5.0).color(
                                // Color32::from_catppuccin_colour_with_alpha(catppuccin.flavour.lavender(), 0.5),
                                Color32::from_catppuccin_colour(catppuccin.flavour.lavender()),
                            ));
                        });

                        ui.end_row();

                        // go through all InputAction variants, and make a title for each
                        // then nested go through each inner variant and make a button for each
                        for action in InputAction::iter() {
                            if matches!(action, InputAction::Undefined) {
                                continue;
                            }
                            ui.label(
                                RichText::new(action.to_string())
                                    .italics()
                                    .color(Color32::from_catppuccin_colour(
                                        catppuccin.flavour.base(),
                                    ))
                                    .size(size),
                            );

                            ui.end_row();
                            match action {
                                InputAction::MoveableObject(_) => {
                                    // let map = query_moveable_object_action.single();
                                    for map in query_moveable_object_action.iter() {
                                        for inner_action in map.iter() {
                                            ui.label(inner_action.0.to_string());

                                            inner_action.1.iter().enumerate().for_each(|(i, x)| {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(
                                                        x.to_display_string(),
                                                    ));
                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::MoveableObject(*inner_action.0),
                                                            i,
                                                        );
                                                    }
                                                });
                                            });

                                            for extra_column in 0..(2 - inner_action.1.len()) {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(""));

                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::MoveableObject(*inner_action.0),
                                                            inner_action.1.len() + extra_column,
                                                        );
                                                    }
                                                });
                                            }

                                            ui.end_row();
                                        }

                                    }
                                }
                                InputAction::General(_) => {
                                    // let map = query_general_action.single();
                                    for map in query_general_action.iter() {
                                        for inner_action in map.iter() {
                                            ui.label(inner_action.0.to_string());

                                            inner_action.1.iter().enumerate().for_each(|(i, x)| {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(
                                                        x.to_display_string(),
                                                    ));
                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::General(*inner_action.0),
                                                            i,
                                                        );
                                                    }
                                                });
                                            });

                                            for extra_column in 0..(2 - inner_action.1.len()) {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(""));

                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::General(*inner_action.0),
                                                            inner_action.1.len() + extra_column,
                                                        );
                                                    }
                                                });
                                            }

                                            ui.end_row();
                                        }
                                    }
                                }
                                InputAction::Camera(_) => {
                                    // let map = query_camera_action.iter().next().unwrap();
                                    for map in query_camera_action.iter() {
                                        for inner_action in map.iter() {
                                            ui.label(inner_action.0.to_string());

                                            inner_action.1.iter().enumerate().for_each(|(i, x)| {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(
                                                        x.to_display_string(),
                                                    ));
                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::Camera(*inner_action.0),
                                                            i,
                                                        );
                                                    }
                                                });
                                            });

                                            for extra_column in 0..(2 - inner_action.1.len()) {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(""));

                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::Camera(*inner_action.0),
                                                            inner_action.1.len() + extra_column,
                                                        );
                                                    }
                                                });
                                            }

                                            ui.end_row();
                                        }
                                    }
                                }
                                InputAction::Ui(_) => {
                                    // let map = query_ui_action.single();
                                    for map in query_ui_action.iter() {
                                        for inner_action in map.iter() {
                                            ui.label(inner_action.0.to_string());

                                            inner_action.1.iter().enumerate().for_each(|(i, x)| {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(
                                                        x.to_display_string(),
                                                    ));
                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::Ui(*inner_action.0),
                                                            i,
                                                        );
                                                    }
                                                });
                                            });

                                            for extra_column in 0..(2 - inner_action.1.len()) {
                                                ui.centered_and_justified(|ui| {
                                                    let button_response = ui.button(RichText::new(""));

                                                    if button_response.clicked() {
                                                        *currently_changing = ChangingBinding::new(
                                                            InputAction::Ui(*inner_action.0),
                                                            inner_action.1.len() + extra_column,
                                                        );
                                                    }
                                                });
                                            }

                                            ui.end_row();
                                        }
                                    }
                                }
                                _ => { /* do nothing */ }
                            }
                        }
                    });
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    occupied_screen_space.left = left_panel
        .map(|ref inner| inner.response.rect.width())
        .unwrap_or(0.0);
}

/// `Update` **Bevy** system
/// Listens for any keyboard events to rebind currently changing binding
fn change_binding_keyboard(
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut keyboard_events: EventReader<KeyboardInput>,
) {
    // Listen for keyboard events to rebind currently changing binding
    // if there is, then rebind the map
    for event in keyboard_events.read() {
        let key_code = event.key_code;

        // If the escape key is pressed, then don't change the binding
        if matches!(key_code, KeyCode::Escape) {
            *currently_changing = ChangingBinding::default();
        }

        // If the currently changing binding is not default, then change the binding
        // let mut bindings = match currently_changing.action {
        match currently_changing.action {
            InputAction::Camera(action) => {
                let mut map = query_camera_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::PhysicalKey(key_code)),
                    );
                }
            }
            InputAction::General(action) => {
                let mut map = query_general_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::PhysicalKey(key_code)),
                    );
                }
            }
            InputAction::MoveableObject(action) => {
                let mut map = query_moveable_object_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::PhysicalKey(key_code)),
                    );
                }
            }
            InputAction::Ui(action) => {
                let mut map = query_ui_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::PhysicalKey(key_code)),
                    );
                }
            }
            _ => { /* do nothing */ } // _ => { None }
        }

        *currently_changing = ChangingBinding::default().with_cooldown(0.1);
    }
}

/// **Bevy** `Update` system
/// Listens for any gamepad button events to rebind currently changing binding
fn change_binding_gamepad(
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut gamepad_button_events: EventReader<GamepadButtonInput>,
) {
    // Listen for gamepad button events to rebind currently changing binding
    for event in gamepad_button_events.read() {
        let button = event.button;

        match currently_changing.action {
            InputAction::Camera(action) => {
                let mut map = query_camera_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::GamepadButton(button.button_type)),
                    );
                }
            }
            InputAction::General(action) => {
                let mut map = query_general_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::GamepadButton(button.button_type)),
                    );
                }
            }
            InputAction::MoveableObject(action) => {
                let mut map = query_moveable_object_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::GamepadButton(button.button_type)),
                    );
                }
            }
            InputAction::Ui(action) => {
                let mut map = query_ui_action.single_mut();
                let bindings = map.get_mut(&action);
                if let Some(bindings) = bindings {
                    if bindings.len() > currently_changing.binding {
                        bindings.remove(currently_changing.binding);
                    }
                    bindings.insert(
                        currently_changing.binding,
                        UserInput::Single(InputKind::GamepadButton(button.button_type)),
                    );
                }
            }
            _ => { /* do nothing */ }
        }

        *currently_changing = ChangingBinding::default().with_cooldown(0.1);
    }
}
