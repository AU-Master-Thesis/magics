//! Step functionality for the API.
//!
//! This module provides functionality for stepping the simulation.

use bevy::prelude::*;
use gbp_config::{Config, formation};

use crate::{
    factorgraph::factorgraph::FactorGraph,
    planner::{
        collisions::resources::{RobotEnvironmentCollisions, RobotRobotCollisions},
        robot::{Mission, RadioAntenna, Radius, RobotConnections},
    },
    environment::ObstacleMarker,
    api::plugin::PreviousCollisionCounts,
};

use super::{
    state::ApiState,
    despawned_agents::DespawnedAgentsTracker,
};

/// Store the start time of each step
#[derive(Default)]
pub struct StepTimeTracker {
    pub start_time: Option<f32>,
    pub last_time: Option<f32>,
}

/// Run condition for when a step is in progress
pub fn api_step_in_progress(api_state: Res<ApiState>) -> bool {
    api_state.is_step_requested() &&
    api_state.get_step_iterations_remaining() > 0
}

/// System to process step requests at the beginning of the frame
pub fn process_step_request(
    api_state: Res<ApiState>,
    mut time_virtual: ResMut<Time<Virtual>>,
    config: Res<Config>,
    mut step_tracker: Local<StepTimeTracker>,
) {
    if api_state.is_step_requested() &&
       api_state.get_step_iterations_remaining() == 0 {
        // Log the current virtual time
        let before_time = time_virtual.elapsed_seconds();
        info!("API: Starting fixed step - Virtual time before: {:.6}s", before_time);

        // Store the start time
        step_tracker.start_time = Some(before_time);
        step_tracker.last_time = Some(before_time);

        // Set the number of iterations to run based on the configured iterations_per_step
        let iterations = api_state.get_iterations_per_step();
        api_state.set_step_iterations_remaining(iterations);

        // Unpause the simulation to allow systems to run
        let virtual_time = time_virtual.bypass_change_detection();
        virtual_time.unpause();

        // Calculate fixed delta based on simulation Hz
        let fixed_delta = 1.0 / config.simulation.hz;
        info!("API: Started step with {} iterations using fixed delta of {:.6}s",
              iterations, fixed_delta);
    }
}

/// System to monitor FixedUpdate ticks
pub fn monitor_fixed_update(
    time_virtual: Res<Time<Virtual>>,
    api_state: Res<ApiState>,
    mut step_tracker: Local<StepTimeTracker>,
) {
    let current_time = time_virtual.elapsed_seconds();
    let delta = time_virtual.delta_seconds();
    let remaining = api_state.get_step_iterations_remaining();

    // Calculate time difference from last tick
    let time_diff = if let Some(last) = step_tracker.last_time {
        let diff = current_time - last;
        step_tracker.last_time = Some(current_time);
        diff
    } else {
        step_tracker.last_time = Some(current_time);
        0.0
    };

    // Log detailed time information for each FixedUpdate tick during a step
    info!(
        "FixedUpdate tick - Virtual time: {:.6}s, Delta: {:.6}s, Time since last tick: {:.6}s, Iterations remaining: {}",
        current_time,
        delta,
        time_diff,
        remaining
    );
}

/// System to complete step after iterations
pub fn complete_step_in_fixed_update(
    api_state: Res<ApiState>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut step_tracker: Local<StepTimeTracker>,
    // Add parameters needed for extract_state
    robots: Query<(
        Entity,
        &Transform,
        &FactorGraph,
        &RobotConnections,
        Option<&Mission>,
        Option<&formation::PlanningStrategy>,
        Option<&Radius>,
        Option<&RadioAntenna>,
    )>,
    obstacles: Query<&Transform, With<ObstacleMarker>>,
    config: Res<Config>,
    env_config: Res<gbp_environment::Environment>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
    mut previous_collision_counts: ResMut<PreviousCollisionCounts>,
    mut despawned_agents_tracker: ResMut<DespawnedAgentsTracker>, // Change to mutable ResMut
) {
    let remaining = api_state.decrement_step_iterations_remaining();

    if remaining <= 1 {
        // Log the current virtual time
        let after_time = time_virtual.elapsed_seconds();
        info!("API: Fixed step completed - Virtual time after: {:.6}s", after_time);

        // Calculate and log the total time advancement
        if let Some(start_time) = step_tracker.start_time {
            let time_advancement = after_time - start_time;
            info!("API: Step advanced virtual time by: {:.6}s", time_advancement);
            step_tracker.start_time = None;
        }

        // Extract state at the end of the step
        info!("API: Extracting state at end of step");
        super::extract::extract_state(
            &api_state,
            robots,
            obstacles,
            config,
            env_config,
            &robot_robot_collisions,
            &robot_environment_collisions,
            &mut previous_collision_counts,
            &mut despawned_agents_tracker // Pass mutably
        );

        // Pause the simulation again
        let virtual_time = time_virtual.bypass_change_detection();
        virtual_time.pause();

        // Mark the step as completed
        api_state.complete_step();

        info!("API: Completed step after {} iterations",
              api_state.get_step_iterations_remaining());
    }
}
