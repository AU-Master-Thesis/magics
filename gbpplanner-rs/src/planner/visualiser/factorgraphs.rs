use bevy::prelude::*;
use itertools::Itertools;

use super::{super::FactorGraph, Line, Path, RobotTracker};
use crate::{
    asset_loader::SceneAssets,
    config::{Config, DrawSetting},
    theme::ColorFromCatppuccinColourExt,
};

pub struct FactorGraphVisualiserPlugin;

impl Plugin for FactorGraphVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                init_factorgraphs,
                update_factorgraphs,
                show_or_hide_factorgraphs,
                draw_lines,
            ),
        );
    }
}

/// A **Bevy** [`Component`] to mark an entity as a visualised _factor graph_
#[derive(Component)]
pub struct VariableVisualiser;

/// A **Bevy** [`Component`] to mark a robot that it has a corresponding
/// `FactorGraphVis` entity Useful for easy _exclusion_ in queries
#[derive(Component)]
pub struct HasFactorGraphVisualiser;

/// A **Bevy** [`Update`] system
/// Initialises each new [`FactorGraph`] component to have a matching
/// [`PbrBundle`] and [`FactorGraphVisualiser`] component I.e. if the
/// [`FactorGraph`] component already has a [`FactorGraphVisualiser`], it will
/// be ignored
fn init_factorgraphs(
    mut commands: Commands,
    query: Query<(Entity, &FactorGraph), Without<HasFactorGraphVisualiser>>,
    scene_assets: Res<SceneAssets>,
    config: Res<Config>,
) {
    for (entity, factorgraph) in query.iter() {
        // Mark the robot with `HasFactorGraphVisualiser` to exclude next time
        commands.entity(entity).insert(HasFactorGraphVisualiser);

        factorgraph
            .variables()
            .enumerate()
            .for_each(|(i, (index, v))| {
                let transform = Vec3::new(
                    v.mu[0] as f32,
                    config.visualisation.height.objects,
                    v.mu[1] as f32,
                );

                // Spawn a `FactorGraphVisualiser` component with a corresponding `PbrBundle`
                commands.spawn((
                    RobotTracker::new(entity)
                        .with_variable_id(index.into())
                        .with_order(i),
                    VariableVisualiser,
                    PbrBundle {
                        mesh: scene_assets.meshes.variable.clone(),
                        material: scene_assets.materials.variable.clone(),
                        transform: Transform::from_translation(transform),
                        ..Default::default()
                    },
                ));
            });
    }
}

/// A **Bevy** [`Update`] system
/// Updates the [`Transform`]s of all [`FactorGraphVisualiser`] entities
/// Done by cross-referencing with the [`FactorGraph`] components
/// that have matching [`Entity`] with the `RobotTracker.robot_id`
/// and variables in the [`FactorGraph`] that have matching
/// `RobotTracker.variable_id`
fn update_factorgraphs(
    mut tracker_query: Query<(&RobotTracker, &mut Transform), With<VariableVisualiser>>,
    factorgraph_query: Query<(Entity, &FactorGraph)>,
    config: Res<Config>,
) {
    // Update the `RobotTracker` components
    for (tracker, mut transform) in tracker_query.iter_mut() {
        for (entity, factorgraph) in factorgraph_query.iter() {
            // continue if we're not looking at the right robot
            let not_the_right_robot = tracker.robot_id != entity;
            if not_the_right_robot {
                continue;
            }

            // else look through the variables
            for (index, v) in factorgraph.variables() {
                // continue if we're not looking at the right variable
                if usize::from(index) != tracker.variable_id {
                    continue;
                }

                if !v.finite_covariance() {
                    continue;
                }

                // else update the transform
                // let mean = v.belief.mean();
                transform.translation = Vec3::new(
                    v.mu[0] as f32,
                    config.visualisation.height.objects,
                    v.mu[1] as f32,
                );
            }
        }
    }
}

/// A **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::PredictedTrajectories` the boolean `DrawSettingEvent.value`
/// will be used to set the visibility of the [`VariableVisualiser`] entities
fn show_or_hide_factorgraphs(
    mut query: Query<(&VariableVisualiser, &mut Visibility)>,
    mut draw_setting_event: EventReader<crate::ui::DrawSettingsEvent>,
) {
    for event in draw_setting_event.read() {
        if matches!(event.setting, DrawSetting::PredictedTrajectories) {
            for (_, mut visibility) in query.iter_mut() {
                *visibility = if event.draw {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// A **Bevy** [`Update`] system
/// Draws lines between all variables in each factor graph
///
/// Despawns old lines, and spawns new lines
///
/// Queries variables by [`RobotTracker`] with the [`FactorGraphVisualiser`]
/// component as initialised by the `init_factorgraphs` system
/// -> Will return if this query is empty
fn draw_lines(
    mut gizmos: Gizmos,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    // should_i_draw_lines: Res<ShouldIDrawLines>,
    config: Res<Config>,
    query_variables: Query<(&RobotTracker, &Transform), With<VariableVisualiser>>,
    query_previous_lines: Query<Entity, With<Line>>,
    factorgraph_query: Query<Entity, With<FactorGraph>>,
    catppuccin_theme: Res<crate::theme::CatppuccinTheme>,
) {
    // If there are no variables visualised yet by the `init_factorgraphs` system,
    // return
    if query_variables.iter().count() == 0 {
        return;
    }

    // Remove previous lines
    // TODO: Update lines instead of removing and re-adding
    for entity in query_previous_lines.iter() {
        commands.entity(entity).despawn();
    }

    // If we're not supposed to draw lines, return
    if !config.visualisation.draw.predicted_trajectories {
        return;
    }

    // let line_material =
    // materials.add(Color::from_catppuccin_colour(catppuccin_theme.text()));

    let color = Color::from_catppuccin_colour(catppuccin_theme.text());

    for entity in factorgraph_query.iter() {
        let positions = query_variables
            .iter()
            .filter(|(tracker, _)| tracker.robot_id == entity)
            .sorted_by(|(a, _), (b, _)| a.order.cmp(&b.order))
            .rev()
            .map(|(_, t)| t.translation)
            .collect::<Vec<Vec3>>();

        // let line = Path::new(positions.clone()).with_width(0.2);
        //
        // commands.spawn((
        //     PbrBundle {
        //         mesh: meshes.add(Mesh::from(line)),
        //         material: line_material.clone(),
        //         ..Default::default()
        //     },
        //     Line,
        // ));
        //

        for window in positions.windows(2) {
            let start = window[0];
            let end = window[1];
            gizmos.line(start, end, color);
        }

        // gizmos.primitive_3d(
        //     Polyline3d::<100>::new(positions),
        //     // BoxedPolyline3d::new(positions),
        //     Vec3::ZERO,
        //     Quat::IDENTITY,
        //     Color::from_catppuccin_colour(catppuccin_theme.text()),
        // );
    }
}
