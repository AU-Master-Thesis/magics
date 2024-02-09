use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use leafwing_input_manager::prelude::*;

pub struct MoveableObjectPlugin;

impl Plugin for MoveableObjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MoveableObjectMovementState>()
            .add_state::<MoveableObjectVisibilityState>()
            .add_systems(PostStartup, spawn);
        // .add_systems(Update, (visibility_actions, movement_actions));
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    query: Query<(Entity, With<InputMap<crate::input::InputAction>>)>,
    // query: Query<(Entity, With<ActionState<crate::input::InputAction>>)>,
) {
    if let Ok((entity, _)) = query.get_single() {
        commands.entity(entity).insert(MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(5.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::PURPLE)),
            // transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            ..default()
        });
        // .insert(MoveableObject)
    }
    // commands
    //     .insert(MaterialMesh2dBundle {
    //         mesh: meshes.add(shape::Circle::new(5.).into()).into(),
    //         material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //         // transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
    //         ..default()
    //     })
    //     .insert(MoveableObject);
}
