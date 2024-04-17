use std::{
    collections::{BTreeMap, VecDeque},
    time::Duration,
};

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    time::common_conditions::{on_real_timer, on_timer},
};
use bevy_notify::{ToastEvent, ToastLevel, ToastOptions};
use smol_str::SmolStr;

use crate::{
    config::{Config, Environment, FormationGroup},
    simulation_loader,
};

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

#[derive(Debug)]
pub struct SimulationLoaderPlugin {
    // pub simulations_dir: std::path::PathBuf,
    pub show_toasts: bool,
}

impl Default for SimulationLoaderPlugin {
    fn default() -> Self {
        Self { show_toasts: true }
    }
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

    // fn
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

        app
            .insert_state(SimulationStates::default())
            .insert_resource(SimulationManager::new(simulations))
            .init_resource::<ActiveSimulation>()
            .add_event::<LoadSimulation>()
            .add_event::<ReloadSimulation>()
            .add_event::<EndSimulation>()
            .add_event::<SimulationReloaded>()
            .add_systems(PreStartup, load_initial_simulation)
            .add_systems(Update, echo_state::<SimulationStates>().run_if(state_changed::<SimulationStates>))
            .add_systems(Update, handle_requests.run_if(on_real_timer(Duration::from_millis(500))))
            // .add_systems(OnEnter(SimulationStates::Loading), load_simulation)
            // .add_systems(OnEnter(SimulationStates::Reloading), reload_simulation)
            // .add_systems(
            //     Update,
            //     // show_toast_when_simulation_reloads.run_if(on_event::<SimulationReloaded>()),
            //     show_toast_when_simulation_reloads.run_if(on_event::<ReloadSimulation>()),
            // )
                // .add_systems(PostStartup, load_initial_simulation)
            // .add_systems(OnEnter(SimulationAssetStates::Loaded), load_simulation)
            // .add_systems(PostUpdate, load_simulation)
            .add_systems(
                Update,
                // enter_state(SimulationStates::Reloading).run_if(input_just_pressed(KeyCode::F5))
                reload_simulation.run_if(input_just_pressed(KeyCode::F5)),
            );
        // .add_systems(Update, load_simulation);
    }
}

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
    requests: VecDeque<Request>,
    simulations_loaded: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    Load(SimulationId),
    Reload,
    End,
}

// impl<'world> SimulationManager<'world> {
// impl<'a> SimulationManager<'a> {
impl SimulationManager {
    #[must_use]
    fn new(simulations: Simulations) -> Self {
        let names = simulations.keys().cloned().map(Into::into).collect();
        let simulations = simulations.into_values().collect();

        let requests = VecDeque::from([Request::Load(SimulationId(0))]);

        let active = Some(0);
        Self {
            names,
            simulations,
            active,
            // active: None,
            reload_requested: None,
            requests,
            // requests: VecDeque::new(),
            simulations_loaded: 0,
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
        // let Some(active_simulation_id) = self.active else {
        //     return;
        // };

        if self.active.is_some() {
            self.requests.push_back(Request::Reload);
        }

        // if self.reload_requested.is_none() {
        //     info!("setting reload requested to Some(())");
        //     self.reload_requested = Some(());
        // }
    }

    pub fn load(&mut self, id: SimulationId) {
        // self.active = Some(id.0);
        self.requests.push_back(Request::Load(id));
        // info!("loading simulation with id: {}", id.0);
        // self.reload_requested = Some(());
    }

    #[must_use]
    pub fn ids(&self) -> impl Iterator<Item = SimulationId> + '_ {
        (0..self.simulations.len()).map(SimulationId)
    }

