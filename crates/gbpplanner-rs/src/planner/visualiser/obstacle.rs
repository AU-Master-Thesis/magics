//! Visualize how obstacle factors measure distance to a solid object in the
//! environment.

use bevy::prelude::*;

use crate::{config::Config, factorgraph::prelude::FactorGraph};

pub struct ObstacleFactorVisualizerPlugin;

impl Plugin for ObstacleFactorVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            visualize_obstacle_factors.run_if(enabled),
            // visualize_obstacle_factors.run_if(on_timer(std::time::Duration::from_millis(500))),
        );
    }
}

/// Draw a line between a variables estimated position and the sample point of
/// its connected obstacle factor. It uses the Gizmos API to draw a line between
/// the two points The color of the line is based on the measured value of the
/// obstacle factor. 1.0 -> rgb(255, 0, 0)
/// 0.0 -> rgb(0, 255, 0)
/// 0.5 -> rgb(128, 128, 0)
/// r = 255 * (1 - value)
/// g = 255 * value
/// b = 0
fn visualize_obstacle_factors(mut gizmos: Gizmos, factorgraphs: Query<&FactorGraph>) {
    // for factorgraph in &factorgraphs {
    //     factorgraph
    //         .variable_and_their_obstacle_factors()
    //         .for_each(|(variable, obstacle_factor)| {
    //             let estimated_position = variable.estimated_position_vec2();
    //             let last_measurement = obstacle_factor.last_measurement();
    //             let mut v: f32 = last_measurement.value as f32 * 1e2;
    //             v = v.max(0.0).min(1.0); // clamp to [0, 1]
    //
    //             let r = 1.0 * v;
    //             let g = 1.0 - r;
    //             let color = Color::rgb(r, g, 0.0);
    //
    //             let height = 0.5f32;
    //             let scale: f32 = 1.1;
    //             // [x, y]
    //             // [x, y, 0]
    //             // [x, 0, y]
    //             let start = estimated_position.extend(height).xzy();
    //             let end = scale * last_measurement.pos.extend(height).xzy();
    //             gizmos.line(start, end, color)
    //         })
    // }
}

/// Used to check if the communication graph should be drawn
#[inline]
fn enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.obstacle_factors
}
