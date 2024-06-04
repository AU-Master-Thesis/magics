//! Visualize interrobot factors
//! environment.
use bevy::prelude::*;
use gbp_config::Config;

use crate::{factorgraph::prelude::FactorGraph, planner::robot::RadioAntenna};

pub struct InterRobotFactorVisualizerPlugin;

impl Plugin for InterRobotFactorVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, visualize_interrobot_factors.run_if(enabled));
    }
}

/// **Bevy** run condition for drawing obstacle factors
fn enabled(config: Res<Config>) -> bool {
    // config.visualisation.draw.interrobot_factors
    (config.visualisation.draw.interrobot_factors_safety_distance
        || config.visualisation.draw.interrobot_factors)
        && config.gbp.factors_enabled.interrobot
}

fn visualize_interrobot_factors(
    mut gizmos: Gizmos,
    q: Query<(&FactorGraph, &RadioAntenna)>,
    config: Res<Config>,
) {
    let red = colorgrad::Color::from_linear_rgba(1.0, 0.0, 0.0, 200.0);
    let yellow = colorgrad::Color::from_linear_rgba(1.0, 1.0, 0.0, 200.0);
    let gradient = colorgrad::CustomGradient::new()
        .colors(&[red, yellow])
        .domain(&[0.0, 1.0])
        .mode(colorgrad::BlendMode::Hsv)
        .build()
        .unwrap();

    // let height = 0.5f32;
    let height = -config.visualisation.height.objects;

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

            let dist = estimated_position.distance(external_position);

            if config.visualisation.draw.interrobot_factors {
                if dist < safety_dist as f32 {
                    let offset = 0.15; // 0.3 / 2.0;
                    let start = estimated_position + dir * offset;
                    let end = external_position + dir * offset;
                    let ratio = dist / safety_dist as f32;
                    let color = gradient.at(ratio as f64);
                    let color = Color::rgb(color.r as f32, color.g as f32, color.b as f32);
                    gizmos.line(start.extend(height).xzy(), end.extend(height).xzy(), color);
                }
            }

            if config.visualisation.draw.interrobot_factors_safety_distance {
                let color = if antenna.active {
                    Color::ORANGE
                } else {
                    Color::GRAY
                };
                gizmos
                    .circle(
                        estimated_position.extend(height).xzy(),
                        Direction3d::Y,
                        safety_dist as f32,
                        color,
                    )
                    .segments(18);
            }
        }
    }
}
