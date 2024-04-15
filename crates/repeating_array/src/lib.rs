//! Container adapter to make it convenient to iterate over an array repeatedly
#![deny(missing_docs)]

/// Container adapter to make it convenient to iterate over an array repeatedly
#[derive(Debug)]
pub struct RepeatingArray<T, const N: usize> {
    /// The array to iterate over
    array: [T; N],
    /// The current index in the array, starting at 0
    index: usize,
}

impl<T: Copy, const N: usize> RepeatingArray<T, N> {
    /// Create a new `RepeatingArray`
    #[inline]
    #[must_use]
    pub const fn new(array: [T; N]) -> Self {
        Self { array, index: 0 }
    }

    /// Get the next item or the first one if we are at the end
    pub fn next_or_first(&mut self) -> T {
        let item = self.array[self.index];
        self.index = (self.index + 1) % N;
        item
    }

    /// Reset the index to 0
    #[inline(always)]
    pub fn reset(&mut self) {
        self.index = 0;
    }

    // /// Turn the RepeatingArray into an `Iterator`
    // pub fn into_iter(self) -> std::array::IntoIter<T, N> {
    //     self.array.into_iter()
    // }
}

// impl<T: Copy, const N: usize> std::iter::IntoIterator for RepeatingArray<T,
// N> {     type IntoIter = std::array::IntoIter<T, N>;
//     type Item = T;

//     fn into_iter(self) -> Self::IntoIter {
//         self.array.into_iter()
//     }
// }

impl<T: Copy, const N: usize> Iterator for RepeatingArray<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_or_first())
    }
}

impl<T: Copy, const N: usize> std::iter::ExactSizeIterator for RepeatingArray<T, N> {
    fn len(&self) -> usize {
        N
    }
}

impl<T, const N: usize> std::ops::Index<usize> for RepeatingArray<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.array[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for RepeatingArray<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.array[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    const fn can_be_constructed() {
        let _ = RepeatingArray::new([1, 2, 3, 4]);
    }

    #[test]
    fn can_be_iterated() {
        let mut array = RepeatingArray::new([1, 2, 3, 4]);
        assert_eq!(array.next_or_first(), 1);
        assert_eq!(array.next_or_first(), 2);
        assert_eq!(array.next_or_first(), 3);
        assert_eq!(array.next_or_first(), 4);
        assert_eq!(array.next_or_first(), 1);
    }

    #[test]
    fn can_be_indexed() {
        let array = RepeatingArray::new([1, 2, 3, 4]);
        assert_eq!(array[0], 1);
        assert_eq!(array[1], 2);
        assert_eq!(array[2], 3);
        assert_eq!(array[3], 4);
    }

    #[test]
    fn can_be_indexed_mut() {
        let mut array = RepeatingArray::new([1, 2, 3, 4]);
        array[0] = 5;
        array[1] = 6;
        array[2] = 7;
        array[3] = 8;
        assert_eq!(array[0], 5);
        assert_eq!(array[1], 6);
        assert_eq!(array[2], 7);
        assert_eq!(array[3], 8);
    }
}
