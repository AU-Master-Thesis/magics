use bevy::{asset::LoadedFolder, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, load_obstacle_folder)
        .run();
}

#[derive(Resource)]
struct LoadedObstaclesFolder(Handle<LoadedFolder>);

impl LoadedObstaclesFolder {
    // pub fn loaded(&self) -> bool {
    //     // self.0.is_loaded()
    // }
}

impl FromWorld for LoadedObstaclesFolder {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        LoadedObstaclesFolder(asset_server.load_folder("imgs/obstacles"))
    }
}

// 46   │     let _loaded_folder: Handle<LoadedFolder> =
// asset_server.load_folder("models/torus");
fn load_obstacle_folder(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/obstacle.glb#Scene0"),
            ..default()
        })
        .insert(Name::new("Obstacle"));
}
