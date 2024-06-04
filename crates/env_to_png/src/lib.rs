//! This crate converts an [`Environment`] Configuration from the magics
//! crate to a PNG image. It is possible to customize the colors of the
//! different elements of the environment. And it can blur the edges to mimic an
//! SDF.

use std::num::NonZeroU32;

// use magics::config::Environment;
use gbp_environment::{Environment, PlaceableShape, RegularPolygon};
use gbp_geometry::RelativePoint;
use glam::{Vec2, Vec3Swizzles};
use image::{imageops::FilterType::Triangle, RgbImage};

/// Custom resolution type, as pixels per tile.
#[derive(Clone, Copy, Debug)]
pub struct PixelsPerTile(NonZeroU32);

impl PixelsPerTile {
    /// Create a new PixelsPerTile type.
    pub fn new(value: u32) -> Self {
        Self(NonZeroU32::new(value).expect("Pixels per tile must be non-zero"))
    }

    /// Get the value of the PixelsPerTile type.
    pub fn get(&self) -> u32 {
        self.0.get()
    }
}

/// Custom type for representing tile coordinates unit.
/// Important to differentiate from pixel coordinates unit.
pub type TileIndex = usize;

/// Custom type for representing pixel coordinates unit.
/// Important to differentiate from tile coordinates unit.
pub type PixelIndex = u32;

/// Custom struct for generic coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords<T> {
    pub x: T,
    pub y: T,
}

/// Custom struct type for percentages.
#[derive(Clone, Copy, Debug)]
pub struct Percentage(f32);

impl Percentage {
    /// Make sure the percentage is within the range [0, 1].
    /// If the input value is 2.3, the output should be 2.3 % 1 = 0.3.
    /// Illegal to give negative values.
    pub fn new(value: f32) -> Self {
        assert!(value >= 0.0);
        assert!(value <= 1.0);
        Self(value)
    }

    /// Invert
    pub fn invert(&mut self) {
        self.0 = 1.0 - self.0;
    }

    /// Get the value of the percentage.
    pub fn get(&self) -> f32 {
        self.0
    }
}

/// Custom type for percentage coordinates.
#[derive(Clone, Copy, Debug)]
pub struct PercentageCoords(Coords<Percentage>);

impl PercentageCoords {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            0: Coords {
                x: Percentage::new(x),
                y: Percentage::new(y),
            },
        }
    }

    pub fn x(&self) -> Percentage {
        self.0.x
    }

    pub fn y(&self) -> Percentage {
        self.0.y
    }

    pub fn horizontal_invert(&mut self) -> &Self {
        self.0.y.invert();
        self
    }

    pub fn vertical_invert(&mut self) -> &Self {
        self.0.x.invert();
        self
    }

    pub fn invert(&mut self) -> &Self {
        self.0.x.invert();
        self.0.y.invert();
        self
    }
}

// impl > and < and <= and >= and == and != for Percentage
impl std::cmp::PartialOrd for Percentage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl std::cmp::PartialEq for Percentage {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// impl ops for Percentage
impl std::ops::Add for Percentage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.0 + other.0)
    }
}

impl std::ops::Sub for Percentage {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self::new(self.0 - other.0)
    }
}

/// Custom type for tile coordinates.
pub type TileCoords = Coords<TileIndex>;

/// Custom type for tile dimension coordinates.
pub type TileDimensions = Coords<f32>;

/// Custom type for pixel coordinates.
pub type PixelCoords = Coords<PixelIndex>;

/// Convert [`Environment`] to an SDF image.
pub fn env_to_sdf_image(
    env: &Environment,
    resolution: PixelsPerTile,
    expansion: Percentage,
    blur_percent: Percentage,
) -> anyhow::Result<RgbImage> {
    let image = env_to_image(env, resolution, expansion).unwrap();
    let blur_pixels = blur_percent.0 * resolution.get() as f32;
    // println!("Blur pixels: {}", blur_pixels);
    if blur_pixels < 1.0 {
        return Ok(image);
    }
    let sdf = image::imageops::blur(&image, blur_pixels);

    Ok(sdf)
}

