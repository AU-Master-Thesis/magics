//! Tracking Factor (extension)
use std::{borrow::Cow, cell::Cell, sync::Mutex};

use bevy::{math::Vec2, utils::smallvec::ToSmallVec};
use colored::Colorize;
use gbp_linalg::prelude::*;
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};

use super::{Factor, FactorState, Measurement};

/// Tracking information for each tracking factor to follow
#[derive(Debug)]
pub struct Tracking {
    /// The path to follow
    path:      Option<Vec<Vec2>>,
    /// Which index in the path, the horizon is currently moving towards
    index:     usize,
    /// Tracking record
    /// Implicitly tells which waypoint the factor has reached
    /// e.g. if the record is 3, the factor has been to waypoint 1, 2, and 3
    record:    Mutex<Cell<usize>>,
    /// The smoothing factor in meters
    smoothing: f64,
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            path:      None,
            index:     0,
            record:    Default::default(),
            smoothing: 10.0,
        }
    }
}

impl Tracking {
    pub fn with_path(mut self, path: Vec<Vec2>) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// Increments record, but clamped to the bounds of the path
    fn increment_record(&self) {
        let record = self.record.lock().unwrap();

        let new_record = if let Some(path) = &self.path {
            (record.get() + 1).min(path.len() - 2)
        } else {
            0
        };

        record.set(new_record);
    }
}

#[derive(Debug)]
pub struct TrackingFactor {
    /// Tracking information from global path finder
    tracking: Tracking,
    /// Most recent measurement
    last_measurement: Mutex<Cell<LastMeasurement>>,
}

#[derive(Debug, Clone, Copy)]
pub struct LastMeasurement {
    pub pos:   bevy::math::Vec2,
    pub value: Float,
}

impl Default for LastMeasurement {
    fn default() -> Self {
        Self {
            pos:   Vec2::ZERO,
            value: 0.0,
        }
    }
}

impl TrackingFactor {
    /// An obstacle factor has a single edge to another variable
    pub const NEIGHBORS: usize = 1;

    /// Creates a new [`TrackingFactor`].
    pub fn new(tracking_path: Option<min_len_vec::TwoOrMore<Vec2>>) -> Self {
        // assert!(
        //    tracking_path.len() >= 2,
        //    "Tracking path must have at least 2 points"
        //);

        // let mut tracking = Tracking::default();
        // if let Some(tracking_path) = tracking_path {
        //    tracking.with_path(tracking_path.into());
        //}
        Self {
            // tracking_path,
            // tracking: Tracking::default(),
            tracking: if let Some(path) = tracking_path {
                Tracking::default().with_path(path.into())
            } else {
                Tracking::default()
            },
            // tracking: Tracking::default().with_path(tracking_path),
            // last_measurement: Mutex::new(Cell::new(Vec2::ZERO)),
            last_measurement: Default::default(),
        }
    }

    pub fn with_last_measurement(self, pos: Vec2, value: Float) -> Self {
        self.last_measurement
            .lock()
            .unwrap()
            .set(LastMeasurement { pos, value });

        self
    }

    pub fn with_smoothing(mut self, smoothing: f64) -> Self {
        self.tracking.smoothing = smoothing;
        self
    }

    /// Get the last measurement
    pub fn last_measurement(&self) -> LastMeasurement {
        self.last_measurement.lock().unwrap().get()
    }

    /// Get the tracking path
    pub fn tracking(&self) -> &Tracking {
        &self.tracking
    }

    pub fn set_tracking_path(&mut self, tracking_path: min_len_vec::TwoOrMore<Vec2>) {
        self.tracking.path = Some(tracking_path.into());
    }
}

