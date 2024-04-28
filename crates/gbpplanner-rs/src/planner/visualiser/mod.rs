mod communication;
pub mod communication_radius;
pub mod factorgraphs;
mod obstacle;
mod robot;
mod tracer;
mod uncertainty;
pub mod waypoints;

const Z_FIGHTING_OFFSET: f32 = 0.04;

use bevy::{
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages},
};

use self::{
    communication::CommunicationGraphVisualiserPlugin, communication_radius::CommunicationRadiusVisualizerPlugin,
    factorgraphs::FactorGraphVisualiserPlugin, robot::RobotVisualiserPlugin, tracer::TracerVisualiserPlugin,
    uncertainty::UncertaintyVisualiserPlugin, waypoints::WaypointVisualiserPlugin,
};
use super::RobotId;

/// A **Bevy** `Plugin` for visualising aspects of the planner
/// Includes visualising parts of the factor graph
pub struct VisualiserPlugin;

impl Plugin for VisualiserPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            WaypointVisualiserPlugin,
            FactorGraphVisualiserPlugin,
            CommunicationGraphVisualiserPlugin,
            UncertaintyVisualiserPlugin,
            TracerVisualiserPlugin,
            CommunicationRadiusVisualizerPlugin,
            RobotVisualiserPlugin,
            obstacle::ObstacleFactorVisualizerPlugin,
        ));
    }
}

/// A **Bevy** `Component` for keeping track of a robot
/// Keeps track of the `RobotId` and `Vec2` position
#[derive(Debug, Component)]
pub struct RobotTracker {
    pub robot_id: RobotId,
    pub variable_index: usize,
    pub order: usize,
}

impl RobotTracker {
    // REFACTOR: take all 3 arguments here
    #[must_use]
    pub const fn new(robot_id: RobotId) -> Self {
        Self {
            robot_id,
            variable_index: 0,
            order: 0,
        }
    }

    pub const fn with_variable_index(mut self, id: usize) -> Self {
        self.variable_index = id;
        self
    }

    pub const fn with_order(mut self, order: usize) -> Self {
        self.order = order;
        self
    }
}

/// A **Bevy** marker [`Component`] for lines
/// Generally used to identify previously spawned lines,
/// so they can be updated or removed
#[derive(Component)]
pub struct Line;

/// A **Bevy** marker [`Component`] for a line segment
/// Generally used to identify previously spawned line segments,
/// so they can be updated or removed
#[derive(Component)]
pub struct LineSegment;

/// A **Bevy** [`Component`] for drawing a path or line
/// Contains a list of points and a width used to construct a mesh
#[derive(Debug, Clone)]
struct Path {
    points: Vec<Vec3>,
    width:  f32,
}

impl Path {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self { points, width: 0.1 }
    }

    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl From<Path> for Mesh {
    fn from(line: Path) -> Self {
        let vertices = line.points;
        let width = line.width;

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

        Self::new(
            PrimitiveTopology::TriangleStrip,
            RenderAssetUsages::MAIN_WORLD  | RenderAssetUsages::RENDER_WORLD
        )
        // Add the vertices positions as an attribute
        .with_inserted_attribute(Self::ATTRIBUTE_POSITION, vertices)
    }
}
