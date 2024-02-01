use crate::factorgraph::factor::Factor;
use crate::factorgraph::variable::Variable;

pub struct GbpSettings {
    pub beta: f64,
}
/// A factor graph is a bipartite graph representing the factorization of a function.
/// It is composed of two types of nodes: factors and variables.
#[derive(Debug)]
pub struct FactorGraph<F: Factor, V: Variable> {
    // pub struct FactorGraph {
    // factors: Vec<Box<dyn Factor>>,
    factors: Vec<F>,
    variables: Vec<V>,
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
        self.factors.push(factor);
    }

    pub fn add_variable(&mut self, variable: V) {
        self.variables.push(variable);
    }

    pub fn update_all_beliefs(&mut self) {
        // for variable in self.variables.iter_mut() {
        //     variable.update_belief();
        // }
        todo!()
        // for factor in self.factors.iter_mut() {
        //     factor.update_belief();
        // }
    }

    // linearize_all_factors
    fn compute_factors(&mut self) {
        for factor in self.factors.iter() {
            factor.compute();
        }
    }

    fn robustify_all_factors(&mut self) {
        for factor in self.factors.iter() {
            factor.robustify_loss();
        }
    }

    fn jit_linearisation(&mut self) {
        for factor in self.factors.iter() {
            match factor.measurement_model() {
                MeasurementModel::NonLinear => {
                    let adj_means = factor.adj_means();
                    // factors.iters_since_relin += 1
                    if ((adj_means - factor.linerisation_point()).norm() > self.gbp_settings.beta) {
                        factor.compute();
                    }
                }
            }
        }
    }
}
