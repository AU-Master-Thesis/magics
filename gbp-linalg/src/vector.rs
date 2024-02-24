use nalgebra as na;
use paste::paste;

/// Utility function to update a subvector/slice of a vector
/// `nalgebra` does, IMO, not provide a straight forward way to do this,
/// hence why this function exists.
/// The update `op` can be any function/closure taking two matrix elements ...
///
/// **preconditions:**
/// - `vector.len()` is n
/// - start is s where s < n
/// - `subvector.len()` is m where m + s <= n
/// **postconditions:**
/// - `vector.len()` is n2 where n == n2, i.e. the shape of the vector is preserved
pub fn update_subvector_by<T: na::Scalar + Copy + num::Float, F>(
    vector: &mut na::DVector<T>,
    start: usize,
    subvector: &na::DVector<T>,
    op: F,
) where
    F: Fn(T, T) -> T,
{
    if vector.len() < start + subvector.len() {
        // panic!(format!("The length of vector is {}, but the length of the subvector offset by start, would put it at [{}, {}]", vector.len(), start, start + subvector.leng()));
        std::panic::panic_any(format!("The length of the vector is {}, but the length of the subvector offset by start, would put the view at [{}, {}]", vector.len(), start, start + subvector.len()));
    }
    // Out of bounds is handled by nalgebra
    // nalgebra will call panic if any of the row/column sizes does not fit,
    for i in start..start + subvector.len() {
        vector[i] = op(vector[i], subvector[i - start]);
    }
}

macro_rules! generate_update_subvector_operation {
    ($prefix:ident, $op:expr) => {
        paste! {
            #[allow(dead_code)]
            // #[inline(always)]
            #[inline]
            pub fn [<$prefix _subvector>]<T: na::Scalar + Copy + num::Float>(
            vector: &mut na::DVector<T>,
            start: usize,
            subvector: &na::DVector<T>,
            ) {
                update_subvector_by(vector, start, subvector, $op)
            }
        }
    };
}

generate_update_subvector_operation!(override, |_: T, b: T| b);
generate_update_subvector_operation!(add_assign, |a: T, b: T| a + b);
generate_update_subvector_operation!(sub_assign, |a: T, b: T| a - b);
generate_update_subvector_operation!(mul_assign, |a: T, b: T| a * b);

#[macro_export]
macro_rules! vector_view {
    ($vec:ident, $range:expr) => {
        $vec.view(($range.start, 0), ($range.end - $range.start, 1))
    };

    ($vec:ident, $start:expr, $end:expr) => {
        $vec.view(($start, 0), ($end - $start, 1))
    };
}

#[macro_export]
macro_rules! vector_view_mut {
    ($vec:ident, $range:expr) => {
        $vec.view_mut(($range.start, 0), ($range.end - $range.start, 1))
    };

    ($vec:ident, $start:expr, $end:expr) => {
        $vec.view_mut(($start, 0), ($end - $start, 1))
    };
}

