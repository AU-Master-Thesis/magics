//! Tracking Factor for path following
//!
//! This module implements the tracking factor which guides robots along
//! predefined paths in the planning algorithm.

use std::{borrow::Cow, cell::Cell, ops::Sub, sync::Mutex};

use gbp_linalg::{prelude::*, pretty_print_matrix};
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};

use super::{Factor, FactorState, Measurement};
use crate::{factorgraph::DOFS, types::Vector2};

/// Tracking information for each tracking factor to follow
#[derive(Debug)]
pub struct Tracking {
    /// The path to follow
    path: Option<Vec<Vector2>>,
    /// Which index in the path, the horizon is currently moving towards
    index: usize,
    /// Tracking record
    /// Implicitly tells which waypoint the factor has reached
    /// e.g. if the record is 3, the factor has been to waypoint 1, 2, and 3
    record: Mutex<Cell<usize>>,
    /// Amount of projects being considered
    connections: Mutex<Cell<usize>>,
    /// The tracking config from the `gbp_config` input `config.toml`
    pub config: gbp_config::TrackingSection,
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            path: None,
            index: 1,
            record: Mutex::new(Cell::new(0)),
            connections: Mutex::new(Cell::new(1)),
            config: gbp_config::TrackingSection::default(),
        }
    }
}

impl Tracking {
    /// Set the path for tracking
    pub fn with_path(mut self, path: Vec<Vector2>) -> Self {
        self.path = Some(path);
        self
    }

    /// Set the tracking configuration
    pub fn with_config(mut self, config: gbp_config::TrackingSection) -> Self {
        self.config = config;
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

    /// Get the current record value
    pub fn get_record(&self) -> usize {
        self.record.lock().unwrap().get()
    }
}

/// Factor for tracking a predefined path
#[derive(Debug)]
pub struct TrackingFactor {
    /// Tracking information from global path finder
    tracking: Tracking,
    /// Most recent measurement
    last_measurement: Mutex<Cell<LastMeasurement>>,
    /// Timeout counter for disabling the factor temporarily
    timeout: Mutex<Cell<Option<usize>>>,
}

/// Information about the last measurement
#[derive(Debug, Clone, Copy)]
pub struct LastMeasurement {
    /// Position of measurement
    pub pos: Vector2,
    /// Value of measurement (0-1)
    pub value: Float,
}

impl Default for LastMeasurement {
    fn default() -> Self {
        Self {
            pos: Vector2::new(0.0, 0.0),
            value: 0.0,
        }
    }
}

impl std::fmt::Display for LastMeasurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[pos: ({:.3}, {:.3}), value: {:.4}]",
            self.pos.x, self.pos.y, self.value
        )
    }
}

impl TrackingFactor {
    /// An tracking factor has a single edge to another variable
    pub const NEIGHBORS: usize = 1;

    /// Creates a new [`TrackingFactor`].
    pub fn new(tracking_path: Option<min_len_vec::TwoOrMore<Vector2>>) -> Self {
        Self {
            tracking: Tracking::default()
                .with_path(tracking_path.map_or_else(Vec::new, |p| p.into())),
            last_measurement: Default::default(),
            timeout: Default::default(),
        }
    }

    /// Set the last measurement
    pub fn with_last_measurement(self, pos: Vector2, value: Float) -> Self {
        self.last_measurement
            .lock()
            .unwrap()
            .set(LastMeasurement { pos, value });

        self
    }

    /// Set the tracking configuration
    pub fn with_config(mut self, config: gbp_config::TrackingSection) -> Self {
        self.tracking.config = config;
        self
    }

    /// Get the last measurement
    pub fn last_measurement(&self) -> LastMeasurement {
        self.last_measurement.lock().unwrap().get()
    }

    /// Get the tracking path information
    pub fn tracking(&self) -> &Tracking {
        &self.tracking
    }

    /// Set a new tracking path
    pub fn set_tracking_path(&mut self, tracking_path: min_len_vec::TwoOrMore<Vector2>) {
        self.tracking.path = Some(tracking_path.into());
    }

    /// Set the current index in the tracking path
    pub fn set_tracking_index(&mut self, index: usize) {
        if let Some(path) = &self.tracking.path {
            assert!(index < path.len());
            self.tracking.index = index;
        }
    }

