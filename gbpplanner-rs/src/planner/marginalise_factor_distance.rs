use gbp_multivariate_normal::MultivariateNormal;
use ndarray::prelude::*;
use ndarray_inverse::Inverse;

use gbp_linalg::{prelude::*, pretty_print_matrix, pretty_print_vector};

use crate::planner::message::{Eta, Lam, Mu};

use super::message::Message;

/// Utility function to create `start..start + n`
/// Similar to `Eigen::seqN`
const fn seq_n(start: usize, n: usize) -> std::ops::Range<usize> {
    start..start + n
}

type Aa<'a, T> = MatrixView<'a, T>;
type Ab<'a, T> = MatrixView<'a, T>;
type Ba<'a, T> = MatrixView<'a, T>;
type Bb<'a, T> = MatrixView<'a, T>;

fn extract_submatrices_from_precision_matrix<T: GbpFloat>(
    precision_matrix: &Matrix<T>,
    dofs: usize,
    marg_idx: usize,
) -> (Aa<T>, Ab<T>, Ba<T>, Bb<T>) {
    debug_assert!(precision_matrix.is_square());
    debug_assert_eq!(precision_matrix.nrows() % dofs, 0);
    debug_assert_eq!(precision_matrix.ncols() % dofs, 0);

    let aa = precision_matrix.slice(s![seq_n(marg_idx, dofs), seq_n(marg_idx, dofs)]);

    let ab = if marg_idx == 0 {
        precision_matrix.slice(s![seq_n(marg_idx, dofs), marg_idx + dofs..])
    } else {
        precision_matrix.slice(s![seq_n(marg_idx, dofs), ..marg_idx])
    };

    let ba = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + dofs.., seq_n(marg_idx, dofs)])
    } else {
        precision_matrix.slice(s![..marg_idx, seq_n(marg_idx, dofs)])
    };

    let bb = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + dofs.., marg_idx + dofs..])
    } else {
        precision_matrix.slice(s![..marg_idx, ..marg_idx])
    };

    (aa, ab, ba, bb)
}

