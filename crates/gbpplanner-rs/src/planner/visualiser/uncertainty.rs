#![warn(missing_docs)]
use bevy::prelude::*;

use super::{RobotTracker, Z_FIGHTING_OFFSET};
use crate::{
    asset_loader::Materials,
    bevy_utils::run_conditions::event_exists,
    config::Config,
    factorgraph::prelude::FactorGraph,
    input::DrawSettingsEvent,
    simulation_loader,
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt},
};

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
                    show_or_hide_uncertainty.run_if(event_exists::<DrawSettingsEvent>),
                    // show_or_hide_uncertainty.run_if(event_exists::<DrawSetting<Uncertainty>>),
                    update_uncertainty.run_if(uncertainty_visualizer_enabled),
                    // update_velocity_uncertainty,
                    // remove_all_uncertainty_visualisers.run_if(on_event::<ReloadSimulation>()),
                    // remove_all_uncertainty_visualisers.run_if(on_event::<EndSimulation>()),
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
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn init_uncertainty(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &FactorGraph, &ColorAssociation), Without<HasUncertaintyVisualiser>>,
    // scene_assets: Res<SceneAssets>,
    materials: Res<Materials>,
    config: Res<Config>,
    theme: Res<CatppuccinTheme>,
) {
    query.iter().for_each(|(entity, factorgraph, color_association)| {
        // Mark the robot with `HasUncertaintyVisualiser` to exclude next time
        commands.entity(entity).insert(HasUncertaintyVisualiser);

        factorgraph.variables().for_each(|(index, v)| {
            // let mean = v.belief.mean();
            #[allow(clippy::cast_possible_truncation)]
            let [x, y] = v.estimated_position();
            #[allow(clippy::cast_possible_truncation)]
            let transform = Vec3::new(
                x as f32,
                2.0f32.mul_add(-Z_FIGHTING_OFFSET, config.visualisation.height.objects), /* just under
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
            let covariance = &v.belief.covariance_matrix;

            // half major axis λ₁ and half minor axis λ₂
            // λ₁ = (a + c) / 2 + √((a - c)² / 4 + b²)
            // λ₂ = (a + c) / 2 - √((a - c)² / 4 + b²)
            let a = covariance[(0, 0)];
            let b = covariance[(0, 1)];
            let c = covariance[(1, 1)];

            let (half_major_axis, half_minor_axis) = {
                let first_term = (a + c) / 2.0;
                let second_term = f64::sqrt(b.mul_add(b, (a - c).powi(2) / 4.0));

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
                },
                // pick `y` from the covariance diagonal, but cap it at 10.0
                if half_minor_axis > 20.0 {
                    attenable = false;
                    config.visualisation.uncertainty.max_radius
                } else {
                    c as f32
                },
            ));

            let mut transform = Transform::from_translation(transform)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
            transform.rotate_y(angle);

            let material = if attenable {
                // scene_assets.materials.uncertainty.clone()
                standard_material_assets.add(StandardMaterial {
                    base_color: Color::from_catppuccin_colour_with_alpha(
                        theme.get_display_colour(&color_association.name),
                        0.2,
                    ),
                    ..Default::default()
                })
            } else {
                materials.uncertainty_unattenable.clone()
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
                simulation_loader::Reloadable,
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
#[allow(clippy::type_complexity, clippy::cast_possible_truncation)]
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
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
    config: Res<Config>,
    materials: Res<Materials>,
    // scene_assets: Res<SceneAssets>,
    theme: Res<CatppuccinTheme>,
) {
    // Update the `RobotTracker` components
    for (tracker, mut transform, mut mesh, mut material) in &mut tracker_query {
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

                let mean = &v.belief.mean;
                let covariance = &v.belief.covariance_matrix;
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
                let new_mesh = mesh_assets.add(Ellipse::new(half_width, half_height));

                // info!("{:?}: Updating uncertainty at {:?}, with covariance {:?}", entity,
                // transform, covariance);

                // else update the transform
                *transform = Transform::from_translation(Vec3::new(
                    mean[0] as f32,
                    2.0f32.mul_add(-Z_FIGHTING_OFFSET, config.visualisation.height.objects), /* just under the lines
                                                                                              * (z-fighting
                                                                                              * prevention) */
                    mean[1] as f32,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

                // update the mesh and material
                *mesh = new_mesh;
                *material = if attenable {
                    // scene_assets.materials.uncertainty.clone()
                    standard_material_assets.add(StandardMaterial {
                        base_color: Color::from_catppuccin_colour_with_alpha(
                            theme.get_display_colour(&color_association.name),
                            0.2,
                        ),
                        ..Default::default()
                    })
                } else {
                    materials.uncertainty_unattenable.clone()
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
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
    mut enabled: ResMut<UncertaintyVisualizerEnabled>,
) {
    for event in evr_draw_settings.read() {
        // debug!("received event to toggle draw visibility of gaussian uncertainty");
        if matches!(event.setting, crate::config::DrawSetting::Uncertainty) {
            let new_visibility_state = if event.draw {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            enabled.0 = event.draw;
            for (_, mut visibility) in &mut query {
                *visibility = new_visibility_state;
            }
        }
    }
}

fn remove_all_uncertainty_visualisers(
    mut commands: Commands,
    query: Query<Entity, With<UncertaintyVisualiser>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_velocity_uncertainty(mut gizmos: Gizmos, q: Query<&FactorGraph>) {
    for fgraph in &q {
        for (_, v) in fgraph.variables() {
            // let mean = v.belief.mean.view();
            // covariance matrix
            // [[_, _, _, _],
            //  [_, _, _, _],
            //  [_, _, x, _],
            //  [_, _, _, y]]
            let covariance = v.belief.covariance_matrix.view();

            // Draw a velocity vector
            let pos = v.estimated_position_vec2();
            let vx = covariance[(2, 2)] * 300.0;
            let vy = covariance[(3, 3)] * 300.0;

            // dbg!(&covariance);

            let start = Vec3::new(pos.x, 0.0, pos.y);
            let end = Vec3::new(pos.x + vx as f32, 0.0, pos.y + vy as f32);
            // dbg!((&start, &end));
            gizmos.arrow(start, end, Color::RED);
        }
    }
}
