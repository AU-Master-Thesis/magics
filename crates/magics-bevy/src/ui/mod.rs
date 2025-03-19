//! UI components for the Magics planner
//!
//! This module provides various UI components including control panels,
//! data visualization, settings panels, and metrics displays.

use bevy::prelude::*;

pub mod controls;
pub mod data;
pub mod metrics;
pub mod settings;
pub mod state;

pub use state::{UiScaleType, UiState};

/// Trait for converting action types to display strings for UI
pub trait ToUiString {
    /// Convert action to a display string
    fn to_display_string(&self) -> String;
}

/// Resource to block input actions when UI needs to capture input
#[derive(Debug, Resource, Default)]
pub struct ActionBlock {
    blocked: bool,
}

impl ActionBlock {
    /// Check if actions are blocked
    pub fn is_blocked(&self) -> bool {
        self.blocked
    }

    /// Set block state
    pub fn set_blocked(&mut self, blocked: bool) {
        self.blocked = blocked;
    }

    /// Block actions
    pub fn block(&mut self) {
        self.blocked = true;
    }

    /// Unblock actions
    pub fn unblock(&mut self) {
        self.blocked = false;
    }
}

/// Main UI plugin for Magics Bevy
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ActionBlock>()
            .add_plugins((
                controls::ControlsPlugin,
                data::DataPlugin,
                metrics::MetricsPlugin,
                settings::SettingsPlugin,
            ))
            .add_systems(Update, (handle_ui_block,));
    }
}

/// System to handle action blocking based on UI focus
fn handle_ui_block(
    mut action_block: ResMut<ActionBlock>,
    mut ui_focus_events: EventReader<bevy_egui::EguiUserOutputEvent>,
) {
    let mut should_block = false;

    for event in ui_focus_events.read() {
        match event {
            bevy_egui::EguiUserOutputEvent::HoverUi => {
                should_block = true;
            }
            _ => {}
        }
    }

    action_block.set_blocked(should_block);
}
