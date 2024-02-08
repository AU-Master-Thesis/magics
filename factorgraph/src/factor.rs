use std::rc::Rc;

use nalgebra::{DMatrix, DVector};

use crate::variable::Variable;

struct FactorState {
    measurement: DVector<f64>, // called z_ in gbpplanner
    measurement_precision: DMatrix<f64>,
    /// Number of degrees of freedom
    dofs: usize,
}

#[derive(Debug)]
pub struct Factor {
    pub adjacent_variables: Vec<Rc<Variable>>,
    pub valid: bool,
    pub kind: FactorKind,
    pub state: FactorState,
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

#[derive(Debug)]
pub struct DynamicFactor;

#[derive(Debug)]
pub struct DefaultFactor;

// TODO: obstacle factor, in gbpplanner uses a pointer to an image, which contains an SDF of the obstacles in the environment

enum FactorKind {
    Default(DefaultFactor),
    InterRobot(InterRobotFactor),
    Dynamic(DynamicFactor),
}

trait Model {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64>;
    /// Measurement function
    /// **Note**: This method takes a mutable reference to self, because the interrobot factor 
    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64>;
    fn first_order_jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        let h0 = self.measurement(state, x);
        // DMatrix::from_fn

        // TODO: make sexier
        let mut retv = DMatrix::zeros(h0.nrows(), x.nrows());

        for i in 0..x.nrows() {
            let mut x_copy = x.clone();
            x_copy[i] += self.jacobian_delta();
            let column = (self.measurement(state, &x_copy) - h0) / self.jacobian_delta();
            retv.set_column(i, &column);
        }
        retv
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
    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        x
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
            j.view_mut((0, 0), (0, state.dofs / 2)).copy_from(&(-1.0 / self.safety_distance / radius * x_diff));
            j.view_mut((0, state.dofs), (0, state.dofs + state.dofs / 2)).copy_from(&(1.0 / self.safety_distance / radius * x_diff));
        }
        j
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        let mut h = DMatrix::zeros(state.measurement.nrows(), state.measurement.ncols());
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
            h[(0, 0)] = 1.0 * (1.0 - radius / self.safety_distance);
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
        
    }

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        todo!()
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

    fn measurement(&mut self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
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
