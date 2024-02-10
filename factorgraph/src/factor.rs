use nalgebra::{DMatrix, DVector};

use crate::{message::Payload, variable::Variable};

use std::rc::Rc;

#[derive(Debug)]
struct FactorState {
    measurement: DVector<f64>, // called z_ in gbpplanner
    measurement_precision: DMatrix<f64>,
    /// Strength of the factor
    strength: f64,
    /// Number of degrees of freedom e.g. 4 [x, y, x', y']
    dofs: usize,
}

#[derive(Debug)]
pub struct Factor {
    pub adjacent_variables: Vec<Rc<Variable>>,
    pub valid: bool,
    pub kind: FactorKind,
    pub state: FactorState,
}

impl Factor {

    // Main section: Factor update:
// Messages from connected variables are aggregated. The beliefs are used to create the linearisation point X_.
// The Factor potential is calculated using h_func_ and J_func_
// The factor precision and information is created, and then marginalised to create outgoing messages to its connected variables.

    /// 
    pub fn update() {
        todo!()
    }

    /// Marginalise the factor precision and information and create the outgoing message to the variable.
    pub fn marginalise_factor_distance() -> Payload {
        todo!()
    }
}

/// Interrobot factor: for avoidance of other robots
/// This factor results in a high energy or cost if two robots are planning to be in the same
/// position at the same timestep (collision). This factor is created between variables of two robots.
/// The factor has 0 energy if the variables are further away than the safety distance.
#[derive(Debug, Clone, Copy)]
pub struct InterRobotFactor {
    // TODO: constrain to be positive
    safety_distance: f64,
    ///
    skip: bool,
}

/// Dynamic factor: constant velocity model
#[derive(Debug)]
pub struct DynamicFactor {
    cached_jacobian: DMatrix<f64>,
    pub delta_t: f64, // defined at src/Robot.cpp:64
}

impl DynamicFactor {
    // TODO: not done
    pub fn new(state: &FactorState, delta_t: f64) -> Self  {
        // let (nrows, ncols) = (state.dofs / 2, state.dofs / 2);
        // let eye = DMatrix::identity(nrows, ncols);
        // let zeros = DMatrix::zeros(nrows, ncols);

        // let qc_inv = f64::powi(sigma, -2) * eye;

        // let qi_inv = DMatrix::new(state.dofs, state.dofs);

        // let cached_jacobian = DMatrix::new(state.dofs, state.dofs);
        // Self {
        //     cached_jacobian,
        //     delta_t,
        // }

        todo!()
    }
}

#[derive(Debug)]
pub struct DefaultFactor;

// TODO: obstacle factor, in gbpplanner uses a pointer to an image, which contains an SDF of the obstacles in the environment

#[derive(Debug)]
enum FactorKind {
    Default(DefaultFactor),
    InterRobot(InterRobotFactor),
    Dynamic(DynamicFactor),
}



trait Model {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        self.first_order_jacobian(state, x)
    }
    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the interrobot factor
    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DVector<f64>;
    fn first_order_jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        // Eigen::MatrixXd Factor::jacobianFirstOrder(const Eigen::VectorXd& X0){
        //     Eigen::MatrixXd h0 = h_func_(X0);    // Value at lin point
        //     Eigen::MatrixXd jac_out = Eigen::MatrixXd::Zero(h0.size(),X0.size());
        //     for (int i=0; i<X0.size(); i++){
        //         Eigen::VectorXd X_copy = X0;                                    // Copy of lin point
        //         X_copy(i) += delta_jac;                                         // Perturb by delta
        //         jac_out(Eigen::all, i) = (h_func_(X_copy) - h0) / delta_jac;    // Derivative (first order)
        //     }
        //     return jac_out;
        // };
        

        todo!()
        // let h0 = self.measurement(state, x);
        // DMatrix::from_fn
        // let columns: Vec<_> = (0..x.nrows())
        // .map(|i| {
        //     let mut x_copy = x.clone();
        //     x_copy[i] += self.jacobian_delta(); // Perturb by delta
        //     let column = (self.measurement(state, &x_copy) - h0) / self.jacobian_delta();
        //     column
        // }).collect();

        // // DMatrix::from_columns(h0.nrows(), x.nrows(), columns.as_slice())
        // nalgebra::

        //    todo!() 
        // DMatrix::from_columns(columns)

        // TODO: make sexier
        // let mut retv = DMatrix::zeros(h0.nrows(), x.nrows());
        // retv.set

        // for i in 0..x.nrows() {
        //     let mut x_copy = x.clone();
        //     // let mut x_copy = x.clone();
        //     x_copy[i] += self.jacobian_delta(); // Perturb by delta
        //     let column = (self.measurement(state, &x_copy) - h0) / self.jacobian_delta();
        //     // retv.view_mut((0, i), (retv.nrows(), i)).copy_from(&column);
        //     retv.set_column(i, &column);
        // }
        // retv
    }

    fn jacobian_delta(&self) -> f64;

    /// Whether to skip this factor in the update step
    /// In gbpplanner, this is only used for the interrobot factor.
    /// The other factors are always included in the update step.
    fn skip(&self) -> bool;
}