impl Factor for TrackingFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "TrackingFactor"
    }

    #[inline]
    fn color(&self) -> [u8; 3] {
        // #f4a15a
        [244, 161, 90]
    }

    #[inline]
    fn jacobian(
        &self,
        state: &FactorState,
        linearisation_point: &Vector<Float>,
    ) -> Cow<'_, Matrix<Float>> {
        // Same as PoseFactor
        // TODO: change to not clone x
        // Cow::Owned(self.first_order_jacobian(state, x.clone()))
        let mut linearisation_point = linearisation_point.to_owned();

        let h0 = array![self.last_measurement.lock().unwrap().get().value];
        let mut jacobian = Matrix::<Float>::zeros((h0.len(), linearisation_point.len()));

        let delta = self.jacobian_delta();

        for i in 0..linearisation_point.len() {
            linearisation_point[i] += delta; // perturb by delta
            let Measurement {
                value: h1,
                position: _,
            } = self.measure(state, &linearisation_point);
            let derivatives = (&h1 - &h0) / delta;
            jacobian.column_mut(i).assign(&derivatives);
            linearisation_point[i] -= delta; // reset the perturbation
        }

        Cow::Owned(jacobian)
    }

    // fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Measurement {
        let current_record = self.tracking.record.lock().unwrap().get();
        let x_pos = x.slice(s![0..2]).to_owned();

        println!("x_pos: {}", x_pos.to_string().truecolor(254, 100, 11));

        // 1. Find which line in the `self.tracking.path` to project to, based off of
        //    the `self.tracking.record` e.g. if `self.tracking.record` is 3, then track
        //    the line between waypoint 3 and 4
        let [start, end]: [Vec2; 2] = self
            .tracking
            .path
            .as_ref()
            .unwrap()
            .windows(2)
            .inspect(|it| println!("{it:?}"))
            .nth(dbg!(current_record))
            .unwrap()
            .try_into()
            .expect("My ass");
        let (start, end) = (array![start.x as Float, start.y as Float], array![
            end.x as Float,
            end.y as Float
        ]);

        println!("start: {}\nend: {}", start, end);

        // 2. Project the current position onto the line between `start` and `end`
        let line = &end - &start;
        let projected_point = &start + (&x_pos - &start).dot(&line) / &line.dot(&line) * &line;

        println!("projected: {}", projected_point.to_string().cyan());

        // 3. if within `self.tracking.smoothing` of end, increment
        //    `self.tracking.record`
        let projected_point_to_end = (&end - &projected_point).l1_norm();

        if projected_point_to_end < self.tracking.smoothing.powi(2) {
            // println!("incre", projected_point_to_end);
            self.tracking.increment_record();
        }

        let max_length = 1.0f64;

        // let projected_point = array![projected_point.x as f64, projected_point.y as
        // f64];
        let x_to_projection = &projected_point - &x_pos;
        // // clamp the distance to the max length
        println!("Euc norm: {}", x_to_projection.euclidean_norm());

        let measurement = 1.0 - f64::min(x_to_projection.euclidean_norm(), max_length);
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos:   Vec2::new(projected_point[0] as f32, projected_point[1] as f32),
            value: measurement,
        });

        println!("measurement: {}", measurement);

        Measurement::new(array![measurement]).with_position(concatenate![
            Axis(0),
            projected_point,
            x.slice(s![2..4]).to_owned()
        ])
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        // Same as DynamicFactor for now
        // TODO: Tune this
        // NOTE: Maybe this should be influenced by the distance from variable to the
        // measurement
        1e-2
    }

    #[inline(always)]
    fn skip(&self, _state: &FactorState) -> bool {
        // skip if `self.tracking.path` is empty
        if self.tracking.path.is_none()
            || self.tracking.record.lock().unwrap().get()
                >= self.tracking.path.as_ref().map_or(1, |p| p.len()) - 1
        {
            println!("Skipping factor because path is empty");
            return true;
        }
        false
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        false
    }

    #[inline(always)]
    fn neighbours(&self) -> usize {
        Self::NEIGHBORS
    }
}

impl std::fmt::Display for TrackingFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;

        if let Some(tracking_path) = &self.tracking.path {
            writeln!(f, "tracking_path: {}", tracking_path.len())?;
            let width = (tracking_path.len() as f32).log10().ceil() as usize;
            for (i, pos) in tracking_path.iter().enumerate() {
                writeln!(f, "  [{:>width$}] = [{:>6.2}, {:>6.2}]", i, pos.x, pos.y)?;
            }
        } else {
            writeln!(f, "tracking_path: {}", "None".red())?;
        }
        write!(f, "last_measurement: {:?}", self.last_measurement())
    }
}