    /// Set the linearisation point
    pub fn set_linearisation_point(&mut self, mean: Vector2) {
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos: mean,
            value: 0.0,
        });
    }

    /// Set a timeout for the factor
    pub fn set_timeout(&mut self, iterations: usize) {
        self.timeout.lock().unwrap().set(Some(iterations));
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
        let last_measurement = self.last_measurement.lock().unwrap().get();
        let h0 = array![last_measurement.value];

        let m = last_measurement.pos;
        let pos = linearisation_point.slice(s![..DOFS / 2]).to_owned();

        let temp = array![m.x, m.y];
        let x_diff = pos - temp;

        let mut jacobian = Matrix::<Float>::zeros((h0.len(), DOFS));
        jacobian
            .slice_mut(s![0, ..DOFS / 2])
            .assign(&(1.0 / h0 * &x_diff));

        Cow::Owned(jacobian)
    }

    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Measurement {
        let current_record = self.tracking.record.lock().unwrap().get();
        let x_pos = x.slice(s![0..2]).to_owned();
        let x_vel = x.slice(s![2..4]).to_owned();

        // Find which line in the path to project to
        let lines = self
            .tracking
            .path
            .as_ref()
            .unwrap()
            .windows(2)
            .collect::<Vec<_>>();

        let [current_start, current_end]: &[Vector2; 2] = lines[current_record].try_into().unwrap();
        let (current_start, current_end) = (
            array![current_start.x, current_start.y],
            array![current_end.x, current_end.y],
        );

        // Project the current position onto the line
        let line = &current_end - &current_start;
        let current_projection =
            &current_start + (&x_pos - &current_start).dot(&line) / &line.dot(&line) * &line;

        let consideration_distance = {
            let d = self.tracking.config.switch_padding as Float;
            if self
                .tracking
                .index
                .saturating_sub(self.tracking.get_record())
                > 1
            {
                (d, d * 0.01)
            } else {
                (d, d * 0.01)
            }
        };

        let current_projected_point_to_current_end =
            (&current_end - &current_projection).euclidean_norm();

        // Get projection to previous line only if there is a previous line
        let projection_to_previous = if current_record > 0 {
            // Retrieve the previous line
            let [previous_start, previous_end]: &[Vector2; 2] =
                lines[current_record - 1].try_into().unwrap();
            let (previous_start, previous_end) = (
                array![previous_start.x, previous_start.y],
                array![previous_end.x, previous_end.y],
            );

            // Project the linearisation point onto the line
            let line = &previous_end - &previous_start;
            let previous_projected_point =
                &previous_start + (&x_pos - &previous_start).dot(&line) / &line.dot(&line) * &line;

            // Check if the projected is to be considered
            let current_projected_point_to_previous_end =
                (&previous_end - &current_projection).euclidean_norm();
            let previous_projected_point_to_previous_end =
                (&current_start - &previous_projected_point).euclidean_norm();

            if current_projected_point_to_previous_end < consideration_distance.0
                && current_projected_point_to_previous_end > consideration_distance.1
                && previous_projected_point_to_previous_end < consideration_distance.0
            {
                Some((
                    previous_projected_point,
                    current_projected_point_to_previous_end,
                    previous_projected_point_to_previous_end,
                ))
            } else {
                None
            }
        } else {
            None
        };

        // If close to the end of current line segment, increment record
        if current_projected_point_to_current_end < consideration_distance.0 {
            self.tracking.increment_record();
        }

        // Calculate measurement point based on projections
        let measurement_point = match projection_to_previous {
            Some((previous_projection, _, _)) => {
                // connections should be 2
                self.tracking.connections.lock().unwrap().set(2);

                // vector from `x_pos` to `current_projection`
                let x_to_current = &current_projection - &x_pos;
                // vector from `x_pos` to `previous_projection`
                let x_to_previous = &previous_projection - &x_pos;

                let measurement_vector = &x_to_current + &x_to_previous;
                &x_pos + &measurement_vector
            }
            None => {
                // connections should be 1
                self.tracking.connections.lock().unwrap().set(1);
                current_projection + line.normalized() * x_vel.euclidean_norm() as Float / 5.0
            }
        };

        // Normalize distance
        let x_to_projection = &measurement_point - &x_pos;
        let x_to_projection_distance = x_to_projection.euclidean_norm();
        let attraction_distance_f64 = self.tracking.config.attraction_distance as f64;
        let normalised_distance = if x_to_projection_distance < attraction_distance_f64 {
            x_to_projection_distance / attraction_distance_f64
        } else {
            1.0
        };

        // Use normalized distance as measurement
        let measurement = normalised_distance;

        // Update last measurement and return
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos: Vector2::new(measurement_point[0], measurement_point[1]),
            value: measurement,
        });

        Measurement::new(array![measurement]).with_position(concatenate![
            Axis(0),
            measurement_point,
            x.slice(s![2..4]).to_owned()
        ])
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        1e-8
    }

    #[inline(always)]
    fn skip(&self, _state: &FactorState) -> bool {
        let timeout = self.timeout.lock().unwrap();
        if let Some(left) = timeout.get() {
            if left == 0 {
                timeout.set(None);
            } else {
                timeout.set(Some(left - 1));
                return true;
            }
        }
        // skip if `self.tracking.path` is empty
        if self.tracking.path.is_none()
            || self.tracking.record.lock().unwrap().get()
                >= self.tracking.path.as_ref().map_or(1, |p| p.len()) - 1
        {
            #[cfg(debug_assertions)]
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
        if let Some(tracking_path) = &self.tracking.path {
            writeln!(f, "tracking_path: {}", tracking_path.len())?;
            let width = (tracking_path.len() as f32).log10().ceil() as usize;
            for (i, pos) in tracking_path.iter().enumerate() {
                writeln!(f, "  [{:>width$}] = [{:>6.2}, {:>6.2}]", i, pos.x, pos.y)?;
            }
        } else {
            writeln!(f, "tracking_path: None")?;
        }
        writeln!(f, "index: {}", self.tracking.index)?;
        writeln!(f, "record: {}", self.tracking.record.lock().unwrap().get())?;
        writeln!(f, "config: {:?}", self.tracking.config)?;

        write!(f, "last_measurement: {}", self.last_measurement())
    }
}