impl Model for DefaultFactor {
    /// Default jacobian is the first order taylor series jacobian
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        self.first_order_jacobian(state, x)
    }

    /// Default meaurement function is the identity function
    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DVector<f64> {
        x.clone()
    }

    fn skip(&self) -> bool {
        false
    }

    fn jacobian_delta(&self) -> f64 {
        1e-8
    }
}

impl Model for InterRobotFactor {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        let mut j = DMatrix::zeros(state.measurement.nrows(), state.dofs * 2);
        let x_diff = {
            let offset = state.dofs / 2;
            let mut x_diff = x.rows(0, offset) - x.rows(state.dofs, offset);
            x_diff += 1e-6 * DVector::from_element(offset, 1.0); // Add a tiny random offset to avoid div/0 errors
            x_diff
        };
        let radius = x_diff.norm();
        if radius <= self.safety_distance {
            j.view_mut((0, 0), (0, state.dofs / 2))
                .copy_from(&(-1.0 / self.safety_distance / radius * &x_diff));
            j.view_mut((0, state.dofs), (0, state.dofs + state.dofs / 2))
                .copy_from(&(1.0 / self.safety_distance / radius * &x_diff));
        }
        j
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DVector<f64> {
        // let mut h = DMatrix::zeros(state.measurement.nrows(), state.measurement.ncols());
        let mut h = DVector::zeros(state.measurement.nrows());
        let x_diff = {
            let mut x_diff = x.rows(0, state.dofs / 2) - x.rows(state.dofs, state.dofs / 2);
            // NOTE: In gbplanner, they weight this by the robot id, why they do this is unclear
            // as a robot id should be unique, and not have any semantics of distance/weight.
            x_diff += 1e-6 * DVector::from_element(state.dofs / 2, 1.0);
            x_diff
        };

        let radius = x_diff.norm();
        if radius <= self.safety_distance {
            self.skip = false;
            // gbpplanner: h(0) = 1.f*(1 - r/safety_distance_);
            // NOTE: in Eigen, indexing a matrix with a single index corresponds to indexing the matrix as a flattened array in column-major order.

            // h[(0, 0)] = 1.0 * (1.0 - radius / self.safety_distance);
            h[0] = 1.0 * (1.0 - radius / self.safety_distance);
        } else {
            self.skip = true;
        }

        h
    }

    fn jacobian_delta(&self) -> f64 {
        1e-2
    }

    fn skip(&self) -> bool {
        self.skip
    }
}

impl Model for DynamicFactor {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        // self.cache_jacobian
        self.cached_jacobian.clone()
        // todo!()
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DVector<f64> {
        &self.cached_jacobian * x
    }

    fn skip(&self) -> bool {
        false
    }

    fn jacobian_delta(&self) -> f64 {
        1e-2
    }
}

impl Model for FactorKind {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        match self {
            FactorKind::Default(f) => f.jacobian(state, x),
            FactorKind::InterRobot(f) => f.jacobian(state, x),
            FactorKind::Dynamic(f) => f.jacobian(state, x),
        }
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DVector<f64> {
        match self {
            FactorKind::Default(f) => f.measurement(state, x),
            FactorKind::InterRobot(f) => f.measurement(state, x),
            FactorKind::Dynamic(f) => f.measurement(state, x),
        }
    }

    fn skip(&self) -> bool {
        match self {
            FactorKind::Default(f) => f.skip(),
            FactorKind::InterRobot(f) => f.skip(),
            FactorKind::Dynamic(f) => f.skip(),
        }
    }

    fn jacobian_delta(&self) -> f64 {
        match self {
            FactorKind::Default(f) => f.jacobian_delta(),
            FactorKind::InterRobot(f) => f.jacobian_delta(),
            FactorKind::Dynamic(f) => f.jacobian_delta(),
        }
    }
}
