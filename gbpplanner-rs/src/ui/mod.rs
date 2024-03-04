use std::iter::Scan;

use bevy::{
    input::{
        gamepad::{self, GamepadButtonChangedEvent, GamepadButtonInput},
        keyboard::KeyboardInput,
    },
    prelude::*,
    window::WindowTheme,
};
use bevy_egui::{
    egui::{self, Color32, RichText, Visuals},
    EguiContexts, EguiPlugin,
};
use catppuccin::Flavour;
use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;
use leafwing_input_manager::{
    axislike::{
        AxisType, DualAxis, MouseMotionAxisType, MouseWheelAxisType, SingleAxis, VirtualAxis,
        VirtualDPad,
    },
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
    input_map::InputMap,
    user_input::{InputKind, Modifier, UserInput},
};
use strum::IntoEnumIterator;

use crate::theme::{CatppuccinTheme, CatppuccinThemeVisualsExt};
use crate::{
    input::{CameraAction, GeneralAction, InputAction, MoveableObjectAction, UiAction},
    theme::FromCatppuccinColourExt,
};
use heck::ToTitleCase;
use std::fmt;

//  _     _ _______ _______  ______
//  |     | |______ |______ |_____/
//  |_____| ______| |______ |    \_
//
//  _____ __   _ _______ _______  ______ _______ _______ _______ _______
//    |   | \  |    |    |______ |_____/ |______ |_____| |       |______
//  __|__ |  \_|    |    |______ |    \_ |       |     | |_____  |______
//

pub struct EguiInterfacePlugin;

impl Plugin for EguiInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OccupiedScreenSpace>()
            .init_resource::<ChangingBinding>()
            .init_resource::<UiState>()
            .add_plugins(EguiPlugin)
            .add_systems(Startup, configure_visuals_system)
            .add_systems(Update, ui_binding_panel);
    }
}

pub trait ToDisplayString {
    fn to_display_string(&self) -> String;
}

impl ToDisplayString for UserInput {
    fn to_display_string(&self) -> String {
        match self {
            UserInput::Single(input) => input.to_display_string(),
            UserInput::VirtualDPad(virtual_dpad) => virtual_dpad.to_display_string(),
            UserInput::VirtualAxis(virtual_axis) => virtual_axis.to_display_string(),
            UserInput::Chord(chord) => chord
                .iter()
                .map(|x| x.to_display_string())
                .collect::<Vec<String>>()
                .join(" + "),
        }
    }
}

impl ToDisplayString for VirtualDPad {
    fn to_display_string(&self) -> String {
        format!(
            "{}{}{}{}",
            self.up.to_display_string(),
            self.left.to_display_string(),
            self.down.to_display_string(),
            self.right.to_display_string()
        )
    }
}

impl ToDisplayString for VirtualAxis {
    fn to_display_string(&self) -> String {
        format!(
            "{}{}",
            self.positive.to_display_string(),
            self.negative.to_display_string()
        )
    }
}

