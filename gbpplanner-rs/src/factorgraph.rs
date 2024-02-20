// use factorgraph::FactorGraph;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{Indices, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};
use catppuccin::Flavour;
use color_eyre::owo_colors::OwoColorize;
use itertools::Itertools;

#[derive(Component, Debug, Copy, Clone)]
pub struct Factor(usize);

#[derive(Component, Debug, Copy, Clone)]
pub struct Variable(usize);

#[derive(Component, Debug)]
pub struct MoveMe;

#[derive(Component, Debug)]
pub struct Line;

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>,
}

/// A list of vertices defining a path
#[derive(Debug, Clone)]
struct Path {
    points: Vec<Vec3>,
}

impl From<Path> for Mesh {
    fn from(line: Path) -> Self {
        let vertices = line.points.clone();
        let width = 0.05;

        // // info!("input vertices: {:?}", vertices);
        // let perps = vertices
        //     .chunks_exact(2)
        //     .map(|chunk| {
        //         let (a, b) = (chunk[0], chunk[1]);
        //         let dir = (b - a).normalize();
        //         Vec3::new(-dir.z, 0.0, dir.x)
        //     })
        //     .flat_map(|n| [n, n])
        //     .collect::<Vec<_>>();

        // let mut normals = Vec::<Vec3>::with_capacity(vertices.len());
        // normals.push(perps[0]);
        // for i in 0..perps.len() - 1 {
        //     let n = (perps[i] + perps[i + 1]).normalize();
        //     normals.push(n);
        // }
        // normals.push(perps[perps.len() - 1]);
        // // info!("normals: {:?}", normals);

        // assert_eq!(
        //     vertices.len(),
        //     normals.len(),
        //     "vertices.len() != normals.len()"
        // );

        // let mut expand_by: Vec<f32> = vertices
        //     .iter()
        //     .tuple_windows::<(_, _, _)>()
        //     .map(|(&a, &b, &c)| {
        //         let ab = (b - a).normalize();
        //         let bc = (c - b).normalize();

        //         let kinking_factor = (1.0 - ab.dot(bc)) * width / 2.0;
        //         width + kinking_factor
        //     })
        //     .collect();
        // expand_by.insert(0, width);
        // expand_by.push(width);
        // // info!("expand_by: {:?}", expand_by);

        // assert_eq!(
        //     vertices.len(),
        //     expand_by.len(),
        //     "vertices.len() != expand_by.len()"
        // );

        // let left_vertices = vertices
        //     .iter()
        //     .zip(normals.iter())
        //     .zip(expand_by.iter())
        //     .map(|((&v, &n), &e)| v - n * e / 2.0)
        //     .collect::<Vec<_>>();
        // let right_vertices = vertices
        //     .iter()
        //     .zip(normals.iter())
        //     .zip(expand_by.iter())
        //     .map(|((&v, &n), &e)| v + n * e / 2.0)
        //     .collect::<Vec<_>>();

        let mut left_vertices = Vec::<Vec3>::with_capacity(vertices.len());
        let mut right_vertices = Vec::<Vec3>::with_capacity(vertices.len());

        // let _ = vertices.windows(3).map(|window| {
        //     let (a, b, c) = (window[0], window[1], window[2]);
        //     let ab = (b - a).normalize();
        //     let bc = (c - b).normalize();

        //     let kinking_factor = (1.0 - ab.dot(bc)) * width / 2.0;
        //     let expand_by = width + kinking_factor;

        //     let n = (ab + bc).normalize();
        //     let left = b - n * expand_by / 2.0;
        //     let right = b + n * expand_by / 2.0;

        //     left_vertices.push(left);
        //     right_vertices.push(right);
        // });

        // add the first offset
        let (a, b) = (vertices[0], vertices[1]);
        let ab = (b - a).normalize();
        let n = Vec3::new(ab.z, ab.y, -ab.x);
        let left = a - n * width / 2.0;
        let right = a + n * width / 2.0;
        left_vertices.push(left);
        right_vertices.push(right);

        for window in vertices.windows(3) {
            let (a, b, c) = (window[0], window[1], window[2]);
            let ab = (b - a).normalize();
            let bc = (c - b).normalize();

            let kinking_factor = (1.0 - ab.dot(bc)) * width / 2.0;
            let expand_by = width + kinking_factor;

            let n = {
                let sum = (ab + bc).normalize();
                Vec3::new(sum.z, sum.y, -sum.x)
            };
            let left = b - n * expand_by / 2.0;
            let right = b + n * expand_by / 2.0;

            left_vertices.push(left);
            right_vertices.push(right);
        }

        // add the last offset
        let (a, b) = (vertices[vertices.len() - 2], vertices[vertices.len() - 1]);
        let ab = (b - a).normalize();
        let n = Vec3::new(ab.z, ab.y, -ab.x);
        let left = b - n * width / 2.0;
        let right = b + n * width / 2.0;
        left_vertices.push(left);
        right_vertices.push(right);

        // info!("left_vertices:  {:?}", left_vertices);
        // info!("right_vertices: {:?}", right_vertices);

        // collect all vertices
        let vertices: Vec<Vec3> = left_vertices
            .iter()
            .zip(right_vertices.iter())
            .flat_map(|(l, r)| [*l, *r])
            .collect();
        // info!("output vertices {}: {:?}", vertices.len(), vertices);

        // let normals: Vec<Vec3> = vertices.iter().map(|_| -Vec3::Y).collect();

        Mesh::new(
            PrimitiveTopology::TriangleStrip,
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices).with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    }
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

#[derive(Component)]
pub struct FactorGraph {
    factors: Vec<Factor>,
    variables: Vec<Variable>,
}

pub struct FactorGraphPlugin;

impl Plugin for FactorGraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Sample4)
            .add_plugins(MaterialPlugin::<LineMaterial>::default())
            .add_systems(Startup, insert_dummy_factor_graph)
            .add_systems(Update, (draw_lines, move_variables));
    }
}

