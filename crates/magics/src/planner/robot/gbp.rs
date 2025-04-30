use std::{
    collections::{BTreeSet, HashMap},
    num::NonZeroUsize,
};

use bevy::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::GlobalEntropy;
use gbp_config::Config;
use gbp_linalg::prelude::*;
use ndarray::{array, concatenate, s, Axis};
use rand::Rng;

use super::{
    bundle::{FinishedPath, RadioAntenna, Radius, RobotConnections, RobotId, T0},
    events::GbpScheduleChanged, // Assuming events.rs will exist
    mission::Mission,
    utils::RobotNumberGenerator, // Assuming utils.rs will exist
};
use crate::{
    factorgraph::{
        factor::{ExternalVariableId, FactorNode},
        factorgraph::{FactorGraph, NodeIndex, VariableIndex},
        id::{FactorId, VariableId},
        message::{FactorToVariableMessage, VariableToFactorMessage},
        DOFS,
    },
    planner::robot::VariableTimesteps,
};

#[derive(Clone, Copy, Debug, Component, Resource, derive_more::Into, derive_more::From)]
pub struct GbpIterationSchedule(pub gbp_config::GbpIterationSchedule);

impl GbpIterationSchedule {
    pub fn schedule(&self) -> Box<dyn gbp_schedule::GbpScheduleIterator> {
        let config = gbp_schedule::GbpScheduleParams {
            internal: self.0.internal as u8,
            external: self.0.external as u8,
        };
        self.0.schedule.get(config)
    }
}

impl FromWorld for GbpIterationSchedule {
    fn from_world(world: &mut World) -> Self {
        if let Some(config) = world.get_resource::<Config>() {
            Self(config.gbp.iteration_schedule)
        } else {
            Self(gbp_config::GbpIterationSchedule::default())
        }
    }
}

/// Called `Simulator::calculateRobotNeighbours` in **gbpplanner**
pub fn update_robot_neighbours(
    robots: Query<(Entity, &Transform), With<RobotConnections>>,
    mut query: Query<(Entity, &Transform, &mut RobotConnections)>,
    config: Res<Config>,
) {
    // TODO: use kdtree to speed up, and to have something in the report
    for (robot_id, transform, mut robotstate) in &mut query {
        robotstate.robots_within_comms_range = robots
            .iter()
            .filter_map(|(other_robot_id, other_transform)| {
                if other_robot_id == robot_id
                    || config.robot.communication.radius.get()
                        < transform.translation.distance(other_transform.translation)
                {
                    // Do not compute the distance to self
                    None
                } else {
                    Some(other_robot_id)
                }
            })
            .collect();
    }
}

pub fn delete_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotConnections)>,
) {
    // the set of robots connected with will (possibly) be mutated
    // the robots factorgraph will (possibly) be mutated
    // the other robot with an interrobot factor connected will be mutated

    let mut robots_to_delete_interrobot_factors_between: HashMap<RobotId, RobotId> = HashMap::new();

    for (robot_id, _, mut robotstate) in &mut query {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = robotstate
            .robots_connected_with
            .difference(&robotstate.robots_within_comms_range)
            .copied()
            .collect();

        robots_to_delete_interrobot_factors_between.extend(
            ids_of_robots_connected_with_outside_comms_range
                .iter()
                .map(|id| (robot_id, *id)),
        );

        for id in ids_of_robots_connected_with_outside_comms_range {
            robotstate.robots_connected_with.remove(&id);
        }
    }

    for (robot1, robot2) in robots_to_delete_interrobot_factors_between {
        // Will delete both interrobot factors as,
        // (a, b)
        // (a, c)
        // (b, a)
        // (c, a)
        // (b, d)
        // (d, b)
        // etc.
        // deletes a's interrobot factor connecting to b, a -> b
        // deletes b's interrobot factor connecting to a, b -> a

        if let Ok((_, mut factorgraph1, _)) = query.get_mut(robot1) {
            factorgraph1.delete_interrobot_factors_connected_to(robot2);
        } else {
            error!("Could not find robot1 in the query");
        };

        if let Ok((_, mut factorgraph2, _)) = query.get_mut(robot2) {
            factorgraph2.delete_interrobot_factors_connected_to(robot1);
        } else {
            error!(
                "attempt to delete interrobot factors between robots: {:?} and {:?} failed, \
                 reason: {:?} does not exist!",
                robot1, robot2, robot2
            );
        };
    }
}

