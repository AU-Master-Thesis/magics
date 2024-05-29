use std::sync::Arc;

use bevy::{prelude::*, reflect::Tuple};
use bevy_mod_picking::prelude::*;
use gbp_config::{Config, DrawSetting};
use gbp_environment::{
    Circle, Environment, PlaceableShape, Rectangle, RegularPolygon, TileCoordinates, Triangle,
};
use gbp_global_planner::Colliders;
use parry2d::{
    na::{self, Isometry2, Vector2},
    shape,
};

use crate::{
    asset_loader::Materials, bevy_utils::run_conditions::event_exists, input::DrawSettingsEvent,
    simulation_loader::LoadSimulation,
};

pub struct GenMapPlugin;

impl Plugin for GenMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<events::ObstacleClickedOn>()
            // .init_resource::<Colliders>()
            // .add_systems(Startup, (build_tile_grid, build_obstacles))
            // .add_systems(PostStartup, create_static_colliders)
            .add_systems(
                Update,
                (build_tile_grid.pipe(build_obstacles.pipe(insert_colliders_resource))).chain().run_if(on_event::<LoadSimulation>()),
            )
            .add_systems(
                Update,
                show_or_hide_generated_map.run_if(event_exists::<DrawSettingsEvent>),
            );
    }
}

pub mod events {
    use super::*;

    #[derive(Debug, Event)]
    pub struct ObstacleClickedOn(pub Entity);

    impl From<ListenerInput<Pointer<Click>>> for ObstacleClickedOn {
        #[inline]
        fn from(value: ListenerInput<Pointer<Click>>) -> Self {
            Self(value.target)
        }
    }
}

#[derive(Debug, Component)]
pub struct ObstacleMarker;

// #[derive(Clone)]
// pub struct Collider {
//     pub associated_mesh: Option<Entity>,
//     pub isometry: Isometry2<f32>,
//     pub shape: Arc<dyn shape::Shape>,
// }

// impl Collider {
//     #[inline]
//     pub fn aabb(&self) -> parry2d::bounding_volume::Aabb {
//         self.shape.compute_aabb(&self.isometry)
//     }
// }

// #[derive(Resource, Default, Clone)]
// pub struct Colliders(Vec<Collider>);

// impl Colliders {
//     delegate! {
//         to self.0 {
//             #[call(iter)]
//             pub fn iter(&self) -> impl Iterator<Item = &Collider>;

//             #[call(len)]
//             pub fn len(&self) -> usize;

//             #[call(is_empty)]
//             pub fn is_empty(&self) -> bool;

//             #[call(clear)]
//             pub fn clear(&mut self);
//         }
//     }

//     pub fn push(
//         &mut self,
//         associated_mesh: Option<Entity>,
//         position: Isometry2<f32>,
//         shape: Arc<dyn shape::Shape>,
//     ) {
//         self.0.push(Collider {
//             associated_mesh,
//             isometry: position,
//             shape,
//         });
//     }
// }

