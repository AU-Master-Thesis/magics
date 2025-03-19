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
    input::DrawSettingsEvent,
    SimulationState,
};

/// Plugin for generating map visuals and colliders
pub struct GenMapPlugin;

impl Plugin for GenMapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<events::ObstacleClickedOn>()
            .add_systems(
                Update,
                (build_tile_grid.pipe(build_obstacles.pipe(insert_colliders_resource))),
            )
            .add_systems(
                Update,
                show_or_hide_generated_map,
            );
    }
}

/// Events for obstacle interaction
pub mod events {
    use super::*;

    /// Event triggered when an obstacle is clicked
    #[derive(Debug, Event)]
    pub struct ObstacleClickedOn(pub Entity);

    impl From<ListenerInput<Pointer<Click>>> for ObstacleClickedOn {
        #[inline]
        fn from(value: ListenerInput<Pointer<Click>>) -> Self {
            Self(value.target)
        }
    }
}

/// Component marker for obstacle entities
#[derive(Debug, Component)]
pub struct ObstacleMarker;

/// System to insert colliders resource
fn insert_colliders_resource(In(colliders): In<Colliders>, mut commands: Commands) {
    commands.insert_resource(colliders);
}

/// System to build obstacles based on environment configuration
#[allow(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn build_obstacles(
    In(mut colliders): In<Colliders>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Option<Res<Environment>>,
    config: Option<Res<Config>>,
    materials: ResMut<Assets<StandardMaterial>>,
) -> Colliders {
    // Skip if no environment or config
    let (Some(env_config), Some(config)) = (env_config, config) else {
        return colliders;
    };

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

    // Create a default material for obstacles
    let obstacle_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.3, 0.3, 0.3),
        ..default()
    });

    // Spawn each obstacle from the environment config
    let obstacles_to_spawn = env_config.obstacles.iter().map(|obstacle| {
        let TileCoordinates { row, col } = obstacle.tile_coordinates;

        info!("Spawning obstacle at {:?}", (row, col));

        let tile_offset_x = col as f32;
        let tile_offset_z = row as f32;

        let offset_x = (tile_offset_x - grid_offset_x) * tile_size;
        let offset_z = (tile_offset_z - grid_offset_z) * tile_size;

        let pos_offset = tile_size / 2.0;

        let translation = obstacle.translation;

        // Create the appropriate shape based on the obstacle type
        match &obstacle.shape {
            PlaceableShape::Circle(Circle { radius }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    (1.0 - translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset,
                );

                info!("Spawning circle: r = {}, at {:?}", radius, center);
                let radius = radius.get() as f32 * tile_size;

                // Create cylinder mesh for circle
                let mesh = meshes.add(Cylinder::new(radius, obstacle_height));
                let transform = Transform::from_translation(center);

                info!(
                    "Spawning cylinder: r = {}, h = {}, at {:?}",
                    radius, obstacle_height, transform
                );

                // Create physics shape
                let shape = parry2d::shape::Ball::new(radius);
                let shape: Arc<dyn shape::Shape> = Arc::new(shape);

                let isometry = Isometry2::new(
                    Vector2::new(transform.translation.x, transform.translation.z),
                    na::zero(),
                );

                Some((mesh, transform, isometry, shape))
            }
            PlaceableShape::Rectangle(Rectangle { width, height }) => {
                let center = Vec3::new(
                    (translation.x.get() as f32).mul_add(tile_size, offset_x) - pos_offset,
                    obstacle_height / 2.0,
                    -((translation.y.get() as f32).mul_add(tile_size, offset_z) - pos_offset),
                );

                info!(
                    "Spawning rectangle: width = {}, height = {}, at {:?}",
                    width, height, center
                );

                // Create cuboid mesh for rectangle
                let mesh = meshes.add(Cuboid::new(
                    width.get() as f32 * tile_size / 2.0,
                    obstacle_height,
                    height.get() as f32 * tile_size / 2.0,
                ));

                let transform = Transform::from_translation(center);

                // Create physics shape
                let half_extents = Vector2::new(
                    width.get() as f32 * tile_size / 4.0,
                    height.get() as f32 * tile_size / 4.0,
                );

                let shape = parry2d::shape::Cuboid::new(half_extents);
                let shape: Arc<dyn shape::Shape> = Arc::new(shape);

                let isometry = Isometry2::new(
                    Vector2::new(transform.translation.x, transform.translation.z),
                    na::zero(),
                );

                Some((mesh, transform, isometry, shape))
            }
            // Add additional shape handling for Triangle, RegularPolygon, etc.
            _ => None,
        }
    });

    // Spawn obstacles and add colliders
    obstacles_to_spawn
        .flatten() // filter out None
        .for_each(|(mesh, transform, isometry, shape)| {
            // Spawn the obstacle entity
            let entity = commands.spawn((
                PbrBundle {
                    mesh,
                    material: obstacle_material.clone(),
                    transform,
                    visibility: if config.visualisation.draw.generated_map {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    ..Default::default()
                },
                ObstacleMarker,
                PickableBundle::default(),
                On::<Pointer<Click>>::send_event::<events::ObstacleClickedOn>(),
            )).id();

            // Add the collider
            colliders.push(
                Some(entity),
                isometry,
                shape
            );
        });

    colliders
}

/// System to build the tile grid visualization
fn build_tile_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    env_config: Option<Res<Environment>>,
    config: Option<Res<Config>>,
    materials: ResMut<Assets<StandardMaterial>>,
    obstacles: Query<Entity, With<ObstacleMarker>>,
) -> Colliders {
    // Skip if no environment or config
    let (Some(env_config), Some(config)) = (env_config, config) else {
        return Colliders::default();
    };

    // Despawn existing obstacles
    for entity in &obstacles {
        commands.entity(entity).despawn();
        info!("Despawn obstacle entity: {:?}", entity);
    }

    let tile_grid = &env_config.tiles.grid;
    let obstacle_height = env_config.obstacle_height();
    let obstacle_y = -obstacle_height / 2.0;
    let tile_size = env_config.tile_size();
    let path_width = env_config.path_width();
    let base_dim = tile_size * (1.0 - path_width) / 2.0;

    // Create a material for the grid
    let obstacle_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.3, 0.3, 0.3),
        ..default()
    });

    // Offset to center the grid
    let grid_offset_x = tile_grid.ncols() as f32 / 2.0 - 0.5;
    let grid_offset_z = -(tile_grid.nrows() as f32 / 2.0 - 0.5);
    let pos_offset = path_width.mul_add(tile_size, base_dim) / 2.0;

    let mut colliders = Colliders::default();

    // Simplified grid building - we'll just create some basic elements
    // In a full implementation, we would process each tile character and create the appropriate shapes

    // Add a ground plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(tile_grid.ncols() as f32 * tile_size).into()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.2, 0.2, 0.2),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -0.1, 0.0),
            visibility: if config.visualisation.draw.generated_map {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            ..default()
        },
        ObstacleMarker,
    ));

    colliders
}

/// System to show or hide the generated map
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
