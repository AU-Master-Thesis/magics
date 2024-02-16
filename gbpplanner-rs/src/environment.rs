use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Abs, Fbm, Perlin,
};

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
            .add_state::<HeightMapState>()
            .add_plugins(InfiniteGridPlugin)
            .add_systems(
                Startup,
                (
                    infinite_grid,
                    // test_cubes,
                    lighting,
                    // view_image,
                ),
            )
            .add_systems(Update, obstacles.run_if(environment_png_is_loaded));
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
            },
            z_axis_color: {
                let (r, g, b) = Flavour::Macchiato.blue().into();
                Color::rgb_u8(r, g, b)
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

fn view_image(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(scene_assets.obstacle_image_raw.clone()),
        ..default()
    });

    let mesh = Mesh::from(shape::Quad::new(Vec2::new(10.0, 10.0)));

    // Spawn an entity with the mesh and material, and position it in 3D space
    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: material_handle,
        transform: Transform::from_xyz(0.0, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum HeightMapState {
    #[default]
    Waiting,
    Generated,
}

fn environment_png_is_loaded(
    state: Res<State<HeightMapState>>,
    scene_assets: Res<SceneAssets>,
    image_assets: Res<Assets<Image>>,
) -> bool {
    if let Some(_) = image_assets.get(scene_assets.obstacle_image_raw.clone()) {
        return match state.get() {
            HeightMapState::Waiting => true,
            _ => false,
        };
    }
    false
}

fn obstacles(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    image_assets: Res<Assets<Image>>,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // state: Res<State<HeightMapState>>,
    mut next_state: ResMut<NextState<HeightMapState>>,
) {
    if let Some(image) = image_assets.get(scene_assets.obstacle_image_raw.clone()) {
        next_state.set(HeightMapState::Generated);

        let width = image.texture_descriptor.size.width as usize;
        let height = image.texture_descriptor.size.height as usize;

        let heightmap_data: Vec<f32> = get_heightmap_data_from_image(&image);

        // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // let vertex_positions =
        //     generate_vertex_positions_from_heightmap_data(&heightmap_data, width, height);

        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_positions.clone());
        // mesh.set_indices(Some(bevy::render::mesh::Indices::U32(generate_indices(
        //     width, height,
        // ))));

        // // mesh.insert_attribute(
        // //     Mesh::ATTRIBUTE_NORMAL,
        // //     calculate_normals(&vertex_positions, width, height),
        // // );

        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, calculate_uvs(width, height));
        let vertices_count = (width + 1) * (height + 1);
        let triangle_count = width * height * 6;
        let extent = 10.0;
        let intensity = 1.0;

        // let fbm = Fbm::new(3456789);
        // let noisemap = PlaneMapBuilder::new(&fbm)
        //     .set_size(width, height)
        //     .set_x_bounds(-extent, extent)
        //     .set_y_bounds(-extent, extent)
        //     .build();
        let perlin = Perlin::default();
        let abs: Abs<f64, Perlin, 2> = Abs::new(perlin);
        // let fbm = Fbm::new(abs);

        let noisemap = PlaneMapBuilder::new(abs)
            .set_size(1000, 1000)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();

        // Defining vertices.
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertices_count);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertices_count);

        for d in 0..=width {
            for w in 0..=height {
                let (w_f32, d_f32) = (w as f32, d as f32);

                let pos = [
                    (w_f32 - width as f32 / 2.) * extent as f32 / width as f32,
                    (noisemap.get_value(w, d) as f32) * intensity,
                    // 0.5,
                    // heightmap_data[(d * width + w) % heightmap_data.len()],
                    (d_f32 - height as f32 / 2.) * extent as f32 / height as f32,
                ];
                positions.push(pos);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push([w_f32 / width as f32, d_f32 / height as f32]);
            }
        }

        // Defining triangles.
        let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

        for d in 0..height as u32 {
            for w in 0..width as u32 {
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

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(triangles)));

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

fn get_heightmap_data_from_image(image: &Image) -> Vec<f32> {
    info!("HEIGHTMAP DATA");
    let width = image.texture_descriptor.size.width as usize;
    let height = image.texture_descriptor.size.height as usize;

    let bytes_per_pixel = image.texture_descriptor.format.block_dimensions().0 as usize;
    let buffer_size = width * height * bytes_per_pixel;
    // let mut heightmap_data = Vec::with_capacity(width * height);
    let mut heightmap_data = Vec::with_capacity(buffer_size);

    for y in 0..height {
        for x in 0..width {
            // let pixel_index = (y * width + x) * bytes_per_pixel;
            let pixel_index = y * width + x;

            // Assume the image is grayscale and take the first byte for the grayscale value
            let grayscale_value = image.data[pixel_index] as f32 / 255.0;
            heightmap_data.push(grayscale_value);
        }
    }

    heightmap_data
}

fn generate_vertex_positions_from_heightmap_data(
    heightmap_data: &[f32],
    width: usize,
    height: usize,
) -> Vec<[f32; 3]> {
    let mut positions = Vec::with_capacity(width * height);

    info!("VERTEX POSITIONS");

    // Go through the pixels in the heightmap like this
    // 1   3   5
    // |  /|  /|
    // | / | / |
    // |/  |/  |
    // 2   4   6

    for j in 0..height {
        for i in 0..width {
            // Vertex 1
            // Normalize i and j to range between -0.5..0.5
            let x = i as f32 / width as f32 - 0.5;
            let z = j as f32 / height as f32 - 0.5;
            // Use the heightmap data to set the y coordinate
            // let y = heightmap_data[j * width + i];
            let y = 0.0;

            positions.push([x, y, z]);

            // // Vertex 2
            // if j < height - 1 {
            //     let x = i as f32 / width as f32 - 0.5;
            //     let z = (j + 1) as f32 / height as f32 - 0.5;
            //     // let y = heightmap_data[(j + 1) * width + i];
            //     let y = 0.0;

            //     positions.push([x, y, z]);
            // }
        }
    }

    positions
}

fn generate_indices(width: usize, height: usize) -> Vec<u32> {
    let mut indices = Vec::with_capacity((width - 1) * (height - 1) * 6);

    info!("INDICES");
    for j in 0..height - 1 {
        for i in 0..width - 1 {
            let current_idx = j * width + i;
            let right_idx = current_idx + 1;
            let bottom_idx = current_idx + width;
            let bottom_right_idx = bottom_idx + 1;

            // Triangle 1
            indices.push(current_idx as u32);
            indices.push(bottom_right_idx as u32);
            indices.push(bottom_idx as u32);

            // Triangle 2
            indices.push(current_idx as u32);
            indices.push(right_idx as u32);
            indices.push(bottom_right_idx as u32);
        }
    }

    indices
}

fn calculate_normals(
    vertex_positions: &[[f32; 3]],
    width: usize,
    height: usize,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0; 3]; vertex_positions.len()];

    // Compute normals by cross-product of the vectors from the adjacent vertices
    info!("NORMALS");
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

    info!("UVS");
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
