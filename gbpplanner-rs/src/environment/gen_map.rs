use bevy::prelude::*;

use crate::{asset_loader::SceneAssets, config::Environment};

pub struct GenMapPlugin;

impl Plugin for GenMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_system);
    }
}

#[derive(Component)]
pub struct MapCell {
    x: usize,
    y: usize,
}

/// **Bevy** [`Startup`] _system_.
/// Takes the [`Environment`] configuration and generates a map.
///
/// Transforms an input like:
/// ```text
/// ┌┬┐
/// ┘└┼┬
///   └┘
/// ```
/// Into visual meshes, defining the physical boundaries of the map
/// - The lines are not walls, but paths
/// - Where the empty space are walls/obstacles
///
/// Each cell e.g. cell (0,0) in the above grid "┌" or (3,1) "┬"
/// - Transforms into a 1x1 section of the map - later to be scaled
/// - Each cell's world position is calculated from the cell's position in the grid
///     - Such that the map is centered
/// - Uses the `Environment.width` to determine the width of the paths,
///    - Otherwise, the empty space is filled with solid meshes
fn build_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Res<Environment>,
    scene_assets: Res<SceneAssets>,
) {
    let matrix = &env_config.grid;

    let obstacle_height = env_config.obstacle_height();
    let obstacle_y = obstacle_height / 2.0;

    let tile_size = env_config.tile_size();

    let path_width = env_config.path_width();
    let base_dim = tile_size * (1.0 - path_width) / 2.0;

    // offset caused by the size of the grid
    // - this centers the map
    let grid_offset_x = matrix.cols() as f32 / 2.0 - 0.5;
    let grid_offset_z = matrix.rows() as f32 / 2.0 - 0.5;

    let pos_offset = (base_dim + path_width * tile_size) / 2.0;

    for (y, row) in matrix.iter().enumerate() {
        for (x, cell) in row.chars().enumerate() {
            // offset of the individual cell in the grid
            // used in all match cases
            let cell_offset_x = x as f32;
            let cell_offset_z = y as f32;

            // total offset caused by grid and cell
            let offset_x = (cell_offset_x - grid_offset_x) * tile_size;
            let offset_z = (cell_offset_z - grid_offset_z) * tile_size;
            if let Some(mesh_transforms) = match cell {
                '─' => {
                    // Horizontal straight path
                    // - 2 equal-sized larger cuboid on either side, spanning the entire width of the cell

                    Some(vec![
                        (
                            // left side
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '│' => {
                    // Vertical straight path
                    // - 2 equal-sized larger cuboid on either side, spanning the entire height of the cell

                    Some(vec![
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '┌' => {
                    // Top right hand turn
                    // - 1 cube in the bottom right corner
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the cell
                    // - 1 larger cuboid on the top side, spanning from the right to the above cuboid

                    Some(vec![
                        (
                            // bottom right cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '┐' => {
                    // Top left hand turn
                    // - 1 cube in the bottom left corner
                    // - 1 larger cuboid on the right hand side, spanning the entire height of the cell
                    // - 1 larger cuboid on the top side, spanning from the left to the above cuboid

                    Some(vec![
                        (
                            // bottom left cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '└' => {
                    // Bottom right hand turn
                    // - 1 cube in the top right corner
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the cell
                    // - 1 larger cuboid on the bottom side, spanning from the right to the above cuboid

                    Some(vec![
                        (
                            // top right cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // left side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┘' => {
                    // Bottom left hand turn
                    // - 1 cube in the top left corner
                    // - 1 larger cuboid on the right hand side, spannding the entire height of the cell
                    // - 1 larger cuboid on the bottom side, spanning from the left to the above cuboid

                    Some(vec![
                        (
                            // top left cube
                            meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim)),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size)),
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim)),
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┬' => {
                    // Top T-junction
                    // - 2 equal-sized cubes, one in each bottom corner
                    // - 1 larger cuboid in the top center, spanning the entire width of the cell

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let top = meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // bottom left cube
                            cube.clone(),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube.clone(),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // top center cuboid
                            top,
                            // top center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '┴' => {
                    // Bottom T-junction
                    // - 2 equal-sized cubes, one in each top corner
                    // - 1 larger cuboid in the bottom center, spanning the entire width of the cell

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let bottom = meshes.add(Cuboid::new(tile_size, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // top left cube
                            cube.clone(),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right cube
                            cube.clone(),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom center cuboid
                            bottom,
                            // bottom center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '├' => {
                    // Right T-junction
                    // - 2 equal-sized cubes, one in each right corner
                    // - 1 larger cuboid in the left center, spanning the entire height of the cell

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let left = meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size));

                    Some(vec![
                        (
                            // top right cube
                            cube.clone(),
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube.clone(),
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left center cuboid
                            left,
                            // left center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '┤' => {
                    // Left T-junction
                    // - 2 equal-sized cubes, one in each left corner
                    // - 1 larger cuboid in the right center, spanning the entire height of the cell

                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));
                    let right = meshes.add(Cuboid::new(base_dim, obstacle_height, tile_size));

                    Some(vec![
                        (
                            // top left cube
                            cube.clone(),
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left cube
                            cube.clone(),
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right center cuboid
                            right,
                            // right center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '┼' => {
                    // 4-way intersection
                    // - 4 equal-sized cubes, one in each corner
                    let cube = meshes.add(Cuboid::new(base_dim, obstacle_height, base_dim));

                    Some(vec![
                        (
                            // top left
                            cube.clone(),
                            // top left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right
                            cube.clone(),
                            // top right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left
                            cube.clone(),
                            // bottom left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right
                            cube.clone(),
                            // bottom right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                ' ' => {
                    // Empty space
                    // - 1 larger cuboid, spanning the entire cell
                    Some(vec![(
                        meshes.add(Cuboid::new(tile_size, obstacle_height, tile_size)),
                        Transform::from_translation(Vec3::new(offset_x, obstacle_y, offset_z)),
                    )])
                }
                _ => None,
            } {
                mesh_transforms.iter().for_each(|(mesh, transform)| {
                    commands.spawn((
                        PbrBundle {
                            mesh: mesh.clone(),
                            transform: *transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        MapCell { x, y },
                    ));
                });
            }
        }
    }
}
