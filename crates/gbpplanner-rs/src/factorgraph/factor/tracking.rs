//! Tracking Factor (extension)

use std::{borrow::Cow, cell::Cell, sync::Mutex};

use bevy::{math::Vec2, utils::smallvec::ToSmallVec};
use gbp_linalg::prelude::*;
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};

use super::{Factor, FactorState, Measurement};

/// Tracking information for each tracking factor to follow
#[derive(Debug)]
pub struct Tracking {
    /// The path to follow
    path:   Vec<Vec2>,
    /// Which index in the path, the horizon is currently moving towards
    index:  usize,
    /// Tracking record
    /// Implicitly tells which waypoint the factor has reached
    /// e.g. if the record is 3, the factor has been to waypoint 1, 2, and 3
    record: Mutex<Cell<usize>>,
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            path:   Vec::new(),
            index:  0,
            record: Default::default(),
        }
    }
}

impl Tracking {
    pub fn with_path(mut self, path: Vec<Vec2>) -> Self {
        self.path = path;
        self
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
    pub fn new(tracking_path: Vec<Vec2>) -> Self {
        assert!(
            tracking_path.len() >= 2,
            "Tracking path must have at least 2 points"
        );
        Self {
            // tracking_path,
            // tracking: Tracking::default(),
            tracking: Tracking::default().with_path(tracking_path),
            // last_measurement: Mutex::new(Cell::new(Vec2::ZERO)),
            last_measurement: Default::default(),
        }
    }

    pub fn with_last_measurement(mut self, pos: Vec2, value: Float) -> Self {
        self.last_measurement
            .lock()
            .unwrap()
            .set(LastMeasurement { pos, value });

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
    fn jacobian(&self, state: &FactorState, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        // Same as PoseFactor
        // TODO: change to not clone x
        Cow::Owned(self.first_order_jacobian(state, x.clone()))
    }

    // fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Measurement {
        println!("x: {}", x);
        let x_pos = x.slice(s![0..2]).to_owned();

        // 1. Find which line in the `self.tracking.path` to project to, based off of
        //    the `self.tracking.record` e.g. if `self.tracking.record` is 3, then track
        //    the line between waypoint 3 and 4
        let current_record = self.tracking.record.lock().unwrap().get();
        let [start, end]: [Vec2; 2] = self
            .tracking
            .path
            .windows(2)
            .nth(current_record)
            .unwrap()
            .try_into()
            .expect("My ass");
        let (start, end) = (array![start.x as Float, start.y as Float], array![
            end.x as Float,
            end.y as Float
        ]);

        // 2. Project the current position onto the line between `start` and `end`
        let line = &end - &start;
        // let projected_point = x_pos.project_onto(line);

        // let projected = &p1 + (&x_pos - &p1).dot(&line) / &line.dot(&line) * &line;
        let projected_point = &start + (&x_pos - &start).dot(&line) / &line.dot(&line) * &line;

        // 3. if within 2m of end, increment `self.tracking.record`
        let projected_point_to_end = (&end - &projected_point).l1_norm();
        println!(
            "current_record: {}, distance to end: {}, lin: {}",
            current_record, projected_point_to_end, x_pos
        );
        if projected_point_to_end < 2.0f64.powi(2) {
            println!("to end: {}", projected_point_to_end);
            self.tracking.record.lock().unwrap().set(current_record + 1)
        }

        let max_length = 1.0;

        // let projected_point = array![projected_point.x as f64, projected_point.y as
        // f64];
        let x_to_projection = &projected_point - x.slice(s![0..2]).to_owned();
        // clamp the distance to the max length
        let x_to_projection = if x_to_projection.euclidean_norm() > max_length {
            (x_to_projection.normalized() * max_length).normalized()
        } else {
            x_to_projection.normalized()
        };

        let measurement = x_to_projection.euclidean_norm();
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos:   Vec2::new(projected_point[0] as f32, projected_point[1] as f32),
            value: measurement,
        });

        Measurement::new(array![measurement]).with_position(concatenate![
            Axis(0),
            projected_point,
            x.slice(s![1..3]).to_owned()
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
        if self.tracking.path.is_empty() {
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
        writeln!(f, "tracking_path: {:?}", self.tracking)?;
        write!(f, "last_measurement: {:?}", self.last_measurement())
    }
}
