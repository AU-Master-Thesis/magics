//! Obstacle factor

use std::{
    borrow::Cow,
    cell::Cell,
    sync::{Arc, Mutex},
    // cell::{Cell, RefCell},
    // sync::{Arc, Mutex},
};

use bevy::math::Vec2;
use gbp_linalg::prelude::*;
use ndarray::array;

use super::{Factor, FactorState};
use crate::simulation_loader::SdfImage;

#[derive(Clone)]
pub struct ObstacleFactor {
    /// The signed distance field of the environment
    // obstacle_sdf:     &'static Image,
    // obstacle_sdf: Image,
    obstacle_sdf: SdfImage,
    /// Copy of the `WORLD_SZ` setting from **gbpplanner**, that we store a copy
    /// of here since `ObstacleFactor` needs this information to calculate
    /// `.jacobian_delta()` and `.measurement()`
    world_size:       Float,
    // last_measurement: Arc<Mutex<Cell<Option<LastMeasurement>>>>,
    // last_measurement: Cell<Option<LastMeasurement>>,
    last_measurement: Option<LastMeasurement>,
    // last_measurement: Arc<Cell<LastMeasurement>>,
    // last_measurement: Arc<Mutex<LastMeasurement>>,
}

#[derive(Debug, Clone, Copy)]
pub struct LastMeasurement {
    pub pos:   bevy::math::Vec2,
    pub value: Float,
}

// unsafe impl Sync for LastMeasurement {}
// unsafe impl Send for LastMeasurement {}

impl Default for LastMeasurement {
    fn default() -> Self {
        Self {
            pos:   Vec2::ZERO,
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
    pub fn new(obstacle_sdf: SdfImage, world_size: Float) -> Self {
        Self {
            obstacle_sdf,
            world_size,
            last_measurement: Default::default(),
        }
    }

    // / Returns the last measurement
    // #[inline(always)]
    // pub fn last_measurement(&self) -> Option<LastMeasurement> {
    //     self.last_measurement.try_lock().ok().as_deref().copied()
    //
    //     // self.last_measurement.try_borrow().ok().as_deref().copied()
    // }

    // pub fn last_measurement(&self) -> Option<LastMeasurement> {
    //     self.last_measurement.lock().unwrap().get()
    //     // self.last_measurement.get()
    // }
}

impl Factor for ObstacleFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "ObstacleFactor"
    }

    #[inline]
    fn jacobian(&self, state: &FactorState, x: &Vector<Float>) -> Cow<'_, Matrix<Float>> {
        // Same as PoseFactor
        // TODO: change to not clone x
        Cow::Owned(self.first_order_jacobian(state, x.clone()))
    }

    fn measure(&self, _state: &FactorState, x: &Vector<Float>) -> Vector<Float> {
        // White areas are obstacles, so h(0) should return a 1 for these regions.
        let scale = Float::from(self.obstacle_sdf.width()) / self.world_size;
        // let offset = (self.world_size / 2.0) as usize;
        let offset = self.world_size / 2.0;
        if (x[0] + offset) * scale > Float::from(self.obstacle_sdf.width()) {
            return array![0.0];
        }
        if (x[1] + offset) * scale > Float::from(self.obstacle_sdf.height()) {
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
        // let linear_index = ((self.obstacle_sdf.width() * pixel_y + pixel_x) * 4) as
        // usize; if linear_index >= self.obstacle_sdf.width() as usize {
        //     // if linear_index >= self.obstacle_sdf.data.len() {
        //     warn!(
        //         "linear_index = {}, obstacle_sdf.width() = {}",
        //         linear_index,
        //         self.obstacle_sdf.width()
        //     );
        //     return array![0.0];
        // }
        //
        let Some(pixel) = self.obstacle_sdf.get_pixel_checked(pixel_x, pixel_y) else {
            // measurement point outside of image
            return array![0.0];
        };

        let red_channel = pixel[0];
        let hsv_value = 1.0 - Float::from(red_channel) / 255.0;

        // let red = self.obstacle_sdf.data[linear_index];
        // NOTE: do 1.0 - red to invert the value, as the obstacle sdf is white where
        // there are obstacles in gbpplanner, they do not do the inversion here,
        // but instead invert the entire image, when they load it from disk.
        // let hsv_value = 1.0 - Float::from(red) / 255.0;

        // let mut last_measurement = self.last_measurement.try_borrow_mut().unwrap();
        // self.last_measurement.set(LastMeasurement {
        //     pos:   Vec2::new(x[0] as f32, x[1] as f32),
        //     value: hsv_value,
        // });

        // if let Ok(mut last_measurement) = self.last_measurement.try_borrow_mut() {
        //     last_measurement.pos.x = x[0] as f32;
        //     last_measurement.pos.y = x[1] as f32;
        //     last_measurement.value = hsv_value;
        // }

        // self.last_measurement.pos.x = x[0] as f32;
        // self.last_measurement.pos.y = x[1] as f32;

        // self.last_measurement.set(Some(LastMeasurement {
        //     pos:   Vec2::new(pixel_x as f32, pixel_y as f32),
        //     value: hsv_value,
        // }));

        // let guard = self.last_measurement.lock().unwrap();
        // // guard.set()

        // guard.set(Some(LastMeasurement {
        //     pos:   Vec2::new(pixel_x as f32, pixel_y as f32),
        //     value: hsv_value,
        // }));
        // self.last_measurement.pos.x = pixel_x as f32;
        // self.last_measurement.pos.y = pixel_y as f32;
        // self.last_measurement.value = hsv_value;

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

impl std::fmt::Display for ObstacleFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "world_size: {}", self.world_size)
        // writeln!(f, "")
    }
}
