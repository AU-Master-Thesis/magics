//! Theme support for the UI
//!
//! This module provides theming functionality for the application,
//! allowing for dark and light mode support.

use bevy::prelude::*;
use catppuccin::{Flavour, Palette};

/// Plugin for theme functionality
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ThemeSettings>()
            .add_systems(Startup, setup_theme)
            .add_systems(Update, handle_theme_change);
    }
}

/// Theme settings resource
#[derive(Resource)]
pub struct ThemeSettings {
    /// Whether dark mode is enabled
    pub dark_mode: bool,
    /// Current theme colors
    pub colors: ThemeColors,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        // Default to dark mode
        let colors = ThemeColors::dark();
        Self {
            dark_mode: true,
            colors,
        }
    }
}

/// Theme colors
#[derive(Clone)]
pub struct ThemeColors {
    /// Background color
    pub background: Color,
    /// Text color
    pub text: Color,
    /// Primary accent color
    pub primary: Color,
    /// Secondary accent color
    pub secondary: Color,
    /// Success color
    pub success: Color,
    /// Warning color
    pub warning: Color,
    /// Error color
    pub error: Color,
}

impl ThemeColors {
    /// Create a dark theme
    pub fn dark() -> Self {
        // Use catppuccin Mocha (dark) flavour
        let palette = Palette::mocha();
        Self {
            background: Color::hex(palette.base().hex).unwrap(),
            text: Color::hex(palette.text().hex).unwrap(),
            primary: Color::hex(palette.blue().hex).unwrap(),
            secondary: Color::hex(palette.lavender().hex).unwrap(),
            success: Color::hex(palette.green().hex).unwrap(),
            warning: Color::hex(palette.yellow().hex).unwrap(),
            error: Color::hex(palette.red().hex).unwrap(),
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        // Use catppuccin Latte (light) flavour
        let palette = Palette::latte();
        Self {
            background: Color::hex(palette.base().hex).unwrap(),
            text: Color::hex(palette.text().hex).unwrap(),
            primary: Color::hex(palette.blue().hex).unwrap(),
            secondary: Color::hex(palette.lavender().hex).unwrap(),
            success: Color::hex(palette.green().hex).unwrap(),
            warning: Color::hex(palette.yellow().hex).unwrap(),
            error: Color::hex(palette.red().hex).unwrap(),
        }
    }
}

/// Setup theme based on system preference
fn setup_theme(mut theme_settings: ResMut<ThemeSettings>) {
    // Get system theme preference if possible
    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(mode) = dark_light::detect() {
        theme_settings.dark_mode = mode == dark_light::Mode::Dark;
        theme_settings.colors = if theme_settings.dark_mode {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        };
    }
}

/// Handle theme changes
fn handle_theme_change(
    mut theme_settings: ResMut<ThemeSettings>,
    mut egui_settings: ResMut<bevy_egui::EguiSettings>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Toggle theme with Ctrl+T
    if keyboard_input.pressed(KeyCode::ControlLeft) && keyboard_input.just_pressed(KeyCode::KeyT) {
        theme_settings.dark_mode = !theme_settings.dark_mode;
        theme_settings.colors = if theme_settings.dark_mode {
            ThemeColors::dark()
        } else {
            ThemeColors::light()
        };

        // Update egui visuals
        if theme_settings.dark_mode {
            egui_settings.style.visuals = bevy_egui::egui::Visuals::dark();
        } else {
            egui_settings.style.visuals = bevy_egui::egui::Visuals::light();
        }
    }
}
