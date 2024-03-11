use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use crate::theme::CatppuccinTheme;

#[derive(Component, Debug, Copy, Clone)]
pub struct Factor(usize);

#[derive(Component, Debug, Copy, Clone)]
pub struct Variable(usize);

#[derive(Component, Debug)]
pub struct MoveMe;

/// A marker component for lines
/// Generally used to identify previously spawned lines,
/// so they can be updated or removed
#[derive(Component, Debug)]
pub struct Line;

/// A list of vertices defining a path
#[derive(Debug, Clone)]
struct Path {
    points: Vec<Vec3>,
}

impl From<Path> for Mesh {
    fn from(line: Path) -> Self {
        let vertices = line.points.clone();
        let width = 0.05;

        let mut left_vertices = Vec::<Vec3>::with_capacity(vertices.len());
        let mut right_vertices = Vec::<Vec3>::with_capacity(vertices.len());

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

            let angle = (std::f32::consts::PI - ab.dot(bc).acos()) / 2.0;
            let kinked_width = width / angle.sin();

            let n = {
                let sum = (ab + bc).normalize();
                Vec3::new(sum.z, sum.y, -sum.x)
            };
            let left = b - n * kinked_width / 2.0;
            let right = b + n * kinked_width / 2.0;

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

        // collect all vertices
        let vertices: Vec<Vec3> = left_vertices
            .iter()
            .zip(right_vertices.iter())
            .flat_map(|(l, r)| [*r, *l])
            .collect();

        Mesh::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::MAIN_WORLD  | RenderAssetUsages::RENDER_WORLD
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
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    let variables = vec![Variable(0), Variable(1), Variable(2)];
    let factors = vec![Factor(0), Factor(1)];

    let factor_graph = FactorGraph {
        factors: factors.clone(),
        variables: variables.clone(),
    };

    let alpha = 0.75;
    let variable_material = materials.add({
        let (r, g, b) = catppuccin_theme.blue().into();
        Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
    });
    let factor_material = materials.add({
        let (r, g, b) = catppuccin_theme.green().into();
        Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
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
    all_positions.sort_by(|a, b| a.z.partial_cmp(&b.z).expect("none of the operands are NAN"));
    info!("all_positions: {:?}", all_positions);

    let variable_mesh = meshes.add(
        bevy::math::primitives::Sphere::new(0.3)
            .mesh()
            .ico(4)
            .expect("4 subdivisions is less than the maximum allowed of 80"),
    );
    // let variable_mesh = meshes.add(
    //     shape::Icosphere {
    //         radius: 0.3,
    //         subdivisions: 4,
    //     }
    //     .try_into()
    //     .unwrap(),
    // );

    let factor_mesh = meshes.add(bevy::math::primitives::Cuboid::new(0.25, 0.25, 0.25));
    // let factor_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.25 }));

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
    catppuccin_theme: Res<CatppuccinTheme>,
    query_factors: Query<(&Factor, &Transform)>,
    query_variables: Query<(&Variable, &Transform)>,
    query_previous_lines: Query<Entity, With<Line>>,
) {
    // remove previous lines
    for entity in query_previous_lines.iter() {
        commands.entity(entity).despawn();
    }

    // collect all factor and variable positions
    let factor_positions: Vec<Vec3> = query_factors.iter().map(|(_, t)| t.translation).collect();
    let variable_positions: Vec<Vec3> =
        query_variables.iter().map(|(_, t)| t.translation).collect();

    // all positions sorted by z value
    let mut all_positions = variable_positions.clone();
    all_positions.extend(factor_positions.clone());

    all_positions.sort_by(|a, b| a.z.partial_cmp(&b.z).expect("none of the operands are NAN"));

    let line_material = materials.add({
        let (r, g, b) = catppuccin_theme.text().into();
        Color::rgb_u8(r, g, b)
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
