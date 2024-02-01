use crate::factorgraph::factor::Factor;
use crate::factorgraph::variable::Variable;

use super::factor::MeasurementModel;
use super::{Dropout, UnitInterval};

type NodeId = usize;

#[derive(Debug)]
pub struct GbpSettings {
    /// Absolute distance threshold between linearisation point and adjacent belief means for relinearisation
    pub beta: f64,
    /// Damping for the eta component of the message
    pub damping: f64,
    pub dropout: UnitInterval,
    /// Number of undamped iterations after relinearisation before
    pub num_undamped_iterations: usize,
}

#[derive(Debug)]
struct FactorNode<F: Factor> {
    pub id: NodeId,
    pub iterations_since_relinerisation: usize,
    factor: F,
}

#[derive(Debug)]
struct VariableNode<V: Variable> {
    pub id: NodeId,
    variable: V,
}
// struct FactorWrapper {
//     id
//     number_since_re
//     factor: Factor
// }

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
    pub fn new(gbp_settings: GbpSettings) -> Self {
        Self {
            factors: Vec::new(),
            variables: Vec::new(),
            gbp_settings,
        }
    }

    pub fn add_factor(&mut self, factor: F) {
        let id = self.factors.len();
        self.factors.push(FactorNode {
            id,
            iterations_since_relinerisation: 0,
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
