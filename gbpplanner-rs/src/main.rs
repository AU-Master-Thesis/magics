//! The main entry point of the simulation.
pub(crate) mod asset_loader;
pub(crate) mod config;
mod environment;
mod factorgraph;
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
    core::FrameCount,
    log::LogPlugin,
    prelude::*,
    window::{WindowMode, WindowTheme},
};
use bevy_dev_console::prelude::*;
use bevy_mod_picking::DefaultPickingPlugins;
// use rand_core::RngCore;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use clap::Parser;
use config::Environment;
use iyes_perf_ui::prelude::*;
use rand::{Rng, SeedableRng};

use crate::{
    asset_loader::AssetLoaderPlugin,
    config::{Config, FormationGroup},
    environment::EnvironmentPlugin,
    input::InputPlugin,
    moveable_object::MoveableObjectPlugin,
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

    #[arg(long, group = "display")]
    /// Run the app without a window for rendering the environment
    headless: bool,

    #[arg(short, long, group = "display")]
    /// Start the app in fullscreen mode
    fullscreen: bool,

    #[arg(short, long)]
    /// Enable debug plugins
    debug: bool,

    #[arg(long)]
    /// muda, muda, muda!
    za_warudo: bool,
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

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum DebugState {
    #[default]
    Disabled,
    Enabled,
}

fn main() -> anyhow::Result<()> {
    better_panic::debug_install();

    let cli = Cli::parse();

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

    let window_mode = if cli.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };

    // let mut rng =
    // rand_chacha::ChaCha8Rng::seed_from_u64(config.simulation.random_seed);

    println!("initial window mode: {:?}", window_mode);

    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
        .insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .insert_state(if cli.debug {DebugState::Enabled} else {DebugState::Disabled})
        .add_plugins(EntropyPlugin::<WyRand>::default())
        .add_plugins((
            // ConsoleLogPlugin::default(),

            DefaultPlugins.set(
                // **Bevy**
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "GBP Planner".into(),
                        mode: window_mode,
                        // present_mode: PresentMode::AutoVsync,
                        // fit_canvas_to_parent: true,
                        // prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        visible: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ),
                // ).disable::<LogPlugin>(),
            // DevConsolePlugin,
            DefaultPickingPlugins,
            // FpsCounterPlugin,  // **Bevy**
            ThemePlugin,       // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin,    // Custom
            InputPlugin,       // Custom
            ToggleFullscreenPlugin,
            // MoveableObjectPlugin, // Custom
            // CameraPlugin,        // Custom
            // FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            // FactorGraphPlugin,   // Custom
            EguiInterfacePlugin, // Custom
            PlannerPlugin,       /* Custom
                                  * WorldInspectorPlugin::new(), */

        ))
        // we want Bevy to measure these values for us:
        // .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        // .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        // .add_plugins(PerfUiPlugin)
        // .add_systems(Startup, spawn_perf_ui)
        .add_systems(Update, make_window_visible).add_systems(PostUpdate, end_simulation.run_if(time_exceeds_max_time));

    // eprintln!("{:#?}", app);

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
fn time_exceeds_max_time(time: Res<Time>, config: Res<Config>) -> bool {
    time.elapsed_seconds() > config.simulation.max_time.get()
}

/// Ends the simulation.
fn end_simulation(config: Res<Config>) {
    println!(
        "ending simulation, reason: time elapsed exceeds configured max time: {} seconds",
        config.simulation.max_time.get()
    );
    std::process::exit(0);
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
