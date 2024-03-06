use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
    mouse::MouseButton,
};
use heck::ToTitleCase;
use leafwing_input_manager::{
    axislike::{
        AxisType, DualAxis, MouseMotionAxisType, MouseWheelAxisType, SingleAxis, VirtualAxis,
        VirtualDPad,
    },
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
    user_input::{InputKind, Modifier, UserInput},
};

/// Convert to a `String` suitable for displaying
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
