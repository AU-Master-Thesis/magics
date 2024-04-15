use bevy::{
    prelude::*,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;

use crate::{
    asset_loader::SceneAssets, bevy_utils::run_conditions::event_exists, config,
    input::DrawSettingsEvent, theme::CatppuccinTheme,
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
            .add_systems(Startup, (infinite_grid, lighting, flat_map))
            .add_systems(Update,
                (
                    obstacles.run_if(environment_png_is_loaded),
                    show_or_hide_height_map.run_if(event_exists::<DrawSettingsEvent>),
                    show_or_hide_flat_map.run_if(event_exists::<DrawSettingsEvent>))
                );
    }
}

/// **Bevy** [`Startup`] system to spawn the an infinite grid
/// Using the [`InfiniteGridPlugin`] from the `bevy_infinite_grid` crate
fn infinite_grid(mut commands: Commands, catppuccin_theme: Res<CatppuccinTheme>) {
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
fn lighting(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 5.0 + Vec3::Z * 8.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
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
pub struct FlatMap;

/// **Bevy** [`Startup`] system
/// Makes a simple quad plane to show the map png.
fn flat_map(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<config::Config>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(scene_assets.obstacle_image_raw.clone()),
        ..default()
    });

    // Spawn an entity with the mesh and material, and position it in 3D space
    commands.spawn((FlatMap, PbrBundle {
        mesh: scene_assets.meshes.plane.clone(),
        material: material_handle,
        visibility: if config.visualisation.draw.sdf {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        transform: Transform::from_xyz(0.0, -0.1, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    }));
}

/// **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::flat_map` the boolean `DrawSettingEvent.value` will be used to
/// set the visibility of the [`VariableVisualiser`] entities
fn show_or_hide_flat_map(
    mut query: Query<(&FlatMap, &mut Visibility)>,
    mut draw_setting_event: EventReader<DrawSettingsEvent>,
) {
    for event in draw_setting_event.read() {
        if matches!(event.setting, config::DrawSetting::Sdf) {
            for (_, mut visibility) in query.iter_mut() {
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
    scene_assets: Res<SceneAssets>,
    image_assets: Res<Assets<Image>>,
) -> bool {
    if image_assets
        .get(scene_assets.obstacle_image_raw.clone())
        .is_some()
    {
        return matches!(state.get(), HeightMapState::Waiting);
    }
    false
}

/// **Bevy** [`Update`] system
/// Spawn the heightmap obstacles as soon as the obstacle image is loaded by
/// using the `environment_png_is_loaded` run criteria.
fn obstacles(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    image_assets: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<HeightMapState>>,
    config: Res<config::Config>,
) {
    let Some(image) = image_assets.get(scene_assets.obstacle_image_raw.clone()) else {
        return;
    };

    next_state.set(HeightMapState::Generated);

    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;
    let bytes_per_pixel = image.texture_descriptor.format.block_dimensions().0 as usize;
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
            heightmap.push(1.0 - image.data[(w * height + h) * channels] as f32 / 255.0);
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
                heightmap[d * width + w] * intensity - 0.1,
                (d_f32 - height as f32 / 2.) * extent / height as f32,
            ];
            positions.push(pos);
            uvs.push([w_f32 / width as f32, d_f32 / height as f32]);
        }
    }

    // Defining triangles.
    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

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

    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(scene_assets.obstacle_image_raw.clone()),
        // base_color: Color::rgb(0.5, 0.5, 0.85),
        ..default()
    });

    commands.spawn((HeightMap, PbrBundle {
        mesh: meshes.add(mesh),
        material: material_handle,
        visibility: if config.visualisation.draw.height_map {
            Visibility::Visible
        } else {
            Visibility::Hidden
        },
        ..default()
    }));
}

/// **Bevy** [`Component`] to represent the heightmap.
/// Serves as a marker to identify the heightmap entity.
#[derive(Component)]
pub struct HeightMap;

/// **Bevy** [`Update`] system
/// Reads [`DrawSettingEvent`], where if `DrawSettingEvent.setting ==
/// DrawSetting::height_map` the boolean `DrawSettingEvent.value` will be used
/// to set the visibility of the [`HeightMap`] entities
fn show_or_hide_height_map(
    mut query: Query<(&HeightMap, &mut Visibility)>,
    mut draw_setting_event: EventReader<DrawSettingsEvent>,
) {
    for event in draw_setting_event.read() {
        if matches!(event.setting, config::DrawSetting::HeightMap) {
            for (_, mut visibility) in query.iter_mut() {
                if event.draw {
                    *visibility = Visibility::Visible;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
