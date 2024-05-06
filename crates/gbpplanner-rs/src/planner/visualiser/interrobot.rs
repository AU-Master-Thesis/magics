//! Visualize interrobot factors
//! environment.
use bevy::prelude::*;

use crate::{config::Config, factorgraph::prelude::FactorGraph, planner::robot::RadioAntenna};

pub struct InterRobotFactorVisualizerPlugin;

impl Plugin for InterRobotFactorVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            visualize_interrobot_factors.run_if(enabled),
            // visualize_obstacle_factors.run_if(on_timer(std::time::Duration::from_millis(500))),
        );
    }
}

/// **Bevy** run condition for drawing obstacle factors
fn enabled(config: Res<Config>) -> bool {
    // config.visualisation.draw.interrobot_factors
    config.visualisation.draw.interrobot_factors_safety_distance
        || config.visualisation.draw.interrobot_factors
}

fn visualize_interrobot_factors(
    mut gizmos: Gizmos,
    q: Query<(&FactorGraph, &RadioAntenna)>,
    config: Res<Config>,
) {
    for (factorgraph, antenna) in &q {
        for (variable, interrobot) in factorgraph.variable_and_inter_robot_factors() {
            let estimated_position = variable.estimated_position_vec2();
            // get the estimated position of the variable that the interrobot factor is
            // connected to in the external factor graph
            let Ok((external_factorgraph, _)) = q.get(interrobot.external_variable.factorgraph_id)
            else {
                continue;
            };

            let external_variable = external_factorgraph
                .get_variable(interrobot.external_variable.variable_index)
                .expect("external variable exists");

            let external_position = external_variable.estimated_position_vec2();
            let dir = (external_position - estimated_position).normalize();
            // rotate 90deg clockwise
            let dir = Vec2::new(dir.y, -dir.x);
            // let distance_sq = estimated_position.distance_squared(external_position);
            let safety_dist = interrobot.safety_distance();

            let (line_color, circle_color) = if antenna.active {
                let dist = estimated_position.distance(external_position);
                if dist < safety_dist as f32 {
                    let ratio = dist / safety_dist as f32;
                    let g = 1.0 * ratio;
                    let r = 1.0 - g;
                    (Color::rgb(r, g, 0.0), Color::ORANGE)
                    // Color::RED
                } else {
                    (Color::GREEN, Color::ORANGE)
                }
            } else {
                // greyed out to indicate that the connection is not active/used
                (Color::GRAY, Color::GRAY)
            };

            let height = 0.5f32;

            if config.visualisation.draw.interrobot_factors {
                let offset = 0.15; // 0.3 / 2.0;
                let start = estimated_position + dir * offset;
                let end = external_position + dir * offset;
                gizmos.line(
                    start.extend(height).xzy(),
                    end.extend(height).xzy(),
                    line_color,
                );
            }

            if config.visualisation.draw.interrobot_factors_safety_distance {
                gizmos
                    .circle(
                        estimated_position.extend(height).xzy(),
                        Direction3d::Y,
                        safety_dist as f32,
                        circle_color,
                    )
                    .segments(18);
            }
        }
    }
}
