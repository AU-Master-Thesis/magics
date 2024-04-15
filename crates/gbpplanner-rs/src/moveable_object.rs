use bevy::prelude::*;

// use leafwing_input_manager::prelude::*;
use crate::{asset_loader::SceneAssets, environment, movement::MovingObjectBundle};

const SCALE: f32 = 0.2;
const START_TRANSLATION: Vec3 = Vec3::new(0., 0., 0.);
pub const SPEED: f32 = 5.0; // m/s
pub const BOOST_SPEED: f32 = 50.0; // m/s
pub const ANGULAR_SPEED: f32 = 1.0; // rad/s
pub const BOOST_ANGULAR_SPEED: f32 = 5.0; // rad/s

pub struct MoveableObjectPlugin;

impl Plugin for MoveableObjectPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_state::<MoveableObjectMovementState>()
            .init_state::<MoveableObjectVisibilityState>()
            // .add_state::<MoveableObjectMovementState>()
            // .add_state::<MoveableObjectVisibilityState>()
            .add_systems(Startup, spawn);
    }
}

#[derive(Component)]
pub struct MoveableObject;

/// Here, we define a State for Scenario.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectMovementState {
    #[default]
    Default,
    Boost,
}

// define visibility state for the moveable object
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectVisibilityState {
    #[default]
    Visible,
    Hidden,
}

fn spawn(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    // mut event_writer: EventWriter<RobotSpawnedEvent>,
) {
    let mut transform = Transform::from_translation(START_TRANSLATION);
    transform.scale = Vec3::splat(SCALE);
    let offset = Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0;
    let follow_camera_flag = environment::follow_cameras::FollowCameraMe::from_vec3(offset);
    commands.spawn((
        MovingObjectBundle {
            model: SceneBundle {
                // scene: scene_assets.roomba.clone(),
                scene: scene_assets.roomba.clone(),
                transform,
                ..default()
            },
            ..default()
        },
        MoveableObject,
        // environment::follow_cameras::FollowCameraMe {
        //     offset: Some(offset),
        // },
        follow_camera_flag,
        crate::movement::Local,
    ));

    // event_writer.send(RobotSpawnedEvent {
    //     entity,
    //     transform,
    //     follow_camera_flag,
    // });
}
