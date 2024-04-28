use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, PrimitiveTopology, TextureDimension, TextureFormat},
    },
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;

use crate::{
    asset_loader::{Meshes, Obstacles},
    config::{self, Config},
    input::DrawSettingsEvent,
    simulation_loader::{self, LoadSimulation, Sdf},
    theme::CatppuccinTheme,
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let (r, g, b) = Flavour::Macchiato.base().into();
        app.insert_resource(ClearColor(Color::rgb_u8(r, g, b)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            })
            // .add_state::<HeightMapState>()
            .init_state::<HeightMapState>()
            .add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, (
                spawn_infinite_grid,
                spawn_directional_light,
            ))
            .add_systems(
                Update,
                spawn_sdf_map_representation.run_if(resource_changed::<Sdf>),
            )
            .add_systems(Update,
                (
                    // obstacles.run_if(environment_png_is_loaded),
                    obstacles.run_if(resource_changed::<Obstacles>),
                    // obstacles.run_if(on_event::<LoadSimulation>()),
                    show_or_hide_height_map,
                    show_or_hide_flat_map,
                )
            );
    }
}

/// **Bevy** [`Startup`] system to spawn the an infinite grid
/// Using the [`InfiniteGridPlugin`] from the `bevy_infinite_grid` crate
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn spawn_infinite_grid(mut commands: Commands, catppuccin_theme: Res<CatppuccinTheme>) {
    let grid_colour = catppuccin_theme.grid_colour();

    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            shadow_color: None,
            major_line_color: grid_colour,
            minor_line_color: grid_colour,
            x_axis_color: {
                let (r, g, b) = catppuccin_theme.maroon().into();
                Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8)
            },
            z_axis_color: {
                let (r, g, b) = catppuccin_theme.blue().into();
                Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8)
            },
            ..default()
        },
        ..default()
    });
}

/// **Bevy** [`Startup`] system
/// Spawns a directional light.
fn spawn_directional_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 5.0 + Vec3::Z * 8.0).looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}

/// **Bevy** [`State`] representing whether the heightmap.
/// 1. is `Waiting` for the image asset to be loaded.
/// 2. has been `Generated` from the image asset.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum HeightMapState {
    #[default]
    Waiting,
    Generated,
}

/// **Bevy** [`Component`] to represent the flat map.
/// Serves as a marker to identify the flat map entity.
#[derive(Component)]
pub struct SdfMapRepresentation;

