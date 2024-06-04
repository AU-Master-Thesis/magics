use bevy::prelude::*;

fn main() -> anyhow::Result<()> {
    let image_plugin = ImagePlugin::default_nearest();
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            window_theme: None,
            visible: true,
            canvas: Some("#bevy".to_string()),
            // Tells wasm not to override default event handling, like F5 and Ctrl+R
            prevent_default_event_handling: false,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = App::new();

    app.insert_resource(AssetMetaCheck::Never) // needed for wasm build to
        // bevy builtin plugins
        .add_plugins(DefaultPlugins
            .set(window_plugin)
            .set(image_plugin)
        )
        // third-party plugins
        .add_plugins((
            bevy_egui::EguiPlugin,
            bevy_mod_picking::DefaultPickingPlugins,
        ))

        // our plugins
        .add_plugins((
            simulation_loader::SimulationLoaderPlugin::default(),
            despawn_entity_after::DespawnEntityAfterPlugin,
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
        // .add_systems(Update, draw_coordinate_system.run_if(input_just_pressed(KeyCode::F1)))
        .add_systems(PostUpdate, end_simulation.run_if(virtual_time_exceeds_max_time));

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
