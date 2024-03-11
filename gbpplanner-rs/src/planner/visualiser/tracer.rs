use std::collections::HashMap;

use bevy::{prelude::*, render::view::visibility};

use crate::{
    config::Config,
    planner::{RobotId, RobotState},
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

pub struct TracerVisualiserPlugin;

impl Plugin for TracerVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Traces>()
            .add_systems(Update, (track_robots, draw_traces));
    }
}

/// **Bevy** [`Resource`] to store all robot traces
#[derive(Default, Resource)]
pub struct Traces(pub HashMap<RobotId, Vec<Vec3>>);

/// **Bevy** [`Update`] system
/// To update the [`Traces`] resource
fn track_robots(
    mut traces: ResMut<Traces>,
    query: Query<(RobotId, &Transform), (With<RobotState>, Changed<Transform>)>,
) {
    for (robot_id, transform) in query.iter() {
        traces
            .0
            .entry(robot_id)
            .or_insert_with(Vec::new)
            .push(transform.translation);
    }
}

/// **Bevy** [`Update`] system
/// To draw the robot traces; using the [`Traces`] resource
fn draw_traces(
    mut gizmos: Gizmos,
    traces: Res<Traces>,
    // mut commands: Commands,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    catppuccin_theme: Res<CatppuccinTheme>,
    config: Res<Config>,
) {
    if !config.visualisation.draw.paths {
        return;
    }

    for (robot_id, trace) in traces.0.iter() {
        let colours = catppuccin_theme.colours().into_iter().collect::<Vec<_>>();
        let index = robot_id.index() as usize % 14;
        let color = colours[index];

        gizmos.primitive_3d(
            Polyline3d::<100>::new(trace.clone()),
            Vec3::ZERO,
            Quat::IDENTITY,
            Color::from_catppuccin_colour(color),
            // color,
        );
    }
}
