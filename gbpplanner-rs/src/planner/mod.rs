mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod multivariate_normal;
mod robot;
mod spawner;
mod variable;

pub type Timestep = u32;

// pub trait Scalar: num_traits::Float + Copy + std::fmt::Debug {}
// impl Scalar for f32 {}
// impl Scalar for f64 {}

// only available on nightly :(
// pub type Vector<T> = ndarray::Array1<T: Scalar>;
// pub type Matrix<T> = ndarray::Array2<T: Scalar>;
pub type Vector<T> = ndarray::Array1<T>;
pub type Matrix<T> = ndarray::Array2<T>;

use self::robot::RobotPlugin;
use self::spawner::SpawnerPlugin;
use bevy::prelude::*;
// use ndarray_linalg::Lapack;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            (
                // RobotPlugin,
                SpawnerPlugin
            ),
        );
    }
}
