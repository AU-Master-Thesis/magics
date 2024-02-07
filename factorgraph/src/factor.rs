
use std::rc::Rc;

use nalgebra::{DMatrix, DVector};

use crate::variable::Variable;


const JACOBIAN_DELTA: f64 = 1e-8;

struct FactorState {
    measurement: DVector<f64>,
    measurement_precision: DMatrix<f64>,
}



#[derive(Debug)]
pub struct Factor {

    pub adjacent_variables: Vec<Rc<Variable>>,
    pub valid: bool,
    pub kind: FactorKind,
    pub state: FactorState,
}


// struct 


#[derive(Debug, Clone, Copy)]
pub struct InterRobotFactor {
    safety_distance: f64
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
    fn measurement(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64>;
    fn first_order_jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        let h0 = self.measurement(state, x);
        // DMatrix::from_fn

        // TODO: make sexier
        let mut retv = DMatrix::zeros(h0.nrows(), x.nrows());
        
        for i in 0..x.nrows() {
            let mut x_copy = x.clone();
            x_copy[i] += JACOBIAN_DELTA;
            let column = (self.measurement(state, &x_copy) - h0) / JACOBIAN_DELTA;
            retv.set_column(i, &column);
        }
        retv
    }

}


impl Model for DefaultFactor {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        self.first_order_jacobian(state, x)
    }

    /// Default meaurement function is the identity function
    fn measurement(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        x
    }
}

impl Model for InterRobotFactor {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        todo!()
    }

    fn measurement(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        todo!()
    }
}

impl Model for DynamicFactor {
    fn jacobian(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        todo!()
    }

    fn measurement(&self, state: &FactorState, x: &DVector<f64>) -> DMatrix<f64> {
        todo!()
    }
}
