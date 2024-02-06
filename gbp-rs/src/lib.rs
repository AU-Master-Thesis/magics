pub mod factorgraph;
pub mod gaussian;

pub mod prelude {
    pub use crate::factorgraph::factor::Factor;
    pub use crate::factorgraph::factorgraph::FactorGraph;
    pub use crate::factorgraph::message::Message;
    pub use crate::factorgraph::variable::Variable;
    pub use crate::gaussian;
}
