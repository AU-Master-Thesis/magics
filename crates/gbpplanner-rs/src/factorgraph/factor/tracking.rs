//! Tracking Factor (extension)

use std::{borrow::Cow, cell::Cell, sync::Mutex};

use bevy::math::Vec2;
use gbp_linalg::prelude::*;
use ndarray::{array, s};

use super::{Factor, FactorState};

#[derive(Debug)]
pub struct TrackingFactor {
    /// Tracking path (Likely from RRT)
    // tracking_path: Vec<Vec2>,
    tracking_path: Option<Vec<Vec2>>,

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
        Self {
            tracking_path:    tracking_path.map(|x| x.into()),
            // last_measurement: Mutex::new(Cell::new(Vec2::ZERO)),
            last_measurement: Default::default(),
        }
    }

    /// Get the last measurement
    pub fn last_measurement(&self) -> LastMeasurement {
        self.last_measurement.lock().unwrap().get()
    }

    pub fn set_tracking_path(&mut self, tracking_path: min_len_vec::TwoOrMore<Vec2>) {
        self.tracking_path = Some(tracking_path.into());
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

    fn measure(&self, _state: &FactorState, linearization_point: &Vector<Float>) -> Vector<Float> {
        // 1. Window pairs of rrt path
        // 1.1. Find the line defined by the two points

        let (projected_point, _, _, _) = self
            .tracking_path
            .as_ref()
            .unwrap()
            .windows(2)
            .map(|window| {
                let p2 = array![window[1].x as Float, window[1].y as Float];
                let p1 = array![window[0].x as Float, window[0].y as Float];

                let line = &p2 - &p1;

                let x_pos = linearization_point.slice(s![0..2]).to_owned();

                // project x_pos onto the line
                let projected = &p1 + (&x_pos - &p1).dot(&line) / &line.dot(&line) * &line;
                let distance = (&x_pos - &projected).euclidean_norm();

                (projected, distance, line, p1)
            })
            .filter(|(projected, _, line, p1)| {
                let p1_to_projected_l2 = (projected - p1).l2_norm();
                let p1_to_p2_l2 = line.l2_norm();
                let p2_to_projected_l2 = (projected - line).l2_norm();

                let projected_is_between_p1_p2 = p1_to_projected_l2 < p1_to_p2_l2;
                let projected_is_outside_radius_of_p2 = p2_to_projected_l2 > 2.0f64.powi(2);

                projected_is_between_p1_p2 && projected_is_outside_radius_of_p2
            })
            .min_by(|(_, a, _, _), (_, b, _, _)| a.partial_cmp(b).unwrap())
            .expect("There should be some line to consider");

        // current speed is the magnitude of the velocity x[2..4]
        // let speed = x.slice(s![2..4]).euclidean_norm();

        // let future_point = projected_point + speed * lines[min_index].2.normalized();
        // let future_point = projected_point + 2.0 * line.normalized();
        // let future_point = array![projected_point[1], projected_point[0]];

        let max_length = 2.0;

        let x_to_projection = &projected_point - linearization_point.slice(s![0..2]).to_owned();
        // clamp the distance to the max length
        let x_to_projection = if x_to_projection.euclidean_norm() > max_length {
            (x_to_projection.normalized() * max_length).normalized()
        } else {
            x_to_projection.normalized()
        };

        // invert measurement to make it 'pull' the variable towards the path
        let measurement = -x_to_projection.euclidean_norm();

        // self.last_measurement
        //     .lock()
        //     .unwrap()
        //     .set(Vec2::new(measurement[0] as f32, measurement[1] as f32));
        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos:   Vec2::new(projected_point[0] as f32, projected_point[1] as f32),
            value: measurement,
        });

        array![measurement]
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        // Same as DynamicFactor for now
        // TODO: Tune this
        // NOTE: Maybe this should be influenced by the distance from variable to the
        // measurement
        1e-8
    }

    #[inline(always)]
    fn skip(&self, _state: &FactorState) -> bool {
        self.tracking_path.is_none()
        // false
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

        if let Some(tracking_path) = &self.tracking_path {
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
