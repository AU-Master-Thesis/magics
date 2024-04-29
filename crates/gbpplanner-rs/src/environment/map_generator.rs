use std::sync::Arc;

use bevy::prelude::*;
use gbp_environment::{
    Circle, Environment, PlaceableShape, Rectangle, RegularPolygon, TileCoordinates, Triangle,
};
use parry2d::{
    na::{self, Isometry2, Vector2},
    shape,
};
use serde::{Deserialize, Serialize};

use crate::{
    asset_loader::Materials,
    bevy_utils::run_conditions::event_exists,
    // config::{environment::PlaceableShape, Config, DrawSetting, Environment},
    config::{Config, DrawSetting},
    input::DrawSettingsEvent,
    simulation_loader::LoadSimulation,
};

// static COLLIDERS: once_cell::sync::Lazy<std::sync::RwLock<Colliders>> =
//     once_cell::sync::Lazy::new(std::sync::RwLock::default);

pub struct GenMapPlugin;

impl Plugin for GenMapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Colliders>()
            // .add_systems(Startup, (build_tile_grid, build_obstacles))
            // .add_systems(PostStartup, create_static_colliders)
            .add_systems(
                Update,
                (clear_colliders, build_tile_grid, build_obstacles).chain().run_if(on_event::<LoadSimulation>()),
            )
            .add_systems(
                Update,
                show_or_hide_generated_map.run_if(event_exists::<DrawSettingsEvent>),
            );
    }
}

#[derive(Debug, Component)]
pub struct ObstacleMarker;

pub trait DebugShape: shape::Shape + std::fmt::Debug {}

impl DebugShape for shape::Cuboid {}

#[derive(Resource, Default)]
pub struct Colliders(Vec<(Isometry2<f32>, Arc<dyn shape::Shape>)>);
// where
//     S: shape::Shape + std::fmt::Debug;

