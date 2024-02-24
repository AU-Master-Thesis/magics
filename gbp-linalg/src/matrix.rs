use nalgebra as na;
use paste::paste;

/// Utility function to update a submatrix/block of a matrix
/// `nalgebra` does, IMO, not provide a straight forward way to do this,
/// hence why this function exists.
/// The update `op` can be any function/closure taking two matrix elements ...
///
/// **preconditions:**
/// - `matrix.shape()` is (m, n)
/// - start is (r0, c0) where r0 < m && r1 < n
/// - `submatrix.shape()` is (q, r) where q + r0 <= m && r + r1 <= n
/// **postconditions:**
/// - `matrix.shape()` is (m2, n2) where m == m2 && n == n2, i.e. the shape of the matrix is preserved
pub fn update_submatrix_by<T: na::Scalar + Copy + num::Float, F>(
    matrix: &mut na::DMatrix<T>,
    start: (usize, usize),
    submatrix: &na::DMatrix<T>,
    op: F,
) where
    F: Fn(T, T) -> T,
{
    // Out of bounds is handled by nalgebra
    // nalgebra will call panic if any of the row/column sizes does not fit,
    let (nrows, ncols) = submatrix.shape();
    for r in start.0..start.0 + nrows {
        for c in start.1..start.1 + ncols {
            matrix[(r, c)] = op(matrix[(r, c)], submatrix[(r - start.0, c - start.1)]);
        }
    }
}

macro_rules! generate_update_submatrix_operation {
    ($prefix:ident, $op:expr) => {
        paste! {
            #[allow(dead_code)]
            // #[inline(always)]
            #[inline]
            pub fn [<$prefix _submatrix>]<T: na::Scalar + Copy + num::Float>(
                matrix: &mut na::DMatrix<T>,
                start: (usize, usize),
                submatrix: &na::DMatrix<T>,
            ) {
                update_submatrix_by(matrix, start, submatrix, $op)
            }
        }
    };
}

generate_update_submatrix_operation!(override, |_: T, b: T| b);
generate_update_submatrix_operation!(add_assign, |a: T, b: T| a + b);
generate_update_submatrix_operation!(sub_assign, |a: T, b: T| a - b);
generate_update_submatrix_operation!(mul_assign, |a: T, b: T| a * b);

// pub fn override_submatrix<T: na::Scalar + Copy + num::Float>(
//     matrix: &mut DMatrix<T>,
//     start: (usize, usize),
//     submatrix: &DMatrix<T>,
// ) {
//     update_submatrix_by(matrix, start, submatrix, |_, b| b)
// }

// pub fn add_assign_submatrix<T: na::Scalar + Copy + num::Float>(
//     matrix: &mut DMatrix<T>,
//     start: (usize, usize),
//     submatrix: &DMatrix<T>,
// ) {
//     update_submatrix_by(matrix, start, submatrix, |a, b| a + b)
// }

// pub fn sub_assign_submatrix<T: na::Scalar + Copy + num::Float>(
//     matrix: &mut DMatrix<T>,
//     start: (usize, usize),
//     submatrix: &DMatrix<T>,
// ) {
//     update_submatrix_by(matrix, start, submatrix, |a, b| a - b)
// }
// pub fn mul_assign_submatrix<T: na::Scalar + Copy + num::Float>(
//     matrix: &mut DMatrix<T>,
//     start: (usize, usize),
//     submatrix: &DMatrix<T>,
// ) {
//     update_submatrix_by(matrix, start, submatrix, |a, b| a * b)
// }

#[cfg(test)]
mod tests {
    use nalgebra as na;
    use pretty_assertions::assert_eq;

    #[test]
    fn override_submatrix() {
        let mut m = na::dmatrix![
            1., 2. ,3.;
            4., 5., 6.;
            7., 8., 9.
        ];

        let sm = na::dmatrix![10., 11.; 12., 13.];

        super::override_submatrix(&mut m, (0, 0), &sm);

        assert_eq!(
            m,
            na::dmatrix![
                10., 11., 3.;
                12., 13., 6.;
                7., 8., 9.
            ]
        );
    }

    #[test]
    #[should_panic]
    fn start_row_outside_the_shape_of_the_matrix_should_panic() {
        let mut m = na::dmatrix![
            1., 2., 3.;
            4., 5., 6.;
            7., 8., 9.;
            10., 11., 12.
        ];

        let sm = na::dmatrix![42.];
        let start = (m.nrows(), 0);

        super::add_assign_submatrix(&mut m, start, &sm);
    }
    #[test]
    #[should_panic]
    fn start_column_outside_the_shape_of_the_matrix_should_panic() {
        let mut m = na::dmatrix![
            1., 2., 3.;
            4., 5., 6.;
            7., 8., 9.;
            10., 11., 12.
        ];

        let sm = na::dmatrix![42.];
        let start = (0, m.ncols());

        super::add_assign_submatrix(&mut m, start, &sm);
    }

    #[test]
    #[should_panic]
    fn start_outside_the_shape_of_the_matrix_should_panic() {
        let mut m = na::dmatrix![
            1., 2., 3.;
            4., 5., 6.;
            7., 8., 9.;
            10., 11., 12.
        ];

        let sm = na::dmatrix![42.];
        let start = m.shape();

        super::add_assign_submatrix(&mut m, start, &sm);
    }

    #[test]
    fn add_assign_submatrix() {
        let mut m = na::dmatrix![
            1., 2. ,3.;
            4., 5., 6.;
            7., 8., 9.;
            10. ,11., 12.
        ];
        let sm = na::dmatrix![
            7., 8., 9.;
            10., 11., 12.
        ];

        super::add_assign_submatrix(&mut m, (2, 0), &sm);

        assert_eq!(
            m,
            na::dmatrix![
                1., 2. ,3.;
                4., 5., 6.;
                14., 16., 18.;
                20. ,22., 24.
            ]
        )
    }
}
