//! Simple bevy plugin that exposes components to despawn an entity after a
//! given duration
use bevy::prelude::*;

/// **Bevy** Plugin that exposes components to despawn an entity after a given
/// duration
#[derive(Default)]
pub struct DespawnEntityAfterPlugin;

impl Plugin for DespawnEntityAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (real_despawn_entity_after, virtual_despawn_entity_after),
        )
        .add_systems(FixedUpdate, fixed_despawn_entity_after);
    }
}

/// **Bevy** components exposed by this plugin
pub mod components {
    use bevy::prelude::*;

    /// Despawns an entity after a given duration
    #[derive(Component)]
    pub struct DespawnEntityAfter<T: Default = ()> {
        pub entity_to_despawn: Entity,
        pub timer: bevy::time::Timer,
        despawn_recursive: bool,
        _marker: std::marker::PhantomData<T>,
    }

    impl<T: Default> DespawnEntityAfter<T> {
        pub fn new(entity_to_despawn: Entity, duration: std::time::Duration) -> Self {
            Self {
                entity_to_despawn,
                despawn_recursive: false,
                timer: Timer::new(duration, TimerMode::Once),
                _marker: std::marker::PhantomData,
            }
        }

        /// Despawns the entity recursively i.e. will use
        /// `Commands::despawn_recursive` instead of `Commands::despawn`
        pub fn recursive(mut self) -> Self {
            self.despawn_recursive = true;
            self
        }

        pub(super) fn despawn(&self, commands: &mut Commands) {
            if self.despawn_recursive {
                commands.entity(self.entity_to_despawn).despawn_recursive();
            } else {
                commands.entity(self.entity_to_despawn).despawn();
            }
        }
    }

    /// Despawns entities after a given duration
    #[derive(Component)]
    pub struct DespawnEntitiesAfter<T: Default = ()> {
        pub entities_to_despawn: Vec<Entity>,
        pub timer: bevy::time::Timer,
        despawn_recursive: bool,
        _marker: std::marker::PhantomData<T>,
    }

    impl<T: Default> DespawnEntitiesAfter<T> {
        pub fn new(
            entities_to_despawn: impl IntoIterator<Item = Entity>,
            duration: std::time::Duration,
        ) -> Self {
            Self {
                entities_to_despawn: entities_to_despawn.into_iter().collect(),
                despawn_recursive: false,
                timer: Timer::new(duration, TimerMode::Once),
                _marker: std::marker::PhantomData,
            }
        }

        /// Despawns the entity recursively i.e. will use
        /// `Commands::despawn_recursive` instead of `Commands::despawn`
        pub fn recursive(mut self) -> Self {
            self.despawn_recursive = true;
            self
        }

        pub(super) fn despawn(&self, commands: &mut Commands) {
            if self.despawn_recursive {
                for entity in &self.entities_to_despawn {
                    commands.entity(*entity).despawn_recursive();
                }
            } else {
                for entity in &self.entities_to_despawn {
                    commands.entity(*entity).despawn();
                }
            }
        }
    }
}

macro_rules! impl_despawn_entity_after {
    ($clock:ty, $fn_prefix:ident) => {
        paste::paste! {
            fn [<$fn_prefix _despawn_entity_after>](
                mut commands: Commands,
                mut q_despawn_entity: Query<(Entity, &mut components::DespawnEntityAfter<$clock>)>,
                // mut despawn_markers: Query<&mut components::DespawnAfter<$clock>>,
                mut q_despawn_entities: Query<(Entity, &mut components::DespawnEntitiesAfter<$clock>)>,
                time: Res<Time<$clock>>,
            ) {
                for (entity, mut despawner) in q_despawn_entity.iter_mut() {
                    despawner.timer.tick(time.delta());
                    if despawner.timer.finished() {
                        despawner.despawn(&mut commands);
                        commands.entity(entity).despawn();
                    }
                }

                for (entity, mut despawner) in q_despawn_entities.iter_mut() {
                    despawner.timer.tick(time.delta());
                    if despawner.timer.finished() {
                        despawner.despawn(&mut commands);
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    };
}

impl_despawn_entity_after!(Real, real);
impl_despawn_entity_after!(Fixed, fixed);
impl_despawn_entity_after!(Virtual, virtual);