    #[must_use]
    pub fn id_from_name(&self, name: &str) -> Option<SimulationId> {
        self.names.iter().position(|n| n == name).map(SimulationId)
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
pub struct ReloadSimulation(pub SimulationId);

#[derive(Event)]
pub struct EndSimulation(pub SimulationId);

// TODO: send an simulation generation or id with
#[derive(Event, Default)]
pub struct SimulationReloaded;

/// Marker component used to mark entities which can be reloaded as part of a
/// scene reload
#[derive(Component)]
pub struct Reloadable;

fn reload_simulation(mut simulation_manager: ResMut<SimulationManager>) {
    simulation_manager.reload();
}

// fn reload_scene(world: &mut World, keyboard_input: Res<ButtonInput<KeyCode>>)
// {
// fn reload_simulation(world: &mut World) {
//     // if !keyboard_input.any_pressed([KeyCode::F5]) {
//     //     return;
//     // }
//
//     let mut query = world.query_filtered::<Entity, With<Reloadable>>();
//     let matching_entities = query.iter(world).collect::<Vec<Entity>>();
//     let n_matching_entities = matching_entities.len();
//
//     info!("despawning reloadable entities in scene");
//     for entity in matching_entities {
//         world.despawn(entity);
//     }
//     info!(
//         "reloadable entities in scene despawned: {}",
//         n_matching_entities
//     );
//
//     let new_virtual_clock = Time::<Virtual>::default();
//     // let mut time = world.resource_mut::<Time<Virtual>>();
//
//     world.insert_resource::<Time<Virtual>>(new_virtual_clock);
//
//     world.send_event_default::<SimulationReloaded>();
//
//     // world.send_event::<ReloadSimulation>()
//
//     // time.pause();
//
//     // let time = time.bypass_change_detection();
//     // *time = new_virtual_clock;
//
//     // let mut time = time.as_deref_mut();
//
//     // *time.as_deref_mut() = new_virtual_clock;
//
//     // time = new_virtual_clock;
// }
//
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

fn show_toast_when_simulation_state_changes(
    mut evw_toast: EventWriter<ToastEvent>,
    state: Res<State<SimulationStates>>,
) {
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

// fn reload_simulation(
//     // mut evw_reload_simulation: EventWriter<SimulationReloaded>,
//     // mut end_simulation: EventWriter<EndSimulation>,
//     mut simulation_manager: ResMut<SimulationManager>,
// ) {
//     simulation_manager.reload();
//     // info!("ending simulation");
//     // end_simulation.send(EndSimulation(SimulationId(0)));
// }

// TODO: use in app
#[derive(Debug, Default, States, PartialEq, Eq, Hash, Clone, Copy, derive_more::IsVariant)]
pub enum SimulationStates {
    #[default]
    Loading,
    Starting,
    Running,
    Paused,
    Reloading,
    Ended,
    // Finished,
}

impl SimulationStates {
    fn transition(&mut self) {
        use SimulationStates::*;
        *self = match self {
            Loading => Starting,
            Starting => Running,
            Running => Running,
            Paused => Paused,
            Reloading => Loading,
            Ended => Ended,
        }
    }
}

impl std::fmt::Display for SimulationStates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Loading => write!(f, "Loading"),
            Self::Starting => write!(f, "Starting"),
            Self::Running => write!(f, "Running"),
            Self::Paused => write!(f, "Paused"),
            Self::Reloading => write!(f, "Reloading"),
            Self::Ended => write!(f, "Ended"),
        }
    }
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
    // simulation_manager: Res<SimulationManager>,
    // mut evw_load_simulation: EventWriter<LoadSimulation>,
    world: &mut World,
) {
    let simulation_manager = world
        .get_resource::<SimulationManager>()
        .expect("simulation manager has been inserted");

    // insert initial config, formation and environment resource

    let Some(simulation_id) = simulation_manager.active_id() else {
        panic!("no initial simulation set to active");
    };

    // if let Some(config) = simulation_manager.get_config_for(simulation_id) {
    //     let config = config.to_owned();
    //     world.insert_resource(config);
    // }

    if let Some(environment) = simulation_manager.get_environment_for(simulation_id) {
        let environment = environment.to_owned();
        world.insert_resource(environment);
    }

    // if let Some(formation) =
    // simulation_manager.get_formation_for(simulation_id) {
    //     let formation = formation.to_owned();
    //     world.insert_resource(formation);
    // }

    // if let Some(id) = simulation_manager.active_id() {
    //     evw_load_simulation.send(LoadSimulation(id));
    //     info!("sent load simulation event with id: {}", id.0);
    // }
}

fn load_simulation(
    mut commands: Commands,
    mut evw_load_simulation: EventWriter<LoadSimulation>,
    // mut evw_end_simulation: EventWriter<EndSimulation>,
    mut simulation_manager: ResMut<SimulationManager>,
    mut next_state: ResMut<NextState<SimulationStates>>,
    ephemeral_entities: Query<Entity, With<Reloadable>>,
) {
    if simulation_manager.reload_requested.is_some() {
        for entity in &ephemeral_entities {
            info!("despawning ephemeral entity: {:?}", entity);
            // commands.entity(entity).despawn();

            // commands.entity(entity).despawn_descendants();
            commands.entity(entity).despawn_recursive();
        }

        let id = simulation_manager.active.map(SimulationId).unwrap();
        info!("sent load simulation event with id: {}", id.0);

        evw_load_simulation.send(LoadSimulation(id));
        simulation_manager.reload_requested = None;
    }
}

