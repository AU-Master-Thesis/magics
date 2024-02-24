pub mod matrix;
pub mod vector;

// use core::panic;
// use std::{fmt::Display, panic::panic_any};

// use nalgebra::{DMatrix, DVector, Dim, Scalar};

// // #[derive(Debug)]
// // struct MatrixShape {
// //     nrows: usize,
// //     ncols: usize,
// // }

// // impl std::fmt::Display for MatrixShape {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         write!(f, "{:?}", self)
// //     }
// // }

// // #[derive(Debug)]
// // enum InsertBlockMatrixError {
// //     BlockLargerThanMatrix {
// //         matrix_dims: MatrixShape,
// //         block_dims: MatrixShape
// //     },
// //     StartOutsideMatrix {
// //         start: (usize, usize),
// //         matrix_dims: MatrixShape,
// //     },
// //     StartPlusBlockExceedsMatrix {
// //         start: (usize, usize),
// //         matrix_dims: MatrixShape,
// //         block_dims: MatrixShape
// //     }
// // }

// // impl std::fmt::Display for InsertBlockMatrixError {
// //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// //         use InsertBlockMatrixError::*;
// //         match self {
// //             BlockLargerThanMatrix { matrix_dims, block_dims } => write!(f, "The dimensions of the block matrix ({}) exceeds the dimensions of the matrix ({})", block_dims, matrix_dims),
// //             StartOutsideMatrix { start, matrix_dims } => write!(f, "The start offset ({}, {}) is outside the dimensions of the matrix ({})", start.0, start.1, matrix_dims ),
// //             StartPlusBlockExceedsMatrix { start, matrix_dims, block_dims } => write!(f, ""),
// //         }
// //     }
// // }

// // impl std::error::Error for InsertBlockMatrixError {}

// fn insert_block_matrix<T: Scalar + Copy>(
//     matrix: &mut DMatrix<T>,
//     start: (usize, usize),
//     block: &DMatrix<T>,
// ) {
//     debug_assert!(
//         start.0 <= matrix.nrows() && start.1 <= matrix.ncols(),
//         "start: ({}, {}) not inside matrix dims: ({}, {})",
//         start.0,
//         start.1,
//         matrix.nrows(),
//         matrix.ncols()
//     );
//     debug_assert!(
//         block.nrows() <= matrix.nrows() && block.ncols() <= block.nrows(),
//         "block's dims ({}, {}) exceeds the matrix's ({}, {})",
//         block.nrows(),
//         block.ncols(),
//         matrix.nrows(),
//         matrix.ncols()
//     );

//     debug_assert!(
//         matrix.nrows() - start.0 >= block.nrows() || matrix.ncols() - start.1 >= block.ncols(),
//         "inserting block with dims ({}, {}) at ({}, {}) would exceed the matrix's dims ({}, {})",
//         block.nrows(),
//         block.ncols(),
//         start.0,
//         start.1,
//         matrix.nrows(),
//         matrix.ncols()
//     );

//     for r in 0..block.nrows() {
//         for c in 0..block.ncols() {
//             matrix[(r + start.0, c + start.1)] = block[(r, c)];
//         }
//     }
// }

// fn main() {
//     let (nrows, ncols) = (4, 4);
//     let mut m = DMatrix::<f32>::zeros(nrows, ncols);

//     let I = DMatrix::<f32>::identity(nrows / 2, ncols / 2);
//     println!("I = {}", I);

//     let fives = nalgebra::dmatrix![5.0, 5.0; 5.0, 5.0];

//     // let mut view = m.view_mut((0, 0), (nrows / 2, ncols / 2));
//     // view[(0, 1)] = 3.0;

//     insert_block_matrix(&mut m, (0, 0), &I);
//     insert_block_matrix(&mut m, (0, 2), &fives);
//     insert_block_matrix(&mut m, (2, 0), &(2.0 * &fives));
//     insert_block_matrix(&mut m, (2, 2), &(4.0 * &fives));
//     // view = I.as_view_mut();

//     // println!(
//     //     "(({}, {}), ({}, {})) = {}",
//     //     0,
//     //     0,
//     //     nrows / 2,
//     //     ncols / 2,
//     //     view
//     // );

//     println!("{}", m);
// }

// use std::ops::Range;

// use nalgebra as na;

// pub trait DVectorExtensions {
//     type Scalar: na::Scalar + Copy;
// type RStride = na::RStride<Self::Scalar, na::Const<1>>;

// fn slice(
//     &self,
//     range: Range<usize>,
// ) -> na::Matrix<
//     Self::Scalar,
//     na::Dyn,
//     na::Const<1>,
//     na::VecStorage<Self::Scalar, na::Dyn, na::Const<1>>,
// >;

// fn slice(
//     &self,
//     range: Range<usize>,
// ) -> na::MatrixView<
//     '_,
//     Self::Scalar,
//     na::Dyn,
//     na::Const<1>,
//     na::RStride<Self::Scalar, na::Const<1>, na::Const<1>>,
//     na::CStride<Self::Scalar, na::Const<1>, na::Const<1>>,
// >;

// fn slice(
//     &self,
//     range: Range<usize>,
// ) -> na::DVectorView<
//     '_,
//     Self::Scalar,
//     na::RStride<Self::Scalar, na::Const<1>>,
//     na::CStride<Self::Scalar, na::Const<1>>,
// >;
// }

// impl<Scalar: na::Scalar + Copy> DVectorExtensions for na::DVector<Scalar> {
//     type Scalar = Scalar;

//     fn slice(
//         &self,
//         range: Range<usize>,
//     ) -> na::MatrixView<
//         '_,
//         Self::Scalar,
//         na::Dyn,
//         na::Const<1>,
//         na::RStride<Self::Scalar, na::Const<1>, na::Const<1>>,
//         na::CStride<Self::Scalar, na::Const<1>, na::Const<1>>,
//     > {
//         todo!()
//         // self.view((range.start, 0), (range.end, 1)) as na::VectorView<Self::Scalar>
//     }

// fn slice(
//     &self,
//     range: Range<usize>,
// ) -> na::DVectorView<
//     '_,
//     Self::Scalar,
//     na::RStride<Self::Scalar, na::Const<1>>,
//     na::CStride<Self::Scalar, na::Const<1>>,
// > {
//     todo!()
//     // self.view((range.start, 0), (range.end, 1)) // as na::DVectorView<Self::Scalar>
// }

// fn slice(
//     &self,
//     range: Range<usize>,
// ) -> na::Matrix<
//     Self::Scalar,
//     na::Dyn,
//     na::Const<1>,
//     na::VecStorage<Self::Scalar, na::Dyn, na::Const<1>>,
// > {
//     self.view((range.start, 0), (range.end, 1)).into_owned()
// }
// }
