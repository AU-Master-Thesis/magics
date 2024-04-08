use std::{collections::BTreeMap, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use itertools::Itertools;
use ringbuf::{HeapRb, Rb, StaticRb};

const MAX_TRACE_LENGTH: usize = 100;
const SAMPLE_DELAY: f32 = 0.5;

use crate::{
    config::Config,
    planner::{
        robot::{DespawnRobotEvent, SpawnRobotEvent},
        RobotId, RobotState,
    },
    robot_spawner::RobotSpawnedEvent,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

pub struct TracerVisualiserPlugin;

impl Plugin for TracerVisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Traces>().add_systems(
            Update,
            (
                track_initial_robot_positions,
                track_robots.run_if(on_timer(Duration::from_secs_f32(SAMPLE_DELAY))),
                draw_traces.run_if(draw_paths_enabled),
                // remove_trace_of_despawned_robot,
            ),
        );
    }
}

/// **Bevy** [`Resource`] to store all robot traces
// Uses a ring buffer to store the traces, to ensure a maximum fixed size.
#[derive(Default, Resource)]
// pub struct Traces(pub BTreeMap<RobotId, Vec<Vec3>>);
// pub struct Traces(pub BTreeMap<RobotId, HeapRb<Vec3>>);
pub struct Traces(pub BTreeMap<RobotId, StaticRb<Vec3, MAX_TRACE_LENGTH>>);

fn remove_trace_of_despawned_robot(
    mut traces: ResMut<Traces>,
    mut despawn_robot_event: EventReader<DespawnRobotEvent>,
) {
    for DespawnRobotEvent(robot_id) in despawn_robot_event.read() {
        match traces.0.remove(robot_id) {
            Some(_) => info!("removed trace of robot: {:?}", robot_id),
            None => error!(
                "attempted to remove trace of untracked robot: {:?}",
                robot_id
            ),
        }
    }
}

fn track_initial_robot_positions(
    query: Query<(RobotId, &Transform), With<RobotState>>,
    mut traces: ResMut<Traces>,
    mut spawn_robot_event: EventReader<SpawnRobotEvent>,
) {
    spawn_robot_event
        .read()
        .for_each(|SpawnRobotEvent(robot_id)| {
            for (other_robot_id, transform) in query.iter() {
                if other_robot_id == *robot_id {
                    let _ = traces
                        .0
                        .entry(*robot_id)
                        .or_default()
                        .push_overwrite(transform.translation);
                }
            }
        });
}

/// **Bevy** [`Update`] system
/// To update the [`Traces`] resource
fn track_robots(
    query: Query<(RobotId, &Transform), (With<RobotState>, Changed<Transform>)>,
    mut traces: ResMut<Traces>,
) {
    debug!("sampling robot positions");

    for (robot_id, transform) in query.iter() {
        let _ = traces
            .0
            .entry(robot_id)
            .or_default()
            // .or_insert_with(StaticRb::default)
            // .or_insert_with(|| HeapRb::new(MAX_TRACE_LENGTH))
            .push_overwrite(transform.translation);
        // .or_insert_with(Vec::new)
        // .push(transform.translation);
    }
}

#[inline]
fn draw_paths_enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.paths
}

/// **Bevy** [`Update`] system
/// To draw the robot traces; using the [`Traces`] resource
fn draw_traces(mut gizmos: Gizmos, traces: Res<Traces>, catppuccin_theme: Res<CatppuccinTheme>) {
    // if !config.visualisation.draw.paths {
    //     // error!("draw_traces: visualisation.draw.paths is false");
    //     return;
    // }

    // TODO: avoid allocating a new iterator every frame
    let mut colours = catppuccin_theme.colours().into_iter().cycle();
    // let colours = catppuccin_theme.colours().into_iter().collect::<Vec<_>>();

    for (robot_id, trace) in traces.0.iter() {
        let color = colours.next().unwrap();
        // PERF: compute the color once, and store it in the traces resource or a Local
        // let color = colours[robot_id.index() as usize % colours.len()];
        let color = Color::from_catppuccin_colour(color);

        // error!("trace: {:?}", trace);

        // use a window of length 2 to iterate over the trace, and draw a line between
        // each pair of points
        // for window in trace.windows(2) {
        for (start, end) in trace.iter().tuple_windows() {
            // let start = window[0];
            // let end = window[1];
            gizmos.line(*start, *end, color);
        }

        // let initial_position = trace.first().unwrap();
        // gizmos.primitive_3d(
        //     Polyline3d::<100>::new(trace.clone()),
        //     *initial_position,
        //     // Vec3::ZERO,
        //     Quat::IDENTITY,
        //     color,
        // );
    }
}
