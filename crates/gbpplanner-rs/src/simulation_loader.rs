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
use gbp_config::{Config, FormationGroup};
use gbp_environment::Environment;
use smol_str::SmolStr;

/// Which simulation to load initially
#[derive(Debug, Default)]
pub enum InitialSimulation {
    /// Use the first simulation found in the `config/simulations` folder
    /// Ordered lexiographically
    #[default]
    FirstFoundInFolder,
    /// Use the simulation with the given name
    Name(String),
}

#[derive(Debug)]
pub struct SimulationLoaderPlugin {
    // pub simulations_dir: std::path::PathBuf,
    pub show_toasts: bool,
    pub initial_simulation: InitialSimulation,
    pub reload_after: Option<Duration>,
}

impl Default for SimulationLoaderPlugin {
    fn default() -> Self {
        Self {
            show_toasts: true,
            initial_simulation: InitialSimulation::FirstFoundInFolder,
            reload_after: None,
        }
    }
}

impl SimulationLoaderPlugin {
    pub fn reload_after(mut self, duration: Duration) -> Self {
        self.reload_after = Some(duration);
        self
    }
}

pub type SdfImage = image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;
pub type RawImage = image::ImageBuffer<image::Rgb<u8>, Vec<u8>>;

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct Sdf(pub SdfImage);

#[derive(Debug, Clone, Resource, Deref, DerefMut)]
pub struct Raw(pub RawImage);

// #[derive(Debug)]
// pub struct Simulations(HashMap<String, Simulation>);
// #[derive(Resource)]
// struct Simulations(BTreeMap<String, Simulation>);
type Simulations = BTreeMap<String, Simulation>;

const SIMULATIONS_DIR: &'static str = "./config/simulations";

impl SimulationLoaderPlugin {
    pub fn new(show_toasts: bool, initial_simulation: Option<String>) -> Self {
        Self {
            show_toasts,
            initial_simulation: initial_simulation
                .map_or(InitialSimulation::FirstFoundInFolder, |name| {
                    InitialSimulation::Name(name)
                }),
            reload_after: Some(Duration::from_secs(80)), // for experiments purposes to run
            // overnight

            //..Default::default()
        }
    }

    // fn reload_after_system( &self,
    //    time: Res<Time<Fixed>>,
    //    mut evw_reload_simulation: EventWriter<ReloadSimulation>,
    //    simulation_manager: Res<SimulationManager>,
    //) {
    //    if self
    //        .reload_after
    //        .as_ref()
    //        .is_some_and(|after| time.elapsed() >= *after)
    //    {
    //        if let Some(id) = simulation_manager.active_id() {
    //            evw_reload_simulation.send(ReloadSimulation(id));
    //        };
    //    }
    //}
}

fn reload_after(
    duration: Duration,
) -> impl FnMut(Res<Time<Fixed>>, EventWriter<ReloadSimulation>, ResMut<SimulationManager>) {
    move |time: Res<Time<Fixed>>,
          mut evw_reload_simulation: EventWriter<ReloadSimulation>,
          mut simulation_manager: ResMut<SimulationManager>| {
        println!("elapsed: {:?}", time.elapsed());
        if time.elapsed() >= duration {
            if let Some(id) = simulation_manager.active_id() {
                simulation_manager.reload();
                // evw_reload_simulation.send(ReloadSimulation(id));
            };
        }
    }
}

// fn elapsed_time_exceeds(duration: Duration) -> impl FnMut(Res<Time>) -> bool
// + Clone {    move |time: Res<Time>| time.elapsed() >= duration
//}

fn elapsed_virtual_time_exceeds(
    duration: Duration,
) -> impl FnMut(Res<Time<Virtual>>) -> bool + Clone {
    move |time: Res<Time<Virtual>>| time.elapsed() >= duration
}

