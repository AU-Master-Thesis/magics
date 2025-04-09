use bevy::prelude::*;
use std::num::NonZeroUsize;

use crate::simulation_loader::{LoadSimulation, ReloadSimulation};

#[derive(Resource)]
pub struct RobotNumberGenerator(usize); // Made pub

impl Default for RobotNumberGenerator {
    fn default() -> Self {
        Self(1)
    }
}

impl RobotNumberGenerator {
    pub fn next(&mut self) -> NonZeroUsize {
        let next = self.0;
        self.0 += 1;
        next.try_into().unwrap()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub fn reset_robot_number_generator(mut robot_number_generator: ResMut<RobotNumberGenerator>) {
    robot_number_generator.reset();
}

// Placeholder for CreateVariableTimesteps trait and impls if needed later
// trait CreateVariableTimesteps {
//     fn create_variable_timesteps(n: NonZeroUsize) -> Vec<u32>;
// }
//
// struct EvenlySpacedVariableTimesteps;
//
// struct GbpplannerVariableTimesteps;
//
// impl CreateVariableTimesteps for GbpplannerVariableTimesteps {
//     fn create_variable_timesteps(n: NonZeroUsize) -> Vec<u32> {
//         todo!()
//         // get_variable_timesteps()
//     }
// }
