// region: Modules
mod bundle;
mod debug;
mod events;
mod gbp;
mod manual;
mod mission;
mod utils;
// endregion

// region: Imports
use bevy::prelude::*;
use gbp_config::Config;

use crate::{
    bevy_utils::run_conditions::time::virtual_time_is_paused,
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

// Re-export items needed by the plugin or potentially other modules
pub use self::{
    bundle::{
        FinishedPath, RadioAntenna, Radius, RobotBundle, RobotConnections, StateVector, T0,
        VariableTimesteps, Ball, RobotId,
    },
    debug::on_robot_clicked,
    events::{
        attach_despawn_timer_when_robot_finishes_route, progress_missions, reached_waypoint,
        request_snapshot_of_robot_when_it_finishes_its_route, GbpScheduleChanged, RobotDespawned,
        RobotFinishedRoute, RobotReachedWaypoint, RobotSpawned,
    },
    gbp::{
        create_interrobot_factors, delete_interrobot_factors, iterate_gbp_v2, on_gbp_schedule_changed,
        update_failed_comms, update_prior_of_current_state_v3, update_prior_of_horizon_state,
        update_robot_neighbours, GbpIterationSchedule,
    },
    manual::{finish_manual_step, start_manual_step, ManualModeState},
    mission::{Mission, MissionState, Route},
    utils::{reset_robot_number_generator, RobotNumberGenerator},
};
// endregion

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GbpIterationSchedule>()
            .init_resource::<RobotNumberGenerator>()
            .insert_state(ManualModeState::Disabled)
            .add_event::<RobotSpawned>()
            .add_event::<RobotDespawned>()
            .add_event::<RobotFinishedRoute>()
            .add_event::<RobotReachedWaypoint>()
            .add_event::<GbpScheduleChanged>()
            .add_systems(PreUpdate, start_manual_step.run_if(virtual_time_is_paused))
            .add_systems(
                Update,
                reset_robot_number_generator
                    .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
            )
            .add_systems(
                Update,
                (
                    on_robot_clicked,
                    on_gbp_schedule_changed,
                    attach_despawn_timer_when_robot_finishes_route,
                    request_snapshot_of_robot_when_it_finishes_its_route,
                    progress_missions.run_if(resource_exists::<gbp_global_planner::Colliders>),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    reached_waypoint,
                    // progress_missions.run_if(resource_exists::<gbp_global_planner::Colliders>),
                )
                    .run_if(not(virtual_time_is_paused)),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_robot_neighbours,
                    delete_interrobot_factors,
                    create_interrobot_factors,
                    update_failed_comms,
                    update_prior_of_horizon_state,
                    update_prior_of_current_state_v3,
                    iterate_gbp_v2,
                    finish_manual_step.run_if(ManualModeState::enabled),
                )
                    .chain()
                    .run_if(not(virtual_time_is_paused)),
            );
    }
}