/// Convert [`Environment`] to an image.
pub fn env_to_image(
    env: &Environment,
    resolution: PixelsPerTile,
    expansion: Percentage,
) -> anyhow::Result<RgbImage> {
    let tile_size = env.tile_size();
    let (ncols, nrows) = (env.tiles.grid.ncols(), env.tiles.grid.nrows());

    let mut image = RgbImage::new(
        ncols as u32 * resolution.get(),
        nrows as u32 * resolution.get(),
    );

    // Start by making the whole image white.
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel_coords = PixelCoords { x, y };
            let tile_coords = image_to_tile_coords(pixel_coords, resolution);
            let tile_dimensions = image_to_tile_units(pixel_coords, resolution, tile_size);
            let percentage_coords = tile_units_to_percentage(tile_dimensions, tile_size);

            if let Some(tile) = env.tiles.grid.get_tile(tile_coords.y, tile_coords.x) {
                if is_tile_obstacle(
                    tile,
                    Percentage::new(env.path_width()),
                    percentage_coords,
                    expansion,
                ) || is_placeable_obstacle(&env, tile_coords, percentage_coords, expansion)
                {
                    image.put_pixel(x, y, image::Rgb([0, 0, 0]));
                } else {
                    image.put_pixel(x, y, image::Rgb([255, 255, 255]));
                }
            } else {
                Err(anyhow::anyhow!("Tile not found"))?;
            }
        }
    }

    Ok(image)
}

/// Convert from image index to tile dimensions
/// That is; if PixelsPerTile is 100, and the env.tile_size() is 10,
/// then pixel (23, 56) is (23 / 100 * 10, 56 / 100 * 10) = (2.3, 5.6) units in
/// the environment.
fn image_to_tile_units(
    pixel_coords: PixelCoords,
    resolution: PixelsPerTile,
    tile_size: f32,
) -> TileDimensions {
    let (x, y) = (pixel_coords.x as f32 + 0.5, pixel_coords.y as f32 + 0.5);
    TileDimensions {
        x: (x / resolution.get() as f32 * tile_size),
        y: (y / resolution.get() as f32 * tile_size),
    }
}

/// Convert from tile dimensions to tile percentage
/// That is; if the env.tile_size() is 10, and the tile is at (2.3, 5.6),
/// then the percentage is (2.3 / 10 * 100, 5.6 / 10 * 100) = (23, 56) percent
/// into the tile.
fn tile_units_to_percentage(tile_dimensions: TileDimensions, tile_size: f32) -> PercentageCoords {
    let (x, y) = (tile_dimensions.x, tile_dimensions.y);

    // PercentageCoords {
    //     x: Percentage::new(offset_modulus(x, tile_size)),
    //     y: Percentage::new(offset_modulus(y, tile_size)),
    // }
    PercentageCoords::new(offset_modulus(x, tile_size), offset_modulus(y, tile_size))
}

/// Offset modulus
/// https://www.desmos.com/calculator/mm8gyvgjje
fn offset_modulus(value: f32, modulus: f32) -> f32 {
    -(f32::ceil(value / modulus) * modulus - value) / modulus + 1.0
}

/// Convert from pixel coordinates to tile coordinates
/// That is; if PixelsPerTile is 100, and the env.tile_size() is 10,
/// then pixel (134, 240) is (floor(134 / resolution), floor(240 / resolution))
/// = (1, 2) tile coords.
fn image_to_tile_coords(pixel_coords: PixelCoords, resolution: PixelsPerTile) -> TileCoords {
    let (x, y) = (pixel_coords.x as f32, pixel_coords.y as f32);

    let tile_coords = TileCoords {
        x: f32::floor(x / resolution.get() as f32) as TileIndex,
        y: f32::floor(y / resolution.get() as f32) as TileIndex,
    };

    tile_coords
}

// impl conversion from `Coords<Percentage>` to `gbp_geometry::RelativePoint`
impl TryFrom<PercentageCoords> for RelativePoint {
    type Error = unit_interval::UnitIntervalError;

