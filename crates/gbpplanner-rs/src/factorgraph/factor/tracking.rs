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

        let lines = self
            .tracking_path
            .windows(2)
            .map(|window| {
                let p2 = array![window[1].x as Float, window[1].y as Float];
                let p1 = array![window[0].x as Float, window[0].y as Float];

                let line = &p2 - &p1;
                let normal = array![-line[1], line[0]].normalized();

                (p1, p2, line, normal)
            })
            .collect::<Vec<_>>();

        // 2. Project the linearisation point onto all the lines
        // 2.1. Choose the closest line and projection

        let mut min_distance = Float::INFINITY;
        // let mut projected_point = Vec2::ZERO;
        let mut projected_point = Vector::<Float>::zeros(2);
        let mut min_index = 0;

        for (i, (p1, p2, line, normal)) in lines.iter().enumerate() {
            let p = x.slice(s![0..2]).to_owned();
            let v = x.slice(s![2..4]);

            let p1p = &p - p1;
            let p1p_dot_n = p1p.dot(normal);
            let v_dot_n = v.dot(normal);

            let projected = &p - p1p_dot_n * normal + v_dot_n * normal;

            let distance = (&projected - p).euclidean_norm();

            if distance < min_distance {
                min_distance = distance;
                projected_point = projected;
                min_index = i;
            }
        }

        // 2.2. Move the projected point X seconds into the future along the
        // line, using the current velocity The future point is the resulting
        // measurement

        // current speed is the magnitude of the velocity x[2..4]
        let speed = x.slice(s![2..4]).euclidean_norm();

        let future_point = projected_point + speed * lines[min_index].2.normalized();
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
