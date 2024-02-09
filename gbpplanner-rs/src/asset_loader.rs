// https://github.com/marcelchampagne/bevy-basics/blob/main/episode-3/src/asset_loader.rs
use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct SceneAssets {
    pub roomba: Handle<Scene>,
    pub object: Handle<Scene>,
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
        roomba: asset_server.load("roomba.glb#Scene0"),
        // Cardboard Boxes by Quaternius (https://poly.pizza/m/bs6ikOeTrR)
        object: asset_server.load("box.glb#Scene0"),
    }
}
