//! Visualize how obstacle factors measure distance to a solid object in the
//! environment.

use bevy::prelude::*;

use crate::{config::Config, factorgraph::prelude::FactorGraph};

#[derive(Default)]
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

// #[derive(Clone, Copy, Event, PartialEq, Eq)]
// pub enum ChangeBooleanSetting<T> {
//     On,
//     Off,
//     Toggle,
// }

// // fn system(ev: EventReader<ChangeBooleanSetting<Visual<ObstacleFactor>>>)
// {} fn system(ev: EventReader<events::ChangeBooleanSetting>) {}

// pub struct Visual<T>;
// pub struct ObstacleFactor;

// pub mod events {
//     use bevy::prelude::*;

//     pub type ChangeBooleanSetting =
//         super::ChangeBooleanSetting<super::Visual<super::ObstacleFactor>>;

//     fn handle_event() {}
// }

mod resources {
    use bevy::prelude::*;

    #[derive(Resource)]
    struct Settings {
        enabled: bool,
    }

    impl FromWorld for Settings {
        fn from_world(world: &mut World) -> Self {
            if let Some(config) = world.get_resource::<crate::config::Config>() {
                Self {
                    enabled: config.visualisation.draw.obstacle_factors,
                }
            } else {
                Self { enabled: false }
            }
        }
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
fn visualize_obstacle_factors(
    mut gizmos: Gizmos,
    factorgraphs: Query<&FactorGraph>,
    config: Res<Config>,
) {
    let height = -config.visualisation.height.objects;

    for factorgraph in &factorgraphs {
        for (variable, obstacle_factor) in factorgraph.variable_and_their_obstacle_factors() {
            // let estimated_position = variable.estimated_position_vec2();
            let last_measurement = obstacle_factor.last_measurement();

            // let Some(last_measurement) = obstacle_factor.last_measurement() else {
            //     continue;
            // };

            // let scale = 1e2;
            let v: f32 = (last_measurement.value as f32 * 1e2).clamp(0.0, 1.0);

            let red = 1.0 * v;
            let green = 1.0 - red;
            let color = Color::rgb(red, green, 0.0);

            // let height = 0.5f32;
            // let scale: f32 = 1.1;
            // [x, y]
            // [x, y, 0]
            // [x, 0, y]
            // let start = estimated_position.extend(height).xzy();
            // let end = scale * last_measurement.pos.extend(height).xzy();
            // gizmos.line(start, end, color);
            gizmos.circle(
                last_measurement.pos.extend(height).xzy(),
                Direction3d::Y,
                // Vec3::NEG_Z.di,
                0.5,
                color,
            );
        }
    }
}

/// **Bevy** run condition for drawing obstacle factors
#[inline]
fn enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.obstacle_factors
}