fn echo_state<S: States>() -> impl Fn(Res<State<S>>) {
    move |state: Res<State<S>>| {
        info!("{} state is: {:?}", std::any::type_name::<S>(), state.get());
    }
}
// fn echo_state<S: States>(state: Res<State<S>>) {
//     info!("state: {:?}", state.get());
// }
//
// fn echo_state(state: ResMut<State<SimulationStates>>) {
//     info!("state: {:?}", state.get());
// }

fn enter_state(state: SimulationStates) -> impl FnMut(ResMut<NextState<SimulationStates>>) {
    move |mut next_state: ResMut<NextState<SimulationStates>>| {
        next_state.set(state);
    }
}

// fn restart_virtual_time

// fn restart_time<T>(world: &mut World) {
//     if let Some(time) = world.get_resource::<Time<T>>() {
//         // preserve its pause/unpause state and reset only its time value
//         // let time = time.bypass_change_detection();
//         // *time = T::default();
//     }
// }
//

fn handle_requests(
    mut commands: Commands,
    mut simulation_manager: ResMut<SimulationManager>,
    mut evw_load_simulation: EventWriter<LoadSimulation>,
    mut evw_reload_simulation: EventWriter<ReloadSimulation>,
    mut evw_end_simulation: EventWriter<EndSimulation>,
    mut evw_toast: EventWriter<ToastEvent>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut config: ResMut<Config>,
    mut environment: ResMut<Environment>,
    reloadable_entites: Query<Entity, With<Reloadable>>,
) {
    let Some(request) = simulation_manager.requests.pop_front() else {
        return;
    };

    info!("handling request: {:?}", request);
    info!("requests pending: {:?}", simulation_manager.requests.len());

    match request {
        Request::Load(_) | Request::Reload => {
            let is_paused = virtual_time.is_paused();

            // let new_virtual_clock = Time::<Virtual>::default();
            let virtual_time = virtual_time.bypass_change_detection();
            *virtual_time = Time::<Virtual>::default();
            if is_paused {
                virtual_time.unpause();
            }
        }
        _ => {}
    }

    match request {
        // Request::Load(id) if simulation_loader.active.is_none() => {
        //     simulation_manager.active = Some(id.0);
        // }
        Request::Load(id)
            if simulation_manager.active.is_some_and(|active| {
                info!("active: {}, id.0 = {}", active, id.0);

                dbg!(active == id.0)
            }) =>
        {
            warn!("simulation already loaded with id: {}", id.0);
            evw_toast.send(ToastEvent::warning("simulation already loaded"));
        }
        Request::Load(id) => {
            for entity in &reloadable_entites {
                // commands.entity(entity).despawn_recursive();
                commands.entity(entity).despawn();
            }
            simulation_manager.active = Some(id.0);
            // load config
            *config = simulation_manager.simulations[id.0].config.clone();
            *environment = simulation_manager.simulations[id.0].environment.clone();

            evw_load_simulation.send(LoadSimulation(id));
            info!("sent load simulation event with id: {}", id.0);
            simulation_manager.simulations_loaded += 1;
            let simulation_name = &simulation_manager.names[id.0];

            evw_toast.send(ToastEvent {
                caption: format!("simulation loaded: {}", simulation_name),
                options: ToastOptions {
                    level: ToastLevel::Success,
                    show_progress_bar: false,
                    duration: Some(Duration::from_secs(1)),
                    ..Default::default()
                },
            });
            // evw_toast.send(ToastEvent::info(format!(
            //     "simulation loaded: {}",
            //     simulation_name
            // )));
        }
        Request::Reload => match simulation_manager.active {
            Some(index) => {
                for entity in &reloadable_entites {
                    // commands.entity(entity).despawn_recursive();
                    commands.entity(entity).despawn();
                }
                evw_reload_simulation.send(ReloadSimulation(SimulationId(index)));
                info!("sent reload simulation event with id: {}", index);
                simulation_manager.simulations_loaded += 1;
                evw_toast.send(ToastEvent {
                    caption: "simulation reloaded".into(),
                    options: ToastOptions {
                        level: ToastLevel::Success,
                        show_progress_bar: false,
                        duration: Some(Duration::from_secs(1)),
                        ..Default::default()
                    },
                });
                // evw_toast.send(ToastEvent::info("reloaded simulation"));
            }
            None => {
                error!("no active simulation, cannot reload");
            }
        },
        Request::End => match simulation_manager.active {
            Some(index) => {
                simulation_manager.active = None;
                evw_end_simulation.send(EndSimulation(SimulationId(index)));
                info!("sent end simulation event with id: {}", index);
            }
            None => {
                error!("no active simulation to end");
            }
        },
    }
}