#[macro_export]
macro_rules! vector_concat {
    ($first:ident, $($rest:ident),+ $(,)?) => {{
        let total_length = $first.len() $(+ $rest.len())+;
        let mut concatenated = ::nalgebra::DVector::zeros(total_length);
        // #[allow(unused_assignments)]
        let mut _offset = 0;
        for i in 0..$first.len() {
            concatenated[i] = $first[i];
        }
        _offset += $first.len();
        $(
            for i in _offset.._offset+$rest.len() {
                concatenated[i] = $rest[i - _offset];
            }
            _offset += $rest.len();
        )+

        concatenated

        // let mut iter = $first.into_iter();
        // ::nalgebra::DVector::from_iterator(3, iter)
        // $(iter.chain($rest.into_iter()))+;
        // ::nalgebra::DVector::from_iterator(total_length, iter)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // fn float_eq(lhs: f32, rhs: f32) -> bool {
    //     f32::abs(lhs - rhs) <= f32::EPSILON
    // }

    #[test]
    fn view_of_static_vector() {
        let v = na::vector![1, 2, 3, 4, 5, 6, 7, 8];
        let first4 = vector_view!(v, 0..4);
        assert_eq!(first4, na::vector![1, 2, 3, 4]);
        let last4 = vector_view!(v, 4, v.len());
        assert_eq!(last4, na::vector![5, 6, 7, 8]);
        let middle2 = vector_view!(v, v.len() / 2 - 1, v.len() / 2 + 1);
        assert_eq!(middle2, na::vector![4, 5]);
    }

    #[test]
    #[should_panic]
    fn view_of_static_vector_outside_span_should_panic() {
        let v = na::vector![1, 2, 3, 4, 5, 6, 7, 8];
        let _ = vector_view!(v, 4..12);
    }

    #[test]
    fn view_of_dynamic_vector() {
        let v = na::dvector![1, 2, 3, 4, 5, 6, 7, 8];
        let first4 = vector_view!(v, 0..4);
        assert_eq!(first4, na::dvector![1, 2, 3, 4]);
        let last4 = vector_view!(v, 4, v.len());
        assert_eq!(last4, na::dvector![5, 6, 7, 8]);
        let middle2 = vector_view!(v, v.len() / 2 - 1, v.len() / 2 + 1);
        assert_eq!(middle2, na::dvector![4, 5]);
    }

    #[test]
    #[should_panic]
    fn view_of_dynamic_vector_outside_span_should_panic() {
        let v = na::dvector![1, 2, 3, 4, 5, 6, 7, 8];
        let _ = vector_view!(v, 1, 9);
    }

    #[test]
    fn mutable_view_of_dynamic_vector() {
        let mut v = na::dvector![1, 2, 3, 4, 5, 6, 7, 8];

        let mut first4 = vector_view_mut!(v, 0..4);
        assert_eq!(first4, na::dvector![1, 2, 3, 4]);
        first4[0] = 42;
        assert_eq!(first4, na::dvector![42, 2, 3, 4]);
        first4[3] = 69;
        assert_eq!(first4, na::dvector![42, 2, 3, 69]);

        let mut last4 = vector_view_mut!(v, 4, v.len());
        assert_eq!(last4, na::dvector![5, 6, 7, 8]);
        last4[0] = 23;
        assert_eq!(last4, na::dvector![23, 6, 7, 8]);
        last4[3] = 46;
        assert_eq!(last4, na::dvector![23, 6, 7, 46]);
        assert_eq!(v, na::dvector![42, 2, 3, 69, 23, 6, 7, 46]);
    }

    #[test]
    #[should_panic]
    fn mutable_view_of_dynamic_vector_outside_span_should_panic() {
        let mut v = na::dvector![1, 2, 3, 4, 5, 6, 7, 8];
        let _ = vector_view_mut!(v, 1, 9);
    }

    #[test]
    fn concatenate_two_vectors() {
        let a = na::dvector![1., 2., 3.];
        let b = na::dvector![4., 5., 6., 7.];
        let c = vector_concat![a, b];
        assert_eq!(c, na::dvector![1., 2., 3., 4., 5., 6., 7.]);
    }

    #[test]
    fn concatenate_three_vectors() {
        let a = na::dvector![1., 2., 3.];
        let b = na::dvector![4., 5., 6., 7.];
        let c = na::dvector![8., 9.];
        let d = vector_concat![a, b, c];
        let _: na::DVector<f32> = na::dvector![];
        assert_eq!(d, na::dvector![1., 2., 3., 4., 5., 6., 7., 8., 9.]);
    }

    #[test]
    fn override_subvector() {
        let mut v = na::dvector![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let sv = na::dvector![10., 11., 12., 13.];

        super::override_subvector(&mut v, 0, &sv);

        assert_eq!(v, na::dvector![10., 11., 12., 13., 5., 6., 7., 8., 9.]);
    }

    #[test]
    fn sub_assign_subvector() {
        let mut v = na::dvector![1., 2., 3., 4., 5., 6., 7., 8., 9.];
        let sv = na::dvector![10., 11., 12., 13.];

        super::sub_assign_subvector(&mut v, 3, &sv);
        assert_eq!(v, na::dvector![1., 2., 3., -6., -6., -6., -6., 8., 9.]);
    }

    #[test]
    fn add_assign_subvector() {
        let mut v = na::dvector![1., 2., 3., 4., 5., 6., 7., 8., 9., 10.];
        let sv = na::dvector![10., 11., 12., 13.];

        super::add_assign_subvector(&mut v, 5, &sv);
        assert_eq!(v, na::dvector![1., 2., 3., 4., 5., 16., 18., 20., 22., 10.]);
    }
}
