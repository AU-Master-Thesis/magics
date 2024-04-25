use bevy::input::{
    gamepad::{GamepadAxisType, GamepadButtonType},
    keyboard::KeyCode,
    mouse::MouseButton,
};
use heck::ToTitleCase;
use leafwing_input_manager::{
    axislike::{AxisType, DualAxis, MouseMotionAxisType, MouseWheelAxisType, SingleAxis, VirtualAxis, VirtualDPad},
    buttonlike::{MouseMotionDirection, MouseWheelDirection},
    user_input::{InputKind, Modifier, UserInput},
};

/// Convert to a `String` suitable for displaying
pub trait ToDisplayString {
    fn to_display_string(&self) -> String;
}

// impl <T: ToString> ToDisplayString for T {
//     fn to_display_string(&self) -> String {
//         self.to_string()
//     }
// }

impl ToDisplayString for UserInput {
    fn to_display_string(&self) -> String {
        match self {
            Self::Single(input) => input.to_display_string(),
            Self::VirtualDPad(virtual_dpad) => virtual_dpad.to_display_string(),
            Self::VirtualAxis(virtual_axis) => virtual_axis.to_display_string(),
            Self::Chord(chord) => chord
                .iter()
                .map(ToDisplayString::to_display_string)
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
            Self::GamepadButton(gamepad_button) => gamepad_button.to_display_string(),
            Self::SingleAxis(single_axis) => single_axis.to_display_string(),
            Self::DualAxis(dual_axis) => dual_axis.to_display_string(),
            Self::PhysicalKey(key_code) => key_code.to_display_string(),
            // Self::KeyLocation(key_location) => key_location.to_display_string(),
            Self::Modifier(modifier) => modifier.to_display_string(),
            Self::Mouse(mouse) => mouse.to_display_string(),
            Self::MouseWheel(mouse_wheel_direction) => mouse_wheel_direction.to_display_string(),
            Self::MouseMotion(mouse_motion) => mouse_motion.to_display_string(),
            _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for MouseMotionDirection {
    fn to_display_string(&self) -> String {
        match self {
            Self::Up => "󰍽 ↑".to_string(),
            Self::Down => "󰍽 ↓".to_string(),
            Self::Left => "󰍽 ←".to_string(),
            Self::Right => "󰍽 →".to_string(),
        }
    }
}

impl ToDisplayString for MouseWheelDirection {
    fn to_display_string(&self) -> String {
        match self {
            Self::Up => "󰍽󰠳 ↑".to_string(),    // Mouse Wheel Up
            Self::Down => "󰍽󰠳 ↓".to_string(),  // Mouse Wheel Down
            Self::Left => "󰍽󰠳 ←".to_string(),  // Mouse Wheel Left
            Self::Right => "󰍽󰠳 →".to_string(), // Mouse Wheel Right
        }
    }
}

impl ToDisplayString for MouseButton {
    fn to_display_string(&self) -> String {
        match self {
            Self::Left => "LMB".to_string(),
            Self::Right => "RMB".to_string(),
            Self::Middle => "MMB".to_string(),
            Self::Other(x) => format!("Mouse {x}"),
            // _ => unreachable!(),
            _ => " ".to_string(),
        }
    }
}

impl ToDisplayString for Modifier {
    fn to_display_string(&self) -> String {
        match self {
            Self::Alt => "Alt".to_string(),
            Self::Control => "Ctrl".to_string(),
            Self::Shift => "Shift".to_string(),
            Self::Super => "Super".to_string(),
        }
    }
}

impl ToDisplayString for KeyCode {
    fn to_display_string(&self) -> String {
        match self {
            Self::Digit0 => "0".to_string(),
            Self::Digit1 => "1".to_string(),
            Self::Digit2 => "2".to_string(),
            Self::Digit3 => "3".to_string(),
            Self::Digit4 => "4".to_string(),
            Self::Digit5 => "5".to_string(),
            Self::Digit6 => "6".to_string(),
            Self::Digit7 => "7".to_string(),
            Self::Digit8 => "8".to_string(),
            Self::Digit9 => "9".to_string(),
            Self::KeyA => "A".to_string(),
            Self::KeyB => "B".to_string(),
            Self::KeyC => "C".to_string(),
            Self::KeyD => "D".to_string(),
            Self::KeyE => "E".to_string(),
            Self::KeyF => "F".to_string(),
            Self::KeyG => "G".to_string(),
            Self::KeyH => "H".to_string(),
            Self::KeyI => "I".to_string(),
            Self::KeyJ => "J".to_string(),
            Self::KeyK => "K".to_string(),
            Self::KeyL => "L".to_string(),
            Self::KeyM => "M".to_string(),
            Self::KeyN => "N".to_string(),
            Self::KeyO => "O".to_string(),
            Self::KeyP => "P".to_string(),
            Self::KeyQ => "Q".to_string(),
            Self::KeyR => "R".to_string(),
            Self::KeyS => "S".to_string(),
            Self::KeyT => "T".to_string(),
            Self::KeyU => "U".to_string(),
            Self::KeyV => "V".to_string(),
            Self::KeyW => "W".to_string(),
            Self::KeyX => "X".to_string(),
            Self::KeyY => "Y".to_string(),
            Self::KeyZ => "Z".to_string(),
            Self::ArrowUp => "↑".to_string(),
            Self::ArrowDown => "↓".to_string(),
            Self::ArrowLeft => "←".to_string(),
            Self::ArrowRight => "→".to_string(),
            Self::Tab => "".to_string(),              // Tab  
            Self::Enter => "󰌑".to_string(),            // Enter 󰌑
            Self::Space => "󱁐".to_string(),            // Space 󱁐
            Self::ShiftLeft => "󰧇 Left".to_string(),   // Shift Left
            Self::ShiftRight => "󰧇 Right".to_string(), // Shift Right
            _ => format!("{:?}", self).to_title_case(),
        }
    }
}

impl ToDisplayString for DualAxis {
    fn to_display_string(&self) -> String {
        match (self.x.axis_type, self.y.axis_type) {
            (
                AxisType::Gamepad(GamepadAxisType::LeftStickX | GamepadAxisType::LeftStickY),
                AxisType::Gamepad(GamepadAxisType::LeftStickY | GamepadAxisType::LeftStickX),
            ) => "L3 󰆾".to_string(), // Left Stick Ⓛ
            (
                AxisType::Gamepad(GamepadAxisType::RightStickX | GamepadAxisType::RightStickY),
                AxisType::Gamepad(GamepadAxisType::RightStickY | GamepadAxisType::RightStickX),
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
            Self::South => "󰸴".to_string(), // Cross/A
            Self::East => "󰸷".to_string(),  // Circle/B
            Self::North => "󰸸".to_string(), // Triangle/Y
            Self::West => "󰸵".to_string(),  // Square/X
            Self::C => "C".to_string(),
            Self::Z => "Z".to_string(),
            Self::LeftTrigger => "L1".to_string(),   // Left bumper
            Self::RightTrigger => "R1".to_string(),  // Right bumper
            Self::LeftTrigger2 => "L2".to_string(),  // Left Trigger
            Self::RightTrigger2 => "R2".to_string(), // Right Trigger
            Self::Select => "Select".to_string(),
            Self::Start => "Start".to_string(),
            Self::Mode => "Mode".to_string(),
            Self::LeftThumb => "L3 ↓".to_string(),  // Left Stick Press Down Ⓛ
            Self::RightThumb => "R3 ↓".to_string(), // Right Stick Press Down Ⓡ
            Self::DPadUp => "󰹁".to_string(),        // DPad Up
            Self::DPadDown => "󰸽".to_string(),      // DPad Down
            Self::DPadLeft => "󰸾".to_string(),      // DPad Left
            Self::DPadRight => "󰹀".to_string(),     // DPad Right
            Self::Other(x) => format!("Gamepad {x}"),
            // _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for SingleAxis {
    fn to_display_string(&self) -> String {
        match self.axis_type {
            AxisType::Gamepad(gamepad_axis) => gamepad_axis.to_display_string(),
            AxisType::MouseWheel(mouse_wheel_direction) => mouse_wheel_direction.to_display_string(),
            AxisType::MouseMotion(mouse_motion) => mouse_motion.to_display_string(),
        }
    }
}

impl ToDisplayString for GamepadAxisType {
    fn to_display_string(&self) -> String {
        match self {
            Self::LeftStickX => "L3 󰹳".to_string(),  // Left Stick Axis X Ⓛ
            Self::LeftStickY => "L3 󰹹".to_string(),  // Left Stick Axis Y Ⓛ
            Self::LeftZ => "L3 ↓".to_string(),       // Left Stick Axis Z (Press down) Ⓛ
            Self::RightStickX => "R3 󰹳".to_string(), // Right Stick Axis X Ⓡ
            Self::RightStickY => "R3 󰹹".to_string(), // Right Stick Axis Y Ⓡ
            Self::RightZ => "R3 ↓".to_string(),      // Right Stick Axis Z (Press down) Ⓡ
            Self::Other(x) => format!("Gamepad {x}"),
            // _ => "Unknown".to_string(),
        }
    }
}

impl ToDisplayString for MouseWheelAxisType {
    fn to_display_string(&self) -> String {
        match self {
            Self::X => "󰍽󰠳 󰹳".to_string(), // Mouse Wheel Axis X (Horizontal)
            Self::Y => "󰍽󰠳 󰹹".to_string(), // Mouse Wheel Axis Y (Vertical)
        }
    }
}

impl ToDisplayString for MouseMotionAxisType {
    fn to_display_string(&self) -> String {
        match self {
            Self::X => "󰍽 󰹳".to_string(), // Mouse Wheel Axis X (Horizontal)
            Self::Y => "󰍽 󰹹".to_string(), // Mouse Wheel Axis Y (Vertical)
        }
    }
}
