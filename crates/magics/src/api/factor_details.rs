//! Module for extracting detailed information about factor graph components.

use std::collections::HashMap;

use bevy::prelude::*;
use gbp_linalg::VectorNorm;

use crate::{
    api::state::{
        DynamicFactorInfo, FactorDetails, InterRobotFactorInfo, ObstacleFactorInfo,
        TrackingFactorInfo, VariableInfo,
    },
    factorgraph::{
        factor::Factor,
        factorgraph::{FactorGraph, FactorIndex},
    },
};

/// Function to extract detailed information about factor graph components.
pub fn extract_factor_details(factor_graph: &FactorGraph) -> FactorDetails {
    let mut factor_details = FactorDetails::default();

    // Create a map of variable node indices to our variable indices
    let mut variable_index_map = HashMap::new();

    // Extract variable information
    for (i, (var_index, variable)) in factor_graph.variables().enumerate() {
        // Store the mapping from node index to our variable index
        variable_index_map.insert(var_index.0.index(), i);

        // Properly convert Vector<Float> to [f64; 4]
        let mean: [f64; 4] = variable
            .belief
            .mean
            .as_slice()
            .expect("Array is not contiguous")
            .try_into()
            .expect("Slice length mismatch");

        // Convert covariance matrix to array format [f64; 16]
        let covariance: [f64; 16] = variable
            .belief
            .covariance_matrix
            .as_slice()
            .expect("Array2 is not contiguous")
            .try_into()
            .expect("Matrix does not have exactly 16 elements");

        let node_index = variable.node_index().index();

        let estimated_position = variable.estimated_position();
        let estimated_velocity = variable.estimated_velocity();

        // Get the factorgraph_id
        let factorgraph_id = factor_graph.id().index() as u32;

        factor_details.variables.push(VariableInfo {
            index: node_index,
            factorgraph_id,
            mean,
            covariance,
            estimated_position,
            estimated_velocity,
        });
    }

    // Extract obstacle factor information
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(obstacle_factor) = factor.kind.try_as_obstacle_ref() {
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map
                        .get(&variable.node_index().index())
                        .copied()
                        .unwrap_or(0);

                    let last_measurement = obstacle_factor.last_measurement();

                    factor_details.obstacle_factors.push(ObstacleFactorInfo {
                        variable_index,
                        sdf_value: last_measurement.value,
                        position: [last_measurement.pos.x, last_measurement.pos.y],
                    });
                }
            }
        }
    }

    // Extract inter-robot factor information
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(interrobot_factor) = factor.kind.try_as_inter_robot_ref() {
            let factor_node = factor_graph.get_factor(FactorIndex(factor_index)).unwrap();

            let skip = interrobot_factor.skip(&factor_node.state);
            let dist_xy = interrobot_factor
                .diff_between_estimated_positions(&factor_node.state.linearisation_point);
            let dist = dist_xy.euclidean_norm();
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map
                        .get(&variable.node_index().index())
                        .copied()
                        .unwrap_or(0);

                    // Get the safety distance
                    let safety_distance = interrobot_factor.safety_distance() as f32;

                    // Get the external factor graph ID and convert to u32
                    let external_id: u32 =
                        interrobot_factor.external_variable.factorgraph_id.index();

                    // interrobot_factor.skip is a bool, but we are not sure when this is set, but
                    // for now we are using it.
                    factor_details
                        .interrobot_factors
                        .push(InterRobotFactorInfo {
                            variable_index,                              /* Index of the variable
                                                                          * in
                                                                          * our variables list */
                            external_robot_id: external_id as u32, // the same as robot id
                            external_factorgraph_id: external_id as u32, // the same as robot id
                            external_variable_index: interrobot_factor
                                .external_variable
                                .variable_index
                                .into(),
                            safety_distance, // Safety distance for factor to be active
                            distance_between_variables: dist, // Distance between the two variables
                            active: !skip,   // Factor is active if skip is false
                        });
                }
            }
        }
    }

    // Extract tracking factor information
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(tracking_factor) = factor.kind.try_as_tracking_ref() {
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map
                        .get(&variable.node_index().index())
                        .copied()
                        .unwrap_or(0);

                    let tracking = tracking_factor.tracking();
                    let last_measurement = tracking_factor.last_measurement();

                    // Convert tracking path to the expected format using the helper method
                    let tracking_path = tracking
                        .get_path()
                        .map(|path| path.iter().map(|v| [v.x, v.y]).collect())
                        .unwrap_or_default();

                    // In the measure method of TrackingFactor, x_to_projection_distance is
                    // calculated but not stored in LastMeasurement. We need to
                    // recalculate it here.
                    let x_pos = variable.estimated_position();
                    let projected_pos =
                        [last_measurement.pos.x as f64, last_measurement.pos.y as f64];
                    let dx = x_pos[0] - projected_pos[0];
                    let dy = x_pos[1] - projected_pos[1];
                    let distance_to_path = (dx * dx + dy * dy).sqrt();

                    factor_details.tracking_factors.push(TrackingFactorInfo {
                        variable_index,
                        tracking_path,
                        tracking_index: tracking.get_index(),
                        projected_position: [last_measurement.pos.x, last_measurement.pos.y],
                        path_deviation: last_measurement.value as f32,
                        distance_to_path,
                    });
                }
            }
        }
    }

    // Extract dynamic factor information
    // Dynamic factors connect two variables, so we need to find both
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(_dynamic_factor) = factor.kind.try_as_dynamic_ref() {
            // Find the variables this factor is connected to
            if let Some(neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                let variables: Vec<_> = neighbors.collect();

                if variables.len() == 2 {
                    // Find the indices of these variables in our variables list
                    let from_variable_index = variable_index_map
                        .get(&variables[0].node_index().index())
                        .copied()
                        .unwrap_or(0);
                    let to_variable_index = variable_index_map
                        .get(&variables[1].node_index().index())
                        .copied()
                        .unwrap_or(0);

                    // Use a default delta_t value
                    let delta_t = _dynamic_factor.delta_t as f32;

                    factor_details.dynamic_factors.push(DynamicFactorInfo {
                        from_variable_index,
                        to_variable_index,
                        delta_t,
                    });
                }
            }
        }
    }

    factor_details
}