impl ToDisplayString for InputKind {
    fn to_display_string(&self) -> String {
        match self {
            InputKind::GamepadButton(gamepad_button) => gamepad_button.to_display_string(),
            InputKind::SingleAxis(single_axis) => single_axis.to_display_string(),
            InputKind::DualAxis(dual_axis) => dual_axis.to_display_string(),
            InputKind::PhysicalKey(key_code) => key_code.to_display_string(),
            // InputKind::KeyLocation(key_location) => key_location.to_display_string(),
            InputKind::Modifier(modifier) => modifier.to_display_string(),
            InputKind::Mouse(mouse) => mouse.to_display_string(),
            InputKind::MouseWheel(mouse_wheel_direction) => {
                mouse_wheel_direction.to_display_string()
            }
            InputKind::MouseMotion(mouse_motion) => mouse_motion.to_display_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for MouseMotionDirection {
    fn to_display_string(&self) -> String {
        match self {
            MouseMotionDirection::Up => "󰍽 ↑".to_string(),
            MouseMotionDirection::Down => "󰍽 ↓".to_string(),
            MouseMotionDirection::Left => "󰍽 ←".to_string(),
            MouseMotionDirection::Right => "󰍽 →".to_string(),
        }
    }
}

impl ToDisplayString for MouseWheelDirection {
    fn to_display_string(&self) -> String {
        match self {
            MouseWheelDirection::Up => "󰍽󰠳 ↑".to_string(), // Mouse Wheel Up
            MouseWheelDirection::Down => "󰍽󰠳 ↓".to_string(), // Mouse Wheel Down
            MouseWheelDirection::Left => "󰍽󰠳 ←".to_string(), // Mouse Wheel Left
            MouseWheelDirection::Right => "󰍽󰠳 →".to_string(), // Mouse Wheel Right
        }
    }
}

impl ToDisplayString for MouseButton {
    fn to_display_string(&self) -> String {
        match self {
            MouseButton::Left => "Left".to_string(),
            MouseButton::Right => "Right".to_string(),
            MouseButton::Middle => "Middle".to_string(),
            MouseButton::Other(x) => format!("Mouse {}", x).to_string(),
            _ => unreachable!(),
        }
    }
}

impl ToDisplayString for Modifier {
    fn to_display_string(&self) -> String {
        match self {
            Modifier::Alt => "Alt".to_string(),
            Modifier::Control => "Control".to_string(),
            Modifier::Shift => "Shift".to_string(),
            Modifier::Super => "Super".to_string(),
        }
    }
}

impl ToDisplayString for KeyCode {
    fn to_display_string(&self) -> String {
        match self {
            KeyCode::Digit0 => "0".to_string(),
            KeyCode::Digit1 => "1".to_string(),
            KeyCode::Digit2 => "2".to_string(),
            KeyCode::Digit3 => "3".to_string(),
            KeyCode::Digit4 => "4".to_string(),
            KeyCode::Digit5 => "5".to_string(),
            KeyCode::Digit6 => "6".to_string(),
            KeyCode::Digit7 => "7".to_string(),
            KeyCode::Digit8 => "8".to_string(),
            KeyCode::Digit9 => "9".to_string(),
            KeyCode::KeyA => "A".to_string(),
            KeyCode::KeyB => "B".to_string(),
            KeyCode::KeyC => "C".to_string(),
            KeyCode::KeyD => "D".to_string(),
            KeyCode::KeyE => "E".to_string(),
            KeyCode::KeyF => "F".to_string(),
            KeyCode::KeyG => "G".to_string(),
            KeyCode::KeyH => "H".to_string(),
            KeyCode::KeyI => "I".to_string(),
            KeyCode::KeyJ => "J".to_string(),
            KeyCode::KeyK => "K".to_string(),
            KeyCode::KeyL => "L".to_string(),
            KeyCode::KeyM => "M".to_string(),
            KeyCode::KeyN => "N".to_string(),
            KeyCode::KeyO => "O".to_string(),
            KeyCode::KeyP => "P".to_string(),
            KeyCode::KeyQ => "Q".to_string(),
            KeyCode::KeyR => "R".to_string(),
            KeyCode::KeyS => "S".to_string(),
            KeyCode::KeyT => "T".to_string(),
            KeyCode::KeyU => "U".to_string(),
            KeyCode::KeyV => "V".to_string(),
            KeyCode::KeyW => "W".to_string(),
            KeyCode::KeyX => "X".to_string(),
            KeyCode::KeyY => "Y".to_string(),
            KeyCode::KeyZ => "Z".to_string(),
            KeyCode::ArrowUp => "↑".to_string(),
            KeyCode::ArrowDown => "↓".to_string(),
            KeyCode::ArrowLeft => "←".to_string(),
            KeyCode::ArrowRight => "→".to_string(),
            KeyCode::Tab => "".to_string(),              // Tab  
            KeyCode::Enter => "󰌑".to_string(),            // Enter 󰌑
            KeyCode::Space => "󱁐".to_string(),            // Space 󱁐
            KeyCode::ShiftLeft => "󰧇 Left".to_string(),   // Shift Left
            KeyCode::ShiftRight => "󰧇 Right".to_string(), // Shift Right
            _ => format!("{:?}", self).to_title_case(),
        }
    }
}

impl ToDisplayString for DualAxis {
    fn to_display_string(&self) -> String {
        match (self.x.axis_type, self.y.axis_type) {
            (
                AxisType::Gamepad(GamepadAxisType::LeftStickX),
                AxisType::Gamepad(GamepadAxisType::LeftStickY),
            ) => "L3 󰆾".to_string(), // Left Stick Ⓛ
            (
                AxisType::Gamepad(GamepadAxisType::LeftStickY),
                AxisType::Gamepad(GamepadAxisType::LeftStickX),
            ) => "L3 󰆾".to_string(), // Left Stick Ⓛ
            (
                AxisType::Gamepad(GamepadAxisType::RightStickX),
                AxisType::Gamepad(GamepadAxisType::RightStickY),
            ) => "R3 󰆾".to_string(), // Right Stick Ⓡ
            (
                AxisType::Gamepad(GamepadAxisType::RightStickY),
                AxisType::Gamepad(GamepadAxisType::RightStickX),
            ) => "R3 󰆾".to_string(), // Right Stick Ⓡ
            (AxisType::MouseMotion(_), AxisType::MouseMotion(_)) => {
                "󰍽 󰆾".to_string() //  Mouse Motion
            }
            (AxisType::MouseWheel(_), AxisType::MouseWheel(_)) => "󰍽󰠳".to_string(), // Mouse Wheel
            _ => "Not yet implemented".to_string(),
        }
    }
}

impl ToDisplayString for GamepadButtonType {
    fn to_display_string(&self) -> String {
        match self {
            GamepadButtonType::South => "󰸴".to_string(), // Cross/A
            GamepadButtonType::East => "󰸷".to_string(),  // Circle/B
            GamepadButtonType::North => "󰸸".to_string(), // Triangle/Y
            GamepadButtonType::West => "󰸵".to_string(),  // Square/X
            GamepadButtonType::C => "C".to_string(),
            GamepadButtonType::Z => "Z".to_string(),
            GamepadButtonType::LeftTrigger => "L1".to_string(), // Left bumper
            GamepadButtonType::RightTrigger => "R1".to_string(), // Right bumper
            GamepadButtonType::LeftTrigger2 => "L2".to_string(), // Left Trigger
            GamepadButtonType::RightTrigger2 => "R2".to_string(), // Right Trigger
            GamepadButtonType::Select => "Select".to_string(),
            GamepadButtonType::Start => "Start".to_string(),
            GamepadButtonType::Mode => "Mode".to_string(),
            GamepadButtonType::LeftThumb => "L3 ↓".to_string(), // Left Stick Press Down Ⓛ
            GamepadButtonType::RightThumb => "R3 ↓".to_string(), // Right Stick Press Down Ⓡ
            GamepadButtonType::DPadUp => "󰹁".to_string(),       // DPad Up
            GamepadButtonType::DPadDown => "󰸽".to_string(),     // DPad Down
            GamepadButtonType::DPadLeft => "󰸾".to_string(),     // DPad Left
            GamepadButtonType::DPadRight => "󰹀".to_string(),    // DPad Right
            GamepadButtonType::Other(x) => format!("Gamepad {}", x).to_string(),
            // _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for SingleAxis {
    fn to_display_string(&self) -> String {
        match self.axis_type {
            AxisType::Gamepad(gamepad_axis) => gamepad_axis.to_display_string(),
            AxisType::MouseWheel(mouse_wheel_direction) => {
                mouse_wheel_direction.to_display_string()
            }
            AxisType::MouseMotion(mouse_motion) => mouse_motion.to_display_string(),
        }
    }
}

impl ToDisplayString for GamepadAxisType {
    fn to_display_string(&self) -> String {
        match self {
            GamepadAxisType::LeftStickX => "L3 󰹳".to_string(), // Left Stick Axis X Ⓛ
            GamepadAxisType::LeftStickY => "L3 󰹹".to_string(), // Left Stick Axis Y Ⓛ
            GamepadAxisType::LeftZ => "L3 ↓".to_string(),      // Left Stick Axis Z (Press down) Ⓛ
            GamepadAxisType::RightStickX => "R3 󰹳".to_string(), // Right Stick Axis X Ⓡ
            GamepadAxisType::RightStickY => "R3 󰹹".to_string(), // Right Stick Axis Y Ⓡ
            GamepadAxisType::RightZ => "R3 ↓".to_string(),     // Right Stick Axis Z (Press down) Ⓡ
            GamepadAxisType::Other(x) => format!("Gamepad {}", x).to_string(),
            // _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for MouseWheelAxisType {
    fn to_display_string(&self) -> String {
        match self {
            MouseWheelAxisType::X => "󰍽󰠳 󰹳".to_string(), // Mouse Wheel Axis X (Horizontal)
            MouseWheelAxisType::Y => "󰍽󰠳 󰹹".to_string(), // Mouse Wheel Axis Y (Vertical)
        }
    }
}

impl ToDisplayString for MouseMotionAxisType {
    fn to_display_string(&self) -> String {
        match self {
            MouseMotionAxisType::X => "󰍽 󰹳".to_string(), // Mouse Wheel Axis X (Horizontal)
            MouseMotionAxisType::Y => "󰍽 󰹹".to_string(), // Mouse Wheel Axis Y (Vertical)
        }
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
fn configure_visuals_system(mut contexts: EguiContexts, windows: Query<&Window>) {
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

/// **Bevy** `Resource` to store the currently changing binding
/// If this is not `default`, then all input will be captured and the binding will be updated
#[derive(Debug, Default, Resource)]
pub struct ChangingBinding {
    pub action: InputAction,
    pub binding: usize,
}

impl ChangingBinding {
    pub fn is_changing(&self) -> bool {
        !matches!(self.action, InputAction::Undefined)
    }
}

/// `Update` **Bevy** system to render the `egui` UI
/// Uses the `UiState` to understand which panels are open and should be rendered
fn ui_binding_panel(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    ui_state: ResMut<UiState>,
    mut query_camera_action: Query<&mut InputMap<CameraAction>>,
    mut query_general_action: Query<&mut InputMap<GeneralAction>>,
    mut query_moveable_object_action: Query<&mut InputMap<MoveableObjectAction>>,
    mut query_ui_action: Query<&mut InputMap<UiAction>>,
    catppuccin: Res<CatppuccinTheme>,
    mut currently_changing: ResMut<ChangingBinding>,
    mut keyboard_events: EventReader<KeyboardInput>,
    mut gamepad_button_events: EventReader<GamepadButtonInput>,
) {
    let ctx = contexts.ctx_mut();

    // info!("Currently changing: {:?}", currently_changing);
    // let grid_row_color = |r: usize, style: &mut egui::style::Style| {
    //     if r == 0 {
    //         style.visuals.widgets.noninteractive.bg_fill = Some(Color32::from_catppuccin_colour_ref(
    //             &catppuccin.flavour.lavender(),
    //         ));
    //     }
    // };

    let grid_row_color = catppuccin.flavour.mantle();
    // let grid_title_color = catppuccin.flavour.lavender();
    // let mut grid_title_colors = [
    //     catppuccin.flavour.green(),
    //     catppuccin.flavour.blue(),
    //     catppuccin.flavour.mauve(),
    //     catppuccin.flavour.maroon(),
    // ]
    // .into_iter()
    // .cycle();
    // title colours
    let grid_title_colors = [
        catppuccin.flavour.green(),
        catppuccin.flavour.blue(),
        catppuccin.flavour.mauve(),
        catppuccin.flavour.maroon(),
        catppuccin.flavour.lavender(),
    ];

    let mut gtc_iter = grid_title_colors.iter().cycle();

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
                                                        // button_response.highlight();
                                                        *currently_changing = ChangingBinding {
                                                            action: InputAction::MoveableObject(
                                                                *inner_action.0,
                                                            ),
                                                            binding: i,
                                                        };
                                                    }
                                                });
                                            });

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
                                                        // button_response.highlight();
                                                        *currently_changing = ChangingBinding {
                                                            action: InputAction::General(
                                                                *inner_action.0,
                                                            ),
                                                            binding: i,
                                                        };
                                                    }
                                                });
                                            });

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
                                                        // button_response.highlight();
                                                        *currently_changing = ChangingBinding {
                                                            action: InputAction::Camera(
                                                                *inner_action.0,
                                                            ),
                                                            binding: i,
                                                        };
                                                    }
                                                });
                                            });

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
                                                        // button_response.highlight();
                                                        *currently_changing = ChangingBinding {
                                                            action: InputAction::Ui(
                                                                *inner_action.0,
                                                            ),
                                                            binding: i,
                                                        };
                                                    }
                                                });
                                            });

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

    // check for any input at all (keyboard, mouse, gamepad, etc.)
    // if there is, then rebind the map
    for event in keyboard_events.read() {
        let key_code = event.key_code;
        // consume the event

        // If the escape key is pressed, then don't change the binding
        if matches!(key_code, KeyCode::Escape) {
            *currently_changing = ChangingBinding::default();
        }

        // If the currently changing binding is not default, then change the binding
        match currently_changing.action {
            InputAction::Camera(action) => {
                let mut map = query_camera_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(action, UserInput::Single(InputKind::PhysicalKey(key_code)));
            }
            InputAction::General(action) => {
                let mut map = query_general_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(action, UserInput::Single(InputKind::PhysicalKey(key_code)));
            }
            InputAction::MoveableObject(action) => {
                let mut map = query_moveable_object_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(action, UserInput::Single(InputKind::PhysicalKey(key_code)));
            }
            InputAction::Ui(action) => {
                let mut map = query_ui_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(action, UserInput::Single(InputKind::PhysicalKey(key_code)));
            }
            _ => { /* do nothing */ }
        }
        *currently_changing = ChangingBinding::default();
    }

    for event in gamepad_button_events.read() {
        let button = event.button;

        match currently_changing.action {
            InputAction::Camera(action) => {
                let mut map = query_camera_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(
                    action,
                    UserInput::Single(InputKind::GamepadButton(button.button_type)),
                );
            }
            InputAction::General(action) => {
                let mut map = query_general_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(
                    action,
                    UserInput::Single(InputKind::GamepadButton(button.button_type)),
                );
            }
            InputAction::MoveableObject(action) => {
                let mut map = query_moveable_object_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(
                    action,
                    UserInput::Single(InputKind::GamepadButton(button.button_type)),
                );
            }
            InputAction::Ui(action) => {
                let mut map = query_ui_action.single_mut();
                map.remove_at(&action, currently_changing.binding);
                map.insert(
                    action,
                    UserInput::Single(InputKind::GamepadButton(button.button_type)),
                );
            }
            _ => { /* do nothing */ }
        }

        *currently_changing = ChangingBinding::default();
    }

    occupied_screen_space.left = left_panel
        .map(|ref inner| inner.response.rect.width())
        .unwrap_or(0.0);

    // occupied_screen_space.left = if left_panel.is_some() {
    //     left_panel.unwrap().response.rect.width()
    // } else {
    //     0.0
    // };
    // occupied_screen_space.left = if left_panel.is_some() {
    //     left_panel.unwrap().response.rect.width()
    // } else {
    //     0.0
    // };
}
