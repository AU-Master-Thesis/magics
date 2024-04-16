use std::collections::BTreeMap;

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, reflect::TypePath, utils::HashMap,
};
use bevy_asset_loader::{asset_collection::AssetCollection, prelude::*};
use bevy_notify::{ToastEvent, ToastLevel, ToastOptions};
use smol_str::SmolStr;

use crate::config::{Config, Environment, FormationGroup};

#[derive(Debug, thiserror::Error)]
pub enum SimulationLoaderPluginError {
    // #[error("The given simulations directory does not exist")]
    // SimulationsDirectoryNotExists(#[from] std::io::Error),
    #[error("No simulations found in {0}")]
    NoSimulationsFound(std::path::PathBuf),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

// #[derive(AssetCollection, Resource)]
// struct SimulationAssets {
//     #[asset(path = "./config/simulations", collection(mapped))]
//     folder: bevy::utils::HashMap<String, UntypedHandle>,
// }

#[derive(Debug, Default)]
pub struct SimulationLoaderPlugin {
    // pub simulations_dir: std::path::PathBuf,
}

// #[derive(Debug)]
// pub struct Simulations(HashMap<String, Simulation>);
type Simulations = BTreeMap<String, Simulation>;

impl SimulationLoaderPlugin {
    pub const SIMULATIONS_DIR: &'static str = "./config/simulations";
    // pub fn new(simulations_dir: std::path::PathBuf) -> Result<Self,
    // SimulationLoaderPluginError> {     if !simulations_dir.is_dir() {
    //         Err(From::from(std::io::Error::other(
    //             std::io::ErrorKind::NotFound,
    //         )))
    //     } else {
    //         Ok(Self { simulations_dir })
    //     }
    // }
}

impl Plugin for SimulationLoaderPlugin {
    fn build(&self, app: &mut App) {
        // let simulations: LoadableSimulations =
        // std::fs::read_dir(Self::SIMULATIONS_DIR)
        let simulations: BTreeMap<_, _> = std::fs::read_dir(Self::SIMULATIONS_DIR)
            .expect("failed to read simulation directory")
            .map(|dir| {
                let dir = dir.expect("failed to read simulation directory");

                let config_path = dir.path().join("config.toml");
                if !config_path.is_file() {
                    panic!(
                        "config.toml not found in simulation directory: {}",
                        dir.path().display()
                    );
                }

                let formation_path = dir.path().join("formation.ron");

                if !formation_path.is_file() {
                    panic!(
                        "formation.ron not found in simulation directory: {}",
                        dir.path().display()
                    );
                }
                let environment_path = dir.path().join("environment.yaml");

                if !environment_path.is_file() {
                    panic!(
                        "environment.yaml not found in simulation directory: {}",
                        dir.path().display()
                    );
                }

                let config =
                    Config::from_file(config_path.clone()).expect("file contains valid config");
                let formation = FormationGroup::from_file(formation_path.clone())
                    .expect("file contains a valid formation group(s)");

                let environment = Environment::from_file(environment_path.clone())
                    .expect("file contains valid environment");

                // dbg!(&config_path);
                // dbg!(&formation_path);
                // dbg!(&environment_path);

                // check config.toml
                // check environment.yaml
                // check formation.ron
                // dbg!(&dir);
                //
                let name = dir
                    .file_name()
                    .into_string()
                    .expect("failed to parse simulation name");
                let simulation = Simulation {
                    name: name.clone(),
                    config,
                    environment,
                    formation_group: formation,
                };

                (name, simulation)
            })
            .collect();

        if simulations.is_empty() {
            panic!("No simulations found in {}", Self::SIMULATIONS_DIR);
        }

        // app
        //     .init_state::<SimulationAssetStates>()
        //     .add_loading_state(
        //         LoadingState::new(SimulationAssetStates::Loading)
        //         .continue_to_state(SimulationAssetStates::Loaded)
        //         .load_collection::<SimulationAssets>()
        //     .register_dynamic_asset_collection::<CustomDynamicAssetCollection>()
        //         .with_dynamic_assets_file::<CustomDynamicAssetCollection>("custom.
        // simulation.ron"),

        //     )

        // init_collection::<SimulationAssets>()
        // .insert_resource(SimulationManager::new(self.simulations_dir.clone()))
        app.insert_resource(SimulationManager::new(simulations))
            .init_resource::<ActiveSimulation>()
            .add_event::<LoadSimulation>()
            .add_event::<EndSimulation>()
            .add_event::<SimulationReloaded>()
            .add_systems(
                Update,
                show_toast_when_simulation_reloads.run_if(on_event::<SimulationReloaded>()),
            )
                .add_systems(PostStartup, load_initial_simulation)
            // .add_systems(OnEnter(SimulationAssetStates::Loaded), load_simulation)
            .add_systems(PostUpdate, load_simulation)
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
    // simulations_dir: std::path::PathBuf,
    // names: Vec<String>,
    names: Vec<SmolStr>,
    simulations: Vec<Simulation>,
    // simulations: Simulations,
    active: Option<usize>,
    reload_requested: Option<()>,
}

// impl<'world> SimulationManager<'world> {
// impl<'a> SimulationManager<'a> {
impl SimulationManager {
    #[must_use]
    fn new(simulations: Simulations) -> Self {
        let names = simulations.keys().cloned().map(Into::into).collect();
        let simulations = simulations.into_values().collect();

        let active = Some(0);
        Self {
            names,
            simulations,
            active,
            // active: None,
            reload_requested: None,
        }
    }