    fn try_from(percentage_coords: PercentageCoords) -> Result<Self, Self::Error> {
        RelativePoint::new(
            percentage_coords.x().0 as f64,
            percentage_coords.y().0 as f64,
        )
    }
}

impl From<PercentageCoords> for Vec2 {
    fn from(percentage_coords: PercentageCoords) -> Self {
        Vec2::new(
            percentage_coords.x().0 as f32,
            percentage_coords.y().0 as f32,
        )
    }
}

/// Given tile coordinates, and coordinate in tile percentage, return whether
/// the coordinate is within a placeable obstacle placed in that tile.
fn is_placeable_obstacle(
    env: &Environment,
    tile_coords: TileCoords,
    percentage: PercentageCoords,
    expansion: Percentage,
) -> bool {
    let inverted_percentage = percentage.clone();
    for obstacle in env.obstacles.iter() {
        // let obstale_tile_coords = &obstacle.tile_coordinates;
        let obstacle_tile_coords = TileCoords {
            x: obstacle.tile_coordinates.col,
            y: obstacle.tile_coordinates.row,
        };

        if tile_coords != obstacle_tile_coords {
            continue;
        }

        // TODO: Expand the obstacle by the expansion percentage first
        let expanded_shape = obstacle.shape.expanded(expansion.0 as f64);
        let translated = Vec2::from(inverted_percentage) - Vec2::from(obstacle.translation); // - translation_offset;
                                                                                             // rotate the translated coordinated by the obstacle rotation
        let rotation_offset = match obstacle.shape {
            PlaceableShape::RegularPolygon(RegularPolygon { sides, radius: _ }) => {
                std::f32::consts::FRAC_PI_2
                    + std::f32::consts::FRAC_PI_2
                    + if sides % 2 != 0 {
                        std::f32::consts::PI / sides as f32
                    } else {
                        0.0
                    }
            }
            _ => std::f32::consts::FRAC_PI_2,
        };

        let rotated =
            glam::Quat::from_rotation_z(obstacle.rotation.as_radians() as f32 + rotation_offset)
                .mul_vec3(translated.extend(0.0))
                .xy();

        let inside_placeable_shape = expanded_shape.inside(rotated);

        if inside_placeable_shape {
            return true;
        }
    }

    false
}