pub fn create_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotConnections, &Radius)>,
    config: Res<Config>,
    mut robot_number_gen: ResMut<RobotNumberGenerator>,
) {
    // a mapping between a robot and the other robots it should create a interrobot
    // factor to e.g:
    // {a -> [b, c, d], b -> [a, c], c -> [a, b], d -> [c]}
    let new_connections_to_establish: HashMap<RobotId, Vec<RobotId>> = query
        .iter()
        .map(|(entity, _, robotstate, _)| {
            let new_connections = robotstate
                .robots_within_comms_range
                .difference(&robotstate.robots_connected_with)
                .copied()
                .collect::<Vec<_>>();

            (entity, new_connections)
        })
        .collect();

    // PERF(kpbaks): store a slice instead of a Vec<NodeIndex>
    let variable_indices_of_each_factorgraph: HashMap<RobotId, Vec<NodeIndex>> = query
        .iter()
        .map(|(robot_id, factorgraph, _, _)| {
            let variable_indices = factorgraph
                .variable_indices_ordered_by_creation()
                .skip(1) // skip current variable
                .collect::<Vec<_>>();

            let num_variables = factorgraph.node_count().variables;
            debug_assert_eq!(num_variables - 1, variable_indices.len());
            (robot_id, variable_indices)
        })
        .collect();

    let mut external_edges_to_add = Vec::new();

    for (robot_id, mut factorgraph, mut robotstate, radius) in &mut query {
        let num_variables_current = factorgraph.node_count().variables; // Number of variables for THIS robot
        for other_robot_id in new_connections_to_establish
            .get(&robot_id)
            .expect("the key is in the map")
        {
            let other_variable_indices = variable_indices_of_each_factorgraph
                .get(other_robot_id)
                .expect("the key is in the map"); // Indices for the OTHER robot

            // Determine the safe upper bound for the loop
            let num_variables_other = other_variable_indices.len() + 1;
            let loop_bound = std::cmp::min(num_variables_current, num_variables_other);

            // Loop goes from 1 up to num_variables (exclusive) of the CURRENT robot
            for i in 1..loop_bound {
                // Use loop_bound instead of num_variables_current
                let initial_measurement = Vector::<Float>::zeros(DOFS);
                let external_variable_id = ExternalVariableId::new(
                    *other_robot_id,
                    VariableIndex(other_variable_indices[i - 1]),
                );

                let interrobot_factor = FactorNode::new_interrobot_factor(
                    factorgraph.id(),
                    Float::from(config.gbp.sigma_factor_interrobot),
                    initial_measurement,
                    Float::from(radius.0).try_into().expect("> 0.0"),
                    Float::from(config.robot.inter_robot_safety_distance_multiplier.get())
                        .try_into()
                        .expect("> 0.0"),
                    external_variable_id,
                    robot_number_gen.next(),
                    config.gbp.factors_enabled.interrobot,
                );

                let factor_index = factorgraph.add_factor(interrobot_factor);

                let variable_index = factorgraph
                    .nth_variable_index(i)
                    .expect("there should be an i'th variable");

                let factor_id = FactorId::new(robot_id, factor_index);
                let graph_id = factorgraph.id();
                factorgraph.add_internal_edge(VariableId::new(graph_id, variable_index), factor_id);
                external_edges_to_add.push((robot_id, factor_index, *other_robot_id, i));
            }

            robotstate.robots_connected_with.insert(*other_robot_id);
        }
    }

    let mut temp = Vec::new();

    for (robot_id, factor_index, other_robot_id, i) in external_edges_to_add {
        let Ok((_, mut other_factorgraph, _, _)) = query.get_mut(other_robot_id) else {
            error!(
                "Could not find other_robot_id {:?} in query",
                other_robot_id
            );
            continue;
        };

        other_factorgraph.add_external_edge(FactorId::new(robot_id, factor_index), i);

        let (nth_variable_index, nth_variable) = other_factorgraph
            .nth_variable(i)
            .expect("the i'th variable should exist");

        let variable_message = nth_variable.prepare_message();
        let variable_id = VariableId::new(other_robot_id, nth_variable_index);

        temp.push((robot_id, factor_index, variable_message, variable_id));
    }

    for (robot_id, factor_index, variable_message, variable_id) in temp {
        let Ok((_, mut factorgraph, _, _)) = query.get_mut(robot_id) else {
            error!("Could not find robot_id {:?} in query", robot_id);
            continue;
        };

        if let Some(factor) = factorgraph.get_factor_mut(factor_index) {
            factor.receive_message_from(variable_id, variable_message.clone());
        } else {
            error!(
                "factorgraph {:?} has no factor with index {:?}",
                robot_id, factor_index
            );
        }
    }
}