    // pub fn active(&self) -> Option<SimulationId> {
    //     self.active
    // }

    pub fn active(&self) -> Option<&Simulation> {
        let active = self.active?;
        self.simulations.get(active)
    }

    pub fn active_id(&self) -> Option<SimulationId> {
        self.active.map(SimulationId)
    }

    pub fn active_name(&self) -> Option<&str> {
        self.names.get(self.active?).map(|s| s.as_str())
    }

    pub fn names(&self) -> impl Iterator<Item = &SmolStr> {
        self.names.iter()

        // self.simulations.keys().map(|s| s.as_str())
    }

    pub fn ids_and_names(&self) -> impl Iterator<Item = (SimulationId, SmolStr)> + '_ {
        (0..self.simulations.len())
            .map(SimulationId)
            // .zip(self.names.iter().map(|s| s.as_str()))
            .zip(self.names.iter().map(Clone::clone))
    }

    pub fn reload(&mut self) {
        if self.reload_requested.is_none() {
            info!("setting reload requested to Some(())");
            self.reload_requested = Some(());
        }
    }

    #[must_use]
    pub fn ids(&self) -> impl Iterator<Item = SimulationId> + '_ {
        (0..self.simulations.len()).map(SimulationId)
    }

    #[must_use]
    pub fn id_from_name(&self, name: &str) -> Option<SimulationId> {
        self.names.iter().position(|n| n == name).map(SimulationId)
    }

    pub fn load(&mut self, id: SimulationId) {
        self.active = Some(id.0);
        info!("loading simulation with id: {}", id.0);
        self.reload_requested = Some(());
    }

    // pub fn get_

    // #[must_use]
    // pub fn new(simulations_dir: std::path::PathBuf) -> Self {
    //     Self { simulations_dir }
    // }

    pub fn get_config_for(&self, id: SimulationId) -> Option<&Config> {
        self.simulations.get(id.0).map(|s| &s.config)
        // todo!()
    }

    pub fn get_environment_for(&self, id: SimulationId) -> Option<&Environment> {
        self.simulations.get(id.0).map(|s| &s.environment)
    }

    pub fn get_formation_group_for(&self, id: SimulationId) -> Option<&FormationGroup> {
        self.simulations.get(id.0).map(|s| &s.formation_group)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
pub struct LoadSimulation(pub SimulationId);

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
pub enum SimulationStates {
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

// fn load_simulation() {}

// #[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
// enum SimulationAssetStates {
//     #[default]
//     Loading,
//     Loaded,
// }

// #[derive(serde::Deserialize, Asset, TypePath)]
// pub struct CustomDynamicAssetCollection(HashMap<String,
// SimulationDynamicAsset>);

// impl DynamicAssetCollection for CustomDynamicAssetCollection {
//     fn register(&self, dynamic_assets: &mut DynamicAssets) {
//         for (key, asset) in self.0.iter() {
//             dynamic_assets.register_asset(key, Box::new(asset.clone()));
//         }
//     }
// }

// #[derive(serde::Deserialize, Debug, Clone)]
// enum SimulationDynamicAsset {
//     Config,
//     Environment,
//     Formation,
// }

fn load_initial_simulation(
    simulation_manager: Res<SimulationManager>,
    mut evw_load_simulation: EventWriter<LoadSimulation>,
) {
    if let Some(id) = simulation_manager.active_id() {
        evw_load_simulation.send(LoadSimulation(id));
        info!("sent load simulation event with id: {}", id.0);
    }

    // if simulation_manager.is_changed() {
    //     evw_load_simulation.send(LoadSimulation(SimulationId(0)));
    // }
}

fn load_simulation(
    mut commands: Commands,
    mut evw_load_simulation: EventWriter<LoadSimulation>,
    // mut evw_end_simulation: EventWriter<EndSimulation>,
    mut simulation_manager: ResMut<SimulationManager>,

    ephemeral_entities: Query<Entity, With<Ephemeral>>,
) {
    if simulation_manager.reload_requested.is_some() {
        for entity in &ephemeral_entities {
            info!("despawning ephemeral entity: {:?}", entity);
            commands.entity(entity).despawn();
        }

        let id = simulation_manager.active.map(SimulationId).unwrap();
        info!("sent load simulation event with id: {}", id.0);

        evw_load_simulation.send(LoadSimulation(id));
        simulation_manager.reload_requested = None;
    }
}
