#![warn(missing_docs)]
use bevy::prelude::*;

// use gbp_linalg::pretty_print_matrix;
use super::{super::FactorGraph, RobotTracker, Z_FIGHTING_OFFSET};
use crate::{asset_loader::SceneAssets, config::Config, theme::ColorAssociation};

/// Plugin that adds the functionality to visualise the position uncertainty of
/// each variable in a factorgraph.
/// The uncertainty is visualised as a 2D ellipse, around the mean of the
/// position.
pub struct UncertaintyVisualiserPlugin;

impl Plugin for UncertaintyVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UncertaintyVisualizerEnabled>()
            .add_systems(
                Update,
                (
                    init_uncertainty,
                    show_or_hide_uncertainty,
                    update_uncertainty.run_if(uncertainty_visualizer_enabled),
                ),
            );
    }
}

/// A plugin local resource to keep track of whether to enable/disable
/// visualisation
#[derive(Resource)]
struct UncertaintyVisualizerEnabled(bool);

fn uncertainty_visualizer_enabled(res: Res<UncertaintyVisualizerEnabled>) -> bool {
    res.0
}

impl Default for UncertaintyVisualizerEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// A **Bevy** [`Component`] to mark an entity as visualised _uncertainty_
/// gaussian
#[derive(Component)]
pub struct UncertaintyVisualiser;

/// A **Bevy** [`Component`] to mark a robot that it has a corresponding
/// `UncertaintyVis` entity Useful for easy _exclusion_ in queries
#[derive(Component)]
pub struct HasUncertaintyVisualiser;

