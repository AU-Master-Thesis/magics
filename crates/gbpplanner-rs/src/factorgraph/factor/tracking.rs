//! Obstacle factor

use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    sync::Mutex,
};

use bevy::math::Vec2;
use gbp_linalg::prelude::*;
use ndarray::{array, s};

use super::{Factor, FactorState};

#[derive(Debug)]
pub struct TrackingFactor {
    /// Reference to the tracking path (Likely from RRT)
    tracking_path: Vec<Vec2>,

    /// Most recent measurement
    last_measurement: Mutex<Cell<Vec2>>,
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
            tracking_path,
            last_measurement: Mutex::new(Cell::new(Vec2::ZERO)),
        }
    }

    /// Get the last measurement
    pub fn last_measurement(&self) -> Vec2 {
        self.last_measurement.lock().unwrap().get()
    }
}

impl Factor for TrackingFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "TrackingFactor"
    }

    #[inline]
    fn jacobian(&self, state: &FactorState, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        // Same as PoseFactor
        // TODO: change to not clone x
        Cow::Owned(self.first_order_jacobian(state, x.clone()))
    }

    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        // 1. Window pairs of rrt path
        // 1.1. Find the line defined by the two points

        let (projected_point, distance, line) = self
            .tracking_path
            .windows(2)
            // .filter(|window| {
            //     let p2 = array![window[1].x as Float, window[1].y as Float];

            //     // if x_pos is within some radius of p2, don't consider this line
            //     let x_pos = x.slice(s![0..2]).to_owned();
            //     let distance = (&x_pos - &p2).euclidean_norm();

            //     // if distance is greater than 1.0, consider this line
            //     let outside_radius_from_p2 = distance > 1.0;

            //     // filter out if on other side of p2
            //     let p1 = array![window[0].x as Float, window[0].y as Float];
            //     let p1_to_p2 = &p2 - &p1;
            //     let p2_to_x = &x_pos - &p2;

            //     // if the cross product is negative, then the point is on the other side
            //     let between_p1_p2 = p1_to_p2.cross(&p2_to_x) < 0.0;

            //     outside_radius_from_p2 && between_p1_p2
            // })
            .map(|window| {
                let p2 = array![window[1].x as Float, window[1].y as Float];
                let p1 = array![window[0].x as Float, window[0].y as Float];

                let line = &p2 - &p1;

                let x_pos = x.slice(s![0..2]).to_owned();

                // project x_pos onto the line
                let projected = &p1 + (&x_pos - &p1).dot(&line) / &line.dot(&line) * &line;
                let distance = (&x_pos - &projected).euclidean_norm();

                (projected, distance, line, p2)
            })
            .filter(|(projected, distance, line, p2)| {
                let dir_projected_to_p2 = (p2 - projected).normalized();
                let dir_line = line.normalized();

                let line_projected = line + dir_projected_to_p2;
                let squared_line_length = dir_projected_to_p2.l2_norm();

                let projected_is_between_p1_p2 = line_projected.l2_norm() < line.l2_norm();
                let projected_is_outside_radius_of_p2 = squared_line_length > 2.0f64.powi(2);

                projected_is_between_p1_p2 && projected_is_outside_radius_of_p2
            })
            .min_by(|(_, a, _, _), (_, b, _, _)| a.partial_cmp(b).unwrap())
            .expect("There should be some line to consider");

        // current speed is the magnitude of the velocity x[2..4]
        // let speed = x.slice(s![2..4]).euclidean_norm();

        // let future_point = projected_point + speed * lines[min_index].2.normalized();
        let future_point = projected_point + 2.0 * line.normalized();
        self.last_measurement
            .lock()
            .unwrap()
            .set(Vec2::new(future_point[0] as f32, future_point[1] as f32));
        future_point
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
