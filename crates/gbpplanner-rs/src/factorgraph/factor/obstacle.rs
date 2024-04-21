//! Obstacle factor

use bevy::{log::warn, math::Vec2, render::texture::Image};
use gbp_linalg::prelude::*;
use ndarray::array;

use super::{Factor, FactorState};

#[derive(Clone)]
pub struct ObstacleFactor {
    /// The signed distance field of the environment
    obstacle_sdf:     &'static Image,
    /// Copy of the `WORLD_SZ` setting from **gbpplanner**, that we store a copy
    /// of here since `ObstacleFactor` needs this information to calculate
    /// `.jacobian_delta()` and `.measurement()`
    world_size:       Float,
    // last_measurement: Option<LastMeasurement>,
    last_measurement: LastMeasurement,
}

#[derive(Clone)]
pub struct LastMeasurement {
    pub pos:   bevy::math::Vec2,
    // x:     Float,
    // y:     Float,
    pub value: Float,
}

impl Default for LastMeasurement {
    fn default() -> Self {
        Self {
            pos:   Vec2::ZERO,
            // x:     0.0,
            // y:     0.0,
            value: 0.0,
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for ObstacleFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use custom impl instead of `derive(Debug)`, to not print the entire `Image`
        // as a pixel array
        f.debug_struct("ObstacleFactor")
            // .field("obstacle_sdf", &self.obstacle_sdf)
            .field("world_size", &self.world_size)
            .finish()
    }
}

impl ObstacleFactor {
    /// An obstacle factor has a single edge to another variable
    pub const NEIGHBORS: usize = 1;

    /// Creates a new [`ObstacleFactor`].
    #[must_use]
    pub fn new(obstacle_sdf: &'static Image, world_size: Float) -> Self {
        Self {
            obstacle_sdf,
            world_size,
            last_measurement: Default::default(),
        }
    }

    /// Returns the last measurement
    #[inline(always)]
    pub fn last_measurement(&self) -> &LastMeasurement {
        &self.last_measurement
    }
}

impl Factor for ObstacleFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "ObstacleFactor"
    }

    #[inline]
    fn jacobian(&mut self, state: &FactorState, x: &Vector<Float>) -> Matrix<Float> {
        // Same as PoseFactor
        // TODO: change to not clone x
        self.first_order_jacobian(state, x.clone())
    }

    fn measure(&mut self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        // pretty_print_vector!(x);
        // debug_assert!(x.len() >= 2, "x.len() = {}", x.len());
        // White areas are obstacles, so h(0) should return a 1 for these regions.
        let scale = Float::from(self.obstacle_sdf.width()) / self.world_size;
        // let offset = (self.world_size / 2.0) as usize;
        let offset = self.world_size / 2.0;
        if (x[0] + offset) * scale > Float::from(self.obstacle_sdf.width()) {
            // warn!(
            //     "x[0] + offset = {}, scale = {}, width = {}",
            //     (x[0] + offset) * scale,
            //     scale,
            //     self.obstacle_sdf.width()
            // );
            return array![0.0];
        }
        if (x[1] + offset) * scale > Float::from(self.obstacle_sdf.height()) {
            // warn!(
            //     "x[1] + offset = {}, scale = {}, height = {}",
            //     (x[1] + offset) * scale,
            //     scale,
            //     self.obstacle_sdf.height()
            // );
            return array![0.0];
        }
        // dbg!(offset);
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let pixel_x = ((x[0] + offset) * scale) as u32;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let pixel_y = ((x[1] + offset) * scale) as u32;
        // println!("pixel_x = {}, pixel_y = {}", pixel_x, pixel_y);
        // dbg!(pixel_x, pixel_y);
        // assert_eq!((self.obstacle_sdf.width() * self.obstacle_sdf.height() * 4) as
        // usize, self.obstacle_sdf.data.len()); multiply by 4 because the image
        // is in RGBA format, and we simply use th R channel to determine value,
        // as the image is grayscale
        // TODO: assert that the image's data is laid out in row-major order
        let linear_index = ((self.obstacle_sdf.width() * pixel_y + pixel_x) * 4) as usize;
        if linear_index >= self.obstacle_sdf.data.len() {
            warn!(
                "linear_index = {}, obstacle_sdf.data.len() = {}",
                linear_index,
                self.obstacle_sdf.data.len()
            );
            return array![0.0];
        }
        let red = self.obstacle_sdf.data[linear_index];
        // NOTE: do 1.0 - red to invert the value, as the obstacle sdf is white where
        // there are obstacles in gbpplanner, they do not do the inversion here,
        // but instead invert the entire image, when they load it from disk.
        let hsv_value = 1.0 - Float::from(red) / 255.0;

        self.last_measurement.pos.x = x[0] as f32;
        self.last_measurement.pos.x = x[1] as f32;
        // self.last_measurement.pos.x = pixel_x as f32;
        // self.last_measurement.pos.y = pixel_y as f32;
        self.last_measurement.value = hsv_value;

        // let hsv_value = pixel as Float / 255.0;
        // if hsv_value <= 0.5 {
        //     println!("image(x={}, y={}).z {} (scale = {})", pixel_x, pixel_y,
        // hsv_value, scale); }
        // dbg!(hsv_value);

        // println!("hsv_value = {}", hsv_value);

        array![hsv_value]
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        self.world_size / Float::from(self.obstacle_sdf.width())
    }

    #[inline(always)]
    fn skip(&mut self, _state: &FactorState) -> bool {
        false
    }

    #[inline(always)]
    fn linear(&self) -> bool {
        false
    }
}
