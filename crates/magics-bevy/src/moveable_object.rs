use bevy::prelude::*;

use crate::{
    environment::FollowCameraMe,
    movement::MovingObjectBundle,
};

/// Constants for moveable object behavior
pub const SCALE: f32 = 0.2;
pub const START_TRANSLATION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
pub const SPEED: f32 = 5.0; // m/s
pub const BOOST_SPEED: f32 = 50.0; // m/s
pub const ANGULAR_SPEED: f32 = 1.0; // rad/s
pub const BOOST_ANGULAR_SPEED: f32 = 5.0; // rad/s

/// Plugin for moveable object functionality
pub struct MoveableObjectPlugin;

impl Plugin for MoveableObjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MoveableObjectMovementState>()
            .init_state::<MoveableObjectVisibilityState>();
    }
}

/// Component marker for a moveable object
#[derive(Component)]
pub struct MoveableObject;

/// Movement state for the moveable object
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectMovementState {
    #[default]
    Default,
    Boost,
}

/// Visibility state for the moveable object
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MoveableObjectVisibilityState {
    #[default]
    Visible,
    Hidden,
}

/// Spawn a moveable object in the world
pub fn spawn_moveable_object(
    commands: &mut Commands, 
    model: Handle<Scene>,
    transform: Option<Transform>,
) -> Entity {
    let mut transform = transform.unwrap_or_else(|| Transform::from_translation(START_TRANSLATION));
    transform.scale = Vec3::splat(SCALE);
    
    // Create a follow camera offset
    let offset = Vec3::new(0.0, 5.0, -10.0).normalize() * 10.0;
    
    commands.spawn((
        MovingObjectBundle {
            model: SceneBundle {
                scene: model,
                transform,
                ..default()
            },
            ..default()
        },
        MoveableObject,
        FollowCameraMe::from(offset),
        crate::movement::Local,
    )).id()
}