pub fn marginalise_factor_distance(
    information_vector: Vector<Float>,
    precision_matrix: Matrix<Float>,
    variable_dofs: usize,
    marginalisation_idx: usize,
) -> Result<Message, &'static str> {
    let ndofs = variable_dofs;
    let marg_idx = marginalisation_idx;

    debug_assert_eq!(information_vector.len(), precision_matrix.nrows());
    debug_assert_eq!(precision_matrix.nrows(), precision_matrix.ncols());
    // pretty_print_vector!(&information_vector);
    // pretty_print_matrix!(&precision_matrix);

    let factor_only_connected_to_one_variable = information_vector.len() == variable_dofs;
    if factor_only_connected_to_one_variable {
        let mu = Vector::<Float>::zeros(information_vector.len());
        return Ok(Message::new(
            Eta(information_vector),
            Lam(precision_matrix),
            Mu(mu),
        ));
        // dbg!(&information_vector);
        // dbg!(&precision_matrix);
        // TODO: return None
        // let mvn = MultivariateNormal::from_information_and_precision(
        //     information_vector,
        //     precision_matrix,
        // )
        // .inspect_err(|_| {
        //     // pretty_print_matrix!(&precision_matrix);
        // })
        // .expect(
        //     "the given information vector and precision matrix is a valid multivariate gaussian",
        // );
        // return Ok(Message::new(mvn));
    }

    // eprintln!(
    //     "information_vector shape = {:?}, ndofs = {:?}",
    //     information_vector.shape(),
    //     variable_dofs
    // );
    // eprintln!("precision_matrix shape = {:?}", precision_matrix.shape());

    // eprintln!("show me precision_matrix = \n{:?}", precision_matrix);

    // let iv = &information_vector;
    // let pm = &precision_matrix;

    let eta_a = information_vector.slice(s![seq_n(marg_idx, ndofs)]);
    assert_eq!(eta_a.len(), ndofs);

    let eta_b = if marg_idx == 0 {
        information_vector.slice(s![ndofs..])
    } else {
        information_vector.slice(s![..marg_idx])
    };
    assert_eq!(eta_b.len(), information_vector.len() - ndofs);

    let lam_aa = precision_matrix.slice(s![seq_n(marg_idx, ndofs), seq_n(marg_idx, ndofs)]);

    let lam_ab = if marg_idx == 0 {
        precision_matrix.slice(s![seq_n(marg_idx, ndofs), marg_idx + ndofs..])
    } else {
        precision_matrix.slice(s![seq_n(marg_idx, ndofs), ..marg_idx])
    };

    assert_eq!(lam_ab.shape(), &[ndofs, precision_matrix.ncols() - ndofs]);

    // eprintln!("margin_idx = {}", marg_idx);

    let lam_ba = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + ndofs.., seq_n(marg_idx, ndofs)])
    } else {
        precision_matrix.slice(s![..marg_idx, seq_n(marg_idx, ndofs)])
    };

    let lam_bb = if marg_idx == 0 {
        precision_matrix.slice(s![marg_idx + ndofs.., marg_idx + ndofs..])
    } else {
        precision_matrix.slice(s![..marg_idx, ..marg_idx])
    };

    // assert_eq!(
    //     lam_bb.shape(),
    //     &[
    //         precision_matrix.shape()[0] - ndofs,
    //         precision_matrix.shape()[1] - ndofs
    //     ]
    // );

    // eprintln!("lam_bb = {:?}", lam_bb);

    let Some(lam_bb_inv) = lam_bb.to_owned().inv() else {
        return Ok(Message::empty(ndofs));
    };

    // let lam_bb_inv = lam_bb.to_owned().inv().expect("should have an inverse");
    // let information_vector = (&eta_a - &lam_ab).dot(&lam_bb_inv).dot(&eta_b);
    let information_vector = &eta_a - &lam_ab.dot(&lam_bb_inv).dot(&eta_b);
    // eprintln!("information_vector = {:?}", information_vector);
    let precision_matrix = &lam_aa - &lam_ab.dot(&lam_bb_inv).dot(&lam_ba);
    // eprintln!("precision_matrix = {:?}", precision_matrix);

    if precision_matrix.iter().any(|elem| elem.is_infinite()) {
        Ok(Message::empty(information_vector.len()))
        // Message::with_dofs(information_vector.len())
        // Message::zeros(information_vector.len())
    } else {
        let mu = Vector::<Float>::zeros(information_vector.len());
        Ok(Message::new(
            Eta(information_vector),
            Lam(precision_matrix),
            Mu(mu),
        ))

        // let mvn = MultivariateNormal::from_information_and_precision(
        //     information_vector,
        //     precision_matrix,
        // )
        // .expect(
        //     "the given information vector and precision matrix is a valid multivariate gaussian",
        // );
        // Ok(Message::new(mvn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::concatenate;
    use pretty_assertions::assert_eq;

    fn float_eq(lhs: f32, rhs: f32) -> bool {
        f32::abs(lhs - rhs) <= f32::EPSILON
    }

    macro_rules! generate_8x8_precision_matrix {
        () => {{
            let upper_left = array![
                [1., 2., 3., 4.],
                [5., 6., 7., 8.],
                [9., 10., 11., 12.],
                [13., 14., 15., 16.]
            ];

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

        let (aa, ab, ba, bb) = extract_submatrices_from_precision_matrix(&precision_matrix, 4, 0);

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

        let (aa, ab, ba, bb) = extract_submatrices_from_precision_matrix(&precision_matrix, 4, 4);

        assert_eq!(aa, lower_right);
        assert_eq!(ab, lower_left);
        assert_eq!(ba, upper_right);
        assert_eq!(bb, upper_left);
    }

    #[test]
    fn information_vector_length_equal_to_ndofs_do_nothing() {
        #![allow(clippy::unwrap_used)]
        let information_vector: Vector<Float> = array![0., 1., 2., 3.];
        let precision_matrix: Matrix<Float> = array![
            [5., 0.2, 0., 0.],
            [0.2, 5., 0., 0.],
            [0., 0.0, 5., 0.3],
            [0., 0., 0.3, 5.]
        ];

        let ndofs = 4;
        let marginalisation_idx = 0;

        let mut marginalised_msg = marginalise_factor_distance(
            information_vector.clone(),
            precision_matrix.clone(),
            ndofs,
            marginalisation_idx,
        )
        .unwrap();

        let payload = marginalised_msg.take().unwrap();

        assert_eq!(payload.eta, information_vector);
        assert_eq!(payload.lam, precision_matrix);
    }

    // #[test]
    // fn size5x5_marg_idx1_ndofs4() {
    //     let information_vector: Vector<f32> = array![1., 2., 3., 4., 5.];
    //     let precision_matrix: Matrix<f32> = array![
    //         [0.5, 0.1, 0., 0., 0.2],
    //         [0.1, 0.5, 0., 0., 0.],
    //         [0., 0.0, 0.5, 0., 0.],
    //         [0., 0., 0., 0.5, 0.],
    //         [0.2, 0., 0., 0., 0.5]
    //     ];

    //     let ndofs = 4;
    //     let marginalisation_idx = 1;

    //     let marginalised_msg = marginalise_factor_distance(
    //         information_vector,
    //         precision_matrix,
    //         ndofs,
    //         marginalisation_idx,
    //     );

    //     assert_eq!(marginalised_msg.information_vector().len(), ndofs);
    //     assert_eq!(marginalised_msg.precision_matrix().shape(), &[ndofs, ndofs]);

    //     assert_eq!(
    //         marginalised_msg.information_vector(),
    //         array![1.8, 3., 4., 4.6]
    //     );

    //     let result = marginalised_msg
    //         .precision_matrix()
    //         .into_iter()
    //         .collect::<Vec<_>>();
    //     let expected = array![
    //         [0.48, 0., 0., -0.04],
    //         [0., 0.5, 0., 0.,],
    //         [0., 0., 0.5, 0.],
    //         [-0.04, 0., 0., 0.42]
    //     ]
    //     .into_iter()
    //     .collect::<Vec<_>>();
    // }
}
