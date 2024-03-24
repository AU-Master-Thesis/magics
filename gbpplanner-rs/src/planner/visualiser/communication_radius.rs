use bevy::prelude::*;

use crate::{
    config::Config,
    planner::RobotState,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

pub struct CommunicationRadiusVisualizerPlugin;

impl Plugin for CommunicationRadiusVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_communication_radius.run_if(draw_communication_radius_enabled),
        );
    }
}

fn draw_communication_radius_enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.communication_radius
}

fn draw_communication_radius(
    mut gizmos: Gizmos,
    // query: Query<&Transform, (With<RobotState>, Changed<Transform>)>,
    query: Query<(&RobotState, &Transform)>,
    config: Res<Config>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    let active_comms_color = Color::from_catppuccin_colour(catppuccin_theme.sky());
    let segments = 24;
    let radius: f32 = config.robot.communication.radius.into();

    for (robot_state, transform) in query.iter() {
        gizmos
            .circle(
                transform.translation,
                Direction3d::Y,
                radius,
                if robot_state.interrobot_comms_active {
                    active_comms_color
                } else {
                    // Color::RED
                    Color::ORANGE_RED
                },
            )
            .segments(segments);
    }
}