/// At random turn on/off the robots "radio".
/// When the radio is turned of the robot will not be able to communicate with
/// any other robot. The probability of failure is set by the user in the config
/// file. `config.robot.communication.failure_rate`
/// Called `Simulator::setCommsFailure` in **gbpplanner**
pub fn update_failed_comms(
    mut antennas: Query<&mut RadioAntenna>,
    config: Res<Config>,
    mut prng: ResMut<GlobalEntropy<WyRand>>,
) {
    for mut antenna in &mut antennas {
        antenna.active = !prng.gen_bool(config.robot.communication.failure_rate.into());
    }
}

pub fn iterate_gbp_v2(
    mut query: Query<
        (
            &mut FactorGraph,
            &GbpIterationSchedule,
            &RadioAntenna,
            &Mission,
        ),
        With<RobotConnections>,
    >,
    config: Res<Config>,
) {
    println!("iterating gbp");
    let schedule_config = gbp_schedule::GbpScheduleParams {
        internal: config.gbp.iteration_schedule.internal as u8,
        external: config.gbp.iteration_schedule.external as u8,
    };
    let schedule = config.gbp.iteration_schedule.schedule.get(schedule_config);

    for gbp_schedule::GbpScheduleAtIteration { internal, external } in schedule {
        if internal {
            query
                .par_iter_mut()
                .for_each(|(mut factorgraph, _, antenna, mission)| {
                    if !mission.state.idle() {
                        factorgraph.internal_factor_iteration();
                        factorgraph.internal_variable_iteration();
                    }
                });
        }

        if external {
            let mut messages_to_external_variables = vec![];
            for (mut factorgraph, _, antenna, mission) in query.iter_mut() {
                if !antenna.active || mission.state.idle() {
                    continue;
                }
                messages_to_external_variables
                    .extend(factorgraph.external_factor_iteration().drain(..));
            }

            // Send messages to external variables
            for message in messages_to_external_variables.into_iter() {
                let Ok((mut external_factorgraph, _, antenna, mission)) =
                    query.get_mut(message.to.factorgraph_id)
                else {
                    continue;
                };

                // cannot receive any new messages if antenna is turned off
                if !antenna.active || mission.state.idle() {
                    continue;
                }

                if let Some(variable) =
                    external_factorgraph.get_variable_mut(message.to.variable_index)
                {
                    variable.receive_message_from(message.from, message.message);
                }
            }

            let mut messages_to_external_factors = vec![];
            for (mut factorgraph, _, antenna, mission) in query.iter_mut() {
                if !antenna.active || mission.state.idle() {
                    continue;
                }
                messages_to_external_factors
                    .extend(factorgraph.external_variable_iteration().drain(..));
            }

            // Send messages to external factors
            for message in messages_to_external_factors.into_iter() {
                let Ok((mut external_factorgraph, _, antenna, mission)) =
                    query.get_mut(message.to.factorgraph_id)
                else {
                    continue;
                };

                // cannot receive any new messages if antenna is turned off
                if !antenna.active || mission.state.idle() {
                    continue;
                }

                if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
                    factor.receive_message_from(message.from, message.message);
                }
            }
        }
    }
}

