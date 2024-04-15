#![deny(missing_docs)]
use bevy::prelude::*;

use crate::{boolean_bevy_resource, planner::RobotState};

pub struct RobotVisualiserPlugin;

impl Plugin for RobotVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RobotVisualiserEnabled>()
            .add_systems(Update, toggle_visibility_of_robot_meshes);
    }
}

boolean_bevy_resource!(RobotVisualiserEnabled, default = true);

fn toggle_visibility_of_robot_meshes(
    mut enabled: ResMut<RobotVisualiserEnabled>,
    mut query: Query<&mut Visibility, With<RobotState>>,
    mut draw_setting_event: EventReader<crate::input::DrawSettingsEvent>,
) {
    for event in draw_setting_event.read() {
        if matches!(event.setting, crate::config::DrawSetting::Robots) {
            let new_visibility_state = if event.draw {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            enabled.set(event.draw);

            for mut visibility in &mut query {
                *visibility = new_visibility_state;
            }
        }
    }
}
