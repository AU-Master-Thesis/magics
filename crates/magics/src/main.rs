//! The main entry point of the simulation.
pub(crate) mod asset_loader;
mod bevy_utils;
pub mod cli;
pub mod despawn_entity_after;
mod diagnostic;
mod environment;
mod factorgraph;
pub mod goal_area;
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

mod api;

// #[cfg(feature = "dhat-heap")]
// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;

// #[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{path::Path, time::Duration};

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        settings::{Backends, PowerPreference, WgpuSettings},
        RenderPlugin,
    },
    time::common_conditions::once_after_real_delay,
    window::{PrimaryWindow, WindowMode, WindowResolution},
};
use colored::Colorize;

// use rand::{Rng, SeedableRng};
use environment::MainCamera;
use gbp_config::{read_config, Config, FormationGroup};
// use config::{environment::EnvironmentType, Environment};
use gbp_environment::{Environment, EnvironmentType};
use magics::AppState;

use crate::cli::DumpDefault;


#[allow(clippy::too_many_lines)]
fn main() -> anyhow::Result<()> {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

    let cli = cli::parse_arguments();

    if let Some(dump) = cli.dump_default {
        let stdout_is_a_terminal = atty::is(atty::Stream::Stdout);
        match dump {
            DumpDefault::Config => {
                let default = gbp_config::Config::default();
                println!("{}", toml::to_string_pretty(&default)?);
            }
            DumpDefault::Formation => {
                let default = gbp_config::FormationGroup::default();
                let yaml = serde_yaml::to_string(&default)?;
                println!("{yaml}");
            }
            DumpDefault::Environment => {
                let yaml = serde_yaml::to_string(&Environment::default())?;
                println!("{yaml}");
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
        println!("{yaml}");

        return Ok(());
    }

    if cli.list_scenarios {
        let scenario_dir = Path::new("./config/scenarios");
        assert!(scenario_dir.exists());
        let mut directories = Vec::new();
        let entries = scenario_dir.read_dir()?;
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

    if let Some(ref working_dir) = cli.working_dir {
        std::env::set_current_dir(working_dir).expect("the given --working-dir exists");
        eprintln!("changed working_dir to: {:?}", working_dir);
    }

    let window_mode = if cli.fullscreen {
        WindowMode::BorderlessFullscreen
    } else {
        WindowMode::Windowed
    };

    eprintln!("initial window mode: {:?}", window_mode);

    let window_plugin = {
        let default_window_resolution = WindowResolution::default();
        let width = cli
            .width
            .unwrap_or(default_window_resolution.physical_width());
        let height = cli
            .height
            .unwrap_or(default_window_resolution.physical_height());

        WindowPlugin {
            primary_window: Some(Window {
                name: Some(NAME.to_string()),
                focused: true,
                mode: window_mode,
                window_theme: None,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                visible: true,
                resizable: !cli.record,
                resolution: WindowResolution::new(width as f32, height as f32)
                    .with_scale_factor_override(1.0),

                // physical_width: 1280,
                // physical_height: 720,
                // resolution: WindowResolution::default().with_scale_factor_override(1.0),
                ..Default::default()
            }),

            ..Default::default()
        }
    };

    let verbosity = cli.verbosity();
    eprintln!("verbosity level: {:?}", verbosity);

    // bevy app
    let mut app = App::new();

    let image_plugin = ImagePlugin::default_nearest();

    app
        //.add_plugins(default_plugins)
        // bevy builtin plugins
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            .set(image_plugin)
            .set(RenderPlugin {
                                    synchronous_pipeline_compilation: true,
                                    render_creation: WgpuSettings {
                                        backends: Some(Backends::VULKAN),
                                        device_label: Some("NVIDIA".into()), // <-- Explicit device label
                                        ..Default::default()
                                    }
                                    .into(),
                                    ..default()
            })
        )
        // third-party plugins
        .add_plugins((
            bevy_egui::EguiPlugin,
            bevy_mod_picking::DefaultPickingPlugins,
        ))

        // our plugins
        .add_plugins((
            // simulation_loader::SimulationLoaderPlugin::default(),
            despawn_entity_after::DespawnEntityAfterPlugin,
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
            bevy_fullscreen::ToggleFullscreenPlugin::default(),
            goal_area::GoalAreaPlugin,
        ))
        // Only add the API plugin when the "api" feature is enabled
        .add_systems(Update, draw_coordinate_system.run_if(input_just_pressed(KeyCode::F1)))
        .add_systems(PostUpdate, end_simulation.run_if(virtual_time_exceeds_max_time));

    // Only add the API plugin when the "api" feature is enabled
    #[cfg(feature = "api")]
    app.add_plugins(api::ApiPlugin::default());

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

    if cli.record {
        app.add_plugins(export_plugin);
        app.add_systems(
            Update,
            setup_image_export.run_if(once_after_real_delay(Duration::from_secs(1))),
        );
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
fn end_simulation(config: Res<Config>, mut evw_app_exit: EventWriter<bevy::app::AppExit>) {
    println!(
        "ending simulation, reason: time elapsed exceeds configured max time: {} seconds",
        config.simulation.max_time.get()
    );

    evw_app_exit.send(bevy::app::AppExit);
    // std::process::exit(0);
}

fn draw_coordinate_system(mut gizmos: Gizmos, mut enabled: Local<bool>) {
    if *enabled {
        let length = 100.0;
        // gizmos.arrow(Vec3::ZERO, Vec3::new(1.0 * length, 0., 0.), Color::RED);
        // gizmos.arrow(Vec3::ZERO, Vec3::new(0.0, 1.0 * length, 0.), Color::GREEN);
        // gizmos.arrow(Vec3::ZERO, Vec3::new(0., 0., 1.0 * length), Color::BLUE);

        gizmos.line(Vec3::ZERO, Vec3::new(1.0 * length, 0., 0.), Color::RED);
        gizmos.line(Vec3::ZERO, Vec3::new(0.0, 1.0 * length, 0.), Color::GREEN);
        gizmos.line(Vec3::ZERO, Vec3::new(0., 0., 1.0 * length), Color::BLUE);
    }

    *enabled = !*enabled;
}
