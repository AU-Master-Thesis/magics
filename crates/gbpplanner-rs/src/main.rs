//! The main entry point of the simulation.
pub(crate) mod asset_loader;
mod bevy_utils;
pub mod cli;
pub(crate) mod config;
mod diagnostic;
mod environment;
mod factorgraph;
mod input;
mod moveable_object;
mod movement;
pub(crate) mod pause_play;
// mod scene;

pub mod planner;
pub(crate) mod simulation_loader;

pub(crate) mod theme;
pub(crate) mod ui;
pub(crate) mod utils;

pub mod export;

pub(crate) mod escape_codes;
pub(crate) mod macros;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

use std::path::Path;

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use bevy_fullscreen::ToggleFullscreenPlugin;
// use bevy_dev_console::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_notify::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use colored::Colorize;
// use config::{environment::EnvironmentType, Environment};
use gbp_environment::{Environment, EnvironmentType};
use gbpplanner_rs::AppState;
use itertools::Itertools;

// use iyes_perf_ui::prelude::*;

// use rand::{Rng, SeedableRng};

// use iyes_perf_ui::prelude::*;
use crate::{
    asset_loader::AssetLoaderPlugin,
    cli::DumpDefault,
    config::{read_config, Config, FormationGroup},
    environment::EnvironmentPlugin,
    input::InputPlugin,
    movement::MovementPlugin,
    pause_play::PausePlayPlugin,
    planner::PlannerPlugin,
    simulation_loader::SimulationLoaderPlugin,
    theme::ThemePlugin,
    ui::EguiInterfacePlugin,
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[allow(clippy::too_many_lines)]
fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    // if cfg!(all(not(target_arch = "wasm32"), debug_assertions)) {
    if cfg!(not(target_arch = "wasm32")) {
        if cfg!(debug_assertions) {
            better_panic::debug_install();
        } else {
            better_panic::install();
        }
    }

    let cli = cli::parse_arguments();

    if cli.metadata {
        let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>();

        eprintln!(
            "{}:   {}",
            "target arch".green().bold(),
            std::env::consts::ARCH
        );
        eprintln!(
            "{}:     {}",
            "target os".green().bold(),
            std::env::consts::OS
        );
        eprintln!(
            "{}: {}",
            "target family".green().bold(),
            std::env::consts::FAMILY
        );

        eprintln!("{}:          {}", "name".green().bold(), NAME);
        eprintln!("{}:", "authors".green().bold());
        for &author in &authors {
            eprintln!(" - {}", author);
        }
        eprintln!("{}:       {}", "version".green().bold(), VERSION);
        eprintln!("{}:  {}", "manifest_dir".green().bold(), MANIFEST_DIR);
    }

    if let Some(dump) = cli.dump_default {
        let stdout_is_a_terminal = atty::is(atty::Stream::Stdout);
        match dump {
            DumpDefault::Config => {
                let default = config::Config::default();
                if stdout_is_a_terminal {
                    let toml = toml::to_string_pretty(&default)?;
                    bat::PrettyPrinter::new()
                        .input_from_bytes(toml.as_bytes())
                        .language("toml")
                        .print()
                        .unwrap();
                } else {
                    // let stdout = std::io::stdout::lock();
                    println!("{}", toml::to_string_pretty(&default)?);
                }
            }
            DumpDefault::Formation => {
                let default = config::FormationGroup::default();
                let config = ron::ser::PrettyConfig::new().indentor("  ".to_string());

                let yaml = serde_yaml::to_string(&default)?;
                // println!("{ron}");
                if stdout_is_a_terminal {
                    bat::PrettyPrinter::new()
                        .input_from_bytes(yaml.as_bytes())
                        .language("rust")
                        .print()
                        .unwrap();
                } else {
                    println!("{yaml}");
                    // println!("{}", ron::ser::to_string_pretty(&default,
                    // config)?);
                }

                // let ron = ron::ser::to_string_pretty(&default, config)?;
                // // println!("{ron}");
                // if stdout_is_a_terminal {
                //     bat::PrettyPrinter::new()
                //         .input_from_bytes(ron.as_bytes())
                //         .language("rust")
                //         .print()
                //         .unwrap();
                // } else {
                //     println!("{ron}");
                //     // println!("{}", ron::ser::to_string_pretty(&default,
                //     // config)?);
                // }
            }
            DumpDefault::Environment => {
                let yaml = serde_yaml::to_string(&Environment::default())?;
                if stdout_is_a_terminal {
                    bat::PrettyPrinter::new()
                        .input_from_bytes(yaml.as_bytes())
                        .language("yaml")
                        .print()
                        .unwrap();
                } else {
                    println!("{yaml}");
                    // println!("{}",
                    // serde_yaml::to_string(&Environment::default())?);
                }
            }
        };

        return Ok(());
    }
    // dump_environment
    if let Some(dump_environment) = cli.dump_environment {
        let env = match dump_environment {
            EnvironmentType::Intersection => Environment::intersection(),
            EnvironmentType::Circle => Environment::circle(),
            EnvironmentType::Intermediate => Environment::intermediate(),
            EnvironmentType::Complex => Environment::complex(),
            EnvironmentType::Maze => Environment::maze(),
            EnvironmentType::Test => Environment::test(),
        };

        let yaml = serde_yaml::to_string(&env)?;
        let stdout_is_a_terminal = atty::is(atty::Stream::Stdout);
        if stdout_is_a_terminal {
            bat::PrettyPrinter::new()
                .input_from_bytes(yaml.as_bytes())
                .language("yaml")
                .print()
                .unwrap();
        } else {
            println!("{yaml}");
            // println!("{}", serde_yaml::to_string(&env)?);
        }

        return Ok(());
    }

    if cli.list_scenarios {
        let scenario_dir = Path::new("./config/simulations");
        assert!(scenario_dir.exists());
        let mut directories = Vec::new();
        let entries = scenario_dir.read_dir()?; // .sort_by(|a, b| a.file_name().cmp(&b.file_name()));
                                                //
        for entry in entries {
            let entry = entry?.path();
            if entry.is_dir() {
                directories.push(entry.to_string_lossy().to_string());
            }
        }

        // sort directory names, to match order in simulation picker
        directories.sort();

        // Determine the maximum length of the basename for alignment
        let max_basename_length = directories
            .iter()
            .map(|s| Path::new(s).file_name().unwrap().to_string_lossy().len())
            .max()
            .unwrap_or(0);

        for name in directories {
            let basename = Path::new(&name).file_name().unwrap().to_string_lossy();
            if atty::is(atty::Stream::Stdout) {
                println!(
                    "{:width$} {}",
                    basename.green().bold(),
                    name,
                    width = max_basename_length
                );
            } else {
                println!("{:width$} {}", basename, name, width = max_basename_length);
            }
        }

        return Ok(());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(ref working_dir) = cli.working_dir {
            std::env::set_current_dir(working_dir).expect("the given --working-dir exists");
            eprintln!("changed working_dir to: {:?}", working_dir);
        }
        eprintln!(
            "current working dir: {:?}",
            std::env::current_dir().expect("current working dir exists")
        );
    }

    // let (config, formation, environment): (Config, FormationGroup, Environment) =
    // if cli.default {     (
    //         Config::default(),
    //         FormationGroup::default(),
    //         Environment::default(),
    //     )
    // } else {
    //     let config = read_config(cli.config.as_ref())?;
    //     if let Some(ref inner) = cli.config {
    //         println!(
    //             "successfully read config from: {}",
    //             inner.as_os_str().to_string_lossy()
    //         );
    //     }

    //     let formation = FormationGroup::from_ron_file(&config.formation_group)?;
    //     println!(
    //         "successfully read formation config from: {}",
    //         config.formation_group
    //     );
    //     let environment = Environment::from_file(&config.environment)?;
    //     println!(
    //         "successfully read environment config from: {}",
    //         config.environment
    //     );

    //     (config, formation, environment)
    // };

    let window_mode = if cli.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };

    // let mut rng =
    // rand_chacha::ChaCha8Rng::seed_from_u64(config.simulation.random_seed);

    eprintln!("initial window mode: {:?}", window_mode);

    let window_plugin = if cfg!(target_arch = "wasm32") {
        WindowPlugin {
            primary_window: Some(Window {
                window_theme: None,
                visible: true,
                canvas: Some("#bevy".to_string()),
                // Tells wasm not to override default event handling, like F5 and Ctrl+R
                prevent_default_event_handling: false,
                ..Default::default()
            }),
            ..Default::default()
        }
    } else {
        WindowPlugin {
            primary_window: Some(Window {
                name: Some(NAME.to_string()),
                focused: true,
                mode: window_mode,
                window_theme: None,
                visible: true,
                resolution: WindowResolution::default().with_scale_factor_override(1.0),
                ..Default::default()
            }),

            ..Default::default()
        }
    };

    let verbosity = cli.verbosity();
    eprintln!("verbosity level: {:?}", verbosity);

    // bevy app
    let mut app = App::new();

    if cfg!(target_arch = "wasm32") {
        app.insert_resource(AssetMetaCheck::Never); // needed for wasm build to
                                                    // work
    }

    // let mut default_plugins = DefaultPlugins;

    // let log_plugin = if cfg!(debug_assertions) {
    //     // dev build
    //     LogPlugin {
    //         level: bevy::log::Level::DEBUG,
    //         filter: format!("error,wgpu_core=warn,wgpu_hal=warn,{}=debug", NAME),
    //         ..default()
    //     }
    // } else {
    //     // release build
    //     LogPlugin {
    //         level: bevy::log::Level::INFO,
    //         filter: format!("error,wgpu_core=warn,wgpu_hal=warn,{}=info", NAME),
    //         ..default()
    //     }
    // };

    // TODO: load from sim loader instead
    // app.insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
    let hz = 60.0;
    app.insert_resource(Time::<Fixed>::from_hz(hz))
        // bevy builtin plugins
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            // .set(log_plugin)
        )
        // third-party plugins
        .add_plugins((
            bevy_egui::EguiPlugin,
            bevy_mod_picking::DefaultPickingPlugins,
            bevy_rand::prelude::EntropyPlugin::<bevy_prng::WyRand>::default()
        ))

        // our plugins
        .add_plugins((
            // simulation_loader::SimulationLoaderPlugin::default(),
            simulation_loader::SimulationLoaderPlugin::new(true, cli.initial_scenario.clone()),
            pause_play::PausePlayPlugin::default(),
            theme::ThemePlugin,
            asset_loader::AssetLoaderPlugin,
            environment::EnvironmentPlugin,
            movement::MovementPlugin,
            input::InputPlugin,
            ui::EguiInterfacePlugin,
            planner::PlannerPlugin,
            bevy_notify::NotifyPlugin::default(),
            export::ExportPlugin::default(),
            bevy_fullscreen::ToggleFullscreenPlugin::default()
        ))
        .add_systems(Update, draw_coordinate_system)
        .add_systems(PostUpdate, end_simulation.run_if(virtual_time_exceeds_max_time));

    if let Some(schedule) = cli.schedule_graph {
        match schedule {
            cli::BevySchedule::PreStartup => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, PreStartup);
            }
            cli::BevySchedule::Startup => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, Startup);
            }
            cli::BevySchedule::PostStartup => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, PostStartup);
            }
            cli::BevySchedule::PreUpdate => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, PreUpdate);
            }
            cli::BevySchedule::Update => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
            }
            cli::BevySchedule::PostUpdate => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, PostUpdate);
            }
            cli::BevySchedule::FixedUpdate => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, FixedUpdate);
            }
            cli::BevySchedule::Last => {
                bevy_mod_debugdump::print_schedule_graph(&mut app, Last);
            }
        }

        return Ok(());
    }

    app.run();

    Ok(())
}

/// Returns true if the time has exceeded the max configured simulation time.
///
/// # Example
/// ```toml
/// [simulation]
/// max-time = 100.0
/// ```
#[inline]
fn virtual_time_exceeds_max_time(time: Res<Time<Virtual>>, config: Res<Config>) -> bool {
    time.elapsed_seconds() > config.simulation.max_time.get()
}

/// Ends the simulation.
fn end_simulation(config: Res<Config>) {
    println!(
        "ending simulation, reason: time elapsed exceeds configured max time: {} seconds",
        config.simulation.max_time.get()
    );
    // std::process::exit(0);
}

fn draw_coordinate_system(mut gizmos: Gizmos) {
    let length = 100.0;
    gizmos.arrow(Vec3::ZERO, Vec3::new(1.0 * length, 0., 0.), Color::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, 1.0 * length, 0.), Color::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::new(0., 0., 1.0 * length), Color::BLUE);
}
