use bevy::prelude::*;

use crate::{
    config::Config,
    planner::RobotState,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

pub struct CommunicationRadiusVisualizerPlugin;

impl Plugin for CommunicationRadiusVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, communication_radius);
    }
}

fn communication_radius(
    mut gizmos: Gizmos,
    // query: Query<&Transform, (With<RobotState>, Changed<Transform>)>,
    query: Query<&Transform, With<RobotState>>,
    config: Res<Config>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    if !config.visualisation.draw.communication_radius {
        return;
    }

    let color = Color::from_catppuccin_colour(catppuccin_theme.sky());
    let segments = 24;
    let radius: f32 = config.robot.communication.radius.into();

    for transform in query.iter() {
        gizmos
            .circle(transform.translation, Direction3d::Y, radius, color)
            .segments(segments);
    }
}
