use std::{collections::BTreeMap, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use itertools::Itertools;
use ringbuf::{Rb, StaticRb};

const MAX_TRACE_LENGTH: usize = 100;
const SAMPLE_DELAY: f32 = 0.5;

use crate::{
    config::Config,
    planner::{
        robot::{RobotDespawned, RobotSpawned},
        RobotId, RobotState,
    },
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt},
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

pub struct Trace {
    color:       Color,
    ring_buffer: StaticRb<Vec3, MAX_TRACE_LENGTH>,
}

/// **Bevy** [`Resource`] to store all robot traces
// Uses a ring buffer to store the traces, to ensure a maximum fixed size.
#[derive(Default, Resource)]
// pub struct Traces(pub BTreeMap<RobotId, Vec<Vec3>>);
// pub struct Traces(pub BTreeMap<RobotId, HeapRb<Vec3>>);
pub struct Traces(pub BTreeMap<RobotId, Trace>);

#[allow(dead_code)]
fn remove_trace_of_despawned_robot(
    mut traces: ResMut<Traces>,
    mut despawn_robot_event: EventReader<RobotDespawned>,
) {
    for RobotDespawned(robot_id) in despawn_robot_event.read() {
        if traces.0.remove(robot_id).is_some() {
            info!("removed trace of robot: {:?}", robot_id);
        } else {
            error!(
                "attempted to remove trace of untracked robot: {:?}",
                robot_id
            );
        }
    }
}

fn track_initial_robot_positions(
    query: Query<(RobotId, &Transform, &ColorAssociation), With<RobotState>>,
    mut traces: ResMut<Traces>,
    mut spawn_robot_event: EventReader<RobotSpawned>,
    theme: Res<CatppuccinTheme>,
) {
    spawn_robot_event.read().for_each(|RobotSpawned(robot_id)| {
        for (other_robot_id, transform, color_association) in query.iter() {
            // initialise the first position of the robot into the ring buffer
            let mut ring_buffer = StaticRb::default();
            let _ = ring_buffer.push_overwrite(transform.translation);

            if other_robot_id == *robot_id {
                let _ = traces.0.entry(*robot_id).or_insert(Trace {
                    color: Color::from_catppuccin_colour(
                        theme.get_display_colour(&color_association.name),
                    ),
                    ring_buffer,
                });
            }
        }
    });
}

/// **Bevy** [`Update`] system
/// To update the [`Traces`] resource
#[allow(clippy::type_complexity)]
fn track_robots(
    query: Query<(RobotId, &Transform, &ColorAssociation), (With<RobotState>, Changed<Transform>)>,
    mut traces: ResMut<Traces>,
    theme: Res<CatppuccinTheme>,
) {
    debug!("sampling robot positions");

    for (robot_id, transform, color_association) in query.iter() {
        let _ = traces
            .0
            .entry(robot_id)
            .or_insert(Trace {
                color:       Color::from_catppuccin_colour(
                    theme.get_display_colour(&color_association.name),
                ),
                ring_buffer: StaticRb::default(),
            })
            .ring_buffer
            .push_overwrite(transform.translation);
        // .or_insert_with(StaticRb::default)
        // .or_insert_with(|| HeapRb::new(MAX_TRACE_LENGTH))
        // .push_overwrite(transform.translation);
        // .or_insert_with(Vec::new)
        // .push(transform.translation);

        // if let Some(trace) = traces.0.get_mut(&robot_id) {
        //     let _ = trace.ring_buffer.push_overwrite(transform.translation);
        // }
    }
}

#[inline]
fn draw_paths_enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.paths
}

/// **Bevy** [`Update`] system
/// To draw the robot traces; using the [`Traces`] resource
fn draw_traces(
    mut gizmos: Gizmos,
    traces: Res<Traces>,
    // catppuccin_theme: Res<CatppuccinTheme>
) {
    // if !config.visualisation.draw.paths {
    //     // error!("draw_traces: visualisation.draw.paths is false");
    //     return;
    // }

    // TODO: avoid allocating a new iterator every frame
    // let colours = catppuccin_theme.into_display_iter().cycle();
    // let colours = catppuccin_theme.colours().into_iter().collect::<Vec<_>>();

    for trace in traces.0.values() {
        // use a window of length 2 to iterate over the trace, and draw a line between
        // each pair of points
        // for window in trace.windows(2) {
        for (start, end) in trace.ring_buffer.iter().tuple_windows() {
            // let start = window[0];
            // let end = window[1];
            gizmos.line(*start, *end, trace.color);
        }
    }
}
