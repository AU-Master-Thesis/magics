use std::num::NonZeroU32;

use gbpplanner_rs::config::Environment;
use image::RgbImage;

/// Custom resolution type, as pixels per tile.
pub type PixelsPerTile = NonZeroU32;

/// Convert [`Environment`] to a PNG image.
pub fn env_to_png(env: Environment, resolution: PixelsPerTile) -> anyhow::Result<RgbImage> {
    let tile_size = env.tile_size();
    let (ncols, nrows) = (env.tiles.grid.ncols(), env.tiles.grid.nrows());

    let mut image = RgbImage::new(ncols as u32 * resolution.get(), nrows as u32 * resolution.get());

    Ok(image)
}
