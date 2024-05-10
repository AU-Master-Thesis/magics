use crate::{GbpSchedule, GbpScheduleAtTimestep, GbpScheduleConfig, GbpScheduleIterator};

pub struct HalfBeginningHalfEnd;

mod private {
    pub struct HalfBeginningHalfEndIter {
        n:     u8,
        max:   u8,
        index: u8,
    }

    impl HalfBeginningHalfEndIter {
        pub fn new(n: u8, max: u8) -> Self {
            assert!(n <= max, "n must be less than or equal to max");
            Self { n, max, index: 0 }
        }
    }

    impl Iterator for HalfBeginningHalfEndIter {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.max {
                return None;
            }

            // Calculate the number of `true` to be at beginning and end
            let half_n = self.n / 2;
            let remainder = self.n % 2; // This handles cases where n is odd

            // Calculate start and end indexes for `false`
            let start_middle = half_n;
            let end_middle = self.max - half_n - remainder;

            // Determine the result based on the current index
            let result = if self.index < start_middle || self.index >= end_middle {
                true
            } else {
                false
            };

            self.index += 1;
            Some(result)
        }
    }
}

pub struct HalfBeginningHalfEndIter {
    // internal: private::HalfBeginningHalfEndIter,
    // external: private::HalfBeginningHalfEndIter,
    iter: std::iter::Zip<private::HalfBeginningHalfEndIter, private::HalfBeginningHalfEndIter>,
}

impl HalfBeginningHalfEndIter {
    pub fn new(config: GbpScheduleConfig) -> Self {
        let max = config.max();
        Self {
            iter: private::HalfBeginningHalfEndIter::new(config.internal, max)
                .zip(private::HalfBeginningHalfEndIter::new(config.external, max)), /* internal: private::HalfBeginningHalfEndIter::new(config.internal, max),
                                                                                     * external: private::HalfBeginningHalfEndIter::new(config.external, max), */
        }
    }
}

impl Iterator for HalfBeginningHalfEndIter {
    type Item = GbpScheduleAtTimestep;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(internal, external)| GbpScheduleAtTimestep { internal, external })
        // match self.iter.next() {
        //     // match (self.internal.next(), self.external.next()) {
        //     None => None,
        //     (Some(internal), Some(external)) => Some(GbpScheduleTimestep {
        // internal, external }),     _ => unreachable!("both iterators
        // have the same length"), }
    }
}

impl GbpScheduleIterator for HalfBeginningHalfEndIter {}

impl GbpSchedule for HalfBeginningHalfEnd {
    fn schedule(config: GbpScheduleConfig) -> impl GbpScheduleIterator {
        HalfBeginningHalfEndIter::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn ts(internal: bool, external: bool) -> GbpScheduleAtTimestep {
        GbpScheduleAtTimestep { internal, external }
    }

    #[test]
    fn internal_greater_than_external() {
        let config = GbpScheduleConfig {
            internal: 10,
            external: 5,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, false)));

        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, true)));

        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));

        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_less_than_external() {
        let config = GbpScheduleConfig {
            internal: 4,
            external: 6,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));

        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_external_even() {
        let config = GbpScheduleConfig {
            internal: 3,
            external: 3,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
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
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_zero_external_not() {
        let config = GbpScheduleConfig {
            internal: 0,
            external: 2,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
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
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
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
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn edge_cases() {
        let config = GbpScheduleConfig {
            internal: 1,
            external: 2,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);

        let config = GbpScheduleConfig {
            internal: 3,
            external: 2,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);

        let config = GbpScheduleConfig {
            internal: 2,
            external: 5,
        };
        let mut schedule = HalfBeginningHalfEnd::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), None);
    }
}