/// A **Bevy** [`Update`] system
/// Initialises each new [`FactorGraph`] components to have a matching 2D circle
/// and [`UncertaintyVisualiser`] component I.e. if the [`FactorGraph`]
/// component already has a [`HasUncertaintyVisualiser`], it will be ignored
fn init_uncertainty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &FactorGraph, &ColorAssociation), Without<HasUncertaintyVisualiser>>,
    scene_assets: Res<SceneAssets>,
    config: Res<Config>,
) {
    query
        .iter()
        .for_each(|(entity, factorgraph, color_association)| {
            // Mark the robot with `HasUncertaintyVisualiser` to exclude next time
            commands.entity(entity).insert(HasUncertaintyVisualiser);

            factorgraph.variables().for_each(|(index, v)| {
                // let mean = v.belief.mean();
                #[allow(clippy::cast_possible_truncation)]
                let [x, y] = v.estimated_position();
                let transform = Vec3::new(
                    x as f32,
                    config.visualisation.height.objects - 2.0 * Z_FIGHTING_OFFSET, /* just under
                                                                                    * the
                                                                                    * lines (z-fighting
                                                                                    * prevention) */
                    y as f32,
                );

                // if the covariance is too large, we won't be able to visualise it
                // however, with this check, we can visualise it in a different colour
                // such that the user knows that the uncertainty is too large, and
                // that the size/shape of the visualisation is not accurate
                let mut attenable = true;

                // covariance matrix
                // [[a, b, _, _],
                //  [b, c, _, _],
                //  [_, _, _, _],
                //  [_, _, _, _]]
                let covariance = &v.belief.sigma;

                // half major axis λ₁ and half minor axis λ₂
                // λ₁ = (a + c) / 2 + √((a - c)² / 4 + b²)
                // λ₂ = (a + c) / 2 - √((a - c)² / 4 + b²)
                let a = covariance[(0, 0)];
                let b = covariance[(0, 1)];
                let c = covariance[(1, 1)];

                let (half_major_axis, half_minor_axis) = {
                    let first_term = (a + c) / 2.0;
                    let second_term = f64::sqrt((a - c).powi(2) / 4.0 + b * b);

                    (first_term + second_term, first_term - second_term)
                };

                // angle of the major axis with the x-axis
                // θ = arctan²(λ₁ - a, b)
                let angle = f64::atan2(half_major_axis - a, b) as f32;

                let mesh = meshes.add(Ellipse::new(
                    // pick `x` from the covariance diagonal, but cap it at 10.0
                    if half_major_axis > 20.0 {
                        attenable = false;
                        config.visualisation.uncertainty.max_radius
                    } else {
                        a as f32
                        // covariance.diag()[0] as f32
                    },
                    // pick `y` from the covariance diagonal, but cap it at 10.0
                    if half_minor_axis > 20.0 {
                        attenable = false;
                        config.visualisation.uncertainty.max_radius
                    } else {
                        c as f32
                        // covariance.diag()[1] as f32
                    },
                ));

                let mut transform = Transform::from_translation(transform)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
                transform.rotate_y(angle);

                // error!(
                //     "{:?}: Initialising uncertainty at {:?}, with covariance {:?}",
                //     entity, transform, covariance
                // );

                let material = if attenable {
                    // scene_assets.materials.uncertainty.clone()
                    materials.add(StandardMaterial {
                        // base_color: color_association.color.with_a(0.2),
                        base_color: Color::Rgba {
                            red:   color_association.color.r(),
                            green: color_association.color.g(),
                            blue:  color_association.color.b(),
                            alpha: 0.2,
                        },
                        ..Default::default()
                    })
                } else {
                    scene_assets.materials.uncertainty_unattenable.clone()
                };
                let visibility = if config.visualisation.draw.uncertainty {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                // Spawn a `UncertaintyVisualiser` component with a corresponding 2D circle
                commands.spawn((
                    RobotTracker::new(entity).with_variable_index(index.into()),
                    UncertaintyVisualiser,
                    PbrBundle {
                        mesh,
                        material,
                        transform,
                        visibility,
                        ..Default::default()
                    },
                ));
            });
        });
}

/// A **Bevy** [`Update`] system
/// Updates the [`Transform`]s of all [`UncertaintyVisualiser`] entities
/// Update the shape and potentially the material of the
/// [`UncertaintyVisualiser`] entities, depending on how the covariance has
/// changed
///
/// Done by cross-referencing with the [`FactorGraph`] components
/// that have matching [`Entity`] with the `RobotTracker.robot_id`
/// and variables in the [`FactorGraph`] that have matching
/// `RobotTracker.variable_id`
fn update_uncertainty(
    mut tracker_query: Query<
        (
            &RobotTracker,
            &mut Transform,
            &mut Handle<Mesh>,
            &mut Handle<StandardMaterial>,
        ),
        With<UncertaintyVisualiser>,
    >,
    factorgraph_query: Query<(Entity, &FactorGraph, &ColorAssociation)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
    scene_assets: Res<SceneAssets>,
) {
    // Update the `RobotTracker` components
    for (tracker, mut transform, mut mesh, mut material) in tracker_query.iter_mut() {
        for (entity, factorgraph, color_association) in factorgraph_query.iter() {
            // continue if we're not looking at the right robot
            if tracker.robot_id != entity {
                continue;
            }

            // else look through the variables
            for (index, v) in factorgraph.variables() {
                // continue if we're not looking at the right variable
                if usize::from(index) != tracker.variable_index {
                    continue;
                }

                let mean = &v.belief.mu;
                let covariance = &v.belief.sigma;
                // pretty_print_matrix!(covariance);

                let mut attenable = true;
                // pick `x` from the covariance diagonal, but cap it at 10.0
                let half_width = if covariance.diag()[0] > 20.0 {
                    attenable = false;
                    config.visualisation.uncertainty.max_radius
                } else {
                    // 1.0
                    covariance.diag()[0] as f32 * config.visualisation.uncertainty.scale
                };
                // pick `y` from the covariance diagonal, but cap it at 10.0
                let half_height = if covariance.diag()[1] > 20.0 {
                    attenable = false;
                    config.visualisation.uncertainty.max_radius
                } else {
                    // 2.0
                    covariance.diag()[1] as f32 * config.visualisation.uncertainty.scale
                };

                // dbg!((half_width, half_height));

                // error!("creating new ellipse");
                let new_mesh = meshes.add(Ellipse::new(half_width, half_height));

                // info!("{:?}: Updating uncertainty at {:?}, with covariance {:?}", entity,
                // transform, covariance);

                // else update the transform
                *transform = Transform::from_translation(Vec3::new(
                    mean[0] as f32,
                    config.visualisation.height.objects - 2.0 * Z_FIGHTING_OFFSET, // just under the lines (z-fighting prevention)
                    mean[1] as f32,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

                // update the mesh and material
                *mesh = new_mesh;
                *material = if attenable {
                    // scene_assets.materials.uncertainty.clone()
                    materials.add(StandardMaterial {
                        // base_color: color_association.color.with_a(0.2),
                        base_color: Color::Rgba {
                            red:   color_association.color.r(),
                            green: color_association.color.g(),
                            blue:  color_association.color.b(),
                            alpha: 0.2,
                        },
                        ..Default::default()
                    })
                } else {
                    scene_assets.materials.uncertainty_unattenable.clone()
                };
            }
        }
    }
}

/// A **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::Uncertainty` the boolean `DrawSettingEvent.value` will be used
/// to set the visibility of the [`UncertaintyVisualiser`] entities
fn show_or_hide_uncertainty(
    mut query: Query<(&UncertaintyVisualiser, &mut Visibility)>,
    mut draw_setting_event: EventReader<crate::ui::DrawSettingsEvent>,
    mut enabled: ResMut<UncertaintyVisualizerEnabled>,
) {
    for event in draw_setting_event.read() {
        debug!("received event to toggle draw visibility of gaussian uncertainty");
        if matches!(event.setting, crate::config::DrawSetting::Uncertainty) {
            let new_visibility_state = if event.draw {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            enabled.0 = event.draw;
            for (_, mut visibility) in query.iter_mut() {
                *visibility = new_visibility_state;
            }
        }
    }
}
