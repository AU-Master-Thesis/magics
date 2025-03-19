use gbp_linalg::prelude::*;
use ndarray::prelude::*;
use ndarray_inverse::Inverse;

use crate::factorgraph::{
    message::{InformationVec, Mean, PrecisionMatrix},
    prelude::Message,
    DOFS,
};

/// Utility function to create `start..start + n`
/// Similar to `Eigen::seqN`
#[inline]
const fn seq_n(start: usize, n: usize) -> std::ops::Range<usize> {
    start..start + n
}

type Aa<'a, T> = MatrixView<'a, T>;
type Ab<'a, T> = MatrixView<'a, T>;
type Ba<'a, T> = MatrixView<'a, T>;
type Bb<'a, T> = MatrixView<'a, T>;

fn extract_submatrices_from_precision_matrix<T: GbpFloat>(
    precision_matrix: &Matrix<T>,
    marg_idx: usize,
) -> (Aa<T>, Ab<T>, Ba<T>, Bb<T>) {
    debug_assert!(precision_matrix.is_square());
    debug_assert_eq!(precision_matrix.nrows() % DOFS, 0);
    debug_assert_eq!(precision_matrix.ncols() % DOFS, 0);

    let aa = precision_matrix.slice(s![seq_n(marg_idx, DOFS), seq_n(marg_idx, DOFS)]);

    let ab = if marg_idx == 0 {
        precision_matrix.slice(s![seq_n(marg_idx, DOFS), marg_idx + DOFS..])
    } else {
        precision_matrix.slice(s![seq_n(marg_idx, DOFS), ..marg_idx])
    };

    let ba = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + DOFS.., seq_n(marg_idx, DOFS)])
    } else {
        precision_matrix.slice(s![..marg_idx, seq_n(marg_idx, DOFS)])
    };

    let bb = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + DOFS.., marg_idx + DOFS..])
    } else {
        precision_matrix.slice(s![..marg_idx, ..marg_idx])
    };

    (aa, ab, ba, bb)
}

/// Marginalise a factor's precision matrix and information vector to get a message
/// 
/// This is a key part of the GBP algorithm that implements the marginalization
/// operation required for message passing.
#[allow(clippy::similar_names)]
pub fn marginalise_factor_distance(
    information_vector: Vector<Float>,
    precision_matrix: Matrix<Float>,
    marg_idx: usize,
) -> Message {
    debug_assert_eq!(information_vector.len(), precision_matrix.nrows());
    debug_assert_eq!(precision_matrix.nrows(), precision_matrix.ncols());

    let factor_only_connected_to_one_variable = information_vector.len() == DOFS;
    if factor_only_connected_to_one_variable {
        let mean = Vector::<Float>::zeros(information_vector.len());

        return Message::new(
            InformationVec(information_vector.to_owned()),
            PrecisionMatrix(precision_matrix.to_owned()),
            Mean(mean),
        );
    }

    let lam_bb = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + DOFS.., marg_idx + DOFS..])
    } else {
        precision_matrix.slice(s![..marg_idx, ..marg_idx])
    };
    let Some(lam_bb_inv) = lam_bb.to_owned().inv() else {
        return Message::empty();
    };

    let lam_aa = precision_matrix.slice(s![seq_n(marg_idx, DOFS), seq_n(marg_idx, DOFS)]);

    let lam_ab = if marg_idx == 0 {
        precision_matrix.slice(s![seq_n(marg_idx, DOFS), marg_idx + DOFS..])
    } else {
        precision_matrix.slice(s![seq_n(marg_idx, DOFS), ..marg_idx])
    };

    let lam_ba = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + DOFS.., seq_n(marg_idx, DOFS)])
    } else {
        precision_matrix.slice(s![..marg_idx, seq_n(marg_idx, DOFS)])
    };

    let eta_a = information_vector.slice(s![seq_n(marg_idx, DOFS)]);
    debug_assert_eq!(eta_a.len(), DOFS);

    let eta_b = if marg_idx == 0 {
        information_vector.slice(s![DOFS..])
    } else {
        information_vector.slice(s![..marg_idx])
    };
    debug_assert_eq!(eta_b.len(), information_vector.len() - DOFS);

    let information_vector = &eta_a - &lam_ab.dot(&lam_bb_inv).dot(&eta_b);
    let precision_matrix = &lam_aa - &lam_ab.dot(&lam_bb_inv).dot(&lam_ba);

    if precision_matrix.iter().any(|elem| elem.is_infinite()) {
        Message::empty()
    } else {
        let mean = Vector::<Float>::zeros(information_vector.len());
        Message::new(
            InformationVec(information_vector.to_owned()),
            PrecisionMatrix(precision_matrix.to_owned()),
            Mean(mean),
        )
    }
}

