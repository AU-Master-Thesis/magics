#![deny(missing_docs)]
use bevy::prelude::*;
use gbp_config::DrawSetting;

use crate::{boolean_bevy_resource, input::DrawSettingsEvent, planner::RobotConnections};

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
    mut query: Query<&mut Visibility, With<RobotConnections>>,
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, DrawSetting::Robots) {
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
