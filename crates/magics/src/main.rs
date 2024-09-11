#![feature(iter_repeat_n)]
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

// #[cfg(feature = "dhat-heap")]
// #[global_allocator]
// static ALLOC: dhat::Alloc = dhat::Alloc;

// #[cfg(not(feature = "dhat-heap"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{path::Path, time::Duration};

use bevy::{
    asset::AssetMetaCheck,
    input::common_conditions::input_just_pressed,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        RenderPlugin,
    },
    time::common_conditions::once_after_real_delay,
    window::{PrimaryWindow, WindowMode, WindowResolution},
};
use bevy_image_export::{
    ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource,
};
use colored::Colorize;
// use iyes_perf_ui::prelude::*;

// use rand::{Rng, SeedableRng};
use environment::MainCamera;
// use iyes_perf_ui::prelude::*;
use gbp_config::{read_config, Config, FormationGroup};
// use config::{environment::EnvironmentType, Environment};
use gbp_environment::{Environment, EnvironmentType};
use magics::AppState;

use crate::cli::DumpDefault;

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
                let default = gbp_config::Config::default();
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
                let default = gbp_config::FormationGroup::default();
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
        let scenario_dir = Path::new("./config/scenarios");
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

    if cfg!(target_arch = "wasm32") {
        app.insert_resource(AssetMetaCheck::Never); // needed for wasm build to
                                                    // work
    }

    let image_plugin = ImagePlugin::default_nearest();

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
    // let hz = 60.0;
    // app.insert_resource(Time::<Fixed>::from_hz(hz))

    // let default_plugins = if cli.headless {
    //    DefaultPlugins.set(image_plugin)
    //} else {
    //    DefaultPlugins.set(window_plugin).set(image_plugin)
    //};

    let export_plugin = ImageExportPlugin::default();
    let export_threads = export_plugin.threads.clone();

    app
        //.add_plugins(default_plugins)
        // bevy builtin plugins
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            .set(image_plugin)
            .set(RenderPlugin {
                                    synchronous_pipeline_compilation: true,
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
        .add_systems(Update, draw_coordinate_system.run_if(input_just_pressed(KeyCode::F1)))
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

    if cli.record {
        app.add_plugins(export_plugin);
        app.add_systems(
            Update,
            setup_image_export.run_if(once_after_real_delay(Duration::from_secs(1))),
        );
    }

    app.run();

    if cli.record {
        // This line is optional but recommended.
        // It blocks the main thread until all image files have been saved successfully.
        export_threads.finish();

        // std::process::Command::new("ffmpeg")
        //     .arg()
    }

    Ok(())
}

fn setup_image_export(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    main_camera: Query<(Entity, &Camera3d), With<MainCamera>>,
) {
    let (width, height) = {
        let primary_window = primary_window.get_single().unwrap();
        let width = primary_window.resolution.width();
        let height = primary_window.resolution.height();
        (width as u32, height as u32)
    };

    info!("image_export: width={width} height={height}");

    // Create an output texture.
    let output_texture_handle = {
        let size = Extent3d {
            width,
            height,
            // width: 900,
            // height: 900,
            ..default()
        };
        let mut export_texture = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };
        export_texture.resize(size);

        images.add(export_texture)
    };

    let main_camera = main_camera.get_single().unwrap().0;

    commands.entity(main_camera).with_children(|parent| {
        parent.spawn(Camera3dBundle {
            camera: Camera {
                target: RenderTarget::Image(output_texture_handle.clone()),
                ..default()
            },
            ..default()
        });
    });

    // commands
    //     .spawn(Camera3dBundle {
    //         transform: Transform::from_translation(5.0 * Vec3::Z),
    //         ..default()
    //     })
    //     .with_children(|parent| {
    //         parent.spawn(Camera3dBundle {
    //             camera: Camera {
    //                 // Connect the output texture to a camera as a RenderTarget.
    //                 target: RenderTarget::Image(output_texture_handle.clone()),
    //                 ..default()
    //             },
    //             ..default()
    //         });
    //     });

    // Spawn the ImageExportBundle to initiate the export of the output texture.
    commands.spawn(ImageExportBundle {
        source:   export_sources.add(output_texture_handle),
        settings: ImageExportSettings {
            // Frames will be saved to "./out/[#####].png".
            output_dir: "out".into(),
            // Choose "exr" for HDR renders.
            extension:  "png".into(),
        },
    });
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
