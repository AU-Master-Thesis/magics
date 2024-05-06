/// Iterate over the counters in an evenly interleaved fashion
///
/// # Examples
///
/// ```
/// use interleave_evenly::InterleaveEvenly;
/// let a = 10;
/// let b = 4;
/// let mut iter = InterleaveEvenly::new([a, b]);
/// assert_eq!(Some([true, true]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(Some([true, true]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(Some([true, true]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(Some([true, true]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(Some([true, false]), iter.next());
/// assert_eq!(None, iter.next());
/// ```
///
/// # Panics
///
/// This function will panic if `N` is not greater than 0
#[derive(Debug)]
pub struct InterleaveEvenly<const N: usize> {
    /// The accumulated state of each counter. Starts with `[0.0; N]`
    state: [f32; N],
    /// How much to increment the state of each counter whenever it is < self.i
    increments: [f32; N],
    /// The largest of all the counters
    max: usize,
    /// The current iteration index, starts at 1
    i: usize,
}

impl<const N: usize> InterleaveEvenly<N> {
    /// Create a new `InterleaveEvenly` iterator
    ///
    /// # Panics
    ///
    /// Panics if `N` == 0
    pub fn new(times: [usize; N]) -> Self {
        let max: usize = times.iter().max().copied().expect("N > 0");
        #[allow(clippy::cast_precision_loss)]
        let increments = std::array::from_fn(|i| max as f32 / times[i] as f32);

        Self {
            state: [0.0; N],
            increments,
            max,
            i: 1,
        }
    }
}

impl<const N: usize> std::iter::Iterator for InterleaveEvenly<N> {
    type Item = [bool; N];

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.max {
            return None;
        }

        let ready = std::array::from_fn(|k| {
            #[allow(clippy::cast_precision_loss)]
            if self.state[k] < self.i as f32 {
                self.state[k] += self.increments[k];
                true
            } else {
                false
            }
        });

        self.i += 1;

        Some(ready)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn a_greater_than_b() {
        let a = 10;
        let b = 4;

        let mut iter = InterleaveEvenly::new([a, b]);

        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(Some([true, false]), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn b_greater_than_a() {
        let a = 4;
        let b = 10;

        let mut iter = InterleaveEvenly::new([a, b]);

        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(Some([true, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(Some([false, true]), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn three_counters() {
        let a = 4;
        let b = 10;
        let c = 8;

        let mut iter = InterleaveEvenly::new([a, b, c]);

        assert_eq!(Some([true, true, true]), iter.next());
        assert_eq!(Some([false, true, true]), iter.next());
        assert_eq!(Some([true, true, true]), iter.next());
        assert_eq!(Some([false, true, true]), iter.next());
        assert_eq!(Some([false, true, false]), iter.next());
        assert_eq!(Some([true, true, true]), iter.next());
        assert_eq!(Some([false, true, true]), iter.next());
        assert_eq!(Some([true, true, true]), iter.next());
        assert_eq!(Some([false, true, true]), iter.next());
        assert_eq!(Some([false, true, false]), iter.next());
        assert_eq!(None, iter.next());
    }
}
