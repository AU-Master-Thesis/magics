use bevy::prelude::*;

/// Types of UI scaling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScaleType {
    /// No scaling
    None,
    /// Custom scaling factor
    Custom,
    /// Scale based on window size
    Window,
}

impl Default for UiScaleType {
    fn default() -> Self {
        Self::Window
    }
}

/// Resource for tracking UI panel visibility states
#[derive(Resource, Debug, Clone)]
pub struct UiState {
    /// Whether the left panel is visible
    pub left_panel_visible: bool,
    /// Whether the right panel is visible
    pub right_panel_visible: bool,
    /// Whether the top panel is visible
    pub top_panel_visible: bool,
    /// Whether the bottom panel is visible
    pub bottom_panel_visible: bool,
    /// Whether the metrics window is visible
    pub metrics_window_visible: bool,
    /// The current scale type for UI
    pub scale_type: UiScaleType,
    /// The custom scale factor when using UiScaleType::Custom
    pub scale_factor: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            left_panel_visible: true,
            right_panel_visible: true,
            top_panel_visible: true,
            bottom_panel_visible: true,
            metrics_window_visible: true,
            scale_type: UiScaleType::default(),
            scale_factor: 1.0,
        }
    }
}

impl UiState {
    /// Get the current scale factor based on scale type
    pub fn get_scale_factor(&self, window_size: Vec2) -> f32 {
        match self.scale_type {
            UiScaleType::None => 1.0,
            UiScaleType::Custom => self.scale_factor,
            UiScaleType::Window => {
                // Scale based on window dimensions
                let window_scale = window_size.x / 1920.0;
                (window_scale * 100.0).round() / 100.0
            }
        }
    }
}
