use gbp_linalg::{Float, Matrix, Vector};

#[allow(clippy::len_without_is_empty)]
#[derive(Debug, Clone)]
pub struct DummyNormal {
    pub information: Vector<Float>,
    pub precision: Matrix<Float>,
    pub mean: Vector<Float>,
}

impl DummyNormal {}