pub fn update_prior_of_horizon_state(
    config: Res<Config>,
    time: Res<Time>,
    time_fixed: Res<Time<Fixed>>,
    mut query: Query<
        (
            Entity,
            &mut FactorGraph,
            &Mission,
            &mut FinishedPath,
            &Radius,
            &RadioAntenna,
            &VariableTimesteps,
            &T0,
        ),
        With<RobotConnections>,
    >,
    mut all_messages_to_external_factors: Local<Vec<VariableToFactorMessage>>,
) {
    let delta_t = Float::from(time_fixed.delta_seconds());
    let max_speed = Float::from(config.robot.target_speed.get());

    let mut robots_to_despawn: Vec<Entity> = Vec::new();

    for (
        robot_id,
        mut factorgraph,
        mission,
        mut finished_path,
        radius,
        antenna,
        variable_timesteps,
        t0,
    ) in &mut query
    {
        if finished_path.0 || mission.state.idle() {
            continue;
        }

        let Some(next_waypoint) = mission.next_waypoint() else {
            // debug!(
            //     "robot {:?} finished at {:?}",
            //     robot_id,
            //     mission.finished_at()
            // );
            finished_path.0 = true;
            robots_to_despawn.push(robot_id);
            continue;
        };

        if config.gbp.iteration_schedule.internal == 0 {
            continue;
        }

        // Get the current robot position (var0) - clone the data to avoid borrow issues
        let current_position = {
            let (_, current_variable) = factorgraph
                .nth_variable(0)
                .expect("factorgraph should have a current variable");
            current_variable.belief.mean.slice(s![..2]).to_owned()
        };

        // Now we can borrow factorgraph mutably
        let (horizon_variable_index, horizon_variable) = factorgraph.last_variable_mut().unwrap();
        let estimated_position = horizon_variable.belief.mean.slice(s![..2]);

        let next_waypoint_pos = array![
            Float::from(next_waypoint.position().x),
            Float::from(next_waypoint.position().y)
        ];

        // debug!(
        //     "Robot {:?} - Current position: [{:.2}, {:.2}], Horizon position: [{:.2},
        // {:.2}], \      Waypoint: [{:.2}, {:.2}]",
        //     robot_id,
        //     current_position[0],
        //     current_position[1],
        //     estimated_position[0],
        //     estimated_position[1],
        //     next_waypoint_pos[0],
        //     next_waypoint_pos[1]
        // );

        // Calculate vector toward waypoint and desired velocity
        let horizon2waypoint = next_waypoint_pos - estimated_position;
        let horizon2goal_dist = horizon2waypoint.euclidean_norm();

        let new_velocity = Float::min(max_speed, horizon2goal_dist) * horizon2waypoint.normalized();

        // Calculate unbounded new position
        let unbounded_new_position = estimated_position.into_owned() + (&new_velocity * delta_t);

        // Calculate maximum allowed distance from current position
        let t0_value = Float::from(**t0);
        let last_timestep = Float::from(*variable_timesteps.0.last().unwrap_or(&0));
        let time_diff_to_end = t0_value * last_timestep;

        let max_allowed_distance = max_speed * time_diff_to_end;

        // Check if unbounded position exceeds maximum allowed distance
        let current_to_new = &unbounded_new_position - &current_position;
        let distance_to_new = current_to_new.euclidean_norm();

        // debug!(
        //     "Robot {:?} - Unbounded new pos: [{:.2}, {:.2}], Max allowed distance:
        // {:.2}, Actual \      distance: {:.2}, t0: {:.4}, last_timestep:
        // {:.1}",     robot_id,
        //     unbounded_new_position[0],
        //     unbounded_new_position[1],
        //     max_allowed_distance,
        //     distance_to_new,
        //     t0_value,
        //     last_timestep
        // );

        // Bound the new position if necessary
        let new_position = if distance_to_new > max_allowed_distance {
            // Clamp to maximum allowed distance
            let bounded =
                current_position.to_owned() + max_allowed_distance * current_to_new.normalized();
            // debug!(
            //     "Robot {:?} - BOUNDING APPLIED! Bounded position: [{:.2}, {:.2}]",
            //     robot_id, bounded[0], bounded[1]
            // );
            bounded
        } else {
            // debug!("Robot {:?} - No bounding needed", robot_id);
            unbounded_new_position
        };

        // Update horizon state with new position and velocity
        let new_mean = concatenate![Axis(0), new_position, new_velocity];
        horizon_variable.belief.mean.clone_from(&new_mean);

        let messages_to_external_factors =
            factorgraph.change_prior_of_variable(horizon_variable_index, new_mean);
        all_messages_to_external_factors.extend(messages_to_external_factors);
    }

    // Send messages to external factors
    for message in all_messages_to_external_factors.drain(..) {
        let Ok((_, mut external_factorgraph, _, _, _, _, _, _)) =
            query.get_mut(message.to.factorgraph_id)
        else {
            continue;
        };

        if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
            factor.receive_message_from(message.from, message.message);
        }
    }
}