impl Colliders {
    pub fn push(&mut self, position: Isometry2<f32>, shape: Arc<dyn shape::Shape>) {
        self.0.push((position, shape));
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Isometry2<f32>, Arc<dyn shape::Shape>)> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// fn create_static_colliders(colliders: Res<Colliders>) {
//     COLLIDERS.write().expect("Not poisoned pls").0 = colliders.0.clone();
// }

/// **Bevy** [`Startup`] _system_.
/// Takes the [`Environment`] configuration and generates all specified
/// [`Obstacles`].
///
/// [`Obstacles`] example:
/// ```rust
/// Obstacles(vec![
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::circle(0.1, (0.5, 0.5)),
///         Angle::new(0.0).unwrap(),
///     ),
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::triangle(0.1, 0.1, 0.5),
///         Angle::new(0.0).unwrap(),
///     ),
///     Obstacle::new(
///         (0, 0),
///         PlaceableShape::square(0.1, (0.75, 0.5)),
///         Angle::new(0.0).unwrap(),
///     ),
/// ]),
/// ```
///
/// Placement of all shapes is given as a `(x, y)` percentage local to a
/// specific tile
#[allow(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn build_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Res<Environment>,
    config: Res<Config>,
    // scene_assets: Res<SceneAssets>,
    materials: Res<Materials>,
) {
    let tile_grid = &env_config.tiles.grid;
    let tile_size = env_config.tile_size();
    let obstacle_height = env_config.obstacle_height();

    let grid_offset_x = tile_grid.ncols() as f32 / 2.0 - 0.5;
    let grid_offset_z = tile_grid.nrows() as f32 / 2.0 - 0.5;

    info!("Spawning obstacles");
    info!("{:?}", env_config.obstacles);
    info!(
        "env_config.obstacles.iter().count() = {:?}",
        env_config.obstacles.iter().count()
    );

    let obstacles_to_spawn = env_config.obstacles.iter().map(|obstacle| {
        let TileCoordinates { row, col } = obstacle.tile_coordinates;

        info!("Spawning obstacle at {:?}", (row, col));

        let tile_offset_x = col as f32;
        let tile_offset_z = row as f32;

        let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
        let offset_z = (tile_offset_z - grid_offset_z) * tile_size;

        let pos_offset = tile_size / 2.0;

        let translation = obstacle.translation;

        // Construct the correct shape
        match obstacle.shape {
            PlaceableShape::Circle(Circle { radius }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    (translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                info!("Spawning circle: r = {}, at {:?}", radius, center);
                let radius = radius.get() as f32 * tile_size;

                let mesh = meshes.add(Cylinder::new(radius, obstacle_height));
                let transform = Transform::from_translation(center);

                info!(
                    "Spawning cylinder: r = {}, h = {}, at {:?}",
                    radius, obstacle_height, transform
                );

                Some((mesh, transform))
            }
            PlaceableShape::Triangle(Triangle {
                base_length,
                height,
                mid_point,
            }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    // obstacle_height / 2.0,
                    0.0,
                    (translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                // Example triangle
                // |\
                // | \
                // |__\
                // In this case the `base_length` is 1.0, `height` is 3.0, and the `mid_point`
                // is 0.0, the 3.0 height is placed at the very left of the triangle.
                // This creates a right-angled triangle.
                // One could also have a negative `mid_point`, which would make the current
                // right-angle more obtuse, or a positive `mid_point`, which
                // would make the current right-angle more acute.

                let points = vec![
                    // bottom-left corner
                    Vec2::new(
                        -base_length.get() as f32 / 2.0 * tile_size,
                        -height.get() as f32 / 2.0 * tile_size,
                    ),
                    // bottom-right corner
                    Vec2::new(
                        base_length.get() as f32 / 2.0 * tile_size,
                        -height.get() as f32 / 2.0 * tile_size,
                    ),
                    // top corner
                    Vec2::new(
                        (mid_point as f32 - 0.5) * base_length.get() as f32 * tile_size,
                        height.get() as f32 / 2.0 * tile_size,
                    ),
                ];

                info!(
                    "Spawning triangle: base_length = {}, height = {}, at {:?}",
                    base_length, height, center
                );

                let mesh = meshes.add(
                    Mesh::try_from(bevy_more_shapes::Prism::new(obstacle_height, points))
                        .expect("Failed to create triangle mesh"),
                );

                let rotation = Quat::from_rotation_y(
                    std::f32::consts::FRAC_PI_2 + obstacle.rotation.as_radians() as f32,
                );
                let transform = Transform::from_translation(center).with_rotation(rotation);

                Some((mesh, transform))
            }
            PlaceableShape::RegularPolygon(RegularPolygon { sides, side_length }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    (translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                info!(
                    "Spawning regular polygon: sides = {}, side_length = {}, at {:?}",
                    sides, side_length, center
                );

                let mesh = meshes.add(Mesh::from(bevy_more_shapes::Cylinder {
                    height: obstacle_height,
                    radius_bottom: side_length.get() as f32 * tile_size / 2.0,
                    radius_top: side_length.get() as f32 * tile_size / 2.0,
                    radial_segments: sides as u32,
                    height_segments: 1,
                }));

                info!(
                    "obstacle.rotation.as_radians() = {:?}, std::f32::consts::FRAC_PI_4 = {:?}",
                    obstacle.rotation.as_radians() as f32,
                    std::f32::consts::FRAC_PI_4
                );

                let rotation = Quat::from_rotation_y(
                    std::f32::consts::FRAC_PI_4 + obstacle.rotation.as_radians() as f32,
                );
                let transform = Transform::from_translation(center).with_rotation(rotation);

                Some((mesh, transform))
            }
            PlaceableShape::Rectangle(Rectangle { width, height }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    (translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                info!(
                    "Spawning rectangle: width = {}, height = {}, at {:?}",
                    width, height, center
                );

                let mesh = meshes.add(Cuboid::new(
                    height.get() as f32 * tile_size / 2.0,
                    obstacle_height,
                    width.get() as f32 * tile_size / 2.0,
                ));

                let rotation = Quat::from_rotation_y(obstacle.rotation.as_radians() as f32);
                let transform = Transform::from_translation(center).with_rotation(rotation);

                Some((mesh, transform))
            }
        }
    });

    obstacles_to_spawn
        .flatten() // filter out None
        .for_each(|(mesh, transform)| {
            commands.spawn((
                PbrBundle {
                    mesh,
                    material: materials.obstacle.clone(),
                    transform,
                    visibility: if config.visualisation.draw.generated_map {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ..Default::default()
                },
                ObstacleMarker
            ));
        });

    // exit
    // std::process::exit(0);
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
/// Each tile e.g. tile (0,0) in the above grid "┌" or (3,1) "┬"
/// - Transforms into a 1x1 section of the map - later to be scaled
/// - Each tile's world position is calculated from the tile's position in the
///   grid
///     - Such that the map is centered
/// - Uses the `Environment.width` to determine the width of the paths,
///    - Otherwise, the empty space is filled with solid meshes
#[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
fn build_tile_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut colliders: ResMut<Colliders>,
    env_config: Res<Environment>,
    config: Res<Config>,
    materials: Res<Materials>,
    obstacles: Query<Entity, With<ObstacleMarker>>,
) {
    for entity in &obstacles {
        commands.entity(entity).despawn();
        info!("despawn obstacle entity: {:?}", entity);
    }

    let tile_grid = &env_config.tiles.grid;

    let obstacle_height = env_config.obstacle_height();
    let obstacle_y = obstacle_height / 2.0;

    let tile_size = env_config.tile_size();

    let path_width = env_config.path_width();
    let base_dim = tile_size * (1.0 - path_width) / 2.0;

    // offset caused by the size of the grid
    // - this centers the map
    let grid_offset_x = tile_grid.ncols() as f32 / 2.0 - 0.5;
    let grid_offset_z = tile_grid.nrows() as f32 / 2.0 - 0.5;

    let pos_offset = path_width.mul_add(tile_size, base_dim) / 2.0;

    for (y, row) in tile_grid.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            // offset of the individual tile in the grid
            // used in all match cases
            let tile_offset_x = x as f32;
            let tile_offset_z = y as f32;

            // total offset caused by grid and tile
            let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
            let offset_z = (tile_offset_z - grid_offset_z) * tile_size;
            // Vec<(Handle<Mesh>, Transform, parry2d::shape::Cuboid)>
            if let Some(obstacle_information) = match tile {
                '─' => {
                    // Horizontal straight path
                    // - 2 equal-sized larger cuboid on either side, spanning the entire width of
                    //   the tile

                    // let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    let cuboid = Cuboid::new(tile_size, obstacle_height, base_dim);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    Some(vec![
                        (
                            // left side
                            cuboid,
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            cuboid,
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
                    // - 2 equal-sized larger cuboid on either side, spanning the entire height of
                    //   the tile

                    let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    Some(vec![
                        (
                            // left side
                            // mesh_handle.clone(),
                            cuboid,
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right side
                            // mesh_handle.clone(),
                            cuboid,
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╴' => {
                    // Termination from the left
                    // - 2 larger cuboids on the top and bottom, spanning the entire width of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the right, to terminate the path

                    // Top and bottom
                    let cuboid = Cuboid::new(tile_size, obstacle_height, base_dim);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the right
                    let cuboid_plug =
                        Cuboid::new(base_dim, obstacle_height, path_width * tile_size);
                    // let parry_cuboid_plug: parry2d::shape::Cuboid = cuboid_plug.into();

                    Some(vec![
                        (
                            // top
                            cuboid,
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom
                            cuboid,
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right plug
                            cuboid_plug,
                            // right plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╶' => {
                    // Termination from the right
                    // - 2 larger cuboids on the top and bottom, spanning the entire width of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the left, to terminate the path

                    // Top and bottom
                    let cuboid = Cuboid::new(tile_size, obstacle_height, base_dim);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the left
                    let cuboid_plug =
                        Cuboid::new(base_dim, obstacle_height, path_width * tile_size);
                    // let parry_cuboid_plug: parry2d::shape::Cuboid = cuboid_plug.into();

                    Some(vec![
                        (
                            // top
                            cuboid,
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom
                            cuboid,
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left plug
                            cuboid_plug,
                            // left plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╷' => {
                    // Termination from the bottom
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the top, to terminate the path

                    // Left and right
                    let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the top
                    let cuboid_plug =
                        Cuboid::new(path_width * tile_size, obstacle_height, base_dim);
                    // let parry_cuboid_plug: parry2d::shape::Cuboid = cuboid_plug.into();

                    Some(vec![
                        (
                            // left
                            cuboid,
                            // left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right
                            cuboid,
                            // right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top plug
                            cuboid_plug,
                            // top plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                    ])
                }
                '╵' => {
                    // Termination from the top
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the bottom, to terminate the path

                    // Left and right
                    let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the bottom
                    let cuboid_plug =
                        Cuboid::new(path_width * tile_size, obstacle_height, base_dim);
                    // let parry_cuboid_plug: parry2d::shape::Cuboid = cuboid_plug.into();

                    Some(vec![
                        (
                            // left
                            cuboid,
                            // left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // right
                            cuboid,
                            // right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom plug
                            cuboid_plug,
                            // bottom plug transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                    ])
                }
                '┌' => {
                    // Top right hand turn
                    // - 1 cube in the bottom right corner
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the top side, spanning from the right to the above
                    //   cuboid

                    let cuboid_bottom_right = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let cuboid_left = Cuboid::new(base_dim, obstacle_height, tile_size);
                    let cuboid_top = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // bottom right cube
                            cuboid_bottom_right,
                            // bottom right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // left side
                            cuboid_left,
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            cuboid_top,
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
                    // - 1 larger cuboid on the right hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the top side, spanning from the left to the above cuboid

                    let cuboid_bottom_left = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let cuboid_right = Cuboid::new(base_dim, obstacle_height, tile_size);
                    let cuboid_top = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // bottom left cube
                            cuboid_bottom_left,
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // right side
                            cuboid_right,
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // top
                            cuboid_top,
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
                    // - 1 larger cuboid on the left hand side, spanning the entire height of the
                    //   tile
                    // - 1 larger cuboid on the bottom side, spanning from the right to the above
                    //   cuboid

                    let cuboid_top_right = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let cuboid_left = Cuboid::new(base_dim, obstacle_height, tile_size);
                    let cuboid_bottom = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // top right cube
                            cuboid_top_right,
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // left side
                            cuboid_left,
                            // left side transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            cuboid_bottom,
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
                    // - 1 larger cuboid on the right hand side, spannding the entire height of the
                    //   tile
                    // - 1 larger cuboid on the bottom side, spanning from the left to the above
                    //   cuboid

                    let cuboid_top_left = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let cuboid_right = Cuboid::new(base_dim, obstacle_height, tile_size);
                    let cuboid_bottom = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // top left cube
                            cuboid_top_left,
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // right side
                            cuboid_right,
                            // right side transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                        (
                            // bottom
                            cuboid_bottom,
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
                    // - 1 larger cuboid in the top center, spanning the entire width of the tile

                    let cube = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let top = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // bottom left cube
                            cube,
                            // bottom left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube,
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
                    // - 1 larger cuboid in the bottom center, spanning the entire width of the tile

                    let cube = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let bottom = Cuboid::new(tile_size, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // top left cube
                            cube,
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right cube
                            cube,
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
                    // - 1 larger cuboid in the left center, spanning the entire height of the tile

                    let cube = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let left = Cuboid::new(base_dim, obstacle_height, tile_size);

                    Some(vec![
                        (
                            // top right cube
                            cube,
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom right cube
                            cube,
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
                    // - 1 larger cuboid in the right center, spanning the entire height of the tile

                    let cube = Cuboid::new(base_dim, obstacle_height, base_dim);
                    let right = Cuboid::new(base_dim, obstacle_height, tile_size);

                    Some(vec![
                        (
                            // top left cube
                            cube,
                            // top left cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left cube
                            cube,
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

                    let cube = Cuboid::new(base_dim, obstacle_height, base_dim);

                    Some(vec![
                        (
                            // top left
                            cube,
                            // top left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top right
                            cube,
                            // top right transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // bottom left
                            cube,
                            // bottom left transform
                            Transform::from_translation(Vec3::new(
                                offset_x - pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom right
                            cube,
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
                    // Filled space
                    // - 1 larger cuboid, spanning the entire tile

                    let cuboid = Cuboid::new(tile_size, obstacle_height, tile_size);

                    Some(vec![(
                        cuboid,
                        Transform::from_translation(Vec3::new(offset_x, obstacle_y, offset_z)),
                    )])
                }
                _ => None,
            } {
                obstacle_information.iter().for_each(|(cuboid, transform)| {
                    colliders.push(
                        Isometry2::new(
                            Vector2::new(transform.translation.x, transform.translation.z),
                            na::zero(),
                        ),
                        Arc::new(Into::<shape::Cuboid>::into(*cuboid)),
                    );
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(*cuboid),
                            transform: *transform,
                            material: materials.obstacle.clone(),
                            visibility: if config.visualisation.draw.generated_map {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            },
                            ..Default::default()
                        },
                        TileCoordinates::new(x, y),
                        ObstacleMarker,
                    ));
                })
            }
        }
    }
}

/// **Bevy** [`Update`] _system_.
/// Shows or hides the generated map based on event from [`DrawSettingsEvent`].
/// - If `DrawSettingsEvent` is `ShowGeneratedMap`, all generated map entities'
///   visibility is changed according to the `DrawSettingsEvent.draw` boolean
///   field
fn show_or_hide_generated_map(
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
    mut query: Query<&mut Visibility, With<ObstacleMarker>>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, DrawSetting::GeneratedMap) {
            for mut visibility in &mut query {
                *visibility = if event.draw {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }
    }
}

/// **Bevy** system that clear all spawned obstacle colliders. Used to clear
/// existing colliders before a new simulation is loaded.
fn clear_colliders(mut colliders: ResMut<Colliders>) {
    let n_colliders = colliders.0.len();
    colliders.0.clear();
    info!("{} colliders cleared", n_colliders);
}