fn insert_colliders_resource(In(colliders): In<Colliders>, mut commands: Commands) {
    commands.insert_resource(colliders);
}

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
    In(mut colliders): In<Colliders>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Res<Environment>,
    config: Res<Config>,
    // scene_assets: Res<SceneAssets>,
    materials: Res<Materials>,
) -> Colliders {
    let tile_grid = &env_config.tiles.grid;
    let tile_size = env_config.tile_size();
    let obstacle_height = -env_config.obstacle_height();

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
        match &obstacle.shape {
            PlaceableShape::Circle(Circle { radius }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    (1.0 - translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                info!("Spawning circle: r = {}, at {:?}", radius, center);
                let radius = radius.get() as f32 * tile_size;

                let mesh = meshes.add(Cylinder::new(radius, obstacle_height));
                let transform = Transform::from_translation(center);

                info!(
                    "Spawning cylinder: r = {}, h = {}, at {:?}",
                    radius, obstacle_height, transform
                );

                let shape = parry2d::shape::Ball::new(radius);
                let shape: Arc<dyn shape::Shape> = Arc::new(shape);

                let isometry = Isometry2::new(
                    parry2d::na::Vector2::new(transform.translation.x, transform.translation.z),
                    na::zero(),
                );

                Some((mesh, transform, isometry, shape))
            }
            PlaceableShape::Triangle(ref triangle_shape @ Triangle { angles, radius }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    // obstacle_height / 2.0,
                    obstacle_height,
                    -((translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset),
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
                let [p1, p2, p3] = triangle_shape.points().map(|point| {
                    Vec2::new(-point.x as f32 * tile_size, point.y as f32 * tile_size)
                });

                // reflect around the x-axis
                // let p1 = Vec2::new(p1.x, -p1.y);
                // let p2 = Vec2::new(p2.x, -p2.y);
                // let p3 = Vec2::new(p3.x, -p3.y);

                // println!("bottom-left = {:?}", center.xz() + p1);
                // println!("bottom-right = {:?}", center.xz() + p2);
                // println!("top = {:?}", center.xz() + p3);

                // info!(
                //     "Spawning triangle: base_length = {}, height = {}, at {:?}",
                //     base_length, height, center
                // );

                let mesh = meshes.add(
                    Mesh::try_from(bevy_more_shapes::Prism::new(
                        -obstacle_height,
                        [p1, p2, p3].to_vec(),
                    ))
                    .expect("Failed to create triangle mesh"),
                );

                let rotation_angle: f32 =
                    std::f32::consts::FRAC_PI_2 - obstacle.rotation.as_radians() as f32;
                let rotation = Quat::from_rotation_y(rotation_angle);

                let isometry = Isometry2::new(
                    parry2d::na::Vector2::new(center.x, center.z),
                    rotation_angle - std::f32::consts::FRAC_PI_2,
                );

                let transform = Transform::from_translation(center).with_rotation(rotation);

                let rotate = |p: Vec2| {
                    rotation
                        .mul_vec3(p.extend(0.0).xzy())
                        .xz()
                        .to_array()
                        .into()
                };

                let shape = parry2d::shape::Triangle::new(rotate(p1), rotate(p2), rotate(p3));
                let shape: Arc<dyn shape::Shape> = Arc::new(shape);

                Some((mesh, transform, isometry, shape))
            }
            PlaceableShape::RegularPolygon(ref polygon @ RegularPolygon { sides, radius }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    -((translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset),
                );

                info!(
                    "Spawning regular polygon: sides = {}, radius = {}, at {:?}",
                    sides, radius, center
                );

                let mesh = meshes.add(Mesh::from(bevy_more_shapes::Cylinder {
                    height: -obstacle_height,
                    radius_bottom: radius.get() as f32 * tile_size / 2.0,
                    radius_top: radius.get() as f32 * tile_size / 2.0,
                    radial_segments: *sides as u32,
                    height_segments: 1,
                }));

                // info!(
                //     "obstacle.rotation.as_radians() = {:?}, std::f32::consts::FRAC_PI_4 =
                // {:?}",     obstacle.rotation.as_radians() as f32,
                //     std::f32::consts::FRAC_PI_4
                // );

                let rotation_angle =
                    std::f32::consts::FRAC_PI_4 + obstacle.rotation.as_radians() as f32;
                let rotation = Quat::from_rotation_y(
                    rotation_angle, /* std::f32::consts::FRAC_PI_4 +
                                     * obstacle.rotation.as_radians() as f32, */
                );
                let transform = Transform::from_translation(center).with_rotation(rotation);

                // let rotation_offset = match obstacle.shape {
                //             PlaceableShape::RegularPolygon(RegularPolygon { sides, radius })
                // => {                 std::f32::consts::PI
                //                     + if sides % 2 != 0 { std::f32::consts::PI / sides as f32
                //                     } else {
                //                         0.0
                //                     }
                //             }
                //             _ => std::f32::consts::FRAC_PI_2,
                //         };

                use std::f32::consts::{FRAC_PI_2, PI};
                let rotation_offset = PI
                    + match polygon.sides {
                        4 => 0.0,
                        n if n % 2 != 0 => FRAC_PI_2,
                        _ => -FRAC_PI_2,
                    };
                // let rotation_offset = PI
                //     + match polygon.sides { n if n % 2 != 0 => PI / n as f32, _ => 0.0, // n
                //       => FRAC_PI_2 / n as f32,
                //     };

                // + if polygon.sides % 2 != 0 { PI / polygon.sides as f32
                // } else {
                //     0.0
                // };

                // let rotation2 = Quat::from_rotation_y(std::f32::consts::PI / polygon.sides as
                // f32);

                let rotation_angle = obstacle.rotation.as_radians() as f32 + rotation_offset;
                let rotation2 =
                    Quat::from_rotation_z(obstacle.rotation.as_radians() as f32 + rotation_offset);
                // let rotation2 = Quat::from_rotation_z(rotation_offset);

                // let points: Vec<parry2d::math::Point<parry2d::math::Real>> = polygon
                let scale = tile_size / 2.0;
                let points: Vec<_> = polygon
                    .points()
                    .iter()
                    .map(|[x, y]| {
                        let p = Vec3::new(*x as f32, *y as f32, 0.0);
                        // let p = Vec3::new(*x as f32, 0.0, *y as f32);
                        // let p_rotated = rotation.mul_vec3(p);
                        let p_rotated = rotation2.mul_vec3(p);
                        dbg!((&p, &p_rotated));

                        // let p_rotated = p;
                        parry2d::math::Point::new(
                            p_rotated.x * scale,
                            p_rotated.y * scale,
                            // *x as f32 * (tile_size / 2.0),
                            // *y as f32 * (tile_size / 2.0),
                        )
                    })
                    // .inspect(|p| println!("p = {:?}", p))
                    .collect();
                let shape = parry2d::shape::ConvexPolygon::from_convex_hull(points.as_slice())
                    .expect("polygon is always convex");

                let shape: Arc<dyn shape::Shape> = Arc::new(shape);
                let isometry = Isometry2::new(
                    parry2d::na::Vector2::new(transform.translation.x, transform.translation.z),
                    rotation_angle,
                );

                Some((mesh, transform, isometry, shape))
            }
            PlaceableShape::Rectangle(Rectangle { width, height }) => {
                // dbg!((
                //     "Rectangle",
                //     translation.y.get() as f32,
                //     tile_size,
                //     -offset_z,
                //     pos_offset,
                //     width,
                //     height,
                // ));
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    -((translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset),
                );

                info!(
                    "Spawning rectangle: width = {}, height = {}, at {:?}",
                    width, height, center
                );

                let mesh = meshes.add(Cuboid::new(
                    width.get() as f32 * tile_size / 2.0,
                    obstacle_height,
                    height.get() as f32 * tile_size / 2.0,
                ));

                // let rotation = Quat::from_rotation_y(obstacle.rotation.as_radians() as f32);
                // let transform = Transform::from_translation(center).with_rotation(rotation);
                let transform = Transform::from_translation(center);

                let half_extents: parry2d::na::Vector2<parry2d::math::Real> =
                    parry2d::na::Vector2::from_vec(vec![
                        width.get() as f32 * tile_size / 4.0,
                        height.get() as f32 * tile_size / 4.0,
                    ]);

                let shape = parry2d::shape::Cuboid::new(half_extents);

                let shape: Arc<dyn shape::Shape> = Arc::new(shape);

                let isometry = Isometry2::new(
                    parry2d::na::Vector2::new(transform.translation.x, transform.translation.z),
                    na::zero(),
                );

                Some((mesh, transform, isometry, shape))
            }
        }
    });

    obstacles_to_spawn
        .flatten() // filter out None
        .for_each(|(mesh, transform, isometry, shape)| {
            // TODO: remember to get rotation of obstacle, i.e. for triangles
            let entity = commands.spawn((
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
                ObstacleMarker,
                bevy_mod_picking::PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<events::ObstacleClickedOn>(),
            )).id();

            colliders.push(
                Some(entity),
                isometry,
                //Isometry2::new(parry2d::na::Vector2::new(
                //    transform.translation.x,
                //    transform.translation.z,
                //), na::zero()), // FIXME: add rotation
                shape
            );
        });

    colliders
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
    // mut colliders: ResMut<Colliders>,
    env_config: Res<Environment>,
    config: Res<Config>,
    materials: Res<Materials>,
    obstacles: Query<Entity, With<ObstacleMarker>>,
) -> Colliders {
    for entity in &obstacles {
        commands.entity(entity).despawn();
        info!("despawn obstacle entity: {:?}", entity);
    }

    let tile_grid = &env_config.tiles.grid;

    let obstacle_height = env_config.obstacle_height();
    let obstacle_y = -obstacle_height / 2.0;

    let tile_size = env_config.tile_size();

    let path_width = env_config.path_width();
    let base_dim = tile_size * (1.0 - path_width) / 2.0;

    // offset caused by the size of the grid
    // - this centers the map
    let grid_offset_x = tile_grid.ncols() as f32 / 2.0 - 0.5;
    let grid_offset_z = -(tile_grid.nrows() as f32 / 2.0 - 0.5);

    let pos_offset = path_width.mul_add(tile_size, base_dim) / 2.0;

    let mut colliders = Colliders::default();

    for (y, row) in tile_grid.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            // offset of the individual tile in the grid
            // used in all match cases
            let tile_offset_x = x as f32;
            let tile_offset_z = -(y as f32);

            // total offset caused by grid and tile
            let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
            let offset_z = (tile_offset_z - grid_offset_z) * tile_size;
            // Vec<(Handle<Mesh>, Transform, parry2d::shape::Cuboid)>
            if let Some(obstacle_information) = match tile {
                '─' | '-' => {
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
                '│' | '|' => {
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
                    // let cuboid_plug =
                    //     Cuboid::new(base_dim, obstacle_height, path_width * tile_size);
                    let cuboid_plug =
                        Cuboid::new(tile_size / 2.0, obstacle_height, path_width * tile_size);
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
                                offset_x + tile_size / 4.0,
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
                    // let cuboid_plug =
                    //     Cuboid::new(base_dim, obstacle_height, path_width * tile_size);
                    let cuboid_plug =
                        Cuboid::new(tile_size / 2.0, obstacle_height, path_width * tile_size);
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
                                offset_x - tile_size / 4.0,
                                obstacle_y,
                                offset_z,
                            )),
                        ),
                    ])
                }
                '╷' => {
                    // Termination from the top
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the bottom, to terminate the path

                    // Left and right
                    let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the bottom
                    // let cuboid_plug =
                    //     Cuboid::new(path_width * tile_size, obstacle_height, base_dim);
                    let cuboid_plug =
                        Cuboid::new(path_width * tile_size, obstacle_height, tile_size / 2.0);
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
                                offset_z + tile_size / 4.0,
                            )),
                        ),
                    ])
                }
                '╵' => {
                    // Termination from the bottom
                    // - 2 larger cuboids on the left and right, spanning the entire height of the
                    //   tile
                    // - 1 smaller 'plug' cuboid on the top, to terminate the path

                    // Left and right
                    let cuboid = Cuboid::new(base_dim, obstacle_height, tile_size);
                    // let parry_cuboid: parry2d::shape::Cuboid = cuboid.into();
                    // let mesh_handle = meshes.add(cuboid);

                    // Plug at the top
                    // let cuboid_plug =
                    //     Cuboid::new(path_width * tile_size, obstacle_height, base_dim);
                    let cuboid_plug =
                        Cuboid::new(path_width * tile_size, obstacle_height, tile_size / 2.0);
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
                                offset_z - tile_size / 4.0,
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
                            // top
                            cuboid_top,
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
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
                            // top
                            cuboid_top,
                            // top transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
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
                            // bottom
                            cuboid_bottom,
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
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
                            // bottom
                            cuboid_bottom,
                            // bottom transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
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
                                offset_z - pos_offset,
                            )),
                        ),
                        (
                            // top center cuboid
                            top,
                            // top center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z + pos_offset,
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
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // top right cube
                            cube,
                            // top right cube transform
                            Transform::from_translation(Vec3::new(
                                offset_x + pos_offset,
                                obstacle_y,
                                offset_z + pos_offset,
                            )),
                        ),
                        (
                            // bottom center cuboid
                            bottom,
                            // bottom center cuboid transform
                            Transform::from_translation(Vec3::new(
                                offset_x,
                                obstacle_y,
                                offset_z - pos_offset,
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
                for (cuboid, transform) in &obstacle_information {
                    let entity = commands
                        .spawn((
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
                            bevy_mod_picking::PickableBundle::default(),
                            On::<Pointer<Click>>::send_event::<events::ObstacleClickedOn>(),
                            // TODO: add on click handler
                        ))
                        .id();

                    colliders.push(
                        Some(entity),
                        Isometry2::new(
                            Vector2::new(transform.translation.x, transform.translation.z),
                            na::zero(),
                        ),
                        Arc::new(Into::<shape::Cuboid>::into(*cuboid)),
                    );
                }
            }
        }
    }
    colliders
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

/// **Bevy** [`Update`] system that clear all spawned obstacle colliders. Used
/// to clear existing colliders before a new simulation is loaded.
fn clear_colliders(mut colliders: ResMut<Colliders>) {
    let n_colliders = colliders.len();
    colliders.clear();
    info!("{} colliders cleared", n_colliders);
}
