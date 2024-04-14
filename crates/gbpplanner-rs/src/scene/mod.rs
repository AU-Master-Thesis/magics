use bevy::{input::common_conditions::*, prelude::*};

use crate::config::{Config, Environment, FormationGroup};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveSimulation>()
            .add_event::<LoadSimulationEvent>()
            .add_event::<ReloadSimulationEvent>()
            .add_systems(
                PostUpdate,
                reload_scene.run_if(input_just_pressed(KeyCode::F5)),
            );
    }
}

pub struct Scene;

#[derive(Debug)]
pub struct Simulation {
    pub name: String,
    pub config: Config,
    pub environment: Environment,
    pub formation_group: FormationGroup,
}

impl Simulation {
    pub fn new(
        name: String,
        config: Config,
        environment: Environment,
        formation_group: FormationGroup,
    ) -> Self {
        Self {
            name,
            config,
            environment,
            formation_group,
        }
    }
}

// impl Time<Scene> {}

pub struct SimulationId;

#[derive(Resource)]
pub struct ActiveSimulation(Option<SimulationId>);

impl FromWorld for ActiveSimulation {
    fn from_world(world: &mut World) -> Self {
        Self(None)
        // todo!()
    }
}

#[derive(Event)]
pub struct LoadSimulationEvent(SimulationId);

#[derive(Event)]
pub struct ReloadSimulationEvent;

/// Marker component used to mark entities which can be reloaded as part of a
/// scene reload
#[derive(Component)]
pub struct Reloadable;

// fn reload_scene(world: &mut World, keyboard_input: Res<ButtonInput<KeyCode>>)
// {
fn reload_scene(world: &mut World) {
    // if !keyboard_input.any_pressed([KeyCode::F5]) {
    //     return;
    // }

    let mut query = world.query_filtered::<Entity, With<Reloadable>>();
    let matching_entities = query.iter(world).collect::<Vec<Entity>>();
    let n_matching_entities = matching_entities.len();

    info!("despawning reloadable entities in scene");
    for entity in matching_entities {
        world.despawn(entity);
    }
    info!(
        "reloadable entities in scene despawned: {}",
        n_matching_entities
    );

    let new_virtual_clock = Time::<Virtual>::default();
    // let mut time = world.resource_mut::<Time<Virtual>>();

    world.insert_resource::<Time<Virtual>>(new_virtual_clock);

    // time.pause();

    // let time = time.bypass_change_detection();
    // *time = new_virtual_clock;

    // let mut time = time.as_deref_mut();

    // *time.as_deref_mut() = new_virtual_clock;

    // time = new_virtual_clock;
}
