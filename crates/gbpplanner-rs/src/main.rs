#![allow(warnings)]

//! The main entry point of the simulation.
pub(crate) mod asset_loader;
mod bevy_utils;
mod cli;
pub(crate) mod config;
mod diagnostic;
mod environment;
mod factorgraph;
mod input;
mod moveable_object;
mod movement;
pub(crate) mod pause_play;
mod planner;
mod robot_spawner;
mod scene;

pub(crate) mod theme;
pub(crate) mod ui;
pub(crate) mod utils;

pub(crate) mod escape_codes;
pub(crate) mod macros;

use std::path::{Path, PathBuf};

use bevy::{asset::AssetMetaCheck, log::LogPlugin, prelude::*, window::WindowMode};
use bevy_fullscreen::ToggleFullscreenPlugin;
// use bevy_dev_console::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_notify::prelude::*;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use colored::Colorize;
use config::{environment::EnvironmentType, Environment};

// use iyes_perf_ui::prelude::*;
use crate::{
    asset_loader::AssetLoaderPlugin,
    cli::DumpDefault,
    config::{read_config, Config, FormationGroup},
    environment::EnvironmentPlugin,
    input::InputPlugin,
    movement::MovementPlugin,
    planner::PlannerPlugin,
    robot_spawner::RobotSpawnerPlugin,
    scene::ScenePlugin,
    theme::ThemePlugin,
    ui::EguiInterfacePlugin,
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> anyhow::Result<()> {
    if cfg!(all(not(target_arch = "wasm32"), debug_assertions)) {
        eprintln!("installing better_panic panic hook");
        better_panic::debug_install();
    }

    let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>();

    println!(
        "{}:   {}",
        "target arch".green().bold(),
        std::env::consts::ARCH
    );
    println!(
        "{}:     {}",
        "target os".green().bold(),
        std::env::consts::OS
    );
    println!(
        "{}: {}",
        "target family".green().bold(),
        std::env::consts::FAMILY
    );

    println!("{}:          {}", "name".green().bold(), NAME);
    println!("{}:", "authors".green().bold());
    authors.iter().for_each(|&author| {
        println!(" - {}", author);
    });
    println!("{}:       {}", "version".green().bold(), VERSION);
    println!("{}:  {}", "manifest_dir".green().bold(), MANIFEST_DIR);

    let cli = cli::parse_arguments();

    if let Some(dump) = cli.dump_default {
        match dump {
            DumpDefault::Config => {
                let default = config::Config::default();
                println!("{}", toml::to_string_pretty(&default)?);
            }
            DumpDefault::Formation => {
                let default = config::FormationGroup::default();
                let config = ron::ser::PrettyConfig::new().indentor("  ".to_string());
                println!("{}", ron::ser::to_string_pretty(&default, config)?);
            }
            DumpDefault::Environment => {
                println!("{}", serde_yaml::to_string(&Environment::default())?);
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
        };
        println!("{}", serde_yaml::to_string(&env)?);

        return Ok(());
    }

    if cli.default && cli.config.is_some() {
        println!("error: --default and --config <CONFIG_FILE> are mutually exclusive");
        println!("\nFor more information, try '--help'");
        std::process::exit(2);
    }

    let (config, formation, environment): (Config, FormationGroup, Environment) = if cli.default {
        (
            Config::default(),
            FormationGroup::default(),
            Environment::default(),
        )
    } else {
        let config = read_config(cli.config.as_ref())?;
        if let Some(ref inner) = cli.config {
            println!(
                "successfully read config from: {}",
                inner.as_os_str().to_string_lossy()
            );
        }

        let formation = FormationGroup::from_file(&config.formation_group)?;
        println!(
            "successfully read formation config from: {}",
            config.formation_group
        );
        let environment = Environment::from_file(&config.environment)?;
        println!(
            "successfully read environment config from: {}",
            config.environment
        );

        (config, formation, environment)
    };

    let window_mode = if cli.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };

    // let mut rng =
    // rand_chacha::ChaCha8Rng::seed_from_u64(config.simulation.random_seed);

    println!("initial window mode: {:?}", window_mode);

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
                ..Default::default()
            }),
            ..Default::default()
        }
    };

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

    app.insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
        .insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .init_state::<AppState>()
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            // .set(log_plugin)
        )
        // third-party plugins
        .add_plugins(bevy_egui::EguiPlugin)
        // TODO: use
        .add_plugins(EntropyPlugin::<WyRand>::default())

        // our plugins
        .add_plugins((
            DefaultPickingPlugins,
            ThemePlugin,       // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin,    // Custom
            InputPlugin,       // Custom
            // // MoveableObjectPlugin, // Custom
            // // CameraPlugin,        // Custom
            // // FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            // // FactorGraphPlugin,   // Custom
            EguiInterfacePlugin, // Custom
            PlannerPlugin,
            NotifyPlugin::default()
        ))
        .add_plugins(ToggleFullscreenPlugin::default())

        .add_plugins(ScenePlugin)
        // .add_plugins(NotifyPlugin)
        //         .insert_resource(Notifications(Toasts::default()))
        // we want Bevy to measure these values for us:
        // .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        // .add_plugins(PerfUiPlugin)
        // .add_systems(Startup, spawn_perf_ui)
        // .add_systems(Update, make_window_visible)

        // .add_systems(
        //     Update,
        //     create_toast.run_if(input_just_pressed(KeyCode::KeyZ)),
        // )
        .add_systems(PostUpdate, end_simulation.run_if(time_exceeds_max_time));

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
fn time_exceeds_max_time(time: Res<Time>, config: Res<Config>) -> bool {
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

// fn spawn_perf_ui(mut commands: Commands) {
//     commands.spawn(PerfUiCompleteBundle::default());
// }

// /// Makes the window visible after a few frames has been rendered.
// /// This is a **hack** to prevent the window from flickering at startup.
// fn make_window_visible(mut window: Query<&mut Window>, frames:
// Res<FrameCount>) {     // The delay may be different for your app or system.
//     if frames.0 == 3 {
//         // At this point the gpu is ready to show the app so we can make the
// window         // visible. Alternatively, you could toggle the visibility in
// Startup.         // It will work, but it will have one white frame before it
// starts rendering         window.single_mut().visible = true;
//     }
// }

// TODO: use in app
/// Set of distinct states the application can be in.
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
    /* derive_more::IsVariant, */
)]
pub enum AppState {
    /// Start of the application where assets e.g. data in `./assets` is being
    /// loaded into memory
    #[default]
    #[display(fmt = "Loading")]
    Loading,
    // #[display(fmt = "Starting")]
    // Starting,
    /// A simulation is running in the application
    #[display(fmt = "Running")]
    Running,
    // #[display(fmt = "Paused")]
    // Paused,
    // #[display(fmt = "Finished")]
    // Finished,
}

// fn create_toast(mut toast_event: EventWriter<ToastEvent>, mut n:
// Local<usize>) {     *n += 1;

//     toast_event.send(ToastEvent {
//         caption: format!("call: {}", *n),
//         // caption: "hello".into(),
//         options: ToastOptions {
//             level: ToastLevel::Success,
//             // closable: false,
//             // show_progress_bar: false,
//             ..Default::default()
//         },
//     });
// }

// #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
// enum DebugState {
//     #[default]
//     Disabled,
//     Enabled,
// }
