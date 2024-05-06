//! Iterators for common sequences

// TODO: create `LowerTriangular` iterator
// TODO: create `LowerTriangularExcludeDiagonal` iterator

use std::num::NonZeroUsize;

#[inline]
const fn sum_of_first_n(n: usize) -> usize {
    (n + 1) * n / 2
}

/// An iterator over the indices of a upper triangular square matrix.
///
/// # Examples
///
/// ```rust
/// # use std::num::NonZeroUsize;
/// # use seq::upper_triangular;
/// let n = NonZeroUsize::new(4).expect("4 > 0");
/// let mut ut = upper_triangular(n);
/// assert_eq!(ut.next(), Some((0, 0)));
/// assert_eq!(ut.next(), Some((0, 1)));
/// assert_eq!(ut.next(), Some((0, 2)));
/// assert_eq!(ut.next(), Some((0, 3)));
/// assert_eq!(ut.next(), Some((1, 1)));
/// assert_eq!(ut.next(), Some((1, 2)));
/// assert_eq!(ut.next(), Some((1, 3)));
/// assert_eq!(ut.next(), Some((2, 2)));
/// assert_eq!(ut.next(), Some((2, 3)));
/// assert_eq!(ut.next(), Some((3, 3)));
/// assert_eq!(ut.next(), None);
/// ```
#[derive(Clone, Copy)]
pub struct UpperTriangular {
    n:   usize,
    row: usize,
    col: usize,
}

#[allow(clippy::len_without_is_empty)]
impl UpperTriangular {
    /// Return the number of elements of the upper triangular matrix
    #[inline]
    pub const fn len(&self) -> usize {
        sum_of_first_n(self.n + 1)
    }
}

impl std::iter::Iterator for UpperTriangular {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.row < self.n {
            let index = (self.row, self.col);
            if self.col + 1 < self.n {
                self.col += 1;
            } else {
                self.row += 1;
                self.col = self.row;
            }
            Some(index)
        } else {
            None
        }
    }
}

impl std::iter::ExactSizeIterator for UpperTriangular {
    fn len(&self) -> usize {
        self.len()
    }
}

/// Construct a new [`UpperTriangular`] iterator
pub const fn upper_triangular(n: NonZeroUsize) -> UpperTriangular {
    let n = n.get();
    UpperTriangular { n, row: 0, col: 0 }
}

/// An iterator over the indices of an upper triangular square matrix
/// excluding the diagonal.
///
/// Construct with [`upper_triangular_exclude_diagonal`]
///
/// # Examples
///
/// ```rust
/// # use std::num::NonZeroUsize;
/// # use seq::upper_triangular_exclude_diagonal;
/// let n = NonZeroUsize::new(4).expect("4 > 0");
/// let mut ut = upper_triangular_exclude_diagonal(n).expect("4 > 1");
/// assert_eq!(ut.next(), Some((0, 1)));
/// assert_eq!(ut.next(), Some((0, 2)));
/// assert_eq!(ut.next(), Some((0, 3)));
/// assert_eq!(ut.next(), Some((1, 2)));
/// assert_eq!(ut.next(), Some((1, 3)));
/// assert_eq!(ut.next(), Some((2, 3)));
/// assert_eq!(ut.next(), None);
/// ```
#[derive(Clone, Copy)]
pub struct UpperTriangularExcludeDiagonal {
    n:   usize,
    row: usize,
    col: usize,
}

impl UpperTriangularExcludeDiagonal {
    // // TODO: test
    // const fn last(&self) -> (usize, usize) {
    //     (self.row - 2, self.col - 1)
    // }

    #[inline]
    pub const fn len(&self) -> usize {
        sum_of_first_n(self.n)
    }
}

impl std::iter::Iterator for UpperTriangularExcludeDiagonal {
    type Item = (usize, usize);

    // (0, 1,), (0, 2), ... , (0, n - 1), (1, 2), ... , (1, n - 1), ... , (n - 1, n
    // - 1)
    fn next(&mut self) -> Option<Self::Item> {
        if self.row == self.n - 2 && self.col == self.n - 1 {
            None
        } else {
            if self.col == self.n - 1 {
                self.row += 1;
                self.col = self.row + 1;
            } else {
                self.col += 1;
            }
            Some((self.row, self.col))
        }
    }
}

impl std::iter::ExactSizeIterator for UpperTriangularExcludeDiagonal {
    fn len(&self) -> usize {
        self.len()
    }
}

/// Construct a new [`UpperTriangularExcludeDiagonal`] iterator
///
/// # Returns
/// `None` if `n == 1`
#[must_use]
pub fn upper_triangular_exclude_diagonal(
    n: NonZeroUsize,
) -> Option<UpperTriangularExcludeDiagonal> {
    let n = n.get();
    // n cannot be zero, due to the NonZeroUsize constraint
    if n == 1 {
        // A 1x1 matrix, has no upper triangular elements, when also excluding the
        // diagonal
        return None;
    }

    Some(UpperTriangularExcludeDiagonal { n, row: 0, col: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_of_first_n() {
        assert_eq!(0, sum_of_first_n(0));
        assert_eq!(1, sum_of_first_n(1));
        assert_eq!(3, sum_of_first_n(2));
        assert_eq!(6, sum_of_first_n(3));
        assert_eq!(10, sum_of_first_n(4));
        assert_eq!(15, sum_of_first_n(5));
        assert_eq!(55, sum_of_first_n(10));
    }

    #[test]
    fn test_upper_triangular_exclude_diagnonal() {
        let n = NonZeroUsize::new(4).expect("4 > 0");
        let mut ut = upper_triangular_exclude_diagonal(n).expect("4 > 1");
        assert_eq!(ut.next(), Some((0, 1)));
        assert_eq!(ut.next(), Some((0, 2)));
        assert_eq!(ut.next(), Some((0, 3)));
        assert_eq!(ut.next(), Some((1, 2)));
        assert_eq!(ut.next(), Some((1, 3)));
        assert_eq!(ut.next(), Some((2, 3)));
        assert_eq!(ut.next(), None);

        // ut.last();
        // assert_eq!(ut.next(), (4 - 1, 4 - 1));

        let n = NonZeroUsize::new(1).expect("1 > 0");
        let ut = upper_triangular_exclude_diagonal(n);
        assert!(ut.is_none());
    }

    #[test]
    fn test_upper_triangular() {
        let n = NonZeroUsize::new(4).expect("4 > 0");
        let mut ut = upper_triangular(n);
        assert_eq!(ut.next(), Some((0, 0)));
        assert_eq!(ut.next(), Some((0, 1)));
        assert_eq!(ut.next(), Some((0, 2)));
        assert_eq!(ut.next(), Some((0, 3)));
        assert_eq!(ut.next(), Some((1, 1)));
        assert_eq!(ut.next(), Some((1, 2)));
        assert_eq!(ut.next(), Some((1, 3)));
        assert_eq!(ut.next(), Some((2, 2)));
        assert_eq!(ut.next(), Some((2, 3)));
        assert_eq!(ut.next(), Some((3, 3)));
        assert_eq!(ut.next(), None);

        let n = NonZeroUsize::new(1).expect("1 > 0");
        let mut ut = upper_triangular(n);
        assert_eq!(ut.next(), Some((0, 0)));
        assert_eq!(ut.next(), None);
    }
}
