// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/asset_loader.rs
use bevy::prelude::*;

use crate::{
    config::Config,
    theme::{CatppuccinTheme, ColorFromCatppuccinColourExt},
};

/// A sub-category of the `SceneAssets` `Resource` to hold all meshes
#[derive(Debug, Default)]
pub struct Meshes {
    pub robot: Handle<Mesh>,
    pub variable: Handle<Mesh>,
    pub factor: Handle<Mesh>,
    pub waypoint: Handle<Mesh>,
}

// A sub-category of the `SceneAssets` `Resource` to hold all materials
#[derive(Debug, Default)]
pub struct Materials {
    pub robot: Handle<StandardMaterial>,
    pub variable: Handle<StandardMaterial>,
    pub factor: Handle<StandardMaterial>,
    pub waypoint: Handle<StandardMaterial>,
    pub line: Handle<StandardMaterial>,
}

/// A resource to hold all assets in a common place
/// Good practice to load assets once, and then reference them by their `Handle`
#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub main_font: Handle<Font>,
    pub roomba: Handle<Scene>,
    pub object: Handle<Scene>,
    pub obstacle_image_raw: Handle<Image>,
    pub obstacle_image_sdf: Handle<Image>,
    // pub waypoint_material: Handle<StandardMaterial>,
    // pub waypoint_mesh: Handle<Mesh>,
    pub meshes: Meshes,
    pub materials: Materials,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .add_systems(PreStartup, load_assets);
    }
}

/// `PreStartup` system to load assets as soon as possible
fn load_assets(
    mut scene_assets: ResMut<SceneAssets>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    catppuccin_theme: Res<CatppuccinTheme>,
    config: Res<Config>,
) {
    *scene_assets = SceneAssets {
        // Load the main font
        main_font: asset_server.load("fonts/JetBrainsMonoNerdFont-Regular.ttf"),
        // Robot vacuum by Poly by Google [CC-BY] (https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/dQj7UZT-1w0)
        roomba: asset_server.load("models/roomba.glb#Scene0"),
        // Cardboard Boxes by Quaternius (https://poly.pizza/m/bs6ikOeTrR)
        object: asset_server.load("models/box.glb#Scene0"),
        // environment images
        // obstacle_image_raw: asset_server.load("imgs/simple.png"),
        obstacle_image_raw: asset_server.load(format!("imgs/{}.png", config.environment)),
        obstacle_image_sdf: asset_server.load(format!("imgs/{}_sdf.png", config.environment)),
        // waypoint material
        // waypoint_material: materials.add(Color::from_catppuccin_colour_with_alpha(
        //     catppuccin_theme.flavour.maroon(),
        //     0.75,
        // )),
        // waypoint mesh
        meshes: Meshes {
            robot: meshes.add(
                Sphere::new(1.0)
                    .mesh()
                    .ico(4)
                    .expect("4 subdivisions is less than the maximum allowed of 80"),
            ),
            variable: meshes.add(
                Sphere::new(0.3)
                    .mesh()
                    .ico(4)
                    .expect("4 subdivisions is less than the maximum allowed of 80"),
            ),
            factor: meshes.add(Cuboid::new(0.5, 0.5, 0.5)),
            waypoint: meshes.add(
                Sphere::new(0.5)
                    .mesh()
                    .ico(4)
                    .expect("4 subdivisions is less than the maximum allowed of 80"),
            ),
        },
        materials: Materials {
            robot: materials.add(Color::from_catppuccin_colour(
                catppuccin_theme.flavour.green(),
            )),
            variable: materials.add(Color::from_catppuccin_colour_with_alpha(
                catppuccin_theme.flavour.blue(),
                0.75,
            )),
            factor: materials.add(Color::from_catppuccin_colour_with_alpha(
                catppuccin_theme.flavour.mauve(),
                0.75,
            )),
            waypoint: materials.add(Color::from_catppuccin_colour_with_alpha(
                catppuccin_theme.flavour.maroon(),
                0.5,
            )),
            line: materials.add(Color::from_catppuccin_colour(
                catppuccin_theme.flavour.text(),
            )),
        },
    }
}
