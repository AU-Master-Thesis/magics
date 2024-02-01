
use crate::factorgraph::factor::Factor;
use crate::factorgraph::variable::Variable;

/// A factor graph is a bipartite graph representing the factorization of a function.
/// It is composed of two types of nodes: factors and variables.
/// 
pub struct FactorGraph {
    factors: Vec<Box<dyn Factor>>,
    variables: Vec<Variable>,
}


impl FactorGraph {
    pub fn new() -> Self {
        Self {
            factors: Vec::new(),
            variables: Vec::new(),
        }
    }

    pub fn add_factor(&mut self, factor: Box<dyn Factor>) {
        self.factors.push(factor);
    }

    pub fn add_variable(&mut self, variable: Variable) {
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
}
