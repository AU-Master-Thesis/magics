use bevy::prelude::*;
use gbp_config::Config;

use crate::{
    planner::{robot::RadioAntenna, RobotState},
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
    query: Query<(&RadioAntenna, &Transform)>,
    config: Res<Config>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    let active_comms_color = Color::from_catppuccin_colour(catppuccin_theme.sky());
    let segments = 24;
    // let radius: f32 = config.robot.communication.radius.into();

    for (antenna, transform) in query.iter() {
        gizmos
            .circle(
                transform.translation,
                Direction3d::Y,
                antenna.radius,
                if antenna.active {
                    active_comms_color
                } else {
                    // Color::RED
                    Color::ORANGE_RED
                },
            )
            .segments(segments);
    }
}
