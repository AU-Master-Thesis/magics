// use nalgebra as na;

// struct LookaheadParams {
//     horizon: usize,
//     multiple: usize,
// }

// TODO: take a struct as argument for better names

/// Compute the timesteps at which variables in the planned path are placed.
/// For a `lookahead_multiple` of 3, variables are spaced at timesteps:
/// 0,  1, 2, 3,  5, 7, 9, 12, 15, 18, ...
/// e.g. variables ar in groups of size `lookahead_multiple`.
/// The spacing within a group increases by one each time (1, for the first
/// group, 2 for the second etc.) Seems convoluted, but the reasoning was:
/// - The first variable should always be at 1 timestep from the current state
///   (0).
/// - The first few variables should be close together in time
/// - The variables should all be at integer timesteps, but the spacing should
///   sort of increase exponentially.
/// ## Example:
/// ```rust
/// let lookahead_horizon = 20;
/// let lookahead_multiple = 3;
/// assert_eq!(
///     get_variable_timesteps(lookahead_horizon, lookahead_multiple),
///     vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 20]
/// );
/// ```
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn get_variable_timesteps(lookahead_horizon: u32, lookahead_multiple: u32) -> Vec<u32> {
    // println!("lookahead_horizon: {}", lookahead_horizon);
    // println!("lookahead_multiple: {}", lookahead_multiple);
    // A visual argument is given for the estimate of the initial capacity in this
    // desmos graph. https://www.desmos.com/calculator/lxxsuqtgdq
    let estimated_capacity = (2.5 * f32::sqrt(lookahead_horizon as f32)) as usize;
    let mut timesteps = Vec::<u32>::with_capacity(estimated_capacity);
    // timesteps.push(0);

    let n = 1
        + (0.5
            * (-1.0 + f32::sqrt(1.0 + 8.0 * lookahead_horizon as f32 / lookahead_multiple as f32)))
            as u32;
    // TODO: use std::iter::successors instead
    for i in 0..(lookahead_multiple * (n + 1)) {
        let section = i / lookahead_multiple;
        let f = (lookahead_multiple as f32 / 2.0).mul_add(
            section as f32,
            (section as f32).mul_add(-(lookahead_multiple as f32), i as f32),
        ) * (section as f32 + 1.0);

        if f >= lookahead_horizon as f32 {
            timesteps.push(lookahead_horizon);
            break;
        }

        timesteps.push(f as u32);

        // // timesteps[i] = timesteps[i-1] + (i - 1) / lookahead_multiple + 1;
        // let ts = timesteps[timesteps.len() - 1] + ((i - 1) /
        // lookahead_multiple) + 1; if ts >= lookahead_horizon {
        //     timesteps.push(lookahead_horizon);
        //     break;
        // }
        // timesteps.push(ts);
    }

    println!("TIMESTEPS {:?}", timesteps);

    timesteps
}

// pub fn static_matrix_to_dynamic<T: na::Scalar, R: na::Const, C: na:: Const,
// usize>(     m: na::Matrix<T, R, C, na::ArrayStorage<T>>,
// ) -> na::Matrix<T> {
//     let mut d = na::Matrix::<T>::zeros(R, C);
//     for i in 0..R {
//         for j in 0..C {
//             d[(i, j)] = m[(i, j)];
//         }
//     }
//     d
// }

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_get_variable_timesteps() {
        let lookahead_horizon = 4;
        let lookahead_multiple = 3;

        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 4]
        );

        let lookahead_horizon = 30;
        let lookahead_multiple = 3;

        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30]
        );

        let lookahead_horizon = 60;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30, 35, 40, 45, 51, 57, 60]
        );

        let lookahead_horizon = 10;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 10]
        );

        let lookahead_horizon = 20;
        let lookahead_multiple = 5;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 4, 5, 7, 9, 11, 13, 15, 18, 20],
        );
    }
}

// pub fn get_variable_timesteps(lookahead_horizon: usize, lookahead_multiple:
// usize) -> Vec<usize> {     let estimated_capacity = (2.5 *
// f32::sqrt(lookahead_horizon as f32)) as usize;

//     std::iter::successors(Some(0), |&x| {
//         Some(x + ((x - 1) / lookahead_multiple).saturating_add(1))
//             .filter(|&ts| ts < lookahead_horizon)
//     })
//     .take_while(|&ts| ts < lookahead_horizon)
//     .collect()
// }

// pub mod nalgebra {
//     use std::ops::Range;

//     use nalgebra::{DVector, Scalar};

//     pub fn concat_column_vectors(a: &Vector<f32>, b: &Vector<f32>) ->
// Vector<f32> {         let combined_length = a.nrows() + b.nrows();
//         let mut c = DVector::<f32>::zeros(combined_length);
//         for i in 0..a.nrows() {
//             c[i] = a[i];
//         }

//         let offset = a.nrows();
//         for i in 0..b.nrows() {
//             c[offset + i] = b[i];
//         }

//         c
//     }

//     pub fn insert_subvector<T: Scalar + Copy>(
//         lhs: &mut DVector<T>,
//         range: Range<usize>,
//         rhs: &DVector<T>,
//     ) {
//         if rhs.len() != range.len() {
//             panic!("The vector you want to copy from does not have the same
// length as the range");         }

//         if lhs.len() < range.end {
//             panic!("The vector you want to copy to, is not as long as the
// range");         }

//         let offset = range.start;
//         for i in range {
//             lhs[i] = rhs[i - offset];
//         }
//     }

//     #[cfg(test)]
//     mod tests {
//         use super::*;

//         #[test]
//         fn test_concat_row_vectors() {
//             use nalgebra::dvector;
//             let a = dvector![1.0, 2.0, 3.0];
//             let b = dvector![4.0, 5.0];

//             let c = concat_column_vectors(&a, &b);

//             assert_eq!(dvector![1.0, 2.0, 3.0, 4.0, 5.0], c);
//         }
//     }
// }

// pub struct Indent {
//     level:  usize,
//     amount: usize,
// }
//
// pub trait PrettyPrint {
//     fn pretty_format(&self, indent: Indent) -> String;
//     fn pretty_print(&self, indent: Indent) {
//         println!("{}", self.pretty_format(indent));
//     }
// }
