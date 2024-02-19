// use factorgraph::FactorGraph;

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

#[derive(Component, Debug, Copy, Clone)]
pub struct Factor(usize);

#[derive(Component, Debug, Copy, Clone)]
pub struct Variable(usize);

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
struct LineList {
    lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

        Mesh::new(
            // This tells wgpu that the positions are list of lines
            // where every pair is a start and end point
            PrimitiveTopology::LineList,
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
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
        app.add_plugins((MaterialPlugin::<LineMaterial>::default()))
            .add_systems(Startup, insert_dummy_factor_graph);
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

    let variable_material = materials.add(Color::rgb(0.0, 0.0, 1.0).into());
    let factor_material = materials.add(Color::rgb(1.0, 0.0, 0.0).into());
    let line_material = materials.add(Color::rgb(0.0, 1.0, 0.0).into());

    let variable_positions: Vec<Vec3> = variables
        .iter()
        .map(|v| {
            let i = v.0 as f32;
            Vec3::new(0.0, 0.0, i * 5.0 + 6.0)
        })
        .collect();
    let factor_positions: Vec<Vec3> = factors
        .iter()
        .map(|f| {
            let i = f.0 as f32;
            Vec3::new(0.0, 0.0, i * 5.0 + 8.5)
        })
        .collect();

    // all positions sorted by z value
    let mut all_positions = variable_positions.clone();
    all_positions.extend(factor_positions.clone());
    all_positions.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

    let variable_mesh = meshes.add(
        shape::Icosphere {
            radius: 0.3,
            subdivisions: 4,
        }
        .try_into()
        .unwrap(),
    );

    let factor_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.2 }));

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

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(LineList {
                lines: all_positions
                    .iter()
                    .zip(all_positions.iter().skip(1))
                    .map(|(a, b)| (*a, *b))
                    .collect(),
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
    ));
}
