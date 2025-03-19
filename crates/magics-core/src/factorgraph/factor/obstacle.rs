//! Obstacle factor for collision avoidance
//!
//! This module implements the obstacle factor which penalizes robot positions
//! that are close to obstacles in the environment.

use std::{borrow::Cow, cell::Cell, sync::Mutex};

use gbp_linalg::prelude::*;
use ndarray::array;

use super::{Factor, FactorState, Measurement, SdfImage};
use crate::types::Vector2;

/// Obstacle factor for collision avoidance in the environment
pub struct ObstacleFactor {
    /// The signed distance field of the environment
    obstacle_sdf: SdfImage,
    /// World size parameters for coordinate transformation
    world_size: WorldSize,
    /// Last measurement value and position
    last_measurement: Mutex<Cell<LastMeasurement>>,
    /// Delta for jacobian calculation
    jacobian_delta: Float,
}

/// World size parameters for coordinate transformation
#[derive(Debug, Clone, Copy)]
pub struct WorldSize {
    /// Width of the world in world units
    pub width: Float,
    /// Height of the world in world units
    pub height: Float,
}

impl std::fmt::Display for WorldSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(width: {}, height: {})", self.width, self.height)
    }
}

/// Last measurement data for debugging
#[derive(Debug, Clone, Copy)]
pub struct LastMeasurement {
    /// Position of the measurement
    pub pos: Vector2,
    /// Value of the measurement (0-1, where 1 is obstacle)
    pub value: Float,
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

impl Default for LastMeasurement {
    fn default() -> Self {
        Self {
            pos: Vector2::new(0.0, 0.0),
            value: 0.0,
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for ObstacleFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use custom impl instead of `derive(Debug)`, to not print the entire SDF image
        f.debug_struct("ObstacleFactor")
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
            let width = world_size.width / obstacle_sdf.width as Float;
            let height = world_size.height / obstacle_sdf.height as Float;
            (width + height) / 2.0
        };

        Self {
            obstacle_sdf,
            world_size,
            last_measurement: Default::default(),
            jacobian_delta,
        }
    }

    /// Get the last measurement
    pub fn last_measurement(&self) -> LastMeasurement {
        self.last_measurement.lock().unwrap().get()
    }
    
    /// Get a pixel from the SDF image
    fn get_pixel_checked(&self, x: u32, y: u32) -> Option<f32> {
        self.obstacle_sdf.get_pixel_checked(x, y)
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
        Cow::Owned(self.first_order_jacobian(state, linearisation_point.clone()))
    }

    fn measure(&self, _state: &FactorState, linearisation_point: &Vector<Float>) -> Measurement {
        let x_pos = linearisation_point[0];
        let y_pos = linearisation_point[1];
        
        // The robots coordinate system is centered in the image, so we have to offset
        // the pixel index, by half the height in the row index i.e. `y` and
        // half the width in the column index i.e. `x`
        let x_offset = self.world_size.width / 2.0;
        let y_offset = self.world_size.height / 2.0;

        let x_scale = self.obstacle_sdf.width as Float / self.world_size.width;
        let y_scale = self.obstacle_sdf.height as Float / self.world_size.height;

        let x_pixel = ((x_pos + x_offset) * x_scale) as u32;
        // NOTE: the -y_pos is because the y axis is flipped in the image
        let y_pixel = ((-y_pos + y_offset) * y_scale) as u32;

        let pixel_value = self.get_pixel_checked(x_pixel, y_pixel);
        
        let hsv_value = if let Some(pixel_value) = pixel_value {
            // Dark areas are obstacles, so h(0) should return a 1 for these regions.
            1.0 - pixel_value as Float / 255.0
        } else {
            // Measurement point outside of image
            // Return 0.0 to indicate that it is an empty space
            0.0
        };

        self.last_measurement.lock().unwrap().set(LastMeasurement {
            pos: Vector2::new(x_pos, y_pos),
            value: hsv_value,
        });

        Measurement::new(array![hsv_value])
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
