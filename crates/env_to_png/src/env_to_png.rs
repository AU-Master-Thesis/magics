use std::num::NonZeroU32;

// use gbpplanner_rs::config::Environment;
use gbp_environment::Environment;
use gbp_geometry::RelativePoint;
use glam::{Vec2, Vec3Swizzles};
use image::RgbImage;

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
        // Self((value - f32::EPSILON) % 1.0)
        // Self(value % 1.0)
    }
}

/// Custom type for percentage coordinates.
pub type PercentageCords = Coords<Percentage>;

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
            // println!("({}, {}): Pixelcoords", x, y);
            let tile_coords = image_to_tile_coords(pixel_coords.clone(), resolution);
            // println!("({}, {}): Tilecoords", tile_coords.x, tile_coords.y);
            let tile_dimensions = image_to_tile_units(pixel_coords.clone(), resolution, tile_size);
            // println!(
            // "({}, {}): Tileunits",
            // tile_unit_coords.x, tile_unit_coords.y
            // );
            let percentage_coords = tile_units_to_percentage(tile_dimensions, tile_size);
            // if (x == 199 || y == 199) && (tile_coords.x == 1 && tile_coords.y == 1) {
            //     println!("({}, {}): Pixelcoords", x, y);
            //     println!(
            //         "({}, {}): tile dimensions",
            //         tile_dimensions.x, tile_dimensions.y
            //     );
            //     println!(
            //         "({}, {}): Percentage",
            //         percentage_coords.x.0, percentage_coords.y.0
            //     );
            // }

            if let Some(tile) = env.tiles.grid.get_tile(tile_coords.y, tile_coords.x) {
                if is_tile_obstacle(
                    tile,
                    Percentage::new(env.path_width()),
                    percentage_coords,
                    expansion,
                ) || is_placeable_obstacle(&env, tile_coords, percentage_coords, expansion)
                {
                    // println!("({}, {}): Obstacle", x, y);
                    image.put_pixel(x, y, image::Rgb([0, 0, 0]));
                } else {
                    // println!("({}, {}): No obstacle", x, y);
                    // let boundary_pixels: [u32; 4] = std::array::from_fn(|i| (i as u32 + 1) +
                    // 100); if boundary_pixels.contains(&x) ||
                    // boundary_pixels.contains(&y) {     println!("({}, {}):
                    // Not obstacle", x, y); }
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
fn tile_units_to_percentage(tile_dimensions: TileDimensions, tile_size: f32) -> PercentageCords {
    let (x, y) = (tile_dimensions.x, tile_dimensions.y);

    PercentageCords {
        x: Percentage::new(offset_modulus(x, tile_size)),
        y: Percentage::new(offset_modulus(y, tile_size)),
    }
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

    // if x == 99.0 || y == 99.0 {
    //     println!("({}, {}): Pixelcoords", x, y);
    //     println!("({}, {}): Tilecoords", tile_coords.x, tile_coords.y);
    // }

    tile_coords
}

// impl conversion from `Coords<Percentage>` to `gbp_geometry::RelativePoint`
impl TryFrom<PercentageCords> for RelativePoint {
    type Error = unit_interval::UnitIntervalError;

    fn try_from(percentage_coords: PercentageCords) -> Result<Self, Self::Error> {
        RelativePoint::new(percentage_coords.x.0 as f64, percentage_coords.y.0 as f64)
    }
}

impl From<PercentageCords> for Vec2 {
    fn from(percentage_coords: PercentageCords) -> Self {
        Vec2::new(percentage_coords.x.0 as f32, percentage_coords.y.0 as f32)
    }
}

/// Given tile coordinates, and coordinate in tile percentage, return whether
/// the coordinate is whithin a placeable obstacle placed in that tile.
fn is_placeable_obstacle(
    env: &Environment,
    tile_coords: TileCoords,
    percentage: PercentageCords,
    expansion: Percentage,
) -> bool {
    for (i, obstacle) in env.obstacles.iter().enumerate() {
        // println!("Obstacle: {}", i);
        // let obstale_tile_coords = &obstacle.tile_coordinates;
        let obstacle_tile_coords = TileCoords {
            x: obstacle.tile_coordinates.col.clone(),
            y: obstacle.tile_coordinates.row.clone(),
        };

        if tile_coords != obstacle_tile_coords {
            continue;
        }

        // TODO: Expand the obstacle by the expansion percentage first
        let expanded_shape = obstacle.shape.expanded(expansion.0 as f64);

        // translate percentage to obstacle coordinates
        // percentage.x.0 -= obstacle.translation.x.get() as f32;
        let translated = Vec2::from(percentage) - Vec2::from(obstacle.translation);
        // rotate the translated coordinated by the obstacle rotation
        let rotated = glam::Quat::from_rotation_z(obstacle.rotation.as_radians() as f32)
            .mul_vec3(translated.extend(0.0))
            .xy();

        let inside_placeable_shape = expanded_shape.inside(rotated);
        // if inside_placeable_shape {
        //     println!("({:.2}, {:.2}) is inside", percentage.x.0, percentage.y.0);
        // }
        if inside_placeable_shape {
            return true;
        }
    }

    false
}

/// Given tile coordinates, and coordinate in tile percentage, return whether
/// the coordinate is whithin and obstacle.
fn is_tile_obstacle(
    // env: Environment,
    // tile: TileCoordinates,
    tile: char,
    path_width: Percentage,
    percentage: PercentageCords,
    expansion: Percentage,
) -> bool {
    // let tile = env.tiles.grid.get_tile(tile.0, tile.1);
    let path_width = path_width - expansion;
    let almost_full = Percentage::new(1.0 - path_width.clone().0);
    let obstacle_width = Percentage::new(almost_full.0 / 2.0);
    let obstacle_and_path_width = obstacle_width + path_width.clone();

    match tile {
        '─' => {
            if percentage.y < obstacle_width || percentage.y > obstacle_and_path_width {
                return true;
            }
        }
        '│' => {
            if percentage.x < obstacle_width || percentage.x > obstacle_and_path_width {
                return true;
            }
        }
        '╴' => {
            if percentage.y < obstacle_width
                || percentage.y > obstacle_and_path_width
                || percentage.x > almost_full
            {
                return true;
            }
        }
        '╶' => {
            if percentage.y < obstacle_width
                || percentage.y > obstacle_and_path_width
                || percentage.x < path_width
            {
                return true;
            }
        }
        '╷' => {
            if percentage.x < obstacle_width
                || percentage.x > obstacle_and_path_width
                || percentage.y > almost_full
            {
                return true;
            }
        }
        '╵' => {
            if percentage.x < obstacle_width
                || percentage.x > obstacle_and_path_width
                || percentage.y < path_width
            {
                return true;
            }
        }
        '┌' => {
            if percentage.x < obstacle_width
                || percentage.y < obstacle_width
                || (percentage.x > obstacle_and_path_width
                    && percentage.y > obstacle_and_path_width)
            {
                return true;
            }
        }
        '┐' => {
            if percentage.x > obstacle_and_path_width
                || percentage.y < obstacle_width
                || (percentage.x < obstacle_width && percentage.y > obstacle_and_path_width)
            {
                return true;
            }
        }
        '└' => {
            if percentage.x < obstacle_width
                || percentage.y > obstacle_and_path_width
                || (percentage.x > obstacle_and_path_width && percentage.y < obstacle_width)
            {
                return true;
            }
        }
        '┘' => {
            if percentage.x > obstacle_and_path_width
                || percentage.y > obstacle_and_path_width
                || (percentage.x < obstacle_width && percentage.y < obstacle_width)
            {
                return true;
            }
        }
        '┬' => {
            if percentage.y < obstacle_width
                || (percentage.y > obstacle_and_path_width
                    && (percentage.x < obstacle_width || percentage.x > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┴' => {
            if percentage.y > obstacle_and_path_width
                || (percentage.y < obstacle_width
                    && (percentage.x < obstacle_width || percentage.x > obstacle_and_path_width))
            {
                return true;
            }
        }
        '├' => {
            if percentage.x < obstacle_width
                || (percentage.x > obstacle_and_path_width
                    && (percentage.y < obstacle_width || percentage.y > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┤' => {
            if percentage.x > obstacle_and_path_width
                || (percentage.x < obstacle_width
                    && (percentage.y < obstacle_width || percentage.y > obstacle_and_path_width))
            {
                return true;
            }
        }
        '┼' => {
            if (percentage.x < obstacle_width || percentage.x > obstacle_and_path_width)
                && (percentage.y < obstacle_width || percentage.y > obstacle_and_path_width)
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
    fn test_tile_to_image_coords() {
        let tile_coords = TileCoords { x: 2, y: 5 };
        let resolution = PixelsPerTile::new(100);
        let tile_size = 10.0;
        let pixel_coords = tile_to_image_coords(tile_coords, resolution, tile_size);
        assert_eq!(pixel_coords.x, 20);
        assert_eq!(pixel_coords.y, 50);
    }

    #[test]
    fn test_tile_units_to_percentage() {
        let tile_dimensions = TileDimensions { x: 2.3, y: 5.6 };
        let tile_size = 10.0;
        let percentage = tile_units_to_percentage(tile_dimensions, tile_size);
        assert_eq!(percentage.x.0, 0.3);
        assert_eq!(percentage.y.0, 0.6);
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
        let percentage = PercentageCords {
            x: Percentage::new(0.3),
            y: Percentage::new(0.6),
        };
        let expansion = Percentage::new(0.0);
        assert_eq!(
            is_tile_obstacle(tile, path_width, percentage, expansion),
            false
        );

        let tile = '─';
        let path_width = Percentage::new(0.5);
        let percentage = PercentageCords {
            x: Percentage::new(0.1),
            y: Percentage::new(0.6),
        };
        let expansion = Percentage::new(0.0);
        assert_eq!(
            is_tile_obstacle(tile, path_width, percentage, expansion),
            true
        );
    }
}
