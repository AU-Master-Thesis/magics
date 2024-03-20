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

    for (y, row) in matrix.iter().enumerate() {
        for (x, cell) in row.chars().enumerate() {
            match cell {
                '─' => {
                    // Horizontal straight path
                    let width = 1.0 / 2.0 - path_width / 2.0;
                    let cuboid = meshes.add(Cuboid::new(1.0, obstacle_height, width));

                    let top_transform = Transform::from_translation(Vec3::new(
                        x as f32 - matrix.cols() as f32 / 2.0,
                        0.0,
                        y as f32 - matrix.rows() as f32 / 2.0 - width,
                    ));

                    let bottom_transform = Transform::from_translation(Vec3::new(
                        x as f32 - matrix.cols() as f32 / 2.0,
                        0.0,
                        y as f32 - matrix.rows() as f32 / 2.0 + width,
                    ));

                    commands.spawn((
                        PbrBundle {
                            mesh: cuboid.clone(),
                            transform: top_transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        PbrBundle {
                            mesh: cuboid.clone(),
                            transform: bottom_transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        MapCell { x, y },
                    ));
                }
                '│' => {
                    // Vertical straight path
                    let width = 1.0 / 2.0 - path_width / 2.0;
                    let cuboid = meshes.add(Cuboid::new(width, obstacle_height, 1.0));

                    let left_transform = Transform::from_translation(Vec3::new(
                        x as f32 - matrix.cols() as f32 / 2.0 - width,
                        0.0,
                        y as f32 - matrix.rows() as f32 / 2.0,
                    ));

                    let right_transform = Transform::from_translation(Vec3::new(
                        x as f32 - matrix.cols() as f32 / 2.0 + width,
                        0.0,
                        y as f32 - matrix.rows() as f32 / 2.0,
                    ));

                    commands.spawn((
                        PbrBundle {
                            mesh: cuboid.clone(),
                            transform: left_transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        PbrBundle {
                            mesh: cuboid.clone(),
                            transform: right_transform,
                            material: scene_assets.materials.obstacle.clone(),
                            ..Default::default()
                        },
                        MapCell { x, y },
                    ));
                }
                '┌' => {
                    todo!("Top right hand turn")
                }
                '┐' => {
                    todo!("Top left hand turn")
                }
                '└' => {
                    todo!("Bottom right hand turn")
                }
                '┘' => {
                    todo!("Bottom left hand turn")
                }
                '┬' => {
                    todo!("Top T-junction")
                }
                '┴' => {
                    todo!("Bottom T-junction")
                }
                '┼' => {
                    // 4-way intersection
                    // 4 qual-sized cubes, one in each corner
                    info!("Building a 4-way intersection");
                    info!("Path width: {}", path_width);

                    let dim = tile_size * (1.0 - path_width) / 2.0;
                    info!("X,Z Dim: {}", dim);
                    let cube = meshes.add(Cuboid::new(dim, obstacle_height, dim));

                    // offsetting each cube into its corner
                    let pos_offset = (dim + path_width * tile_size) / 2.0;

                    // offset cause by the size of the grid
                    // - this centers the map
                    let grid_offset_x = matrix.cols() as f32 / 2.0 - 0.5;
                    let grid_offset_z = matrix.rows() as f32 / 2.0 - 0.5;

                    // offset of the individual cell in the grid
                    let cell_offset_x = x as f32;
                    let cell_offset_z = y as f32;

                    // summation of the offsets
                    let offset_x = (cell_offset_x - grid_offset_x) * tile_size;
                    let offset_z = (cell_offset_z - grid_offset_z) * tile_size;
                    let transforms = [
                        // top left
                        Transform::from_translation(Vec3::new(
                            offset_x - pos_offset,
                            obstacle_y,
                            offset_z - pos_offset,
                        )),
                        // top right
                        Transform::from_translation(Vec3::new(
                            offset_x + pos_offset,
                            obstacle_y,
                            offset_z - pos_offset,
                        )),
                        // bottom left
                        Transform::from_translation(Vec3::new(
                            offset_x - pos_offset,
                            obstacle_y,
                            offset_z + pos_offset,
                        )),
                        // bottom right
                        Transform::from_translation(Vec3::new(
                            offset_x + pos_offset,
                            obstacle_y,
                            offset_z + pos_offset,
                        )),
                    ];

                    info!("Transforms: {:?}", transforms);

                    transforms.iter().for_each(|transform| {
                        let entity = commands
                            .spawn((
                                PbrBundle {
                                    mesh: cube.clone(),
                                    transform: *transform,
                                    material: scene_assets.materials.obstacle.clone(),
                                    ..Default::default()
                                },
                                MapCell { x, y },
                            ))
                            .id();
                        info!("Intersection entity: {:?}", entity);
                    });
                }
                _ => {}
            }
        }
    }
}
