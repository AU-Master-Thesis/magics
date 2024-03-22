use std::collections::BTreeMap;

use bevy::prelude::*;
use itertools::Itertools;
use ringbuf::{HeapRb, Rb, StaticRb};

const MAX_TRACE_LENGTH: usize = 100;
const SAMPLE_DELAY: f32 = 0.5;

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

// IDEA: use a ring buffer to store the traces, to ensure a maximum fixed size
/// **Bevy** [`Resource`] to store all robot traces
#[derive(Default, Resource)]
// pub struct Traces(pub BTreeMap<RobotId, Vec<Vec3>>);
// pub struct Traces(pub BTreeMap<RobotId, HeapRb<Vec3>>);
pub struct Traces(pub BTreeMap<RobotId, StaticRb<Vec3, MAX_TRACE_LENGTH>>);

// #[derive(Resource)]
pub struct SampleDelay(Timer);

impl Default for SampleDelay {
    fn default() -> Self {
        Self(Timer::from_seconds(SAMPLE_DELAY, TimerMode::Repeating))
    }
}

/// **Bevy** [`Update`] system
/// To update the [`Traces`] resource
fn track_robots(
    query: Query<(RobotId, &Transform), (With<RobotState>, Changed<Transform>)>,
    mut traces: ResMut<Traces>,
    time: Res<Time>,
    mut sample_delay: Local<SampleDelay>,
) {
    if !sample_delay.0.tick(time.delta()).just_finished() {
        return;
    }

    debug!("sampling robot positions");

    for (robot_id, transform) in query.iter() {
        let _ = traces
            .0
            .entry(robot_id)
            .or_insert_with(StaticRb::default)
            // .or_insert_with(|| HeapRb::new(MAX_TRACE_LENGTH))
            .push_overwrite(transform.translation);
        // .or_insert_with(Vec::new)
        // .push(transform.translation);
    }
}

/// **Bevy** [`Update`] system
/// To draw the robot traces; using the [`Traces`] resource
fn draw_traces(
    mut gizmos: Gizmos,
    traces: Res<Traces>,
    catppuccin_theme: Res<CatppuccinTheme>,
    config: Res<Config>,
) {
    if !config.visualisation.draw.paths {
        // error!("draw_traces: visualisation.draw.paths is false");
        return;
    }

    // TODO: avoid allocating a new iterator every frame
    // let mut colours = catppuccin_theme.colours().into_iter().cycle();
    let colours = catppuccin_theme.colours().into_iter().collect::<Vec<_>>();

    for (robot_id, trace) in traces.0.iter() {
        // let color = colours.next().unwrap();
        // PERF: compute the color once, and store it in the traces resource or a Local
        let color = colours[robot_id.index() as usize % colours.len()];
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
