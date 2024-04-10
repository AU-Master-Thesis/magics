//! The main entry point of the simulation.
pub(crate) mod asset_loader;
pub(crate) mod config;
mod diagnostic;
mod environment;
mod input;
mod moveable_object;
mod movement;
mod planner;
mod robot_spawner;

pub(crate) mod theme;
mod toggle_fullscreen;
pub(crate) mod ui;
pub(crate) mod utils;

pub(crate) mod escape_codes;
pub(crate) mod macros;

use std::path::PathBuf;

use bevy::{
    asset::AssetMetaCheck, core::FrameCount, input::common_conditions::input_just_pressed,
    prelude::*, window::WindowMode,
};
// use bevy_dev_console::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_notify::prelude::*;
// use rand_core::RngCore;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use clap::Parser;
use config::Environment;
use iyes_perf_ui::prelude::*;
// use rand::{Rng, SeedableRng};

use crate::{
    asset_loader::AssetLoaderPlugin,
    config::{Config, FormationGroup},
    environment::EnvironmentPlugin,
    input::InputPlugin,
    // moveable_object::MoveableObjectPlugin,
    movement::MovementPlugin,
    planner::PlannerPlugin,
    robot_spawner::RobotSpawnerPlugin,
    theme::ThemePlugin,
    toggle_fullscreen::ToggleFullscreenPlugin,
    ui::EguiInterfacePlugin,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
enum DumpDefault {
    /// Dump the default config to stdout
    Config,
    /// Dump the default formation config to stdout
    Formation,
    /// Dump the default environment config to stdout
    Environment,
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    /// Specify the configuration file to use, overrides the normal
    /// configuration file resolution
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<std::path::PathBuf>,

    /// What default configuration information to optionally dump to stdout
    #[arg(long, value_enum)]
    dump_default: Option<DumpDefault>,

    /// Run the app without a window for rendering the environment
    #[arg(long, group = "display")]
    headless: bool,

    /// Start the app in fullscreen mode
    #[arg(short, long, group = "display")]
    fullscreen: bool,

    /// Enable debug plugins
    #[arg(short, long)]
    debug: bool,

    /// use default values for all configuration, simulation and environment settings
    #[arg(long)]
    default: bool,
}

// fn read_config(cli: &Cli) -> color_eyre::eyre::Result<Config> {
fn read_config<P>(path: Option<P>) -> anyhow::Result<Config>
where
    P: AsRef<std::path::Path>,
{
    if let Some(path) = path {
        Ok(Config::from_file(path)?)
    } else {
        let mut conf_paths = Vec::<PathBuf>::new();

        if let Ok(home) = std::env::var("HOME") {
            let xdg_config_home = std::path::Path::new(&home).join(".config");
            let user_config_dir = xdg_config_home.join("gbpplanner");

            conf_paths.push(user_config_dir.join("config.toml"));
        }

        let cwd = std::env::current_dir()?;

        conf_paths.push(cwd.join("config/config.toml"));

        for conf_path in conf_paths {
            if conf_path.exists() {
                return Ok(Config::from_file(&conf_path)?);
            }
        }

        anyhow::bail!("No config file found")
    }
}

// #[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
// enum DebugState {
//     #[default]
//     Disabled,
//     Enabled,
// }

#[cfg(not(target_arch = "wasm32"))]
fn parse_arguments() -> Cli {
    eprintln!("parsing arguments not on wasm32");
    Cli::parse()
}

#[cfg(target_arch = "wasm32")]
fn parse_arguments() -> Cli {
    eprintln!("parsing arguments on wasm32");
    let mut cli = Cli::parse();
    cli.default = true;
    cli
}

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> anyhow::Result<()> {
    if cfg!(all(not(target_arch = "wasm32"), debug_assertions)) {
        println!("installing better_panic panic hook");
        better_panic::debug_install();
    }

    // if cfg!(target_os = "linux") {}
    // if cfg!(wasm32-unknown-unknown) {}
    // if cfg!(linux) {}

    // if cfg!(windows) {
    //     compile_error!("compiling on wasm32");
    // }

    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>();

    println!("target arch:   {}", std::env::consts::ARCH);
    println!("target os:     {}", std::env::consts::OS);
    println!("target family: {}", std::env::consts::FAMILY);

    println!("name:         {}", NAME);
    println!("authors:");
    authors.iter().for_each(|&author| {
        println!(" - {}", author);
    });
    println!("version:      {}", VERSION);
    println!("manifest_dir: {}", MANIFEST_DIR);

    // let cli  = parse_arguments();

    let cli = if cfg!(not(target_arch = "wasm32")) {
        Cli::parse()
    } else {
        let mut cli = Cli::parse();
        cli.default = true;
        cli
    };

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
                println!("{}", toml::to_string_pretty(&Environment::default())?);
            }
        };

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
        let config = read_config(cli.config)?;
        // if let Some(ref inner) = cli.config {
        //     println!(
        //         "successfully read config from: {}",
        //         inner.as_os_str().to_string_lossy()
        //     );
        // }

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
        app.insert_resource(AssetMetaCheck::Never); // needed for wasm build to work
    }

    app.insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
        .insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .init_state::<SimulationState>()
        .add_plugins(DefaultPlugins.set(window_plugin))
        // third-party plugins
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(EntropyPlugin::<WyRand>::default())

        // our plugins
        .add_plugins((
            DefaultPickingPlugins,
            ThemePlugin,       // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin,    // Custom
            InputPlugin,       // Custom
            ToggleFullscreenPlugin,
            // // MoveableObjectPlugin, // Custom
            // // CameraPlugin,        // Custom
            // // FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            // // FactorGraphPlugin,   // Custom
            EguiInterfacePlugin, // Custom
            PlannerPlugin,
            NotifyPlugin::default()
        ))
        // .add_plugins(NotifyPlugin)
        //         .insert_resource(Notifications(Toasts::default()))
        // we want Bevy to measure these values for us:
        // .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        // .add_plugins(PerfUiPlugin)
        // .add_systems(Startup, spawn_perf_ui)
        // .add_systems(Update, make_window_visible)

        .add_systems(
            Update,
            create_toast.run_if(input_just_pressed(KeyCode::KeyZ)),
        )
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

fn spawn_perf_ui(mut commands: Commands) {
    commands.spawn(PerfUiCompleteBundle::default());
}

/// Makes the window visible after a few frames has been rendered.
/// This is a **hack** to prevent the window from flickering at startup.
fn make_window_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window
        // visible. Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
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

fn create_toast(mut toast_event: EventWriter<ToastEvent>, mut n: Local<usize>) {
    *n += 1;

    toast_event.send(ToastEvent {
        caption: format!("call: {}", *n),
        // caption: "hello".into(),
        options: ToastOptions {
            level: ToastLevel::Success,
            // closable: false,
            // show_progress_bar: false,
            ..Default::default()
        },
    });
}
