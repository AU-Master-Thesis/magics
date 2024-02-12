use bevy::render::mesh::VertexAttributeValues;
use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;

use crate::asset_loader::SceneAssets;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let (r, g, b) = Flavour::Macchiato.base().into();
        app.insert_resource(ClearColor(Color::rgb_u8(r, g, b)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 0.5,
            })
            .add_plugins(InfiniteGridPlugin)
            .add_systems(
                Startup,
                (
                    infinite_grid,
                    // test_cubes,
                    lighting,
                    obstacles, // ergioeorigj
                ),
            );
    }
}

fn infinite_grid(mut commands: Commands) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            // shadow_color: None,
            major_line_color: {
                let (r, g, b) = Flavour::Macchiato.crust().into();
                Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            minor_line_color: {
                let (r, g, b) = Flavour::Macchiato.crust().into();
                Color::rgba_u8(r, g, b, (0.25 * 255.0) as u8)
            },
            x_axis_color: {
                let (r, g, b) = Flavour::Macchiato.maroon().into();
                Color::rgb_u8(r, g, b)
                // Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            z_axis_color: {
                let (r, g, b) = Flavour::Macchiato.blue().into();
                Color::rgb_u8(r, g, b)
                // Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            ..default()
        },
        ..default()
    });
}

fn lighting(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 15.0 + Vec3::Z * 20.0)
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..default()
    });
}

fn test_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = materials.add(StandardMaterial::default());

    // cube
    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        transform: Transform {
            translation: Vec3::new(3., 4., 0.),
            rotation: Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize()),
            scale: Vec3::splat(1.5),
        },
        ..default()
    });

    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
}

fn obstacles(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    image_assets: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if let Some(image) = image_assets.get(scene_assets.obstacle_image_raw.clone()) {
        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;

        let heightmap_data: Vec<f32> = get_heightmap_data_from_image(&image);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let vertex_positions =
            generate_vertex_positions_from_heightmap_data(&heightmap_data, width, height);

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions.clone());

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            calculate_normals(&vertex_positions, width, height),
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, calculate_uvs(width, height));

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6), // Example color
            ..default()
        });

        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: material_handle,
            ..default()
        });
    }
}

// Dummy functions for illustration, replace with actual implementations
fn get_heightmap_data_from_image(image: &Image) -> Vec<f32> {
    let size = image.texture_descriptor.size;
    let width = size.width as usize;
    let height = size.height as usize;

    // Assuming the image is 8 bits per channel and in grayscale format
    (0..width * height)
        .map(|i| {
            let pixel_index = i * 4; // 4 bytes per pixel for RGBA8 format
            let pixel = &image.data[pixel_index..pixel_index + 4];
            // Convert to grayscale by just taking the red channel
            pixel[0] as f32 / 255.0
        })
        .collect()
}

fn generate_vertex_positions_from_heightmap_data(
    heightmap_data: &[f32],
    width: usize,
    height: usize,
) -> Vec<[f32; 3]> {
    let mut positions = Vec::with_capacity(width * height);

    for j in 0..height {
        for i in 0..width {
            // Normalize i and j to range between -0.5..0.5
            let x = i as f32 / width as f32 - 0.5;
            let z = j as f32 / height as f32 - 0.5;
            // Use the heightmap data to set the y coordinate
            let y = heightmap_data[j * width + i];
            positions.push([x, y, z]);
        }
    }

    positions
}

fn calculate_normals(vertex_positions: &[[f32; 3]], width: usize, height: usize) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; vertex_positions.len()];

    // Compute normals by cross-product of the vectors from the adjacent vertices
    for j in 0..height {
        for i in 0..width {
            // Get positions of adjacent vertices
            let current_idx = j * width + i;
            let right_idx = current_idx + 1;
            let bottom_idx = current_idx + width;

            let current_pos = Vec3::from(vertex_positions[current_idx]);
            let right_pos = if i < width - 1 {
                Vec3::from(vertex_positions[right_idx])
            } else {
                current_pos
            };
            let bottom_pos = if j < height - 1 {
                Vec3::from(vertex_positions[bottom_idx])
            } else {
                current_pos
            };

            // Calculate the vectors for the edges sharing the current vertex
            let edge1 = right_pos - current_pos;
            let edge2 = bottom_pos - current_pos;

            // The normal is the cross product of the two edge vectors
            let normal = edge1.cross(edge2).normalize().to_array();
            normals[current_idx] = normal;
        }
    }

    normals
}

fn calculate_uvs(width: usize, height: usize) -> Vec<[f32; 2]> {
    let mut uvs = Vec::with_capacity(width * height);

    for j in 0..height {
        for i in 0..width {
            uvs.push([
                i as f32 / (width - 1) as f32,
                j as f32 / (height - 1) as f32,
            ]);
        }
    }

    uvs
}
