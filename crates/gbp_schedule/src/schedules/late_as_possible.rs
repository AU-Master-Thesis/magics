use crate::{GbpSchedule, GbpScheduleAtTimestep, GbpScheduleConfig, GbpScheduleIter};

pub struct LateAsPossible;

#[derive(Debug)]
pub struct LateAsPossibleIter {
    max: u8,
    config: GbpScheduleConfig,
    internal: Option<bool>,
    external: Option<bool>,
    i: u8,
}

impl LateAsPossibleIter {
    pub fn new(config: GbpScheduleConfig) -> Self {
        let max = config.max();
        Self {
            max,
            config,
            internal: if config.internal == max {
                Some(true)
            } else if config.internal == 0 {
                Some(false)
            } else {
                None
            },
            external: if config.external == max {
                Some(true)
            } else if config.external == 0 {
                Some(false)
            } else {
                None
            },
            // external: 0,
            i: 0,
        }
    }
}

impl std::iter::Iterator for LateAsPossibleIter {
    type Item = GbpScheduleAtTimestep;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.max {
            let ts = Some(GbpScheduleAtTimestep {
                internal: self.internal.unwrap_or(self.i >= self.config.internal),
                external: self.external.unwrap_or(self.i >= self.config.external),
            });
            self.i += 1;
            ts
        } else {
            None
        }
    }
}

impl GbpScheduleIter for LateAsPossibleIter {}

impl GbpSchedule for LateAsPossible {
    fn schedule(config: GbpScheduleConfig) -> impl GbpScheduleIter {
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
