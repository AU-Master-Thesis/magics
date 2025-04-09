//! Module for handling weight updates in the API.

use bevy::prelude::*;
use crate::factorgraph::factorgraph::FactorGraph;
use gbp_config::Config;
use super::state::{ApiState, BatchWeightUpdate};

/// System that applies weight updates from the API.
pub fn apply_weight_updates(
    api_state: Res<ApiState>,
    mut robots: Query<(Entity, &mut FactorGraph)>,
    mut config: ResMut<Config>,
) {
    // Only apply updates if API is active
    if !api_state.is_active() {
        return;
    }

    let mut individual_updates = Vec::new();
    let mut batch_updates = Vec::new();

    // Get all pending individual weight updates
    if let Ok(mut weight_requests) = api_state.weight_requests.write() {
        individual_updates.append(&mut weight_requests);
    }

    // Get all pending batch weight updates
    if let Ok(mut batch_requests) = api_state.batch_weight_requests.write() {
        batch_updates.append(&mut batch_requests);
    }

    // If there are no updates, nothing to do
    if individual_updates.is_empty() && batch_updates.is_empty() {
        return;
    }

    // Process individual updates first
    for update in individual_updates {
        match update.agent_id {
            // System-wide update
            None => {
                info!("Applying system-wide weight update: dynamic={}, obstacle={}, interrobot={}, tracking={}", 
                      update.weights.dynamic, update.weights.obstacle, 
                      update.weights.interrobot, update.weights.tracking);
                
                // Update config
                config.gbp.sigma_factor_dynamics = update.weights.dynamic;
                config.gbp.sigma_factor_obstacle = update.weights.obstacle;
                config.gbp.sigma_factor_interrobot = update.weights.interrobot;
                config.gbp.sigma_factor_tracking = update.weights.tracking;

                // Update all factor graphs
                for (entity, mut factor_graph) in robots.iter_mut() {
                    // Update factor enabled settings
                    let settings = config.gbp.factors_enabled;
                    factor_graph.change_factor_enabled(settings);
                    
                    // BUG FIX: Also update the weights for each factor graph
                    factor_graph.update_factor_weights(update.weights);
                    
                    debug!("Updated weights for robot {:?}: {:?}", entity, factor_graph.factor_weights());
                }
            }
            // Agent-specific update
            Some(agent_id) => {
                debug!("Applying agent-specific weight update for agent {:?}: dynamic={}, obstacle={}, interrobot={}, tracking={}", 
                      agent_id, update.weights.dynamic, update.weights.obstacle, 
                      update.weights.interrobot, update.weights.tracking);
                
                // Find the robot with the matching agent_id
                let mut found = false;
                for (entity, mut factor_graph) in robots.iter_mut() {
                    if entity.index() == agent_id {
                        // Update only this agent's factor graph weights
                        factor_graph.update_factor_weights(update.weights);
                        debug!("Updated weights for agent {:?}: {:?}", entity, factor_graph.factor_weights());
                        found = true;
                        break;
                    }
                }
                
                if !found {
                    // Collect all available agent IDs for debugging
                    let available_ids: Vec<u32> = robots.iter().map(|(e, _)| e.index()).collect();
                    error!("Agent {} not found for weight update. Available agent IDs: {:?}", agent_id, available_ids);
                }
            }
        }
    }

    // Process batch updates
    for batch_update in batch_updates {
        process_batch_weight_update(batch_update, &mut robots, &mut config);
    }

    // Mark weight updates as applied
    api_state.mark_weight_updates_applied();
    info!("All weight updates applied successfully");
}

/// Process a batch weight update
fn process_batch_weight_update(
    batch_update: BatchWeightUpdate,
    robots: &mut Query<(Entity, &mut FactorGraph)>,
    config: &mut ResMut<Config>,
) {
    info!("Processing batch weight update for {} agents", batch_update.agent_weights.len());
    
    // Create a set of agent IDs that have specific weights
    let agent_ids_with_specific_weights: std::collections::HashSet<u32> = 
        batch_update.agent_weights.keys().cloned().collect();
    
    // Apply agent-specific weights
    for (agent_id, weights) in &batch_update.agent_weights {
        let mut found = false;
        for (entity, mut factor_graph) in robots.iter_mut() {
            if entity.index() == *agent_id {
                factor_graph.update_factor_weights(*weights);
                info!("Updated weights for agent {:?}: {:?}", entity, factor_graph.factor_weights());
                found = true;
                break;
            }
        }
        
        if !found {
            // Collect all available agent IDs for debugging
            let available_ids: Vec<u32> = robots.iter().map(|(e, _)| e.index()).collect();
            error!("Agent {} not found for batch weight update. Available agent IDs: {:?}", agent_id, available_ids);
        }
    }
    
    // Apply default weights if provided
    if let Some(default_weights) = batch_update.default_weights {
        info!("Applying default weights from batch update: dynamic={}, obstacle={}, interrobot={}, tracking={}", 
              default_weights.dynamic, default_weights.obstacle, 
              default_weights.interrobot, default_weights.tracking);
        
        // Update config
        config.gbp.sigma_factor_dynamics = default_weights.dynamic;
        config.gbp.sigma_factor_obstacle = default_weights.obstacle;
        config.gbp.sigma_factor_interrobot = default_weights.interrobot;
        config.gbp.sigma_factor_tracking = default_weights.tracking;
        
        // Apply to all agents not explicitly specified in agent_weights
        for (entity, mut factor_graph) in robots.iter_mut() {
            if !agent_ids_with_specific_weights.contains(&entity.index()) {
                // Update factor enabled settings
                let settings = config.gbp.factors_enabled;
                factor_graph.change_factor_enabled(settings);
                
                // Update weights
                factor_graph.update_factor_weights(default_weights);
                debug!("Updated weights for robot {:?} with default weights: {:?}", entity, factor_graph.factor_weights());
            }
        }
    }
}
