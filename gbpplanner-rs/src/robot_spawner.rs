use bevy::prelude::*;

use crate::{
    environment::follow_cameras::FollowCameraMe,
    movement::{
        AngularMovementBundle, AngularVelocity, LinearMovementBundle, Local, MovementBundle,
        Velocity,
    },
};

pub struct RobotSpawnerPlugin;

impl Plugin for RobotSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RobotSpawnedEvent>()
            .init_state::<RobotsState>();
        // .add_state::<RobotsState>();
        // .add_systems(Update, spawn_robot_periodically);
    }
}

// Define the event
#[derive(Event, Debug, Copy, Clone)]
pub struct RobotSpawnedEvent {
    pub entity:             Entity,
    pub transform:          Transform,
    pub follow_camera_flag: FollowCameraMe,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
pub struct RobotsState {
    pub amount: usize,
}

// Define the Robot component
#[derive(Component)]
struct Robot;

// System to spawn a robot
fn spawn_robot_periodically(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut event_writer: EventWriter<RobotSpawnedEvent>,
    state: Res<State<RobotsState>>,
    mut next_state: ResMut<NextState<RobotsState>>,
) {
    if state.amount >= 10 {
        return;
    }

    // spawn a robot every 1 or 5 seconds
    let time_between_spawns = if state.amount >= 5 { 5.0 } else { 1.0 };

    if time.elapsed_seconds() % time_between_spawns < time.delta_seconds() {
        // pick a random location within 20x20 in the xz plane
        let x = rand::random::<f32>() * 20.0 - 10.0;
        let z = rand::random::<f32>() * 20.0 - 10.0;

        // create a cube mesh
        let size = 1.0;
        // let mesh = meshes.add(Mesh::from(bevy::prelude::shape::Cube { size }));
        let mesh = meshes.add(bevy::math::primitives::Cuboid::new(size, size, size));

        // create a material
        let material = materials.add(Color::rgb(0.8, 0.7, 0.6));
        let transform = Transform::from_translation(Vec3::new(x, size / 2.0, z));
        let follow_camera_flag = FollowCameraMe {
            offset: Some(Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0),
        };

        // spawn the robot
        let entity = commands
            .spawn((
                PbrBundle {
                    mesh,
                    material,
                    transform,
                    ..Default::default()
                },
                MovementBundle {
                    linear_movement:  LinearMovementBundle {
                        velocity: Velocity {
                            value: Vec3::new(0.0, 0.0, 1.0),
                        },
                        ..Default::default()
                    },
                    angular_movement: AngularMovementBundle {
                        angular_velocity: AngularVelocity {
                            value: Vec3::new(0.0, 0.2, 0.0),
                        },
                        ..Default::default()
                    },
                },
                Local,
                Robot,
                follow_camera_flag,
            ))
            .id();

        // send the event
        event_writer.send(RobotSpawnedEvent {
            entity,
            transform,
            follow_camera_flag,
        });
        next_state.set(RobotsState {
            amount: state.amount + 1,
        });
    }
}
