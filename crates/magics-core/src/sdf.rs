//! SDF (Signed Distance Field) handling for the simulation environment
//!
//! This module provides utilities for generating and working with Signed Distance Fields
//! extracted from environments.

use std::path::Path;
use image::{ImageBuffer, Rgb};
use gbp_environment::Environment;
use env_to_png::{PixelsPerTile, Percentage};
use anyhow::Result;

/// Type alias for an SDF image
pub type SdfImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// SDF image wrapper
#[derive(Debug, Clone)]
pub struct Sdf(pub SdfImage);

impl Sdf {
    /// Generate an SDF image from an environment
    pub fn from_environment(
        environment: &Environment,
        resolution: u32,
        expansion: f64,
        blur: f64,
    ) -> Result<Self> {
        let sdf_image_buffer = env_to_png::env_to_sdf_image(
            environment,
            PixelsPerTile::new(resolution),
            Percentage::new(expansion as f32),
            Percentage::new(blur as f32),
        )?;

        Ok(Sdf(sdf_image_buffer))
    }

    /// Save the SDF image to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.0.save(path)?;
        Ok(())
    }
    
    /// Get a reference to the inner image
    pub fn inner(&self) -> &SdfImage {
        &self.0
    }
    
    /// Get the dimensions of the SDF image
    pub fn dimensions(&self) -> (u32, u32) {
        self.0.dimensions()
    }
}

impl Default for Sdf {
    fn default() -> Self {
        // Create a small 1x1 black image as a default
        let buffer = ImageBuffer::from_pixel(1, 1, Rgb([0, 0, 0]));
        Sdf(buffer)
    }
}