impl Plugin for SimulationLoaderPlugin {
    fn build(&self, app: &mut App) {
        let reader =
            std::fs::read_dir(SIMULATIONS_DIR).expect("failed to read simulation directory");

        let simulations: BTreeMap<_, _> = reader
            .map(|dir| {
                let dir = dir.unwrap();
                let name = dir
                    .file_name()
                    .into_string()
                    .expect("failed to parse simulation name");
                // println!("about to load: {name:?}");
                let config_path = dir.path().join("config.toml");
                let config = Config::from_file(config_path)
                    .expect(format!("failed to load config for simulation: {name:?}").as_str());
                let environment_path = dir.path().join("environment.yaml");
                let environment = Environment::from_file(environment_path).expect(
                    format!("failed to load environment for simulation: {name:?}").as_str(),
                );
                let formation_path = dir.path().join("formation.yaml");
                let formation = FormationGroup::from_yaml_file(formation_path)
                    .expect(format!("failed to load formation for simulation: {name:?}").as_str());

                // println!("name: {name:?}");
                let sdf_image_buffer = env_to_png::env_to_sdf_image(
                    &environment,
                    env_to_png::PixelsPerTile::new(
                        environment.tiles.settings.sdf.resolution as u32,
                    ),
                    env_to_png::Percentage::new(environment.tiles.settings.sdf.expansion),
                    env_to_png::Percentage::new(environment.tiles.settings.sdf.blur),
                )
                .expect("it all just works");

                // let sdf_path = PathBuf::new()
                //     .join("crates/gbpplanner-rs/assets/imgs/obstacles")
                //     .join(format!("{}.sdf.png", config.environment_image));
                // info!("sdf_path: {sdf_path:?}");
                // let sdf_image_buffer =
                // image::io::Reader::open(sdf_path).unwrap().decode().unwrap();
                // println!(
                //     "sdf_image_buffer: {:?} channels: {:?}",
                //     sdf_image_buffer.dimensions(),
                //     sdf_image_buffer.color()
                // );

                // let raw_path = PathBuf::new()
                //     .join("crates/gbpplanner-rs/assets/imgs/obstacles")
                //     .join(format!("{}.png", config.environment_image));
                // let raw_image_buffer =
                // image::io::Reader::open(raw_path).unwrap().decode().unwrap();

                let simulation = Simulation {
                    name: name.clone(),
                    config,
                    environment,
                    formation_group: formation,
                    sdf: Sdf(sdf_image_buffer.into()),
                    // raw: Raw(raw_image_buffer.into()),
                };

                // println!("loaded: {name:?}");

                (name, simulation)
            })
            .collect();

        assert!(
            !simulations.is_empty(),
            "No simulations found in {}",
            SIMULATIONS_DIR
        );

        let initial_simulation = match &self.initial_simulation {
            InitialSimulation::FirstFoundInFolder => simulations
                .first_key_value()
                .map(|(_, v)| v)
                .expect("there is 1 or more simulations"),
            InitialSimulation::Name(name) => {
                simulations.get(name).expect("simulation with name exists")
            }
        };

        // let initial_simulation = simulations.first_key_value().map(|(_, v)|
        // v).unwrap();

        let config = initial_simulation.config.clone();
        let formation_group = initial_simulation.formation_group.clone();
        let environment = initial_simulation.environment.clone();
        let sdf = initial_simulation.sdf.clone();
        // let raw = initial_simulation.raw.clone();

        let initial_simulation_name = initial_simulation.name.clone();

        app
            .add_plugins(
                    bevy_rand::prelude::EntropyPlugin::<bevy_prng::WyRand>::default(),
            )
            // .add_systems(Startup, load_initial_simulation)
            .insert_resource(config)
            .insert_resource(formation_group)
            .insert_resource(environment)
            .insert_resource(sdf)
            // .insert_resource(raw)
            .add_event::<ReloadSimulation>()
            .add_event::<LoadSimulation>()
            .add_event::<EndSimulation>()
            .add_event::<SaveSettings>()
            .insert_resource(SimulationManager::new(simulations, Some(initial_simulation_name)))
            .add_systems(Update, handle_requests.run_if(on_real_timer(Duration::from_millis(500))))
            .add_systems(
                Update,
                (
                    reload_simulation.run_if(input_just_pressed(KeyCode::F5)),
                    load_next_simulation.run_if(input_just_pressed(KeyCode::F6)),
                    load_previous_simulation.run_if(input_just_pressed(KeyCode::F4)),
                    save_settings.run_if(on_event::<SaveSettings>()),
                )
            );

        if let Some(after) = self.reload_after {
            app.add_systems(
                FixedUpdate,
                (
                    reload_simulation
                        .run_if(on_timer(after).and_then(elapsed_virtual_time_exceeds(after))),
                    //(|time: Res<Time<Virtual>>| println!("time exceeded {:?}", time.elapsed()))
                    //    .run_if(elapsed_virtual_time_exceeds(after)),
                ),
            );
            // app.add_systems(FixedUpdate, reload_after(after));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub name: String,
    pub config: Config,
    pub environment: Environment,
    pub formation_group: FormationGroup,
    // pub sdf: Handle<Image>,
    pub sdf: Sdf,
    // pub raw: Raw,
}

#[derive(Debug, Resource)]
pub struct SimulationManager {
    // _phantom_data: PhantomData<()>,
    // simulations_dir: std::path::PathBuf,
    // names: Vec<String>,
    names: Vec<SmolStr>,
    simulations: Vec<Simulation>,
    // simulations: Simulations,
    active: Option<usize>,
    // reload_requested: Option<()>,
    requests: VecDeque<Request>,
    simulations_loaded: usize,
}

// impl FromWorld for SimulationManager {
//     fn from_world(world: &mut World) -> Self {
//         let asset_server = world.resource::<AssetServer>();
//
//         let reader =
//             std::fs::read_dir(SIMULATIONS_DIR).expect("failed to read
// simulation directory");
//
//         let simulations: BTreeMap<_, _> = reader
//             .map(|dir| {
//                 let dir = dir.unwrap();
//                 let config_path = dir.path().join("config.toml");
//                 let config = Config::from_file(config_path).unwrap();
//                 let environment_path = dir.path().join("environment.yaml");
//                 let environment =
// Environment::from_file(environment_path).unwrap();                 let
// formation_path = dir.path().join("formation.yaml");                 let
// formation = FormationGroup::from_yaml_file(formation_path).unwrap();
//
//                 let sdf_path = PathBuf::new()
//                     .join("imgs/obstacles")
//                     .join(format!("{}.sdf.png", config.environment_image));
//                 info!("sdf_path: {sdf_path:?}");
//                 let sdf = asset_server.load(sdf_path);
//
//                 let name = dir
//                     .file_name()
//                     .into_string()
//                     .expect("failed to parse simulation name");
//
//                 let simulation = Simulation {
//                     name: name.clone(),
//                     config,
//                     environment,
//                     formation_group: formation,
//                     sdf,
//                 };
//
//                 (name, simulation)
//             })
//             .collect();
//
//         assert!(
//             !simulations.is_empty(),
//             "No simulations found in {}",
//             SIMULATIONS_DIR
//         );
//
//         let names = simulations.keys().cloned().map(Into::into).collect();
//         let simulations = simulations.into_values().collect();
//
//         Self {
//             names,
//             simulations,
//             active: None,
//             requests: VecDeque::new(),
//             simulations_loaded: 0,
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Request {
    LoadInitial,
    Load(SimulationId),
    Reload,
    End,
}

impl SimulationManager {
    #[must_use]
    fn new(simulations: Simulations, initial: Option<String>) -> Self {
        let names: Vec<SmolStr> = simulations.keys().cloned().map(Into::into).collect();
        let simulations = simulations.into_values().collect();

        let initial_index = initial
            .and_then(|name| names.iter().position(|n| *n == name))
            .unwrap_or(0);
        // let initial_index = names.iter().position(|n| n == initial).unwrap_or(0);
        // let initial_index = 0;
        let requests = VecDeque::from([Request::Load(SimulationId(initial_index))]);

        let active = Some(initial_index);
        Self {
            names,
            simulations,
            active,
            // active: None,
            requests,
            simulations_loaded: 0,
        }
    }

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

    pub fn load_next(&mut self) {
        let next = self
            .active
            .map_or(0, |id| (id + 1) % self.simulations.len());
        self.load(SimulationId(next));
    }

    pub fn load_previous(&mut self) {
        let next = self.active.map_or(0, |id| {
            if id == 0 {
                self.simulations.len() - 1
            } else {
                id - 1
            }
        });
        self.load(SimulationId(next));
    }

    pub fn ids(&self) -> impl Iterator<Item = SimulationId> + '_ {
        (0..self.simulations.len()).map(SimulationId)
    }

    #[must_use]
    pub fn id_from_name(&self, name: &str) -> Option<SimulationId> {
        self.names.iter().position(|n| n == name).map(SimulationId)
    }

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

    pub fn active_formation_group(&self) -> Option<&FormationGroup> {
        let index = self.active?;
        self.simulations.get(index).map(|s| &s.formation_group)
    }

    pub fn active_config(&self) -> Option<&Config> {
        let index = self.active?;
        self.simulations.get(index).map(|s| &s.config)
    }

    pub fn active_environment(&self) -> Option<&Environment> {
        let index = self.active?;
        self.simulations.get(index).map(|s| &s.environment)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationId(usize);

#[derive(Event)]
pub struct LoadSimulation(pub SimulationId);

#[derive(Event)]
pub struct ReloadSimulation(pub SimulationId);

#[derive(Event)]
pub struct EndSimulation(pub SimulationId);

#[derive(Event)]
pub struct SaveSettings;

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

fn load_initial_simulation(
    // simulation_manager: Res<SimulationManager>,
    // mut evw_load_simulation: EventWriter<LoadSimulation>,
    world: &mut World,
) {
    let (simulation_id, config, environment, formation_group) = {
        let simulation_manager = world
            .get_resource_mut::<SimulationManager>()
            .expect("SimulationManager has been inserted");

        let simulation_index = simulation_manager.active.unwrap_or(0);
        let simulation_id = SimulationId(simulation_index);

        let Some(config) = simulation_manager.get_config_for(simulation_id) else {
            panic!("no config found for simulation id: {}", simulation_id.0);
        };

        let Some(environment) = simulation_manager.get_environment_for(simulation_id) else {
            panic!(
                "no environment found for simulation id: {}",
                simulation_id.0
            );
        };

        let Some(formation_group) = simulation_manager.get_formation_group_for(simulation_id)
        else {
            panic!(
                "no formation group found for simulation id: {}",
                simulation_id.0
            );
        };

        (
            simulation_id,
            config.clone(),
            environment.clone(),
            formation_group.clone(),
        )
    };

    world.insert_resource(environment);
    world.insert_resource(formation_group);
    world.insert_resource(config);

    let mut simulation_manager = world
        .get_resource_mut::<SimulationManager>()
        .expect("SimulationManager has been inserted");
    // simulation_manager.active = Some(0);
    simulation_manager
        .requests
        .push_back(Request::Load(simulation_id));

    // let Some(simulation_id) = simulation_manager.active_id() else {
    //     panic!("no initial simulation set to active");
    // };

    // if let Some(config) = simulation_manager.get_config_for(simulation_id) {
    //     let config = config.to_owned();
    //     world.insert_resource(config);
    // }

    // if let Some(environment) =
    // simulation_manager.get_environment_for(simulation_id) {
    //     let environment = environment.to_owned();
    //     world.insert_resource(environment);
    // }

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

#[allow(clippy::too_many_arguments)]
fn handle_requests(
    mut commands: Commands,
    mut simulation_manager: ResMut<SimulationManager>,
    mut evw_load_simulation: EventWriter<LoadSimulation>,
    mut evw_reload_simulation: EventWriter<ReloadSimulation>,
    mut evw_end_simulation: EventWriter<EndSimulation>,
    mut evw_toast: EventWriter<ToastEvent>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut time_fixed: ResMut<Time<Fixed>>,
    // time_real: Res<Time<Real>>,
    mut config: ResMut<Config>,
    // mut variable_timesteps: ResMut<VariableTimesteps>,
    mut environment: ResMut<Environment>,
    mut sdf: ResMut<Sdf>,
    // mut raw: ResMut<Raw>,
    mut rng: ResMut<bevy_rand::prelude::GlobalEntropy<bevy_prng::WyRand>>,
    reloadable_entities: Query<Entity, With<Reloadable>>,
) {
    let Some(request) = simulation_manager.requests.pop_front() else {
        return;
    };

    info!("handling request: {:?}", request);
    info!("requests pending: {:?}", simulation_manager.requests.len());

    match request {
        Request::LoadInitial => todo!(),
        // Request::Load(id) if simulation_loader.active.is_none() => {
        //     simulation_manager.active = Some(id.0);
        // }
        Request::Load(id)
            if simulation_manager
                .active
                .is_some_and(|active| active == id.0)
                && simulation_manager.simulations_loaded > 0 =>
        {
            warn!("simulation already loaded with id: {}", id.0);
            evw_toast.send(ToastEvent::warning("simulation already loaded"));
        }
        Request::Load(id) => {
            for entity in &reloadable_entities {
                // commands.entity(entity).despawn_recursive();
                commands.entity(entity).despawn();
            }
            simulation_manager.active = Some(id.0);
            // load config

            // app.insert_resource(Time::<Fixed>::from_hz(hz))
            *time_fixed = Time::<Fixed>::from_hz(config.simulation.hz);
            *config = simulation_manager.simulations[id.0].config.clone();
            // config.simulation.t0 =
            *environment = simulation_manager.simulations[id.0].environment.clone();
            *sdf = simulation_manager.simulations[id.0].sdf.clone();

            time_virtual.set_relative_speed(config.simulation.time_scale.get());
            // *raw = simulation_manager.simulations[id.0].raw.clone();
            // *variable_timesteps = VariableTimesteps::from(config.as_ref());
            let seed: [u8; 8] = config.simulation.prng_seed.to_le_bytes();
            rng.reseed(seed);

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
                for entity in &reloadable_entities {
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

                let seed: [u8; 8] = config.simulation.prng_seed.to_le_bytes();
                rng.reseed(seed);
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

    match request {
        Request::Load(_) | Request::Reload => {
            let is_paused = time_virtual.is_paused();
            // let relative_speed = time_virtual.

            let virtual_time = time_virtual.bypass_change_detection();
            *virtual_time = Time::<Virtual>::default();
            if is_paused {
                virtual_time.unpause();
            }

            virtual_time.set_relative_speed(config.simulation.time_scale.get());
        }

        _ => {}
    }
}

#[inline]
fn load_previous_simulation(mut simulation_manager: ResMut<SimulationManager>) {
    simulation_manager.load_previous();
}

#[inline]
fn load_next_simulation(mut simulation_manager: ResMut<SimulationManager>) {
    simulation_manager.load_next();
}

/// Save the current state of the `Config` resource to the config.toml of the
/// current scenario from which it was originally loaded from
fn save_settings(mut simulation_manager: ResMut<SimulationManager>, config: Res<Config>) {
    let Some(name) = simulation_manager.active_name() else {
        return;
    };

    let dir = std::path::Path::new(SIMULATIONS_DIR).join(name);

    // serialize to toml
    let toml = toml::to_string_pretty(config.as_ref()).unwrap();
    std::fs::write(dir.join("config.toml"), toml).unwrap();

    // update the simulation manager instance of the config object, such that if the
    // user loads another scenario, and then this, the current, again the changes
    // will be persisted across this application instance
    let ix = simulation_manager.active.unwrap();
    simulation_manager.simulations[ix].config = config.clone();
    info!("saved settings to: {}", dir.join("config.toml").display());
}
