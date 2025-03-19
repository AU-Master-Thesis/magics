use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;
use gbp_config::{self, Config};
use gbp_environment::Environment;

use crate::{
    input::DrawSettingsEvent,
    theme::CatppuccinTheme,
};

/// Plugin for environment map rendering
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        let (r, g, b) = Flavour::Macchiato.base().into();
        app.insert_resource(ClearColor(Color::rgb_u8(r, g, b)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 1000.0,
            })
            .init_state::<HeightMapState>()
            .add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, (
                spawn_infinite_grid,
                spawn_directional_light,
            ))
            .add_systems(Update, spawn_sdf_map_representation)
            .add_systems(Update, show_or_hide_flat_map);
    }
}

/// System to spawn an infinite grid using bevy_infinite_grid
fn spawn_infinite_grid(mut commands: Commands, catppuccin_theme: Option<Res<CatppuccinTheme>>) {
    // Default grid color
    let grid_colour = catppuccin_theme
        .map(|theme| theme.grid_colour())
        .unwrap_or(Color::rgba(0.15, 0.15, 0.15, 0.6));
    
    let (x_axis_color, z_axis_color) = if let Some(theme) = catppuccin_theme {
        let (r, g, b) = theme.maroon().into();
        let x_color = Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8);
        
        let (r, g, b) = theme.blue().into();
        let z_color = Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8);
        
        (x_color, z_color)
    } else {
        (Color::rgba(0.6, 0.2, 0.2, 0.1), Color::rgba(0.2, 0.2, 0.6, 0.1))
    };

    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            shadow_color: None,
            major_line_color: grid_colour,
            minor_line_color: grid_colour,
            x_axis_color,
            z_axis_color,
            ..default()
        },
        ..default()
    });
}

/// System to spawn a directional light
fn spawn_directional_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 5.0 + Vec3::Z * 8.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}

/// State representing the heightmap loading and generation state
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum HeightMapState {
    #[default]
    Waiting,
    Generated,
}

/// Component to represent the SDF map representation
#[derive(Component)]
pub struct SdfMapRepresentation;

/// Structure representing an SDF for visualization
#[derive(Resource, Clone)]
pub struct Sdf(pub Image);

/// System to spawn an SDF map representation
fn spawn_sdf_map_representation(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
    sdf: Option<Res<Sdf>>,
    meshes: ResMut<Assets<Mesh>>,
    config: Option<Res<Config>>,
    environment: Option<Res<Environment>>,
    existing_sdf_map_representation: Query<Entity, With<SdfMapRepresentation>>,
) {
    // Skip if no SDF, config, or environment is available
    let (Some(sdf), Some(config), Some(environment)) = (sdf, config, environment) else {
        return;
    };
    
    // Despawn any existing map representation
    for entity in &existing_sdf_map_representation {
        commands.entity(entity).despawn_recursive();
        info!("Despawned SDF map representation");
    }

    // Create RGBA data from SDF
    let width = sdf.0.width();
    let height = sdf.0.height();
    let mut rgba_buffer = vec![255u8; width as usize * height as usize * 4];
    let input = sdf.0.as_raw();
    let mut i = 0;
    for chunk in input.chunks(3) {
        rgba_buffer[i..i + 3].copy_from_slice(&chunk[0..3]);
        i += 4;
    }

    // Create the image
    let image = Image::new(
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

    // Add image to assets and create material
    let image_handle = image_assets.add(image);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        ..default()
    });

    // Set visibility based on config
    let visibility = if config.visualisation.draw.sdf {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Calculate map size based on environment
    let (nrows, ncols) = environment.tiles.grid.shape();
    let tile_size = environment.tiles.settings.tile_size;
    let (width, height) = (nrows as f32 * tile_size, ncols as f32 * tile_size);
    
    // Create a rectangle mesh for the map
    let rectangle = bevy::math::primitives::Rectangle::new(height, width);
    let mesh = meshes.add(Mesh::from(rectangle));

    // Spawn the map entity
    commands.spawn((
        SdfMapRepresentation,
        PbrBundle {
            mesh,
            material,
            visibility,
            transform: Transform::from_xyz(0.0, 0.1, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            ..default()
        },
    ));
    
    info!("Spawned SDF map representation");
}

/// System to show or hide the flat map based on DrawSettingsEvent
fn show_or_hide_flat_map(
    mut query: Query<&mut Visibility, With<SdfMapRepresentation>>,
    mut evr_draw_settings: EventReader<DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, gbp_config::DrawSetting::Sdf) {
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
