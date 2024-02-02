use crate::factorgraph::factor::Factor;
use crate::factorgraph::variable::Variable;

use super::factor::MeasurementModel;
use super::{Dropout, UnitInterval};

use typed_builder::TypedBuilder;

type NodeId = usize;

#[derive(Debug, TypedBuilder)]
pub struct GbpSettings {
    /// Damping for the eta component of the message
    damping: f64,
    /// Absolute distance threshold between linearisation point and adjacent belief means for relinearisation
    pub beta: f64,
    /// Number of undamped iterations after relinearisation before
    pub number_of_undamped_iterations: usize,
    pub minimum_linear_iteration: usize,
    /// Chance for dropout to happen
    pub dropout: UnitInterval,
    pub reset_iterations_since_relinearisation: Vec<usize>,
}

impl Default for GbpSettings {
    fn default() -> Self {
        Self {
            damping: 0.0,
            beta: 0.1,
            number_of_undamped_iterations: 5,
            minimum_linear_iteration: 10,
            dropout: UnitInterval::new(0.0).unwrap(),
            reset_iterations_since_relinearisation: vec![],
        }
    }
}

impl GbpSettings {
    fn damping(&self, iterations_since_relinearisation: usize) -> f64 {
        if iterations_since_relinearisation > self.number_of_undamped_iterations {
            self.damping
        } else {
            0.0
        }
    }
}

// impl Default for GbpSettings {
//     fn default() -> Self {
//         Self { beta: , damping: , dropout: , num_undamped_iterations:  }
//     }
// }

#[derive(Debug)]
struct FactorNode<F: Factor> {
    pub id: NodeId,
    pub iterations_since_relinearisation: usize,
    factor: F,
}

#[derive(Debug)]
struct VariableNode<V: Variable> {
    pub id: NodeId,
    variable: V,
}

/// A factor graph is a bipartite graph representing the factorization of a function.
/// It is composed of two types of nodes: factors and variables.
#[derive(Debug)]
pub struct FactorGraph<F: Factor, V: Variable> {
    factors: Vec<FactorNode<F>>,
    variables: Vec<VariableNode<V>>,
    gbp_settings: GbpSettings,
}

// std::unique_ptr

impl<F: Factor, V: Variable> FactorGraph<F, V> {
    pub fn new(gbp_settings: Option<GbpSettings>) -> Self {
        Self {
            factors: Vec::new(),
            variables: Vec::new(),
            gbp_settings: gbp_settings.unwrap_or_default(),
        }
    }

    pub fn add_factor(&mut self, factor: F) {
        let id = self.factors.len();
        self.factors.push(FactorNode {
            id,
            iterations_since_relinearisation: 0,
            factor,
        });
    }

    pub fn add_variable(&mut self, variable: V) {
        let id = self.variables.len();
        self.variables.push(VariableNode { id, variable });
    }

    // linearize_all_factors
    fn compute_factors(&mut self) {
        for factor_node in self.factors.iter() {
            factor_node.factor.compute();
        }
    }

    fn jit_linearisation(&mut self) {
        for factor_node in self.factors.iter() {
            match factor_node.factor.measurement_model() {
                MeasurementModel::NonLinear => {
                    let adj_means = factor_node.factor.adj_means();
                    // factors.iters_since_relin += 1
                    if (adj_means - factor_node.factor.linerisation_point()).norm()
                        > self.gbp_settings.beta
                    {
                        factor_node.factor.compute();
                    }
                }
                MeasurementModel::Linear => {}
            }
        }
    }

    fn robustify_factors(&mut self) {
        for factor_node in self.factors.iter() {
            factor_node.factor.robustify_loss();
        }
    }

    fn synchronous_iteration(&mut self) {
        self.robustify_factors();
        self.jit_linearisation();
        self.compute_messages(Dropout(true));
        self.update_beliefs();
    }

    pub fn update_beliefs(&mut self) {
        // for variable in self.variables.iter_mut() {
        //     variable.update_belief();
        // }
        todo!()
        // for factor in self.factors.iter().iter_mut() {
        //     factor.update_belief();
        // }
    }

    fn compute_messages(&mut self, apply_dropout: Dropout) {
        for factor_node in self.factors.iter_mut() {
            if !apply_dropout.0 || rand::random::<f64>() > self.gbp_settings.dropout.into_inner() {
                // self.gpb_settings.get_damping(factor.iters_since_relin)
                let damping = self.gbp_settings.damping;
                factor_node.factor.compute_messages(damping);
            }
        }
    }
}
