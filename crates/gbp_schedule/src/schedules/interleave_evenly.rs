use crate::{GbpSchedule, GbpScheduleAtIteration, GbpScheduleIterator, GbpScheduleParams};

pub struct InterleaveEvenly;

mod private {
    const fn odd(n: u8) -> bool {
        // n & 0b1 == 1
        n % 2 == 1
    }

    const fn even(n: u8) -> bool {
        n % 2 == 0
    }

    const fn divides(n: u8, d: u8) -> bool {
        n % d == 0
    }

    // enum OddEven {
    //     Odd,
    //     Even,
    // }
    //
    // impl OddEven {
    //     const fn new(n: u8) -> Self {
    //         if n % 2 == 0 {
    //             Self::Even
    //         } else {
    //             Self::Odd
    //         }
    //     }
    // }

    pub(super) struct InterleaveEvenlyIter {
        seq:   Vec<bool>,
        index: usize,
        max:   u8,
    }

    impl InterleaveEvenlyIter {
        fn recurse(slice: &mut [bool], n: u8) {
            let max = slice.len() as u8;
            // let diff = max - n;
            let half = (max / 2) as usize;

            if n == max {
                slice.fill(true);
            } else if n == 0 {
                slice.fill(false);
            } else {
                if odd(n) && odd(max) {
                    if divides(max, n) {
                        let times_divided = max / n;
                        // Example max = 9, n = 3
                        // times_divided = 3
                        // seq = [true, false, false, true, false, false,
                        // true, false, false]
                        let mut iter = std::iter::once(true)
                            .chain(std::iter::repeat(false).take(times_divided as usize - 1))
                            .cycle();
                        slice.fill_with(|| iter.next().unwrap());
                    } else {
                        let n = n / 2;

                        Self::recurse(&mut slice[0..half], n);
                        slice[half] = true;
                        Self::recurse(&mut slice[half + 1..], n);
                        slice[half + 1..].reverse();
                    }
                } else if even(n) && odd(max) {
                    let n = n / 2;
                    Self::recurse(&mut slice[0..half], n);
                    slice[0..half].reverse();
                    slice[half] = false;
                    Self::recurse(&mut slice[half + 1..], n);
                } else if even(n) && even(max) {
                    if divides(max, n) {
                        let times_divided = max / n;
                        // Example max = 8, n = 4
                        // times_divided = 2
                        // seq = [true, false, true, false, true, false, true, false]
                        let mut iter = std::iter::once(true)
                            .chain(std::iter::repeat(false).take(times_divided as usize - 1))
                            .cycle();
                        slice.fill_with(|| iter.next().unwrap());
                    } else {
                        let n = n / 2;
                        Self::recurse(&mut slice[0..half], n);
                        Self::recurse(&mut slice[half..], n);
                    }
                } else {
                    // odd(n) && even(max)
                    // Example max = 8, n = 3
                    let n = n / 2;
                    Self::recurse(&mut slice[0..half], n + 1);
                    slice[0..half].reverse();
                    Self::recurse(&mut slice[half..], n);
                }
            }
        }

        pub fn new(n: u8, max: u8) -> Self {
            assert!(n <= max, "n must be less than or equal to max");

            let mut seq = vec![false; max as usize];
            Self::recurse(&mut seq, n);

            Self { seq, index: 0, max }
        }
    }

    impl Iterator for InterleaveEvenlyIter {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(item) = self.seq.get(self.index).copied() {
                self.index += 1;
                Some(item)
            } else {
                None
            }
        }
    }
}

pub struct InterleaveEvenlyIter {
    iter: std::iter::Zip<private::InterleaveEvenlyIter, private::InterleaveEvenlyIter>,
}

impl InterleaveEvenlyIter {
    pub fn new(config: GbpScheduleParams) -> Self {
        let max = config.max();
        let internal = private::InterleaveEvenlyIter::new(config.internal, max);
        let external = private::InterleaveEvenlyIter::new(config.external, max);
        let iter = internal.zip(external);
        Self { iter }
    }
}

impl Iterator for InterleaveEvenlyIter {
    type Item = GbpScheduleAtIteration;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(internal, external)| GbpScheduleAtIteration { internal, external })
    }
}

impl ExactSizeIterator for InterleaveEvenlyIter {}

impl GbpScheduleIterator for InterleaveEvenlyIter {}

impl GbpSchedule for InterleaveEvenly {
    fn schedule(config: GbpScheduleParams) -> impl GbpScheduleIterator {
        InterleaveEvenlyIter::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn ts(internal: bool, external: bool) -> GbpScheduleAtIteration {
        GbpScheduleAtIteration { internal, external }
    }

    // #[test]
    // fn internal_greater_than_external() {
    //     let config = GbpScheduleConfig {
    //         internal: 10,
    //         external: 5,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));

    //     assert_eq!(schedule.next(), None);
    // }

    // #[test]
    // fn internal_less_than_external() {
    //     let config = GbpScheduleConfig {
    //         internal: 4,
    //         external: 6,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));

    //     assert_eq!(schedule.next(), None);
    // }

    #[test]
    fn internal_external_even() {
        let config = GbpScheduleParams {
            internal: 3,
            external: 3,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn both_zero() {
        let config = GbpScheduleParams {
            internal: 0,
            external: 0,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_zero_external_not() {
        let config = GbpScheduleParams {
            internal: 0,
            external: 2,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn external_zero_internal_not() {
        let config = GbpScheduleParams {
            internal: 2,
            external: 0,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn both_one() {
        let config = GbpScheduleParams {
            internal: 1,
            external: 1,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);
    }

    // #[test]
    // fn five_seven() {
    //     let config = GbpScheduleConfig {
    //         internal: 5,
    //         external: 7,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), None);
    // }

    #[test]
    fn six_seven() {
        let config = GbpScheduleParams {
            internal: 6,
            external: 7,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        // println!("{:?}", schedule.collect::<Vec<_>>());
        // assert!(false);

        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);
    }

    // #[test]
    // fn edge_cases() {
    //     let config = GbpScheduleConfig {
    //         internal: 1,
    //         external: 2,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), None);

    //     let config = GbpScheduleConfig {
    //         internal: 3,
    //         external: 2,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, false)));
    //     assert_eq!(schedule.next(), None);

    //     let config = GbpScheduleConfig {
    //         internal: 2,
    //         external: 5,
    //     };
    //     let mut schedule = InterleaveEvenly::schedule(config);
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(false, true)));
    //     assert_eq!(schedule.next(), Some(ts(true, true)));
    //     assert_eq!(schedule.next(), None);
    // }
}
