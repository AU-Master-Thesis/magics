use gbpplanner_rs::config::Environment;

/// Convert [`Environment`] to a PNG image.
pub fn env_to_png(env: Environment) {
    let tile_size = env.tile_size();
    env.tiles.grid.nrows()
}
