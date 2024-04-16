use std::marker::PhantomData;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_notify::{ToastEvent, ToastLevel, ToastOptions};

use crate::config::{Config, Environment, FormationGroup};

pub struct SimulationLoaderPlugin;

impl Plugin for SimulationLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationManager::default())
            .init_resource::<ActiveSimulation>()
            .add_event::<LoadSimulation>()
            .add_event::<EndSimulation>()
            .add_event::<SimulationReloaded>()
            .add_systems(
                Update,
                show_toast_when_simulation_reloads.run_if(on_event::<SimulationReloaded>()),
            )
            .add_systems(
                Update,
                reload_simulation.run_if(input_just_pressed(KeyCode::F5)),
            );

        // if app.world.get_resource::<Events<LoadSimulation>>().is_some() {}

        //     ;.is_some() {

        // }
    }
}

#[derive(Debug, Component)]
pub struct Ephemeral;

#[derive(Debug)]
pub struct Simulation {
    pub name: String,
    pub config: Config,
    pub environment: Environment,
    pub formation_group: FormationGroup,
}

impl Simulation {
    pub const fn new(
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

#[derive(Debug, Resource, Default)]
pub struct SimulationManager {
    // _phantom_data: PhantomData<()>,
}

// impl<'world> SimulationManager<'world> {
impl SimulationManager {
    // pub fn get_config_for(id: SimulationId) -> Option<&'world Config> {
    //     todo!()
    // }

    // pub fn get_environment_for(id: SimulationId) -> Option<&Environment> {
    //     todo!()
    // }

    // pub fn get_formation_for(id: SimulationId) -> Option<&Formation> {
    //     None
    // }
}

pub struct SimulationId(usize);

#[derive(Resource)]
pub struct ActiveSimulation(Option<SimulationId>);

impl FromWorld for ActiveSimulation {
    fn from_world(_world: &mut World) -> Self {
        Self(None)
        // todo!()
    }
}

#[derive(Event)]
pub struct LoadSimulation(SimulationId);

#[derive(Event)]
pub struct EndSimulation(SimulationId);

// TODO: send an simulation generation or id with
#[derive(Event, Default)]
pub struct SimulationReloaded;

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

    world.send_event_default::<SimulationReloaded>();
    // world.send_event::<ReloadSimulation>()

    // time.pause();

    // let time = time.bypass_change_detection();
    // *time = new_virtual_clock;

    // let mut time = time.as_deref_mut();

    // *time.as_deref_mut() = new_virtual_clock;

    // time = new_virtual_clock;
}

fn show_toast_when_simulation_reloads(mut evw_toast: EventWriter<ToastEvent>) {
    evw_toast.send(ToastEvent {
        caption: "reloaded simulation".into(),
        options: ToastOptions {
            level: ToastLevel::Success,
            closable: false,
            show_progress_bar: false,
            ..Default::default()
        },
    });
}

fn reload_simulation(
    mut evw_reload_simulation: EventWriter<SimulationReloaded>,
    mut end_simulation: EventWriter<EndSimulation>,
) {
    info!("ending simulation");
    end_simulation.send(EndSimulation(SimulationId(0)));
}

// TODO: use in app
#[derive(
    Debug,
    Default,
    States,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    derive_more::Display,
    derive_more::IsVariant,
)]
pub enum SimulationState {
    #[default]
    #[display(fmt = "Loading")]
    Loading,
    #[display(fmt = "Starting")]
    Starting,
    #[display(fmt = "Running")]
    Running,
    #[display(fmt = "Paused")]
    Paused,
    #[display(fmt = "Finished")]
    Finished,
}
