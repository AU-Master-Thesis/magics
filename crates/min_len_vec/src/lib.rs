#![warn(missing_docs)]
//! A simple crate for a vector with a minimum length.

use serde::{Deserialize, Deserializer, Serialize};

/// A vector with a minimum length.
/// It is a wrapper around `Vec<T>` that ensures that the vector has at least
/// `N` elements. It is useful when you want to ensure that a vector has at
/// least a certain number of elements. but don't want to check it every time
/// you access the vector.
#[derive(Debug)]
pub struct MinLenVec<T, const N: usize>(Vec<T>);

/// Error type for `MinLenVec`.
#[derive(Debug, PartialEq)]
pub enum MinLenVecError {
    /// Not enough elements in the vector.
    /// Happens when you call [`MinLenVec::new`] with a vector that has less
    /// than N elements. Or when you call [`MinLenVec::pop`] and the vector
    /// has exactly N elements.
    NotEnoughElements(usize),
}

impl std::fmt::Display for MinLenVecError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::NotEnoughElements(n) => write!(f, "Not enough elements, expected at least {}", n),
        }
    }
}

impl std::error::Error for MinLenVecError {}

/// Result type for `MinLenVec`.
pub type Result<T> = std::result::Result<T, MinLenVecError>;

impl<T, const N: usize> MinLenVec<T, N> {
    /// Create a new `MinLenVec` from a vector.
    /// Returns an error if the vector has less than `N` elements.
    pub fn new(data: Vec<T>) -> Result<Self> {
        if data.len() < N {
            return Err(MinLenVecError::NotEnoughElements(N));
        }
        Ok(Self(data))
    }

    /// Consume the `MinLenVec` and return the inner vector.
    #[inline(always)]
    pub fn into_inner(self) -> Vec<T> {
        self.0
    }

    /// Get the length of the vector.
    /// This is the length of the inner vector, not the minimum length.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    // #[inline(always)]
    // pub fn is_empty(&self) -> bool {
    //     self.0.is_empty()
    // }

    /// Get an iterator of &T to the elements of the vector.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }

    /// Take ownership of the vector and return an iterator of T.
    pub fn into_iter(self) -> std::vec::IntoIter<T> {
        self.0.into_iter()
    }

    /// Get a slice of the elements of the vector.
    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Get a mutable slice of the elements of the vector.
    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }

    /// Push an element to the vector.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    /// Pop an element from the vector.
    /// Returns an error if the vector has exactly `N` elements.
    /// This is to ensure the invariant that the vector always has at least `N`
    /// elements.
    #[inline(always)]
    pub fn pop(&mut self) -> Result<T> {
        if self.0.len() <= N {
            return Err(MinLenVecError::NotEnoughElements(N));
        }
        Ok(self.0.pop().expect("there is always at least N elements"))
    }

    /// Get a reference to the first element of the vector.
    /// Since the vector has at least `N` elements, this will always return a
    /// first element.
    #[inline(always)]
    pub fn first(&self) -> &T {
        &self.0[0]
    }

    /// Get a reference to the last element of the vector.
    /// Since the vector has at least `N` elements, this will always return a
    /// last element.
    #[inline(always)]
    pub fn last(&self) -> &T {
        &self.0[self.0.len() - 1]
    }
}

impl<T, const N: usize> std::ops::Index<usize> for MinLenVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T, const N: usize> std::ops::IndexMut<usize> for MinLenVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for MinLenVec<T, N> {
    type Error = MinLenVecError;

    fn try_from(value: Vec<T>) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<T, const N: usize> From<MinLenVec<T, N>> for Vec<T> {
    fn from(value: MinLenVec<T, N>) -> Self {
        value.into_inner()
    }
}

impl<T: Clone, const N: usize> From<[T; N]> for MinLenVec<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::new(value.to_vec()).expect("there are always N elements")
    }
}

impl<T, const N: usize> Serialize for MinLenVec<T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for MinLenVec<T, N>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        Self::new(v).map_err(serde::de::Error::custom)
    }
}

/// A type alias for a `MinLenVec` with a minimum length of 1.
pub type OneOrMore<T> = MinLenVec<T, 1>;
/// A type alias for a `MinLenVec` with a minimum length of 2.
pub type TwoOrMore<T> = MinLenVec<T, 2>;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_min_len_vec() {
        assert!(matches!(
            MinLenVec::<_, 3>::new(vec![1, 2]),
            Err(MinLenVecError::NotEnoughElements(3))
        ));

        assert!(matches!(
            MinLenVec::<_, 3>::new(vec![1, 2, 3]),
            Ok(MinLenVec(_))
        ));

        assert!(matches!(
            MinLenVec::<_, 1>::new(vec![1.0]),
            Ok(MinLenVec(_))
        ));
        assert!(matches!(
            MinLenVec::<i32, 1>::new(vec![]),
            Err(MinLenVecError::NotEnoughElements(1))
        ));
    }

    #[test]
    fn test_min_len_vec_push() {
        let mut v = MinLenVec::<_, 3>::new(vec![1, 2, 3]).unwrap();
        assert_eq!(v.len(), 3);
        v.push(4);
        assert_eq!(v.len(), 4);
        v.push(5);
        assert_eq!(v.len(), 5);
    }

    #[test]
    fn test_min_len_vec_pop() {
        let mut v = MinLenVec::<_, 3>::new(vec![1, 2, 3, 4, 5]).unwrap();
        assert_eq!(v.len(), 5);
        assert_eq!(v.pop(), Ok(5));
        assert_eq!(v.len(), 4);
        assert_eq!(v.pop(), Ok(4));
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Err(MinLenVecError::NotEnoughElements(3)));
        assert_eq!(v.len(), 3);
        assert_eq!(v.pop(), Err(MinLenVecError::NotEnoughElements(3)));
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_min_len_vec_index() {
        let v = MinLenVec::<_, 3>::new(vec![1, 2, 3]).unwrap();
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn test_min_len_vec_into_inner() {
        let v = MinLenVec::<_, 3>::new(vec![1, 2, 3]).unwrap();
        let inner = v.into_inner();
        assert_eq!(inner, vec![1, 2, 3]);
    }

    #[test]
    fn test_min_len_vec_from() {
        let v = MinLenVec::<_, 3>::from([1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn test_first() {
        let v = MinLenVec::<_, 3>::new(vec![1, 2, 3]).unwrap();
        assert_eq!(v.first(), &1);
    }

    #[test]
    fn test_last() {
        let v = MinLenVec::<_, 4>::new(vec![1, 2, 3, 4]).unwrap();
        assert_eq!(v.last(), &4);
    }
}
