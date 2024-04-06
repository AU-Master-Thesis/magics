#![deny(missing_docs)]
//! Useful function when working with bevy

use bevy::{ecs::prelude::*, hierarchy::DespawnRecursiveExt};

/// Generic system that takes a component as a parameter, and will despawn all
/// entities with that component
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// #[derive(Component)]
/// struct OnSplashScreen;
///
/// #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
/// enum GameState {
///     #[default]
///     Splash,
///     Menu,
///     Game,
/// }
///
/// App::new()
///     .add_systems(
///         OnExit(GameState::Splash),
///         despawn_entities_with_component::<OnSplashScreen>,
///     )
///     .run();
/// ```
pub fn despawn_entities_with_component<T: Component>(
    to_despawn: Query<Entity, With<T>>,
    mut commands: Commands,
) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
