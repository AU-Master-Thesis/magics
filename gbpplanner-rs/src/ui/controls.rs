use bevy::{
    input::{gamepad::GamepadButtonInput, keyboard::KeyboardInput, mouse::MouseButtonInput},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Color32, Layout, RichText, Sense, Vec2},
    EguiContexts,
};
use leafwing_input_manager::{
    input_map::InputMap,
    user_input::{InputKind, UserInput},
};
use strum::IntoEnumIterator;

use super::{custom, OccupiedScreenSpace, ToDisplayString, UiState};
use crate::{
    config::Config,
    input::{
        CameraAction, CameraSensitivity, GeneralAction, InputAction, MoveableObjectAction,
        MoveableObjectSensitivity, UiAction,
    },
    theme::{CatppuccinTheme, FromCatppuccinColourExt},
};

pub struct ControlsPanelPlugin;

impl Plugin for ControlsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChangingBinding>().add_systems(
            Update,
            (
                ui_controls_panel,
                change_binding_keyboard,
                change_binding_gamepad,
                change_binding_mouse,
                binding_cooldown_system,
            ),
        );
    }
}

/// **Bevy** [`Resource`] to store the currently changing binding
/// If this is not `default`, then all input will be captured and the binding
/// will be updated Blocks ALL actions (including UI actions) while changing a
/// binding
#[derive(Debug, Default, Resource)]
pub struct ChangingBinding {
    pub action:  InputAction,
    pub binding: usize,
    cooldown:    f32,
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

    // Refresh the cooldown
    pub fn refresh_cooldown(&mut self) {
        self.cooldown = 0.1;
    }
}

fn binding_cooldown_system(time: Res<Time<Real>>, mut currently_changing: ResMut<ChangingBinding>) {
    if currently_changing.on_cooldown() {
        // info!("Cooldown: {}", currently_changing.cooldown);
        currently_changing.decrease_cooldown(time.delta_seconds());
    }
}

