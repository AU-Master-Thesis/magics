use crate::{GbpSchedule, GbpScheduleAtTimestep, GbpScheduleConfig, GbpScheduleIterator};

pub struct LateAsPossible;

// #[derive(Debug)]
pub struct LateAsPossibleIter {
    // max: u8,
    // config: GbpScheduleConfig,
    // internal: Option<bool>,
    // external: Option<bool>,
    // i: u8,
    iter: std::iter::Zip<private::LateAsPossibleIter, private::LateAsPossibleIter>,
}

mod private {

    pub(super) struct LateAsPossibleIter {
        n:     u8,
        max:   u8,
        index: u8,
    }

    impl LateAsPossibleIter {
        pub fn new(n: u8, max: u8) -> Self {
            assert!(n <= max, "n must be less than or equal to max");
            Self { n, max, index: 0 }
        }
    }

    impl Iterator for LateAsPossibleIter {
        type Item = bool;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.max {
                return None;
            }

            let result = if self.n == self.max {
                true
            } else if self.n == 0 {
                false
            } else {
                self.index >= self.max - self.n
            };

            // let result = self.index >= self.n;
            self.index += 1;
            Some(result)
        }
    }
}

impl LateAsPossibleIter {
    pub fn new(config: GbpScheduleConfig) -> Self {
        // use std::iter::repeat_n;
        // let max = config.max() as usize;
        let max = config.max();
        // let internal = if config.internal == 0 {
        //     repeat_n(false, max)
        // } else if config.external == max {
        //     repeat_n(true, max)
        // } else {
        //     repeat_n(false, max - config.internal as usize)
        //         .chain(repeat_n(true, config.internal as usize))
        // };
        let internal = private::LateAsPossibleIter::new(config.internal, max);
        let external = private::LateAsPossibleIter::new(config.external, max);

        let iter = internal.zip(external);
        Self { iter }
    }
}

impl std::iter::Iterator for LateAsPossibleIter {
    type Item = GbpScheduleAtTimestep;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(internal, external)| GbpScheduleAtTimestep { internal, external })
    }
}

impl GbpScheduleIterator for LateAsPossibleIter {}

impl GbpSchedule for LateAsPossible {
    fn schedule(config: GbpScheduleConfig) -> impl GbpScheduleIterator {
        LateAsPossibleIter::new(config)
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
        let mut schedule = LateAsPossible::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));

        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));

        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_less_than_external() {
        let config = GbpScheduleConfig {
            internal: 3,
            external: 6,
        };
        let mut schedule = LateAsPossible::schedule(config);
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));

        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));
        assert_eq!(schedule.next(), Some(ts(true, true)));

        assert_eq!(schedule.next(), None);

        let config = GbpScheduleConfig {
            internal: 3,
            external: 10,
        };
        let mut schedule = LateAsPossible::schedule(config);
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));
        assert_eq!(schedule.next(), Some(ts(false, true)));

        assert_eq!(schedule.next(), Some(ts(true, true)));
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
        let mut schedule = LateAsPossible::schedule(config);
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
        let mut schedule = LateAsPossible::schedule(config);
        assert_eq!(schedule.next(), None);
    }

    #[test]
    fn internal_zero_external_not() {
        let config = GbpScheduleConfig {
            internal: 0,
            external: 2,
        };
        let mut schedule = LateAsPossible::schedule(config);
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
        let mut schedule = LateAsPossible::schedule(config);
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), Some(ts(true, false)));
        assert_eq!(schedule.next(), None);
    }
}
