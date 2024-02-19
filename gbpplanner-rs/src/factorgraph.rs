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
// use bevy_prototype_lyon::prelude::*;
use itertools::Itertools;

#[derive(Component, Debug, Copy, Clone)]
pub struct Factor(usize);

#[derive(Component, Debug, Copy, Clone)]
pub struct Variable(usize);

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
        // let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();
        // calculate perpendicualar vectors for every line
        // remember the plane is in xz
        // such that I can offset the vertices to create a line with a thickness
        let vertices = line.points.clone();
        let width = 0.05;

        info!("input vertices: {:?}", vertices);
        let perps = vertices
            .chunks_exact(2)
            .map(|chunk| {
                let (a, b) = (chunk[0], chunk[1]);
                let dir = (b - a).normalize();
                Vec3::new(-dir.z, 0.0, dir.x)
            })
            .flat_map(|n| [n, n])
            .collect::<Vec<_>>();

        let mut normals = Vec::<Vec3>::with_capacity(vertices.len());
        normals.push(perps[0]);
        for i in 0..perps.len() - 1 {
            let n = (perps[i] + perps[i + 1]).normalize();
            normals.push(n);
        }
        normals.push(perps[perps.len() - 1]);
        info!("normals: {:?}", normals);

        assert_eq!(
            vertices.len(),
            normals.len(),
            "vertices.len() != normals.len()"
        );

        let mut expand_by: Vec<f32> = vertices
            .iter()
            .tuple_windows::<(_, _, _)>()
            .map(|(&a, &b, &c)| {
                let ab = (b - a).normalize();
                let bc = (c - b).normalize();

                let kinking_factor = (1.0 - ab.dot(bc)) * width / 2.0;
                width + kinking_factor
            })
            .collect();
        expand_by.insert(0, width);
        expand_by.push(width);
        info!("expand_by: {:?}", expand_by);

        assert_eq!(
            vertices.len(),
            expand_by.len(),
            "vertices.len() != expand_by.len()"
        );

        let left_vertices = vertices
            .iter()
            .zip(normals.iter())
            .zip(expand_by.iter())
            .map(|((&v, &n), &e)| v - n * e / 2.0)
            .collect::<Vec<_>>();
        let right_vertices = vertices
            .iter()
            .zip(normals.iter())
            .zip(expand_by.iter())
            .map(|((&v, &n), &e)| v + n * e / 2.0)
            .collect::<Vec<_>>();

        info!("left_vertices: {:?}", left_vertices);
        info!("right_vertices: {:?}", right_vertices);

        // collect all vertices
        let vertices = left_vertices
            .iter()
            .zip(right_vertices.iter())
            .flat_map(|(l, r)| [*l, *r])
            .collect::<Vec<_>>();
        info!("output vertices {}: {:?}", vertices.len(), vertices);

        // let normals: Vec<Vec3> = vertices.iter().map(|_| Vec3::Y).collect();

        // // create indices for the triangles
        // let mut indices = Vec::<u32>::with_capacity((vertices.len() - 1) * 6);
        // // info!("vertices.len(): {}", vertices.len());

        // for i in 0..left_vertices.len() - 1 {
        //     let i = i as u32;

        //     let t1_v1 = i;
        //     let t1_v2 = i + 1;
        //     let t1_v3 = i + 2;
        //     // let t1_v2 = i + left_vertices.len() as u32;
        //     // let t1_v3 = i + 1;
        //     indices.extend_from_slice(&[t1_v1, t1_v2, t1_v3]);

        //     let t2_v1 = i + 1;
        //     let t2_v2 = i + left_vertices.len() as u32;
        //     let t2_v3 = i + left_vertices.len() as u32 + 1;
        //     indices.extend_from_slice(&[t2_v1, t2_v2, t2_v3]);
        // }
        // info!("indices: {:?}", indices);

        Mesh::new(
            // This tells wgpu that the positions are list of lines
            // where every pair is a start and end point
            // PrimitiveTopology::LineList,
            // PrimitiveTopology::TriangleList,
            PrimitiveTopology::TriangleStrip,
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices) //.with_indices(Some(Indices::U32(indices)))
                                                                     //.with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
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
            .add_systems(Startup, insert_dummy_factor_graph);
    }
}

fn insert_dummy_factor_graph(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let variables = vec![Variable(0), Variable(1)];
    let factors = vec![Factor(0)];

    let factor_graph = FactorGraph {
        factors: factors.clone(),
        variables: variables.clone(),
    };

    let variable_material = materials.add(Color::rgb(0.0, 0.0, 1.0).into());
    let factor_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let line_material = materials.add(Color::rgb(0.0, 1.0, 0.0).into());

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
        ));
    }

    // let mut path_builder = PathBuilder::new();
    // path_builder.move_to(all_positions[0].xz());
    // for position in all_positions.iter().skip(1) {
    //     path_builder.line_to(position.xz());
    // }
    // path_builder.close();
    // let path = path_builder.build();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(Path {
                points: all_positions,
            })),
            material: line_material,
            ..Default::default()
        },
        // Segment3dBundle {
        //     start: all_positions[0],
        //     end: all_positions[1],
        //     thickness: 0.1,
        //     material: line_material,
        // },
        // ShapeBundle {
        //     path,
        //     spatial: SpatialBundle {
        //         transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // },
        // Stroke::new(Color::WHITE, 10.0),
    ));
}
