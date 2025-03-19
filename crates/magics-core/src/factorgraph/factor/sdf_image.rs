//! SDF Image implementation for obstacle factors
//!
//! This module contains the SDF (Signed Distance Field) image implementation
//! used by obstacle factors to calculate distances from obstacles.

/// A Signed Distance Field image representation
///
/// This structure stores a flattened array of distance values
/// where each value represents the distance to the nearest obstacle.
/// Positive values are outside obstacles, negative values are inside.
#[derive(Debug, Clone)]
pub struct SdfImage {
    /// Width of the image in pixels
    pub width: usize,
    /// Height of the image in pixels
    pub height: usize,
    /// Raw image data (flattened row-major order)
    pub data: Vec<f32>,
}

impl SdfImage {
    /// Create a new SDF image with given dimensions
    pub fn new(width: usize, height: usize, data: Vec<f32>) -> Self {
        // Ensure data size matches dimensions
        assert_eq!(width * height, data.len(), 
            "SDF image data length ({}) doesn't match dimensions ({}x{}={})",
            data.len(), width, height, width * height);
        
        Self {
            width,
            height,
            data,
        }
    }
    
    /// Get the value at the given coordinates
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        debug_assert!(x < self.width && y < self.height, 
            "Coordinates out of bounds: ({}, {}) for dimensions {}x{}", 
            x, y, self.width, self.height);
        
        self.data[y * self.width + x]
    }
    
    /// Get the width of the image
    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }
    
    /// Get the height of the image
    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }
    
    /// Get a pixel if it's within bounds
    pub fn get_pixel_checked(&self, x: u32, y: u32) -> Option<f32> {
        if x < self.width as u32 && y < self.height as u32 {
            let index = (y as usize * self.width) + x as usize;
            if index < self.data.len() {
                return Some(self.data[index]);
            }
        }
        None
    }
}

impl Default for SdfImage {
    fn default() -> Self {
        // Create a small blank image by default
        Self {
            width: 1,
            height: 1,
            data: vec![0.0],
        }
    }
}
