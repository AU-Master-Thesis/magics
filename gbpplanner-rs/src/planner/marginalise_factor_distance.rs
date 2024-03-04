use ndarray::{concatenate, prelude::*};
use ndarray_inverse::Inverse;

use super::{factorgraph::Message, Matrix, Vector};

// pub type Vector<T> = Array1<T>;
// pub type Matrix<T> = Array2<T>;

// pub trait Scalar: num_traits::Float + Copy {}
// impl Scalar for f32 {}
// impl Scalar for f64 {}

// #[derive(Debug)]
// struct Message<T: Scalar> {
//     pub information_vector: Vector<T>,
//     pub precision_matrix: Matrix<T>,
// }

// impl<T: Scalar> Message<T> {
//     pub fn zeroize(&mut self) {
//         self.information_vector.fill(T::zero());
//         self.precision_matrix.fill(T::zero());
//     }
// }

const fn seq_n(start: usize, n: usize) -> std::ops::Range<usize> {
    start..start + n
}

// TODO: possible duplicate function?
// fn marginalise_factor_distance<T: Scalar>(
pub fn marginalise_factor_distance(
    information_vector: Vector<f32>,
    precision_matrix: Matrix<f32>,
    dofs_of_variable: usize,
    marginalisation_idx: usize,
) -> Message {
    if information_vector.len() == dofs_of_variable {
        return Message::new(information_vector, precision_matrix).expect(
            "the given information vector and precision matrix is a valid multivariate gaussian",
        );
    }

    let ndofs = dofs_of_variable;
    let marg_idx = marginalisation_idx;
    let iv = &information_vector;
    let pm = &precision_matrix;

    let eta_a = information_vector.slice(s![seq_n(marginalisation_idx, dofs_of_variable)]);
    assert_eq!(eta_a.len(), ndofs);
    let eta_b = concatenate![
        Axis(0),
        information_vector.slice(s![0..marginalisation_idx]),
        information_vector.slice(s![marginalisation_idx + dofs_of_variable..])
    ];

    assert_eq!(eta_b.len(), information_vector.len() - ndofs);
    let lam_aa = precision_matrix.slice(s![
        seq_n(marginalisation_idx, dofs_of_variable),
        seq_n(marginalisation_idx, dofs_of_variable)
    ]);

    let lam_ab = concatenate![
        Axis(1),
        dbg!(precision_matrix.slice(s![seq_n(marg_idx, ndofs), ..marg_idx])),
        dbg!(precision_matrix.slice(s![seq_n(marg_idx, ndofs), marg_idx + ndofs..]))
    ];

    let lam_ba = if marg_idx + ndofs == information_vector.len() {
        concatenate![
            Axis(1),
            dbg!(precision_matrix.slice(s![..marg_idx, marg_idx..marg_idx + ndofs]))
        ]
    } else {
        concatenate![
            Axis(1),
            dbg!(precision_matrix.slice(s![..marg_idx, marg_idx..marg_idx + ndofs])),
            dbg!(precision_matrix.slice(s![marg_idx + ndofs.., seq_n(marg_idx, ndofs)]))
        ]
    };

    let lam_bb = concatenate![
        Axis(0),
        concatenate![
            Axis(1),
            precision_matrix.slice(s![..marg_idx, ..marg_idx]),
            precision_matrix.slice(s![..marg_idx, marg_idx + ndofs..])
        ],
        concatenate![
            Axis(1),
            precision_matrix.slice(s![marg_idx + ndofs.., ..marg_idx]),
            precision_matrix.slice(s![marg_idx + ndofs.., marg_idx + ndofs..])
        ]
    ];

    assert_eq!(
        lam_bb.shape(),
        &[
            precision_matrix.shape()[0] - ndofs,
            precision_matrix.shape()[1] - ndofs
        ]
    );

    let lam_bb_inv = lam_bb.inv().expect("should have an inverse");
    // let information_vector = (&eta_a - &lam_ab).dot(&lam_bb_inv).dot(&eta_b);
    let information_vector = &eta_a - &lam_ab.dot(&lam_bb_inv).dot(&eta_b);
    // eprintln!("information_vector = {:?}", information_vector);
    let precision_matrix = &lam_aa - &lam_ab.dot(&lam_bb_inv).dot(&lam_ba);
    // eprintln!("precision_matrix = {:?}", precision_matrix);

    if precision_matrix.iter().any(|elem| elem.is_infinite()) {
        Message::with_dofs(information_vector.len())
        // Message::zeros(information_vector.len())
    } else {
        Message::new(information_vector, precision_matrix).expect(
            "the given information vector and precision matrix is a valid multivariate gaussian",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn float_eq(lhs: f32, rhs: f32) -> bool {
        f32::abs(lhs - rhs) <= f32::EPSILON
    }

    #[test]
    fn concat_single_matrix() {
        let a = array![[0., 1.], [2., 3.]];
        let b = concatenate![Axis(1), a];
        assert_eq!(b, a);
    }

    #[test]
    fn inverse_of_single_element_matrix() {
        let a = array![[0.5]];
        let a_inv = a.inv().expect("should have an inverse");

        assert_eq!(a_inv, array![[2.]]);
    }

    #[test]
    fn inverse_of_2x2_matrix() {
        let a = array![[0.5, 0.], [0., 2.]];
        let a_inv = a.inv().expect("should have an inverse");
        eprintln!("a_inv = {:?}", a_inv);
        assert_eq!(a_inv, array![[2., 0.], [0., 0.5]]);
    }

    #[test]
    fn concat_block_matrix() {
        let a = array![
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16]
        ];

        let b = concatenate![
            Axis(0),
            concatenate![Axis(1), a.slice(s![..3, ..3]), a.slice(s![..3, 3..]),],
            concatenate![Axis(1), a.slice(s![3.., ..3]), a.slice(s![3.., 3..])]
        ];

        assert_eq!(a, b);
    }

    #[test]
    fn ranges() {
        let a = ..5usize;
        assert_eq!(a.end, 5);
        let b = 5..;
        assert_eq!(b.start, 5);

        let c = ..=5;
        assert_eq!(c.end, 5);

        let d = 2..5;
        assert_eq!(d.start, 2);
        assert_eq!(d.end, 5);
        let e = 2..=5;
    }

    // #[test]
    // fn call_marginalise_factor_distance() {
    //     let information_vector: Vector<f32> = array![0., 1., 2., 3.];
    //     let precision_matrix: Matrix<f32> = array![
    //         [5., 0.2, 0., 0.],
    //         [0.2, 5., 0., 0.],
    //         [0., 0.0, 5., 0.3],
    //         [0., 0., 0.3, 5.]
    //     ];

    //     let ndofs = 3;
    //     let marginalisation_idx = 0;

    //     let marginalised_msg = marginalise_factor_distance(
    //         information_vector,
    //         precision_matrix,
    //         ndofs,
    //         marginalisation_idx,
    //     );

    //     println!("{:?}", marginalised_msg);

    //     assert!(false);
    // }

    #[test]
    fn information_vector_length_equal_to_ndofs_do_nothing() {
        let information_vector: Vector<f32> = array![0., 1., 2., 3.];
        let precision_matrix: Matrix<f32> = array![
            [5., 0.2, 0., 0.],
            [0.2, 5., 0., 0.],
            [0., 0.0, 5., 0.3],
            [0., 0., 0.3, 5.]
        ];

        let ndofs = 4;
        let marginalisation_idx = 0;

        let marginalised_msg = marginalise_factor_distance(
            information_vector,
            precision_matrix,
            ndofs,
            marginalisation_idx,
        );

        assert_eq!(
            marginalised_msg.gaussian.information_vector(),
            array![0., 1., 2., 3.]
        );
        assert_eq!(
            marginalised_msg.gaussian.precision_matrix(),
            array![
                [5., 0.2, 0., 0.],
                [0.2, 5., 0., 0.],
                [0., 0.0, 5., 0.3],
                [0., 0., 0.3, 5.]
            ]
        );
    }

    #[test]
    fn size5x5_marg_idx1_ndofs4() {
        let information_vector: Vector<f32> = array![1., 2., 3., 4., 5.];
        let precision_matrix: Matrix<f32> = array![
            [0.5, 0.1, 0., 0., 0.2],
            [0.1, 0.5, 0., 0., 0.],
            [0., 0.0, 0.5, 0., 0.],
            [0., 0., 0., 0.5, 0.],
            [0.2, 0., 0., 0., 0.5]
        ];

        let ndofs = 4;
        let marginalisation_idx = 1;

        let marginalised_msg = marginalise_factor_distance(
            information_vector,
            precision_matrix,
            ndofs,
            marginalisation_idx,
        );

        assert_eq!(marginalised_msg.gaussian.information_vector().len(), ndofs);
        assert_eq!(
            marginalised_msg.gaussian.precision_matrix().shape(),
            &[ndofs, ndofs]
        );

        assert_eq!(
            marginalised_msg.gaussian.information_vector(),
            array![1.8, 3., 4., 4.6]
        );

        let result = marginalised_msg
            .gaussian
            .precision_matrix()
            .into_iter()
            .collect::<Vec<_>>();
        let expected = array![
            [0.48, 0., 0., -0.04],
            [0., 0.5, 0., 0.,],
            [0., 0., 0.5, 0.],
            [-0.04, 0., 0., 0.42]
        ]
        .into_iter()
        .collect::<Vec<_>>();
    }
}
