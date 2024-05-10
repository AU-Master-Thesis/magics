use crate::{GbpSchedule, GbpScheduleAtTimestep, GbpScheduleConfig, GbpScheduleIter};

pub struct InterleaveEvenly;

mod private {
    pub struct InterleaveEvenlyIter {
        seq:   Vec<bool>,
        index: usize,
        max:   u8, /* n: u8,
                    * max: u8,
                    * current: f32,
                    * increment: f32,
                    * index: u8,
                    * zero: bool, */
    }

    const fn odd(n: u8) -> bool {
        // n & 0b1 == 1
        n % 2 == 1
    }

    const fn even(n: u8) -> bool {
        n % 2 == 0
    }

    enum OddEven {
        Odd,
        Even,
    }

    impl OddEven {
        const fn new(n: u8) -> Self {
            if n % 2 == 0 {
                Self::Even
            } else {
                Self::Odd
            }
        }
    }

    impl InterleaveEvenlyIter {
        fn recurse(slice: &mut [bool], n: u8) {
            use OddEven::{Even, Odd};
            let max = slice.len() as u8;
            let diff = max - n;
            let half = (max / 2) as usize;

            if n == max {
                for i in 0..slice.len() {
                    slice[i] = true;
                }
                return;
            }

            if n == 0 {
                for i in 0..slice.len() {
                    slice[i] = false;
                }
                return;
            }

            match (OddEven::new(n), OddEven::new(max), OddEven::new(diff)) {
                // 5, 7, 2
                (Odd, Odd, Even) => {
                    slice[half] = true;
                    let lower = n / 2;
                    let upper = lower;
                    assert_eq!(lower + upper, n - 1);

                    Self::recurse(&mut slice[0..half], lower);
                    Self::recurse(&mut slice[half..], upper);
                }
                // 4, 6, 2
                (Even, Even, Even) => {
                    let lower = n / 2;
                    let upper = lower;
                    assert_eq!(lower + upper, n);

                    Self::recurse(&mut slice[0..half], lower);
                    Self::recurse(&mut slice[half..], upper);
                }
                // 6, 7, 1
                (Even, Odd, Odd) => {
                    let lower = n / 2;
                    let upper = lower;
                    assert_eq!(lower + upper, n);

                    Self::recurse(&mut slice[0..half], lower);
                    Self::recurse(&mut slice[(half + 1)..], upper);
                }
                // 7, 8, 1
                (Odd, Even, Odd) => {
                    let lower = n / 2;
                    let upper = n - lower;
                    assert_eq!(lower + upper, n);

                    Self::recurse(&mut slice[0..half], lower);
                    Self::recurse(&mut slice[half..], upper);
                }
                _ => unreachable!(),
            }
        }

        pub fn new(n: u8, max: u8) -> Self {
            assert!(n <= max, "n must be less than or equal to max");
            let diff = max - n;

            let seq: Vec<bool> = if diff == max {
                vec![false; max as usize]
            } else if diff == 0 {
                vec![true; max as usize]
            } else {
                let mut seq = vec![true; max as usize];
                Self::recurse(&mut seq, n);
                dbg!(&seq);

                assert_eq!(seq.iter().filter(|&b| *b).count(), n as usize);

                seq
            };

            assert_eq!(seq.len(), max as usize);

            Self { seq, index: 0, max }
        }
    }

    impl Iterator for InterleaveEvenlyIter {
        type Item = bool;

        // fn next(&mut self) -> Option<Self::Item> {
        //     if self.index > self.max {
        //         return None;
        //     }

        //     if self.zero {
        //         return Some(false);
        //     }

        //     let on = if self.current < self.index as f32 {
        //         self.current += self.increment;
        //         true
        //     } else {
        //         false
        //     };

        //     self.index += 1;

        //     Some(on)
        // }

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(item) = self.seq.get(self.index) {
                self.index += 1;
                Some(*item)
            } else {
                None
            }
        }

        // fn next(&mut self) -> Option<Self::Item> {
        //     if self.index >= self.max {
        //         return None;
        //     }

        //     let half_diff = (self.max - self.n) / 2;
        //     let start = half_diff;
        //     let end = start + self.n - 1;

        //     // Determine the value based on the index
        //     let result = if self.index >= start && self.index <= end {
        //         // Within the range of 'n' values, we alternate starting from the
        // first true         // index
        //         (self.index - start) % 2 == 0
        //     } else {
        //         // Outside the range, we invert the alternating pattern based on the
        // distance         // from the start/end
        //         if self.index < start {
        //             (start - self.index) % 2 == 1
        //         } else {
        //             (self.index - end) % 2 == 1
        //         }
        //     };

        //     self.index += 1;
        //     Some(result)
        // }
    }
}

pub struct InterleaveEvenlyIter {
    iter: std::iter::Zip<private::InterleaveEvenlyIter, private::InterleaveEvenlyIter>,
}

impl InterleaveEvenlyIter {
    pub fn new(config: GbpScheduleConfig) -> Self {
        let max = config.max();
        let internal = private::InterleaveEvenlyIter::new(config.internal, max);
        let external = private::InterleaveEvenlyIter::new(config.external, max);
        let iter = internal.zip(external);
        Self { iter }
    }
}

impl Iterator for InterleaveEvenlyIter {
    type Item = GbpScheduleAtTimestep;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(internal, external)| GbpScheduleAtTimestep { internal, external })
    }
}

impl ExactSizeIterator for InterleaveEvenlyIter {}

impl GbpScheduleIter for InterleaveEvenlyIter {}

impl GbpSchedule for InterleaveEvenly {
    fn schedule(config: GbpScheduleConfig) -> impl GbpScheduleIter {
        InterleaveEvenlyIter::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn ts(internal: bool, external: bool) -> GbpScheduleAtTimestep {
        GbpScheduleAtTimestep { internal, external }
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
        let config = GbpScheduleConfig {
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
        let config = GbpScheduleConfig {
            internal: 0,
            external: 0,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_zero_external_not() {
        let config = GbpScheduleConfig {
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
        let config = GbpScheduleConfig {
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
        let config = GbpScheduleConfig {
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
        let config = GbpScheduleConfig {
            internal: 6,
            external: 7,
        };
        let mut schedule = InterleaveEvenly::schedule(config);
        println!("{:?}", schedule.collect::<Vec<_>>());
        assert!(false);

        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), Some(ts(false, true)));
        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), Some(ts(true, true)));
        // assert_eq!(schedule.next(), None);
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