fn insert_dummy_factor_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let variables = vec![Variable(0), Variable(1), Variable(2)];
    let factors = vec![Factor(0), Factor(1)];

    let factor_graph = FactorGraph {
        factors: factors.clone(),
        variables: variables.clone(),
    };

    let alpha = 0.75;
    let variable_material = materials.add({
        let (r, g, b) = Flavour::Macchiato.blue().into();
        Color::rgba_u8(r, g, b, (alpha * 255.0) as u8).into()
    });
    let factor_material = materials.add({
        let (r, g, b) = Flavour::Macchiato.green().into();
        Color::rgba_u8(r, g, b, (alpha * 255.0) as u8).into()
    });

    let variable_positions: Vec<Vec3> = variables
        .iter()
        .map(|v| {
            let i = v.0 as f32;
            Vec3::new(0.0, 0.5, i * 5.0 + 6.0)
        })
        .collect();
    let factor_positions: Vec<Vec3> = factors
        .iter()
        .map(|f| {
            let i = f.0 as f32;
            Vec3::new(0.0, 0.5, i * 5.0 + 8.5)
        })
        .collect();
    info!("variable_positions: {:?}", variable_positions);
    info!("factor_positions: {:?}", factor_positions);

    // all positions sorted by z value
    let mut all_positions = variable_positions.clone();
    all_positions.extend(factor_positions.clone());
    all_positions.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());
    info!("all_positions: {:?}", all_positions);

    let variable_mesh = meshes.add(
        shape::Icosphere {
            radius: 0.3,
            subdivisions: 4,
        }
        .try_into()
        .unwrap(),
    );

    let factor_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));

    for (i, variable) in factor_graph.variables.iter().enumerate() {
        println!("Spawning variable: {:?}", variable);

        commands.spawn((
            *variable,
            PbrBundle {
                mesh: variable_mesh.clone(),
                material: variable_material.clone(),
                transform: Transform::from_translation(variable_positions[i]),
                ..Default::default()
            },
        ));
    }

    for (i, factor) in factor_graph.factors.iter().enumerate() {
        println!("Spawning factor: {:?}", factor);

        commands.spawn((
            *factor,
            PbrBundle {
                mesh: factor_mesh.clone(),
                material: factor_material.clone(),
                transform: Transform::from_translation(factor_positions[i]),
                ..Default::default()
            },
            MoveMe,
        ));
    }
}

// TODO: Set vertex positions on the line mesh instead of generating a new one.
fn draw_lines(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query_factors: Query<(&Factor, &Transform)>,
    query_variables: Query<(&Variable, &Transform)>,
    query_previous_lines: Query<Entity, With<Line>>,
) {
    // remove previous lines
    for entity in query_previous_lines.iter() {
        commands.entity(entity).despawn();
    }

    // collect all factor and variable positions
    let factor_positions: Vec<Vec3> =
        query_factors.iter().map(|(_, t)| t.translation).collect();
    let variable_positions: Vec<Vec3> =
        query_variables.iter().map(|(_, t)| t.translation).collect();

    // all positions sorted by z value
    let mut all_positions = variable_positions.clone();
    all_positions.extend(factor_positions.clone());
    all_positions.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

    let line_material = materials.add({
        let (r, g, b) = Flavour::Macchiato.crust().into();
        // Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8).into()
        Color::rgb_u8(r, g, b).into()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Path {
                points: all_positions,
            })),
            material: line_material,
            ..Default::default()
        },
        Line,
    ));
}

fn move_variables(time: Res<Time>, mut query: Query<&mut Transform, With<Variable>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x = time.elapsed_seconds().sin() * 5.0;
    }
}
