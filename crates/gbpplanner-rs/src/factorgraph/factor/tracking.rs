//! Tracking Factor (extension)
use std::{borrow::Cow, cell::Cell, sync::Mutex};

use bevy::{math::Vec2, utils::smallvec::ToSmallVec};
use colored::Colorize;
use gbp_linalg::{prelude::*, pretty_print_matrix};
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};

use super::{Factor, FactorState, Measurement};

/// Tracking information for each tracking factor to follow
#[derive(Debug)]
pub struct Tracking {
    /// The path to follow
    path: Option<Vec<Vec2>>,
    /// Which index in the path, the horizon is currently moving towards
    index: usize,
    /// Tracking record
    /// Implicitly tells which waypoint the factor has reached
    /// e.g. if the record is 3, the factor has been to waypoint 1, 2, and 3
    record: Mutex<Cell<usize>>,
    /// Amount of projects being considered
    connections: Mutex<Cell<usize>>,
    /// The tracking config from the `gbp_config` input `config.toml`
    config: gbp_config::TrackingSection,
}

impl Default for Tracking {
    fn default() -> Self {
        Self {
            path: None,
            index: 0,
            record: Mutex::new(Cell::new(0)),
            connections: Mutex::new(Cell::new(1)),
            config: gbp_config::TrackingSection::default(),
        }
    }
}

impl Tracking {
    pub fn with_path(mut self, path: Vec<Vec2>) -> Self {
        self.path = Some(path);
        self
    }

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
        Self {
            tracking: Tracking::default()
                .with_path(tracking_path.map_or_else(Vec::new, |p| p.into())),
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

    pub fn with_config(mut self, config: gbp_config::TrackingSection) -> Self {
        self.tracking.config = config;
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

        // pretty_print_matrix!(&jacobian);

        Cow::Owned(jacobian)
    }

    // fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Measurement {
        let current_record = self.tracking.record.lock().unwrap().get();
        let x_pos = x.slice(s![0..2]).to_owned();
        let x_vel = x.slice(s![2..4]).to_owned();

        // 1. Find which line in the `self.tracking.path` to project to, based off of
        //    the `self.tracking.record` e.g. if `self.tracking.record` is 3, then track
        //    the line between waypoint 3 and 4
        let lines = self
            .tracking
            .path
            .as_ref()
            .unwrap()
            .windows(2)
            .collect_vec();

        let [current_start, current_end]: &[Vec2; 2] = lines[current_record].try_into().unwrap();
        let (current_start, current_end) = (
            array![current_start.x as Float, current_start.y as Float],
            array![current_end.x as Float, current_end.y as Float],
        );

        // 2. Project the current position onto the line between `current_start` and
        //    `current_end`
        let line = &current_end - &current_start;
        let projected_point =
            &current_start + (&x_pos - &current_start).dot(&line) / &line.dot(&line) * &line;

        // 3. If within `self.tracking.smoothing` of start, project to the previous line
        //    if it exists
        //   as well, and take the average of the two projections
        // let consideration_distance =
        //     x_vel.euclidean_norm() + self.tracking.config.switch_padding.powi(2) as
        // f64;
        let consideration_distance = self.tracking.config.switch_padding as Float;
        let current_projected_point_to_current_end =
            (&current_end - &projected_point).euclidean_norm();
        // let current_projected_point_to_current_start =
        //     (&current_start - &projected_point).euclidean_norm();
        // let vel_distance_contribution = line.normalized() * x_vel.euclidean_norm() as
        // Float;
        // let should_be_considered = current_projected_point_to_current_end <
        // consideration_distance;

        // Get projection to previous line only if there is a previous line
        let projection_to_previous = if current_record > 0 {
            // Retrieve the previous line
            let [previous_start, previous_end]: &[Vec2; 2] =
                lines[current_record - 1].try_into().unwrap();
            let (previous_start, previous_end) = (
                array![previous_start.x as Float, previous_start.y as Float],
                array![previous_end.x as Float, previous_end.y as Float],
            );

            // Project the linearisation point onto the line between `previous_start` and
            // `previous_end`
            let line = &previous_end - &previous_start;
            let previous_projected_point =
                &previous_start + (&x_pos - &previous_start).dot(&line) / &line.dot(&line) * &line;

            // Check if the projected is to be considered
            let current_projected_point_to_previous_end =
                (&previous_end - &projected_point).euclidean_norm();
            let previous_projected_point_to_previous_end =
                (&current_start - &previous_projected_point).euclidean_norm();
            // if previous_projected_point_to_previous_end < consideration_distance {
            if current_projected_point_to_previous_end < consideration_distance
                && previous_projected_point_to_previous_end < consideration_distance
            {
                Some(previous_projected_point)
            } else {
                None
            }
        } else {
            None
        };

        // 4. if within `self.tracking.smoothing` of end, increment
        //    `self.tracking.record`
        if current_projected_point_to_current_end < consideration_distance {
            self.tracking.increment_record();
        }

        // 5. Take the average of the two projections
        let measurement_point = match projection_to_previous {
            Some(projection) => {
                // connections should be 2
                self.tracking.connections.lock().unwrap().set(2);
                (projected_point + projection) / 2.0
            }
            None => {
                // connections should be 1
                self.tracking.connections.lock().unwrap().set(1);
                projected_point // + line.normalized() * x_vel.euclidean_norm()
                                //   as Float
            }
        };

        // 6. Normalise length to `self.tracking.config.attraction_distance`
        let x_to_projection = &measurement_point - &x_pos;
        let x_to_projection_distance = x_to_projection.euclidean_norm();
        let attraction_distance_f64 = self.tracking.config.attraction_distance as f64;
        let normalised_distance = if x_to_projection_distance < attraction_distance_f64 {
            x_to_projection_distance / attraction_distance_f64
        } else {
            1.0
        };

        // 7. Invert the measurement
        // let measurement = 1.0 - normalised_distance;
        let measurement = normalised_distance;

        // Update last measurement and return
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos:   Vec2::new(measurement_point[0] as f32, measurement_point[1] as f32),
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
        // Same as DynamicFactor for now
        // TODO: Tune this
        // NOTE: Maybe this should be influenced by the distance from variable to the
        // measurement
        1e-2
        // let base = 1e-2;
        // base / (2.0
        //     * self.last_measurement.lock().unwrap().get().value
        //     * self.tracking.connections.lock().unwrap().get() as Float)
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
