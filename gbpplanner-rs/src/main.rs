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

use std::path::PathBuf;

use bevy::{
    core::FrameCount,
    prelude::*,
    window::{WindowMode, WindowTheme},
};
use clap::Parser;
use config::{environment::EnvironmentType, Environment};

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

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    /// Specify the configuration file to use, overrides the normal
    /// configuration file resolution
    #[arg(short, long, value_name = "CONFIG_FILE")]
    config: Option<std::path::PathBuf>,

    /// Dump the default config to stdout
    #[arg(long)]
    dump_default_config:      bool,
    /// Dump the default formation config to stdout
    #[arg(long)]
    dump_default_formation:   bool,
    /// Dump the default environment config to stdout
    #[arg(long)]
    dump_default_environment: bool,

    /// The environment to dump
    #[arg(short, long, value_name = "ENVIRONMENT_TYPE")]
    dump_environment: Option<EnvironmentType>,

    /// Run the app without a window for rendering the environment
    #[arg(long)]
    headless: bool,

    /// Start the app in fullscreen mode
    #[arg(short, long)]
    fullscreen: bool,
}

// fn read_config(cli: &Cli) -> color_eyre::eyre::Result<Config> {
fn read_config(cli: &Cli) -> anyhow::Result<Config> {
    if let Some(config_path) = &cli.config {
        Ok(Config::from_file(config_path)?)
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
                return Ok(Config::from_file(&conf_path.to_path_buf())?);
            }
        }

        anyhow::bail!("No config file found");
        // Err(color_eyre::eyre::eyre!("No config file found"))
    }
}

// fn main() -> color_eyre::eyre::Result<()> {
//     color_eyre::install()?;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // if cli.dump_default_config && cli.dump_default_formation {
    //     eprintln!(
    //         "you can not set --dump-default-config and --dump-default-formation
    // at the same time!"     );
    //     std::process::exit(2);
    // }

    // Cannot set more than one of --dump-default-config, --dump-default-formation,
    // --dump-default-environment at one time
    if (cli.dump_default_config as u8
        + cli.dump_default_formation as u8
        + cli.dump_default_environment as u8)
        > 1
    {
        eprintln!(
            "you can not set more than one of --dump-default-config, --dump-default-formation, \
             --dump-default-environment at one time!"
        );
        std::process::exit(2);
    }

    if cli.dump_default_environment {
        let default_environment = Environment::default();
        // Write default config to stdout
        println!("{}", toml::to_string_pretty(&default_environment)?);

        return Ok(());
    }

    if cli.dump_default_formation {
        let default_formation = config::FormationGroup::default();
        // Write default config to stdout\
        println!(
            "{}",
            ron::ser::to_string_pretty(
                &default_formation,
                ron::ser::PrettyConfig::new().indentor("  ".to_string())
            )?
        );

        return Ok(());
    }

    if cli.dump_default_config {
        let default_config = config::Config::default();
        // Write default config to stdout
        println!("{}", toml::to_string_pretty(&default_config)?);
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
        // println!("{}", toml::to_string_pretty(&env)?);
        // println!(
        //     "{}",
        //     ron::ser::to_string_pretty(
        //         &env,
        //         ron::ser::PrettyConfig::new().indentor("  ".to_string())
        //     )?
        // );

        return Ok(());
    }

    let config = read_config(&cli)?;

    // formation
    let formation_file_path = PathBuf::from(&config.formation_group.clone());
    let formation = FormationGroup::from_file(&formation_file_path)?;

    // environment
    let environment_file_path = PathBuf::from(&config.environment.clone());
    println!("environment_file_path: {:?}", environment_file_path);
    let environment = Environment::from_file(&environment_file_path)?;
    let window_mode = if cli.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };

    println!("initial window mode: {:?}", window_mode);

    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(config.simulation.hz))
        .insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .add_plugins((
            DefaultPlugins.set(
                // **Bevy**
                WindowPlugin {
                    primary_window: Some(Window {
                        title: "GBP Planner".into(),
                        // resolution: (1280.0, 720.0).into(),
                        // mode: WindowMode::BorderlessFullscreen,
                        mode: window_mode,
                        // mode: WindowMode::Fullscreen,
                        // present_mode: PresentMode::AutoVsync,
                        // fit_canvas_to_parent: true,
                        // prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        // enable_buttons: bevy::window::EnableButtons {
                        //     maximize: false,
                        //     ..Default::default()
                        // },
                        visible: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ),
            // FpsCounterPlugin,  // **Bevy**
            ThemePlugin,       // Custom
            AssetLoaderPlugin, // Custom
            EnvironmentPlugin, // Custom
            MovementPlugin,    // Custom
            InputPlugin,       // Custom
            ToggleFullscreenPlugin,
            MoveableObjectPlugin, // Custom
            // CameraPlugin,        // Custom
            // FollowCamerasPlugin, // Custom
            RobotSpawnerPlugin, // Custom
            // FactorGraphPlugin,   // Custom
            EguiInterfacePlugin, // Custom
            PlannerPlugin,       /* Custom
                                  * WorldInspectorPlugin::new(), */
        ))
        .add_systems(Update, make_visible);

    // eprintln!("{:#?}", app);

    app.run();

    Ok(())
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window
        // visible. Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
}