/// Makes a simple quad plane to show the map png.
fn spawn_sdf_map_representation(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
    // scene_assets: Res<SceneAssets>,
    // obstacles: Res<Obstacles>,
    sdf: Res<Sdf>,
    meshes: Res<Meshes>,
    // mesh_assets: Res<Assets<Mesh>>,
    config: Res<Config>,
    existing_sdf_map_representation: Query<Entity, With<SdfMapRepresentation>>,
) {
    if let Ok(entity) = existing_sdf_map_representation.get_single() {
        commands.entity(entity).despawn_recursive();
        info!("despawned sdf map representation");
    }

    let width = sdf.0.width();
    let height = sdf.0.height();
    let mut rgba_buffer = vec![255u8; width as usize * height as usize * 4];
    let input = sdf.0.as_raw();
    let mut i = 0;
    for chunk in input.chunks(3) {
        rgba_buffer[i..i + 3].copy_from_slice(&chunk[0..3]);
        i += 4;
    }

    let image = bevy::render::texture::Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba_buffer,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    let image_handle = image_assets.add(image);

    // if let Ok(mut existing_image_handle) =
    // existing_sdf_map_representation.get_single_mut() {     // update the
    // texture of the existing mesh     *existing_image_handle = image_handle;
    //     info!("changed sdf map representation");
    // }

    // TODO: cache material handle, in a hash name by the sim name
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        ..default()
    });

    let visibility = if config.visualisation.draw.sdf {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Spawn an entity with the mesh and material, and position it in 3D space
    // TODO: generate based on sim config
    let mesh = meshes.plane.clone();
    commands.spawn((SdfMapRepresentation, PbrBundle {
        mesh,
        material,
        visibility,
        transform: Transform::from_xyz(0.0, -0.1, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    }));
    info!("spawned sdf map representation");
}

// /// Makes a simple quad plane to show the map png.
// fn spawn_sdf_map_representation(
//     mut commands: Commands,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut image_assets: ResMut<Assets<Image>>,
//     // scene_assets: Res<SceneAssets>,
//     // obstacles: Res<Obstacles>,
//     sdf: Res<Sdf>,
//     meshes: Res<Meshes>,
//     // mesh_assets: Res<Assets<Mesh>>,
//     config: Res<Config>,
//     mut existing_sdf_map_representation: Query<&mut Handle<Image>,
// With<SdfMapRepresentation>>, ) {
//     let width = sdf.0.width();
//     let height = sdf.0.height();
//     let mut rgba_buffer = vec![255u8; width as usize * height as usize * 4];
//     let input = sdf.0.as_raw();
//     let mut i = 0;
//     for chunk in input.chunks(3) {
//         rgba_buffer[i..i + 3].copy_from_slice(&chunk[0..3]);
//         i += 4;
//     }
//
//     let image = bevy::render::texture::Image::new(
//         Extent3d {
//             width,
//             height,
//             depth_or_array_layers: 1,
//         },
//         TextureDimension::D2,
//         rgba_buffer,
//         TextureFormat::Rgba8Unorm,
//         RenderAssetUsages::RENDER_WORLD,
//     );
//     let image_handle = image_assets.add(image);
//
//     if let Ok(mut existing_image_handle) =
// existing_sdf_map_representation.get_single_mut() {         // update the
// texture of the existing mesh         *existing_image_handle = image_handle;
//         info!("changed sdf map representation");
//     } else {
//         let material = materials.add(StandardMaterial {
//             base_color_texture: Some(image_handle),
//             ..default()
//         });
//
//         let visibility = if config.visualisation.draw.sdf {
//             Visibility::Visible
//         } else {
//             Visibility::Hidden
//         };
//
//         // Spawn an entity with the mesh and material, and position it in 3D
// space         // TODO: generate based on sim config
//         let mesh = meshes.plane.clone();
//         commands.spawn((SdfMapRepresentation, PbrBundle {
//             mesh,
//             material,
//             visibility,
//             transform: Transform::from_xyz(0.0, -0.1, 0.0)
//
// .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
//             ..default()
//         }));
//         info!("spawned sdf map representation");
//     }
// }

/// **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::flat_map` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`VariableVisualiser`] entities
fn show_or_hide_flat_map(
    mut query: Query<&mut Visibility, With<SdfMapRepresentation>>,
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, config::DrawSetting::Sdf) {
            for mut visibility in &mut query {
                if event.draw {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}

/// **Bevy** run criteria
/// Checks whether the environment image asset has been loaded.
/// used as a run criteria for the [`obstacles`] system.
fn environment_png_is_loaded(
    state: Res<State<HeightMapState>>,
    // scene_assets: Res<SceneAssets>,
    obstacles: Res<Obstacles>,
    image_assets: Res<Assets<Image>>,
) -> bool {
    image_assets
        .get(obstacles.raw.id())
        .is_some()
        // && matches!(state.get(), HeightMapState::Generated)
        && matches!(state.get(), HeightMapState::Waiting)

    // if image_assets
    //     .get(scene_assets.obstacle_image_raw.clone())
    //     .is_some()
    // {
    //     return matches!(state.get(), HeightMapState::Waiting);
    // }
    // false
}

/// **Bevy** [`Update`] system
/// Spawn the heightmap obstacles as soon as the obstacle image is loaded by
/// using the `environment_png_is_loaded` run criteria.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn obstacles(
    mut commands: Commands,
    obstacles: Res<Obstacles>,
    sdf: Res<Sdf>,
    image_assets: Res<Assets<Image>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut standard_material_assets: ResMut<Assets<StandardMaterial>>,
    mut next_heightmap_state: ResMut<NextState<HeightMapState>>,
    config: Res<Config>,
    asset_server: Res<AssetServer>,
) {
    let Some(load_state) = asset_server.get_load_state(obstacles.raw.id()) else {
        warn!("obstacle image not loaded yet");
        return;
    };

    let bevy::asset::LoadState::Loaded = load_state else {
        warn!("obstacle image not loaded yet");
        return;
    };

    let Some(image) = image_assets.get(obstacles.raw.id()) else {
        warn!("obstacle image not available yet");
        return;
    };

    next_heightmap_state.set(HeightMapState::Generated);

    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;
    // let bytes_per_pixel = image.texture_descriptor.format.block_dimensions().0 as
    // usize;
    let channels = 4;

    let vertices_count = width * height;
    let triangle_count = (width - 1) * (height - 1) * 6;
    let extent = config.simulation.world_size.get();
    let intensity = config.visualisation.height.height_map;

    // info!("image.texture_descriptor.size.width: {}", width);
    // info!("image.texture_descriptor.size.height: {}", height);
    // info!("image.data.len(): {}", image.data.len());
    // info!("bytes_per_pixel: {}", bytes_per_pixel);
    // info!(
    //     "image.data.len() / bytes_per_pixel: {}",
    //     image.data.len() / bytes_per_pixel
    // );
    // info!("vertices_count: {}", vertices_count);
    // info!("triangle_count: {}", triangle_count);

    let mut heightmap = Vec::<f32>::with_capacity(vertices_count);
    for w in 0..width {
        for h in 0..height {
            // heightmap.push((w + h) as f32);
            // heightmap.push(0.0);
            heightmap.push(1.0 - f32::from(image.data[(w * height + h) * channels]) / 255.0);
        }
    }

    // info!("heightmap.len(): {}", heightmap.len());

    // Defining vertices.
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);

    for d in 0..width {
        for w in 0..height {
            let (w_f32, d_f32) = (w as f32, d as f32);

            let pos = [
                (w_f32 - width as f32 / 2.) * extent / width as f32,
                heightmap[d * width + w].mul_add(intensity, -0.1),
                (d_f32 - height as f32 / 2.) * extent / height as f32,
            ];
            positions.push(pos);
            uvs.push([w_f32 / width as f32, d_f32 / height as f32]);
        }
    }

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

    assert!(height > 2);
    for d in 0..(height - 2) as u32 {
        for w in 0..(width - 2) as u32 {
            // First tringle
            triangles.push((d * (width as u32 + 1)) + w);
            triangles.push(((d + 1) * (width as u32 + 1)) + w);
            triangles.push(((d + 1) * (width as u32 + 1)) + w + 1);
            // Second triangle
            triangles.push((d * (width as u32 + 1)) + w);
            triangles.push(((d + 1) * (width as u32 + 1)) + w + 1);
            triangles.push((d * (width as u32 + 1)) + w + 1);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // mesh.set_indices(Some(Indices::U32(triangles)));
    mesh.insert_indices(Indices::U32(triangles));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let material_handle = standard_material_assets.add(StandardMaterial {
        base_color_texture: Some(obstacles.raw.clone()),
        // base_color: Color::rgb(0.5, 0.5, 0.85),
        ..default()
    });

    let visibility = if config.visualisation.draw.height_map {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    commands.spawn((simulation_loader::Reloadable, HeightMap, PbrBundle {
        mesh: mesh_assets.add(mesh),
        material: material_handle,
        visibility,
        ..default()
    }));

    error!("spawned heightmap");
}

/// **Bevy** marker [`Component`] to represent the heightmap.
/// Serves as a marker to identify the heightmap entity.
#[derive(Component)]
pub struct HeightMap;

/// **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::height_map` the boolean `DrawSettingEvent.value` will be used
/// to set the visibility of the [`HeightMap`] entities
fn show_or_hide_height_map(
    mut query: Query<&mut Visibility, With<HeightMap>>,
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, config::DrawSetting::HeightMap) {
            for mut visibility in &mut query {
                if event.draw {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
