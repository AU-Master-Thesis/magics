//! example crate to take the environment config from the config file
//! and generate the necessary environment and colliders for rrt to work
use bevy::prelude::*;
use gbpplanner_rs::{
    asset_loader::AssetLoaderPlugin,
    cli,
    config::{read_config, Config, Environment, FormationGroup},
    environment::EnvironmentPlugin,
    input::{camera::CameraInputPlugin, ChangingBinding, InputPlugin},
    theme::ThemePlugin,
};

fn main() -> anyhow::Result<()> {
    better_panic::debug_install();

    let cli = cli::parse_arguments();

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

    let mut app = App::new();
    app.insert_resource(config)
        .insert_resource(formation)
        .insert_resource(environment)
        .init_resource::<ChangingBinding>()
        .add_plugins((
            DefaultPlugins,
            AssetLoaderPlugin,
            CameraInputPlugin,
            EnvironmentPlugin,
            ThemePlugin,
        ))
        .run();

    Ok(())
}