/// Given tile coordinates, and coordinate in tile percentage, return whether
/// the coordinate is within and obstacle.
fn is_tile_obstacle(
    tile: char,
    path_width: Percentage,
    percentage: PercentageCoords,
    expansion: Percentage,
) -> bool {
    // let tile = env.tiles.grid.get_tile(tile.0, tile.1);
    let path_width = path_width - expansion;
    let almost_full = Percentage::new(1.0 - path_width.clone().0);
    let obstacle_width = Percentage::new(almost_full.0 / 2.0);
    let obstacle_and_path_width = obstacle_width + path_width.clone();
    let half = Percentage::new(0.5 - expansion.get() / 2.0);

    match tile {
        '─' => {
            if percentage.y() < obstacle_width || percentage.y() > obstacle_and_path_width {
                return true;
            }
        }
        '│' => {
            if percentage.x() < obstacle_width || percentage.x() > obstacle_and_path_width {
                return true;
            }
        }
        '╴' => {
            if percentage.y() < obstacle_width
                || percentage.y() > obstacle_and_path_width
                || percentage.x() > Percentage::new(0.5 - expansion.get() / 2.0)
            {
                return true;
            }
        }
        '╶' => {
            if percentage.y() < obstacle_width
                || percentage.y() > obstacle_and_path_width
                || percentage.x() < Percentage::new(0.5 + expansion.get() / 2.0)
            {
                return true;
            }
        }
        '╷' => {
            if percentage.x() < obstacle_width
                || percentage.x() > obstacle_and_path_width
                || percentage.y() < Percentage::new(0.5 + expansion.get() / 2.0)
            {
                return true;
            }
        }
        '╵' => {
            if percentage.x() < obstacle_width
                || percentage.x() > obstacle_and_path_width
                || percentage.y() > Percentage::new(0.5 - expansion.get() / 2.0)
            {
                return true;
            }
        }
        '┌' => {
            if percentage.x() < obstacle_width
                || percentage.y() < obstacle_width
                || (percentage.x() > obstacle_and_path_width
                    && percentage.y() > obstacle_and_path_width)
            {
                return true;
            }
        }
        '┐' => {
            if percentage.x() > obstacle_and_path_width
                || percentage.y() < obstacle_width
                || (percentage.x() < obstacle_width && percentage.y() > obstacle_and_path_width)
            {
                return true;
            }
        }
        '└' => {
            if percentage.x() < obstacle_width
                || percentage.y() > obstacle_and_path_width
                || (percentage.x() > obstacle_and_path_width && percentage.y() < obstacle_width)
            {
                return true;
            }
        }
        '┘' => {
            if percentage.x() > obstacle_and_path_width
                || percentage.y() > obstacle_and_path_width
                || (percentage.x() < obstacle_width && percentage.y() < obstacle_width)
            {
                return true;
            }
        }
        '┬' => {
            if percentage.y() < obstacle_width
                || (percentage.y() > obstacle_and_path_width
                    && (percentage.x() < obstacle_width
                        || percentage.x() > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┴' => {
            if percentage.y() > obstacle_and_path_width
                || (percentage.y() < obstacle_width
                    && (percentage.x() < obstacle_width
                        || percentage.x() > obstacle_and_path_width))
            {
                return true;
            }
        }
        '├' => {
            if percentage.x() < obstacle_width
                || (percentage.x() > obstacle_and_path_width
                    && (percentage.y() < obstacle_width
                        || percentage.y() > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┤' => {
            if percentage.x() > obstacle_and_path_width
                || (percentage.x() < obstacle_width
                    && (percentage.y() < obstacle_width
                        || percentage.y() > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┼' => {
            if (percentage.x() < obstacle_width || percentage.x() > obstacle_and_path_width)
                && (percentage.y() < obstacle_width || percentage.y() > obstacle_and_path_width)
            {
                return true;
            }
        }
        ' ' => {
            return true;
        }
        _ => return false,
    }

    false
}

/// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_to_tile_units() {
        let pixel_coords = PixelCoords { x: 23, y: 56 };
        let resolution = PixelsPerTile::new(100);
        let tile_size = 10.0;
        let tile_dimensions = image_to_tile_units(pixel_coords, resolution, tile_size);
        assert_eq!(tile_dimensions.x, 2.3);
        assert_eq!(tile_dimensions.y, 5.6);
    }

    #[test]
    fn test_tile_units_to_percentage() {
        let tile_dimensions = TileDimensions { x: 2.3, y: 5.6 };
        let tile_size = 10.0;
        let percentage = tile_units_to_percentage(tile_dimensions, tile_size);
        assert_eq!(percentage.x().0, 0.3);
        assert_eq!(percentage.y().0, 0.6);
    }

    #[test]
    fn test_image_to_tile_coords() {
        let pixel_coords = PixelCoords { x: 134, y: 240 };
        let resolution = PixelsPerTile::new(100);
        let tile_coords = image_to_tile_coords(pixel_coords, resolution);
        assert_eq!(tile_coords.x, 1);
        assert_eq!(tile_coords.y, 2);
    }

    #[test]
    fn test_is_obstacle() {
        let tile = '─';
        let path_width = Percentage::new(0.5);
        let percentage = PercentageCoords::new(0.3, 0.6);
        let expansion = Percentage::new(0.0);
        assert_eq!(
            is_tile_obstacle(tile, path_width, percentage, expansion),
            false
        );

        let tile = '─';
        let path_width = Percentage::new(0.5);
        let percentage = PercentageCoords::new(0.1, 0.6);
        let expansion = Percentage::new(0.0);
        assert_eq!(
            is_tile_obstacle(tile, path_width, percentage, expansion),
            true
        );
    }
}
