use std::ops::AddAssign;

use crate::factorgraph::factor::Factor;
use crate::factorgraph::variable::Variable;

use super::factor::MeasurementModel;
use super::{Dropout, UnitInterval};
use crate::gaussian::Gaussian;

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

#[derive(Debug)]
pub struct SolveSettings {
    iterations: usize,
    convergance_threshold: f64,
    include_priors: super::Include,
    log: bool,
}

impl Default for SolveSettings {
    fn default() -> Self {
        Self {
            iterations: 20,
            convergance_threshold: 1e-6,
            include_priors: super::Include(true),
            log: true,
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
    adjacent_variables: Vec<usize>,
}

#[derive(Debug)]
struct VariableNode<V: Variable> {
    pub id: NodeId,
    pub dofs: usize,
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

    pub fn add_variable(&mut self, variable: V, dofs: usize) {
        // TODO: maybe move variable initialisation inside this function
        let id = self.variables.len();
        self.variables.push(VariableNode { id, dofs, variable });
    }

    pub fn add_factor(&mut self, factor: F, adjacent_variables: Vec<usize>) {
        // TODO: maybe move adjacent variable node sorting into here
        let id = self.factors.len();
        self.factors.push(FactorNode {
            id,
            iterations_since_relinearisation: 0,
            factor,
            adjacent_variables,
        });
    }

    pub fn update_beliefs(&mut self) {
        for variable_node in self.variables.iter_mut() {
            variable_node.variable.update_belief();
        }
    }

    fn compute_messages(&mut self, apply_dropout: Dropout) {
        for factor_node in self.factors.iter_mut() {
            if !apply_dropout.0 || rand::random::<f64>() > self.gbp_settings.dropout.into_inner() {
                let damping = self
                    .gbp_settings
                    .damping(factor_node.iterations_since_relinearisation);
                factor_node.factor.compute_messages(damping);
            }
        }
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

    fn solve(&mut self, settings: SolveSettings) {
        let mut energy_log: [f64; 2] = [0.0, 0.0];
        let mut count = 0;

        for i in 0..settings.iterations {
            self.synchronous_iteration();

            if self
                .gbp_settings
                .reset_iterations_since_relinearisation
                .contains(&i)
            {
                for factor_node in self.factors.iter_mut() {
                    factor_node.iterations_since_relinearisation = 1;
                }
            }

            energy_log[0] = energy_log[1];
            energy_log[1] = self.energy(settings.include_priors);
            // energy_log[1] = self.energy();

            if settings.log {
                println!("Iterations: {}\tEnergy: {:.3}", i + 1, energy_log[0]);
            }

            if f64::abs(energy_log[0] - energy_log[1]) < settings.convergance_threshold {
                count += 1;
                if count >= 3 {
                    return;
                }
            } else {
                count = 0;
            }
        }
    }

    /// Computes the sum of all of the squared errors in the graph using the appropriate local loss function
    fn energy(&self, include_priors: super::Include) -> f64 {
        let factor_energy = self
            .factors
            .iter()
            .fold(0.0, |acc, factor_node| acc + factor_node.factor.energy());

        let prior_energy = if include_priors.0 {
            self.variables.iter().fold(0.0, |acc, variable_node| {
                acc + variable_node.variable.prior_energy()
            })
        } else {
            0.0
        };

        factor_energy + prior_energy
    }

    fn get_joint_dim(&self) -> usize {
        self.variables
            .iter()
            .map(|variable_node| variable_node.dofs)
            .sum()
    }

    /// Get the joint distribution over all variables in the information form.
    /// If non-linear factors exist, it is taken at the linearisation point.
    fn joint_distribution(&self) -> Gaussian {
        let dim = self.get_joint_dim();
        let mut joint = Gaussian::new(dim, None, None);

        // Priors
        let mut var_ix = vec![0; self.variables.len()];
        let mut counter = 0;

        for variable_node in self.variables.iter() {
            let variable = &variable_node.variable;

            var_ix[variable_node.id] = counter;

            joint
                .information_vector
                .rows_mut(counter, variable_node.dofs)
                .add_assign(
                    &variable
                        .get_prior()
                        .information_vector
                        .rows(counter, variable_node.dofs),
                );
            joint
                .precision_matrix
                .view_mut(
                    (counter, counter + variable_node.dofs),
                    (counter, counter + variable_node.dofs),
                )
                .add_assign(
                    &variable
                        .get_prior()
                        .precision_matrix
                        .view((counter, variable_node.dofs), (counter, variable_node.dofs)),
                );
            counter += variable_node.dofs;
        }

        // Other factors
        for factor_node in self.factors.iter() {
            let mut fact_ix = 0;
            for &adjacent_variable_node_id in factor_node.adjacent_variables.iter() {
                let adjacent_variable_node = &self.variables[adjacent_variable_node_id];

                // Diagonal contribution of factor
                joint
                    .information_vector
                    .rows_mut(
                        var_ix[adjacent_variable_node_id],
                        adjacent_variable_node.dofs,
                    )
                    .add_assign(
                        factor_node
                            .factor
                            .get_gaussian()
                            .information_vector
                            .rows(fact_ix, adjacent_variable_node.dofs),
                    );

                // joint.lam[var_ix[vID]:var_ix[vID] + adj_var_node.dofs, var_ix[vID]:var_ix[vID] + adj_var_node.dofs] += \
                // factor.factor.lam[factor_ix:factor_ix + adj_var_node.dofs, factor_ix:factor_ix + adj_var_node.dofs]
                joint
                    .precision_matrix
                    .view_mut(
                        (
                            var_ix[adjacent_variable_node_id],
                            adjacent_variable_node.dofs,
                        ),
                        (
                            var_ix[adjacent_variable_node_id],
                            adjacent_variable_node.dofs,
                        ),
                    )
                    .add_assign(factor_node.factor.get_gaussian().precision_matrix.view(
                        (fact_ix, adjacent_variable_node.dofs),
                        (fact_ix, adjacent_variable_node.dofs),
                    ));

                let mut other_fact_ix = 0;
                for &other_adjacent_variable_node_id in factor_node.adjacent_variables.iter() {
                    if &other_adjacent_variable_node_id > &adjacent_variable_node_id {
                        let other_adjacent_variable_node =
                            &self.variables[other_adjacent_variable_node_id];

                        // off diagonal contributions of factor
                        // joint.lam[var_ix[vID]:var_ix[vID] + adj_var_node.dofs, var_ix[other_vID]:var_ix[other_vID] + other_adj_var_node.dofs] += \
                        //     factor.factor.lam[factor_ix:factor_ix + adj_var_node.dofs, other_factor_ix:other_factor_ix + other_adj_var_node.dofs]
                        joint
                            .precision_matrix
                            .view_mut(
                                (
                                    var_ix[adjacent_variable_node_id],
                                    adjacent_variable_node.dofs,
                                ),
                                (
                                    var_ix[adjacent_variable_node_id],
                                    adjacent_variable_node.dofs,
                                ),
                            )
                            .add_assign(factor_node.factor.get_gaussian().precision_matrix.view(
                                (fact_ix, adjacent_variable_node.dofs),
                                (other_fact_ix, adjacent_variable_node.dofs),
                            ));
                        // joint.lam[var_ix[other_vID]:var_ix[other_vID] + other_adj_var_node.dofs, var_ix[vID]:var_ix[vID] + adj_var_node.dofs] += \
                        //     factor.factor.lam[other_factor_ix:other_factor_ix + other_adj_var_node.dofs, factor_ix:factor_ix + adj_var_node.dofs]
                        joint
                            .precision_matrix
                            .view_mut(
                                (
                                    var_ix[other_adjacent_variable_node_id],
                                    other_adjacent_variable_node.dofs,
                                ),
                                (
                                    var_ix[adjacent_variable_node_id],
                                    adjacent_variable_node.dofs,
                                ),
                            )
                            .add_assign(factor_node.factor.get_gaussian().precision_matrix.view(
                                (other_fact_ix, other_adjacent_variable_node.dofs),
                                (fact_ix, adjacent_variable_node.dofs),
                            ));
                        other_fact_ix += other_adjacent_variable_node.dofs;
                    }

                    fact_ix += adjacent_variable_node.dofs;
                }
            }
        }

        joint
    }
}