#[cfg(test)]
mod tests {
    use ndarray::concatenate;
    use pretty_assertions::assert_eq;

    use super::*;

    macro_rules! generate_8x8_precision_matrix {
        () => {{
            let upper_left = array![[1., 2., 3., 4.], [5., 6., 7., 8.], [9., 10., 11., 12.], [
                13., 14., 15., 16.
            ]];

            let upper_right = array![
                [17., 18., 19., 20.],
                [21., 22., 23., 24.],
                [25., 26., 27., 28.],
                [29., 30., 31., 32.]
            ];

            let lower_left = array![
                [33., 34., 35., 36.],
                [37., 38., 39., 40.],
                [41., 42., 43., 44.],
                [45., 46., 47., 48.]
            ];

            let lower_right = array![
                [49., 50., 51., 52.],
                [53., 54., 55., 56.],
                [57., 58., 59., 60.],
                [61., 62., 63., 64.]
            ];

            let precision_matrix = concatenate![
                Axis(0),
                concatenate![Axis(1), upper_left, upper_right],
                concatenate![Axis(1), lower_left, lower_right]
            ];
            (
                precision_matrix,
                upper_left,
                upper_right,
                lower_left,
                lower_right,
            )
        }};
    }

    #[test]
    fn extract_submatrices_from_precision_matrix_with_marg_idx0_dofs4() {
        let (precision_matrix, upper_left, upper_right, lower_left, lower_right) =
            generate_8x8_precision_matrix!();

        assert!(precision_matrix.is_square());

        let (aa, ab, ba, bb) = extract_submatrices_from_precision_matrix(&precision_matrix, 0);

        assert_eq!(aa, upper_left);
        assert_eq!(ab, upper_right);
        assert_eq!(ba, lower_left);
        assert_eq!(bb, lower_right);
    }

    #[test]
    fn extract_submatrices_from_precision_matrix_with_marg_idx4_dofs4() {
        let (precision_matrix, upper_left, upper_right, lower_left, lower_right) =
            generate_8x8_precision_matrix!();

        assert!(precision_matrix.is_square());

        let (aa, ab, ba, bb) = extract_submatrices_from_precision_matrix(&precision_matrix, 4);

        assert_eq!(aa, lower_right);
        assert_eq!(ab, lower_left);
        assert_eq!(ba, upper_right);
        assert_eq!(bb, upper_left);
    }

    #[test]
    fn information_vector_length_equal_to_ndofs_do_nothing() {
        #![allow(clippy::unwrap_used)]
        let information_vector: Vector<Float> = array![0., 1., 2., 3.];
        let precision_matrix: Matrix<Float> =
            array![[5., 0.2, 0., 0.], [0.2, 5., 0., 0.], [0., 0.0, 5., 0.3], [
                0., 0., 0.3, 5.
            ]];

        let marginalisation_idx = 0;

        let mut marginalised_msg = marginalise_factor_distance(
            information_vector.clone(),
            precision_matrix.clone(),
            marginalisation_idx,
        );

        let payload = marginalised_msg.take().unwrap();

        assert_eq!(payload.information_vector, information_vector);
        assert_eq!(payload.precision_matrix, precision_matrix);
    }
}
