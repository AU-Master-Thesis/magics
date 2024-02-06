use crate::gaussian::MultivariateNormal;

use super::{measurement_model::MeasurementModel, message::Message};

// class MeasModel:
// def __init__(self, meas_fn: Callable, jac_fn: Callable, loss: SquaredLoss, *args) -> None:
// self._meas_fn = meas_fn
// self._jac_fn = jac_fn
// self.loss = loss
// self.args = args
// self.linear = True
//
// def jac_fn(self, x: torch.Tensor) -> torch.Tensor:
// return self._jac_fn(x, *self.args)
//
// def meas_fn(self, x: torch.Tensor) -> torch.Tensor:
// return self._meas_fn(x, *self.args)

// #[derive(Debug)]
// pub enum MeasurementModel {
//     Linear,
//     NonLinear,
// }

pub trait Factor {
    fn compute_messages(&mut self, damping: f64) -> Vec<Message>;
    fn energy(&self) -> f64;
    fn residual(&self) -> nalgebra::DVector<f64>;
    fn adj_means(&self) -> nalgebra::DVector<f64>;
    fn compute(&self) -> f64;
    fn robustify_loss(&self);
    fn measurement_model(&self) -> MeasurementModel;
    fn linerisation_point(&self) -> nalgebra::DVector<f64>;
    fn get_gaussian(&self) -> &MultivariateNormal;
}

// [1,,3].iter().

// #[derive(Debug)]
// struct DynamicFactor {
//     id: usize,
//     // messages:
// }

// impl Factor for DynamicFactor {
//     // type ComputedValue = f64;

//     fn get_id(&self) -> usize {
//         self.id
//         // get_id_from_db()
//     }

//     fn compute_messages(&self) -> Vec<Message> {
//         // Implementation goes here
//         Vec::new()
//     }

//     fn compute(&self) -> Self::ComputedValue {
//         // Implementation goes here
//     }
// }
// let f = DynamicFactor {id : 2};

// #[derive(Debug)]
// struct InterRobotFactor;
// #[derive(Debug)]
// struct ObstacleFactor;

// impl Factor for DefaultFactor {}
// impl Factor for DynamicFactor {}
// impl Factor for InterRobotFactor {}
// impl Factor for ObstacleFactor {}

// #[derive(Debug)]
// enum FactorType {
//     Default(DefaultFactor),
//     Dynamic(DynamicFactor),
//     InterRobot(InterRobotFactor),
//     Ocstacle(ObstacleFactor),
// }
