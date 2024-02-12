// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/asset_loader.rs
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub roomba: Handle<Scene>,
    pub object: Handle<Scene>,
    pub obstacle_image_raw: Handle<Image>,
    pub obstacle_image_sdf: Handle<Image>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SceneAssets>()
            .add_systems(Startup, load_assets);
    }
}

fn load_assets(mut scene_assets: ResMut<SceneAssets>, asset_server: Res<AssetServer>) {
    *scene_assets = SceneAssets {
        // Robot vacuum by Poly by Google [CC-BY] (https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/dQj7UZT-1w0)
        roomba: asset_server.load("models/roomba.glb#Scene0"),
        // Cardboard Boxes by Quaternius (https://poly.pizza/m/bs6ikOeTrR)
        object: asset_server.load("models/box.glb#Scene0"),
        // environment images
        obstacle_image_raw: asset_server.load("imgs/very_clutter.png"),
        obstacle_image_sdf: asset_server.load("imgs/very_clutter_sdf.png"),
    }
}
