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

// fn marginalise_factor_distance<T: Scalar>(
pub fn marginalise_factor_distance(
    information_vector: Vector<f32>,
    precision_matrix: Matrix<f32>,
    dofs_of_variable: usize,
    marginalisation_idx: usize,
) -> Message {
    let ndofs = dofs_of_variable;
    let marg_idx = marginalisation_idx;
    let iv = &information_vector;
    let pm = &precision_matrix;
    if information_vector.len() == dofs_of_variable {
        return Message::new(information_vector, precision_matrix);
    }

    // eprintln!("dofs_of_variable = {dofs_of_variable}");
    // eprintln!("marginalisation_idx = {marginalisation_idx}");
    // eprintln!("information_vector.len() = {:?}", information_vector.len());
    // eprintln!("precision_matrix.shape() = {:?}", precision_matrix.shape());

    let eta_a = information_vector.slice(s![seq_n(marginalisation_idx, dofs_of_variable)]);
    // eprintln!("eta_a = {:?}", eta_a);
    assert_eq!(eta_a.len(), ndofs);
    let eta_b = concatenate![
        Axis(0),
        information_vector.slice(s![0..marginalisation_idx]),
        information_vector.slice(s![marginalisation_idx + dofs_of_variable..])
    ];
    // eprintln!("eta_b = {:?}", eta_b);
    assert_eq!(eta_b.len(), information_vector.len() - ndofs);

    let lam_aa = precision_matrix.slice(s![
        seq_n(marginalisation_idx, dofs_of_variable),
        seq_n(marginalisation_idx, dofs_of_variable)
    ]);

    // eprintln!("lam_aa = {:#?}", lam_aa);

    // assert_eq!(lam_aa.shape(), &[])

    // lam_ab << Lam(seqN(marg_idx, n_dofs), seq(0, marg_idx - 1)),
    //           Lam(seqN(marg_idx, n_dofs), seq(marg_idx + n_dofs, last));
    let lam_ab = concatenate![
        Axis(1),
        dbg!(precision_matrix.slice(s![seq_n(marg_idx, ndofs), ..marg_idx])),
        dbg!(precision_matrix.slice(s![seq_n(marg_idx, ndofs), marg_idx + ndofs..]))
    ];

    // eprintln!("lam_ab = {:#?}", lam_ab);

    // lam_ba << Lam(seq(0, marg_idx - 1), seq(marg_idx, marg_idx + n_dofs - 1)),
    //           Lam(seq(marg_idx + n_dofs, last), seqN(marg_idx, n_dofs));
    // let lam_ba = {
    //     let mut lam_ba =
    // }
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
    // let lam_ba = concatenate![
    //     Axis(1),
    //     dbg!(precision_matrix.slice(s![..marg_idx, marg_idx..marg_idx + ndofs])),
    //     dbg!(precision_matrix.slice(s![marg_idx + ndofs.., seqN(marg_idx, ndofs)]))
    // ];

    // eprintln!("lam_ba = {:?}", lam_ba);

    // let lam_bb = {
    //     let shape = precision_matrix.shape();
    //     // let shap
    //     let mut m = Matrix::<f32>::uninit((shape[0] - ndofs, shape[0]-ndofs));
    //     m.slice_mut(s![])
    // };

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

    // eprintln!("lam_bb = {:#?}", lam_bb);

    assert_eq!(
        lam_bb.shape(),
        &[
            precision_matrix.shape()[0] - ndofs,
            precision_matrix.shape()[1] - ndofs
        ]
    );

    // let mut lam_bb = Array::<f32>::uninit((precision_matrix.rows()))
    // let lam_bb = concatenate![
    //     Axis(1),
    //     precision_matrix.slice(s![])

    // ]

    // lam_bb.

    let lam_bb_inv = lam_bb.inv().expect("should have an inverse");

    // eprintln!("lam_bb_inv = {:?}", lam_bb_inv);

    {
        // let information_vector = (&eta_a - &lam_ab).dot(&lam_bb_inv).dot(&eta_b);
        let information_vector = &eta_a - &lam_ab.dot(&lam_bb_inv).dot(&eta_b);
        // eprintln!("information_vector = {:?}", information_vector);
        let precision_matrix = &lam_aa - &lam_ab.dot(&lam_bb_inv).dot(&lam_ba);
        // eprintln!("precision_matrix = {:?}", precision_matrix);

        if precision_matrix.iter().any(|elem| elem.is_infinite()) {
            Message::zeros(information_vector.len())
        } else {
            Message::new(information_vector, precision_matrix)
        }

        // let mut message = Message {
        //     information_vector,
        //     precision_matrix,
        // };

        // if message
        //     .precision_matrix
        //     .iter()
        //     .any(|elem| elem.is_infinite())
        // {
        //     message.zeroize();
        // }

        // message
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
            marginalised_msg.0.information_vector,
            array![0., 1., 2., 3.]
        );
        assert_eq!(
            marginalised_msg.0.precision_matrix,
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

        assert_eq!(marginalised_msg.0.information_vector.len(), ndofs);
        assert_eq!(marginalised_msg.0.precision_matrix.shape(), &[ndofs, ndofs]);

        assert_eq!(
            marginalised_msg.0.information_vector,
            array![1.8, 3., 4., 4.6]
        );

        let result = marginalised_msg
            .0
            .precision_matrix
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