/// Called `Robot::updateCurrent` in **gbpplanner**
#[allow(clippy::cast_possible_truncation)]
pub fn update_prior_of_current_state_v3(
    mut query: Query<
        (
            &mut FactorGraph,
            &mut Transform,
            &T0,
            &Mission,
            &RadioAntenna,
        ),
        With<RobotConnections>,
    >,
    config: Res<Config>,
    time_fixed: Res<Time<Fixed>>,
) {
    for (mut factorgraph, mut transform, &t0, mission, antenna) in &mut query {
        if mission.state.idle() {
            continue;
        }

        let time_scale = time_fixed.delta_seconds() / *t0;
        let (current_variable_index, current_variable) = factorgraph
            .nth_variable(0)
            .expect("factorgraph should have a current variable");
        let (_, next_variable) = factorgraph
            .nth_variable(1)
            .expect("factorgraph should have a next variable");

        let change_in_state =
            Float::from(time_scale) * (&next_variable.belief.mean - &current_variable.belief.mean);

        let mean_updated = &current_variable.belief.mean + &change_in_state;

        let external_factor_messages =
            factorgraph.change_prior_of_variable(current_variable_index, mean_updated);
        assert!(
            external_factor_messages.is_empty(),
            "the current variable is not connected to any external factors"
        );

        // Casts are allowed by the attribute on the function signature
        transform.translation.x += change_in_state[0] as f32;
        transform.translation.z += change_in_state[1] as f32;
    }
}

pub fn on_gbp_schedule_changed(
    mut evr_gbp_schedule_changed: EventReader<GbpScheduleChanged>,
    mut q_robots: Query<(Entity, &mut GbpIterationSchedule)>,
) {
    for GbpScheduleChanged(new_schedule) in evr_gbp_schedule_changed.read() {
        for (entity, mut schedule) in q_robots.iter_mut() {
            *schedule = *new_schedule;
            info!(
                "changed gbp-schedule to: {:?} of entity {:?}",
                new_schedule, entity
            );
        }
    }
}
