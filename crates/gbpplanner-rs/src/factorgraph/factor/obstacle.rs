//! Obstacle factor

use std::{borrow::Cow, cell::Cell, sync::Mutex};

use bevy::math::Vec2;
use gbp_linalg::prelude::*;
use ndarray::array;

use super::{Factor, FactorState};
use crate::simulation_loader::SdfImage;

pub struct ObstacleFactor {
    /// The signed distance field of the environment
    obstacle_sdf:     SdfImage,
    /// Copy of the `WORLD_SZ` setting from **gbpplanner**, that we store a copy
    /// of here since `ObstacleFactor` needs this information to calculate
    /// `.jacobian_delta()` and `.measurement()`
    world_size:       WorldSize,
    // world_size:       Float,
    last_measurement: Mutex<Cell<LastMeasurement>>,
    jacobian_delta:   Float,
}

#[derive(Debug, Clone, Copy)]
pub struct WorldSize {
    pub width:  Float,
    pub height: Float,
}

impl std::fmt::Display for WorldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(width: {}, height: {})", self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LastMeasurement {
    pub pos:   bevy::math::Vec2,
    pub value: Float,
}

impl std::fmt::Display for LastMeasurement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use colored::Colorize;
        let v = self.value * 255.0;
        let r = v as u8;
        let g = 255 - r;

        write!(
            f,
            "[pos: {}, value: {}]",
            self.pos,
            format!("{:.4}", self.value).truecolor(r, g, 0u8)
        )
    }
}

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
    pub fn new(obstacle_sdf: SdfImage, world_size: WorldSize) -> Self {
        let jacobian_delta = {
            let width = world_size.width / Float::from(obstacle_sdf.width());
            let height = world_size.height / Float::from(obstacle_sdf.height());
            (width + height) / 2.0
        };

        Self {
            obstacle_sdf,
            world_size,
            last_measurement: Default::default(),
            jacobian_delta,
        }
    }

    pub fn last_measurement(&self) -> LastMeasurement {
        self.last_measurement.lock().unwrap().get()
    }
}

impl Factor for ObstacleFactor {
    #[inline]
    fn name(&self) -> &'static str {
        "ObstacleFactor"
    }

    fn color(&self) -> [u8; 3] {
        // #ee99a0
        [238, 153, 160]
    }

    #[inline]
    fn jacobian(
        &self,
        state: &FactorState,
        linearisation_point: &Vector<Float>,
    ) -> Cow<'_, Matrix<Float>> {
        // Same as PoseFactor
        // TODO: change to not clone x
        Cow::Owned(self.first_order_jacobian(state, linearisation_point.clone()))
    }

    fn measure(&self, _state: &FactorState, linearisation_point: &Vector<Float>) -> Vector<Float> {
        let x_pos = linearisation_point[0];
        let y_pos = linearisation_point[1];
        // The robots coordinate system is centered in the image, so we have to offset
        // the pixel index, by half the height in the row index i.e. `y` and
        // half the width in the column index i.e. `x`
        let x_offset = self.world_size.width / 2.0;
        let y_offset = self.world_size.height / 2.0;

        let x_scale = Float::from(self.obstacle_sdf.width()) / self.world_size.width;
        let y_scale = Float::from(self.obstacle_sdf.height()) / self.world_size.height;

        let x_pixel = ((x_pos + x_offset) * x_scale) as u32;
        // NOTE: the -y_pos is because the y axis is flipped in the image
        let y_pixel = ((-y_pos + y_offset) * y_scale) as u32;

        // dbg!((
        //     x_pos,
        //     y_pos,
        //     x_offset,
        //     y_offset,
        //     x_scale,
        //     y_scale,
        //     x_pixel,
        //     y_pixel,
        //     self.obstacle_sdf.dimensions()
        // ));

        let Some(pixel) = self.obstacle_sdf.get_pixel_checked(x_pixel, y_pixel) else {
            // let Some(pixel) = self.obstacle_sdf.get_pixel_checked(y_pixel, x_pixel) else
            // { Measurement point outside of image
            // Return 1.0 to indicate that it is an obstacle
            // return array![1.0];
            // Return 0.0 to indicate that it is an empty space
            return array![0.0];
        };

        let red_channel = pixel[0];
        // Dark areas are obstacles, so h(0) should return a 1 for these regions.
        let hsv_value = 1.0 - Float::from(red_channel) / 255.0;

        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos:   Vec2::new(x_pos as f32, y_pos as f32),
            value: hsv_value,
        });

        array![hsv_value]
    }

    #[inline(always)]
    fn jacobian_delta(&self) -> Float {
        self.jacobian_delta
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
        writeln!(f, "world_size: {}", self.world_size)?;
        writeln!(f, "last_measurement: {}", self.last_measurement())
    }
}