/// `Update` **Bevy** system to render the `egui` UI
/// Uses the `UiState` to understand which panels are open and should be
/// rendered
fn ui_controls_panel(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut ui_state: ResMut<UiState>,
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
    catppuccin_theme: Res<CatppuccinTheme>,
    mut currently_changing: ResMut<ChangingBinding>,
    config: Res<Config>,
    mut object_sensitivity: Option<ResMut<MoveableObjectSensitivity>>,
    mut camera_sensitivity: ResMut<CameraSensitivity>,
) {
    let ctx = contexts.ctx_mut();

    let mut title_colors = [
        catppuccin_theme.green(),
        catppuccin_theme.blue(),
        catppuccin_theme.mauve(),
        catppuccin_theme.maroon(),
        catppuccin_theme.lavender(),
    ]
    .into_iter()
    .cycle();

    let left_panel = egui::SidePanel::left("left_panel")
        .default_width(300.0)
        .resizable(false)
        .show_animated(ctx, ui_state.left_panel_visible, |ui| {
            if ui.rect_contains_pointer(ui.max_rect()) && config.interaction.ui_focus_cancels_inputs
            {
                ui_state.mouse_over.left_panel = true;
            } else {
                ui_state.mouse_over.left_panel = false;
            }

            custom::heading(ui, "Controls", None);

            let panel_height = ui.available_rect_before_wrap().height();
            egui::ScrollArea::vertical()
                .max_height(panel_height - 80.0)
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    custom::subheading(
                        ui,
                        "Sensitivity",
                        Some(Color32::from_catppuccin_colour(catppuccin_theme.lavender())),
                    );
                    ui.push_id("sensitivity_table", |ui| {
                        custom::sens_table(ui).body(|mut body| {
                            if let Some(mut object_sensitivity) = object_sensitivity.as_mut() {
                                body.row(custom::ROW_HEIGHT + custom::SPACING, |mut row| {
                                    row.col(|col| {
                                        col.label("Object Movement");
                                    });
                                    row.col(|col| {
                                        col.spacing_mut().slider_width = col.available_width()
                                            - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                                        col.add(
                                            egui::Slider::new(
                                                &mut object_sensitivity.move_sensitivity,
                                                0.0..=2.0,
                                            )
                                            .show_value(true)
                                            .custom_formatter(|x, _| format!("{:.0}", x * 100.0))
                                            .text("%"),
                                        );
                                    });
                                });

                                body.row(custom::ROW_HEIGHT + custom::SPACING, |mut row| {
                                    row.col(|col| {
                                        col.label("Object Rotation");
                                    });
                                    row.col(|col| {
                                        col.spacing_mut().slider_width = col.available_width()
                                            - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                                        col.add(
                                            egui::Slider::new(
                                                &mut object_sensitivity.rotate_sensitivity,
                                                0.0..=2.0,
                                            )
                                            .show_value(true)
                                            .custom_formatter(|x, _| format!("{:.0}", x * 100.0))
                                            .text("%"),
                                        );
                                    });
                                });
                            }
                            // body.row(custom::ROW_HEIGHT + custom::SPACING, |mut row| {
                            //     row.col(|col| {
                            //         col.label("Object Movement");
                            //     });
                            //     row.col(|col| {
                            //         col.spacing_mut().slider_width = col.available_width()
                            //             - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                            //         col.add(
                            //             egui::Slider::new(
                            //                 &mut object_sensitivity.move_sensitivity,
                            //                 0.0..=2.0,
                            //             )
                            //             .show_value(true)
                            //             .custom_formatter(|x, _| format!("{:.0}", x * 100.0))
                            //             .text("%"),
                            //         );
                            //     });
                            // });

                            // body.row(custom::ROW_HEIGHT + custom::SPACING, |mut row| {
                            //     row.col(|col| {
                            //         col.label("Object Rotation");
                            //     });
                            //     row.col(|col| {
                            //         col.spacing_mut().slider_width = col.available_width()
                            //             - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                            //         col.add(
                            //             egui::Slider::new(
                            //                 &mut object_sensitivity.rotate_sensitivity,
                            //                 0.0..=2.0,
                            //             )
                            //             .show_value(true)
                            //             .custom_formatter(|x, _| format!("{:.0}", x * 100.0))
                            //             .text("%"),
                            //         );
                            //     });
                            // });
                            body.row(custom::ROW_HEIGHT + custom::SPACING, |mut row| {
                                row.col(|col| {
                                    col.label("Camera Movement");
                                });
                                row.col(|col| {
                                    col.spacing_mut().slider_width = col.available_width()
                                        - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                                    col.add(
                                        egui::Slider::new(
                                            &mut camera_sensitivity.move_sensitivity,
                                            0.0..=2.0,
                                        )
                                        .show_value(true)
                                        .custom_formatter(|x, _| format!("{:.0}", x * 100.0))
                                        .text("%"),
                                    );
                                });
                            });
                        });

                        custom::separator(ui, None, Some(5.0), Some(5.0));

                        let size = 15.0; // pt

                        ui.push_id("binding_header_table", |ui| {
                            custom::binding_table(ui).striped(false).body(|mut body| {
                                body.row(custom::BINDING_ROW_HEIGHT, |mut row| {
                                    row.col(|col| {
                                        custom::center_y(col, |col| {
                                            col.label(RichText::new("Binding").size(size).color(
                                                Color32::from_catppuccin_colour(
                                                    catppuccin_theme.lavender(),
                                                ),
                                            ));
                                        });
                                    });
                                    row.col(|col| {
                                        custom::fill_x(col, |col| {
                                            col.label(RichText::new("󰌌").size(size + 5.0).color(
                                                Color32::from_catppuccin_colour(
                                                    catppuccin_theme.lavender(),
                                                ),
                                            ));
                                        });
                                    });
                                    row.col(|col| {
                                        custom::fill_x(col, |col| {
                                            col.label(RichText::new("󰊗").size(size + 5.0).color(
                                                Color32::from_catppuccin_colour(
                                                    catppuccin_theme.lavender(),
                                                ),
                                            ));
                                        });
                                    });
                                });
                            });
                        });

                        // go through all InputAction variants, and make a title for each
                        // then nested go through each inner variant and make a button for each

                        for action in InputAction::iter() {
                            if matches!(action, InputAction::Undefined) {
                                continue;
                            }
                            match action {
                                InputAction::MoveableObject(_) => {
                                    if query_moveable_object_action.iter().count() != 0 {
                                        custom::subheading(
                                            ui,
                                            action.to_string().as_str(),
                                            Some(Color32::from_catppuccin_colour(
                                                title_colors.next().expect("From cycle iterator"),
                                            )),
                                        );
                                    }

                                    ui.push_id(format!("{}_table", action.to_string()), |ui| {
                                        custom::binding_table(ui).body(|body| {
                                            // for map in query_moveable_object_action.iter() {

                                            if let Ok(map) =
                                                query_moveable_object_action.get_single_mut()
                                            {
                                                body.rows(
                                                    custom::BINDING_ROW_HEIGHT,
                                                    map.iter().count(),
                                                    |mut row| {
                                                        let row_index = row.index();

                                                        let inner_action =
                                                            map.iter().nth(row_index).expect(
                                                                "Table row amount is equal to map \
                                                                 length",
                                                            );

                                                        row.col(|col| {
                                                            col.with_layout(
                                                                Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |col| {
                                                                    col.label(
                                                                        inner_action.0.to_string(),
                                                                    );
                                                                },
                                                            );
                                                        });

                                                        for r in 0..2 {
                                                            let button_content =
                                                                inner_action.1.get(r).map_or_else(
                                                                    || "".to_string(),
                                                                    |ia| ia.to_display_string(),
                                                                );

                                                            row.col(|col| {
                                                                col.add_space(custom::SPACING);
                                                                let (rect, _) = col
                                                                    .allocate_exact_size(
                                                                    Vec2::new(
                                                                        custom::BINDING_COL_WIDTH
                                                                            - 2.0 * custom::SPACING,
                                                                        custom::BINDING_ROW_HEIGHT
                                                                            - 2.0 * custom::SPACING,
                                                                    ),
                                                                    Sense::hover(),
                                                                );
                                                                col.allocate_ui_at_rect(rect, |ui| {
                                                        custom::fill_x(ui, |ui| {
                                                            if ui.button(RichText::new(
                                                                button_content,
                                                                )).clicked() {
                                                                *currently_changing =
                                                                    ChangingBinding::new(
                                                                        InputAction::MoveableObject(
                                                                            *inner_action.0,
                                                                        ),
                                                                        r,
                                                                    );
                                                            }
                                                        });
                                                    });
                                                            });
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    });
                                }
                                InputAction::General(_) => {
                                    if query_general_action.iter().count() != 0 {
                                        custom::subheading(
                                            ui,
                                            action.to_string().as_str(),
                                            Some(Color32::from_catppuccin_colour(
                                                title_colors.next().expect("From cycle iterator"),
                                            )),
                                        );
                                    }

                                    ui.push_id(format!("{}_table", action.to_string()), |ui| {
                                        custom::binding_table(ui).body(|body| {
                                            // for map in query_general_action.iter() {

                                            if let Ok(map) = query_general_action.get_single_mut() {
                                                body.rows(
                                                    custom::BINDING_ROW_HEIGHT,
                                                    map.iter().count(),
                                                    |mut row| {
                                                        let row_index = row.index();

                                                        let inner_action =
                                                            map.iter().nth(row_index).expect(
                                                                "Table row amount is equal to map \
                                                                 length",
                                                            );

                                                        row.col(|col| {
                                                            col.with_layout(
                                                                Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |col| {
                                                                    col.label(
                                                                        inner_action.0.to_string(),
                                                                    );
                                                                },
                                                            );
                                                        });

                                                        for r in 0..2 {
                                                            let button_content =
                                                                inner_action.1.get(r).map_or_else(
                                                                    || "".to_string(),
                                                                    |ia| ia.to_display_string(),
                                                                );

                                                            row.col(|col| {
                                                                col.add_space(custom::SPACING);
                                                                let (rect, _) = col
                                                                    .allocate_exact_size(
                                                                    Vec2::new(
                                                                        custom::BINDING_COL_WIDTH
                                                                            - 2.0 * custom::SPACING,
                                                                        custom::BINDING_ROW_HEIGHT
                                                                            - 2.0 * custom::SPACING,
                                                                    ),
                                                                    Sense::hover(),
                                                                );
                                                                col.allocate_ui_at_rect(rect, |ui| {
                                                        custom::fill_x(ui, |ui| {
                                                            if ui.button(RichText::new(
                                                                button_content,
                                                                )).clicked() {
                                                                *currently_changing =
                                                                    ChangingBinding::new(
                                                                        InputAction::General(
                                                                            *inner_action.0,
                                                                        ),
                                                                        r,
                                                                    );
                                                            }
                                                        });
                                                    });
                                                            });
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    });
                                }
                                InputAction::Camera(_) => {
                                    if query_camera_action.iter().count() != 0 {
                                        custom::subheading(
                                            ui,
                                            action.to_string().as_str(),
                                            Some(Color32::from_catppuccin_colour(
                                                title_colors.next().expect("From cycle iterator"),
                                            )),
                                        );
                                    }

                                    ui.push_id(format!("{}_table", action.to_string()), |ui| {
                                        custom::binding_table(ui).body(|body| {
                                            // for map in query_camera_action.iter() {

                                            if let Ok(map) = query_camera_action.get_single_mut() {
                                                body.rows(
                                                    custom::BINDING_ROW_HEIGHT,
                                                    map.iter().count(),
                                                    |mut row| {
                                                        let row_index = row.index();

                                                        let inner_action =
                                                            map.iter().nth(row_index).expect(
                                                                "Table row amount is equal to map \
                                                                 length",
                                                            );

                                                        row.col(|col| {
                                                            col.with_layout(
                                                                Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |col| {
                                                                    col.label(
                                                                        inner_action.0.to_string(),
                                                                    );
                                                                },
                                                            );
                                                        });

                                                        for r in 0..2 {
                                                            let button_content =
                                                                inner_action.1.get(r).map_or_else(
                                                                    || "".to_string(),
                                                                    |ia| ia.to_display_string(),
                                                                );

                                                            row.col(|col| {
                                                                col.add_space(custom::SPACING);
                                                                let (rect, _) = col
                                                                    .allocate_exact_size(
                                                                    Vec2::new(
                                                                        custom::BINDING_COL_WIDTH
                                                                            - 2.0 * custom::SPACING,
                                                                        custom::BINDING_ROW_HEIGHT
                                                                            - 2.0 * custom::SPACING,
                                                                    ),
                                                                    Sense::hover(),
                                                                );
                                                                col.allocate_ui_at_rect(rect, |ui| {
                                                        custom::fill_x(ui, |ui| {
                                                            if ui.button(RichText::new(
                                                                button_content,
                                                                )).clicked() {
                                                                *currently_changing =
                                                                    ChangingBinding::new(
                                                                        InputAction::Camera(
                                                                            *inner_action.0,
                                                                        ),
                                                                        r,
                                                                    );
                                                            }
                                                        });
                                                    });
                                                            });
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    });
                                }
                                InputAction::Ui(_) => {
                                    if query_ui_action.iter().count() != 0 {
                                        custom::subheading(
                                            ui,
                                            action.to_string().as_str(),
                                            Some(Color32::from_catppuccin_colour(
                                                title_colors.next().expect("From cycle iterator"),
                                            )),
                                        );
                                    }

                                    ui.push_id(format!("{}_table", action.to_string()), |ui| {
                                        custom::binding_table(ui).body(|body| {
                                            // for map in query_ui_action.iter() {

                                            if let Ok(map) = query_ui_action.get_single_mut() {
                                                body.rows(
                                                    custom::BINDING_ROW_HEIGHT,
                                                    map.iter().count(),
                                                    |mut row| {
                                                        let row_index = row.index();

                                                        let inner_action =
                                                            map.iter().nth(row_index).expect(
                                                                "Table row amount is equal to map \
                                                                 length",
                                                            );

                                                        row.col(|col| {
                                                            col.with_layout(
                                                                Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |col| {
                                                                    col.label(
                                                                        inner_action.0.to_string(),
                                                                    );
                                                                },
                                                            );
                                                        });

                                                        // inner_action.1.iter().enumerate().
                                                        // for_each(
                                                        for r in 0..2 {
                                                            let button_content =
                                                                inner_action.1.get(r).map_or_else(
                                                                    || "".to_string(),
                                                                    |ia| ia.to_display_string(),
                                                                );

                                                            // |(i, x)| {
                                                            row.col(|col| {
                                                                col.add_space(custom::SPACING);
                                                                let (rect, _) = col
                                                                    .allocate_exact_size(
                                                                    Vec2::new(
                                                                        custom::BINDING_COL_WIDTH
                                                                            - 2.0 * custom::SPACING,
                                                                        custom::BINDING_ROW_HEIGHT
                                                                            - 2.0 * custom::SPACING,
                                                                    ),
                                                                    Sense::hover(),
                                                                );
                                                                col.allocate_ui_at_rect(rect, |ui| {
                                                            custom::fill_x(ui, |ui| {
                                                                if ui.button(RichText::new(
                                                                    button_content,
                                                                    )).clicked() {
                                                                    *currently_changing =
                                                                        ChangingBinding::new(
                                                                            InputAction::Ui(
                                                                                *inner_action.0,
                                                                            ),
                                                                            r,
                                                                        );
                                                                }
                                                            });
                                                        });
                                                            });
                                                            // };
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    });
                                }
                                _ => { /* do nothing */ }
                            }
                        }
                    });
                });

            // CURRENT CHANGING BINDING INFO
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if !matches!(currently_changing.action, InputAction::Undefined) {
                    ui.columns(2, |columns| {
                        columns[0].label(
                            RichText::new("Currently binding:").italics().color(
                                Color32::from_catppuccin_colour(catppuccin_theme.overlay2()),
                            ),
                        );
                        columns[1].centered_and_justified(|ui| {
                            let _ = ui.button(currently_changing.action.to_display_string());
                        });
                    });
                } else {
                    ui.label(
                        RichText::new("Select a binding to change")
                            .italics()
                            .color(Color32::from_catppuccin_colour(catppuccin_theme.overlay2())),
                    );
                }
            });

            // buttons to cancel the currently changing binding
            // and to unbind the currently changing binding
            if !matches!(currently_changing.action, InputAction::Undefined) {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.columns(2, |columns| {
                        columns[0].centered_and_justified(|ui| {
                            if ui.button("Cancel (ESC)").clicked() {
                                *currently_changing = ChangingBinding::default();
                            }
                        });
                        columns[1].centered_and_justified(|ui| {
                            if ui.button("Unbind").clicked() {
                                match currently_changing.action {
                                    InputAction::Camera(action) => {
                                        let mut map = query_camera_action.single_mut();
                                        let bindings = map.get_mut(&action);
                                        if let Some(bindings) = bindings {
                                            if bindings.len() > currently_changing.binding {
                                                bindings.remove(currently_changing.binding);
                                            }
                                        }
                                    }
                                    InputAction::General(action) => {
                                        let mut map = query_general_action.single_mut();
                                        let bindings = map.get_mut(&action);
                                        if let Some(bindings) = bindings {
                                            if bindings.len() > currently_changing.binding {
                                                bindings.remove(currently_changing.binding);
                                            }
                                        }
                                    }
                                    InputAction::MoveableObject(action) => {
                                        let mut map = query_moveable_object_action.single_mut();
                                        let bindings = map.get_mut(&action);
                                        if let Some(bindings) = bindings {
                                            if bindings.len() > currently_changing.binding {
                                                bindings.remove(currently_changing.binding);
                                            }
                                        }
                                    }
                                    InputAction::Ui(action) => {
                                        let mut map = query_ui_action.single_mut();
                                        let bindings = map.get_mut(&action);
                                        if let Some(bindings) = bindings {
                                            if bindings.len() > currently_changing.binding {
                                                bindings.remove(currently_changing.binding);
                                            }
                                        }
                                    }
                                    _ => { /* do nothing */ }
                                }
                                *currently_changing = ChangingBinding::default();
                            }
                        });
                    });
                });
            }
        });

    occupied_screen_space.left = left_panel
        .map(|ref inner| inner.response.rect.width())
        .unwrap_or(0.0);
}

/// **Bevy** [`Update`] system
/// Listens for any keyboard events to rebind currently changing binding
fn change_binding_keyboard(
    query_camera_action: Query<&mut InputMap<CameraAction>>,
    query_general_action: Query<&mut InputMap<GeneralAction>>,
    query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    query_ui_action: Query<&mut InputMap<UiAction>>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut keyboard_events: EventReader<KeyboardInput>,
) {
    if !currently_changing.is_changing() {
        return;
    }
    // Listen for keyboard events to rebind currently changing binding
    if let Some(event) = keyboard_events.read().next() {
        let key_code = event.key_code;

        // If the escape key is pressed, then don't change the binding
        if matches!(key_code, KeyCode::Escape) {
            *currently_changing = ChangingBinding::default();
        }

        rebind(
            // currently_changing.action,
            &mut currently_changing,
            UserInput::Single(InputKind::PhysicalKey(key_code)),
            query_camera_action,
            query_general_action,
            query_moveable_object_action,
            query_ui_action,
        );

        *currently_changing = ChangingBinding::default().with_cooldown(0.1);
    }
}

/// **Bevy** [`Update`] system
/// Listens for any gamepad button events to rebind currently changing binding
fn change_binding_gamepad(
    query_camera_action: Query<&mut InputMap<CameraAction>>,
    query_general_action: Query<&mut InputMap<GeneralAction>>,
    query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    query_ui_action: Query<&mut InputMap<UiAction>>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut gamepad_button_events: EventReader<GamepadButtonInput>,
) {
    if !currently_changing.is_changing() {
        return;
    }
    // Listen for gamepad button events to rebind currently changing binding
    if let Some(event) = gamepad_button_events.read().next() {
        let button = event.button;

        rebind(
            // currently_changing.action,
            &mut currently_changing,
            UserInput::Single(InputKind::GamepadButton(button.button_type)),
            query_camera_action,
            query_general_action,
            query_moveable_object_action,
            query_ui_action,
        );

        *currently_changing = ChangingBinding::default().with_cooldown(0.1);
    }
}

/// **Bevy** [`Update`] system
/// Listens for mouse button events to rebind currently changing binding
fn change_binding_mouse(
    query_camera_action: Query<&mut InputMap<CameraAction>>,
    query_general_action: Query<&mut InputMap<GeneralAction>>,
    query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    query_ui_action: Query<&mut InputMap<UiAction>>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut mouse_button_events: EventReader<MouseButtonInput>,
) {
    if !currently_changing.is_changing() {
        return;
    }
    // Listen for mouse button events to rebind currently changing binding
    if let Some(event) = mouse_button_events.read().next() {
        let button = event.button;

        rebind(
            // currently_changing.action,
            &mut currently_changing,
            UserInput::Single(InputKind::Mouse(button)),
            query_camera_action,
            query_general_action,
            query_moveable_object_action,
            query_ui_action,
        );

        *currently_changing = ChangingBinding::default().with_cooldown(0.1);
    }
}

fn rebind(
    // action: InputAction,
    currently_changing: &mut ChangingBinding,
    new_binding: UserInput,
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
) {
    // let action = currently_changing.action;
    match currently_changing.action {
        InputAction::Camera(action) => {
            let Ok(mut map) = query_camera_action.get_single_mut() else {
                return;
            };
            let bindings = map.get_mut(&action);
            if let Some(bindings) = bindings {
                if bindings.len() > currently_changing.binding {
                    bindings.remove(currently_changing.binding);
                }
                bindings.insert(currently_changing.binding, new_binding);
            }
        }
        InputAction::General(action) => {
            let Ok(mut map) = query_general_action.get_single_mut() else {
                return;
            };
            let bindings = map.get_mut(&action);
            if let Some(bindings) = bindings {
                if bindings.len() > currently_changing.binding {
                    bindings.remove(currently_changing.binding);
                }
                bindings.insert(currently_changing.binding, new_binding);
            }
        }
        InputAction::MoveableObject(action) => {
            let Ok(mut map) = query_moveable_object_action.get_single_mut() else {
                return;
            };
            let bindings = map.get_mut(&action);
            if let Some(bindings) = bindings {
                if bindings.len() > currently_changing.binding {
                    bindings.remove(currently_changing.binding);
                }
                bindings.insert(currently_changing.binding, new_binding);
            }
        }
        InputAction::Ui(action) => {
            let Ok(mut map) = query_ui_action.get_single_mut() else {
                return;
            };
            let bindings = map.get_mut(&action);
            if let Some(bindings) = bindings {
                if bindings.len() > currently_changing.binding {
                    bindings.remove(currently_changing.binding);
                }
                bindings.insert(currently_changing.binding, new_binding);
            }
        }
        _ => { /* do nothing */ }
    }
}
