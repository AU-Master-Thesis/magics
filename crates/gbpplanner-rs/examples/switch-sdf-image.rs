use std::time::Duration;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    time::common_conditions::{on_real_timer, once_after_real_delay},
};

#[derive(Resource)]
struct SceneAssets {
    pub sdf: Handle<Image>,
}

impl FromWorld for SceneAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource_mut::<AssetServer>().unwrap();
        let first = Environments::default().asset_path();
        SceneAssets {
            sdf: assets.load(first),
        }
    }
}

#[derive(Component)]
struct Sdf;

fn main() -> anyhow::Result<()> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EntityCountDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .init_resource::<SceneAssets>()
        .init_state::<Environments>()
        .add_systems(Startup, (spawn_camera, create_sprite_bundle))
        .add_systems(Update, render_sdf.run_if(resource_changed::<SceneAssets>))
        .add_systems(Update, load_next_sdf.run_if(on_real_timer(Duration::from_secs(1))))
        .run();

    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum Environments {
    #[default]
    Junction,
    Roundabout,
    CircleCluttered,
    VeryClutter,
}

impl Environments {
    fn next(self) -> Self {
        use Environments::*;
        match self {
            Junction => Roundabout,
            Roundabout => CircleCluttered,
            CircleCluttered => VeryClutter,
            VeryClutter => Junction,
        }
    }

    fn asset_path(self) -> &'static str {
        match self {
            Environments::Junction => "imgs/junction_sdf.png",
            Environments::Roundabout => "imgs/roundabout_sdf.png",
            Environments::CircleCluttered => "imgs/circle_cluttered_sdf.png",
            Environments::VeryClutter => "imgs/very_clutter_sdf.png",
        }
    }
}

fn load_next_sdf(
    state: Res<State<Environments>>,
    mut next_state: ResMut<NextState<Environments>>,
    mut scene_assets: ResMut<SceneAssets>,
    assets: ResMut<AssetServer>,
) {
    let current = state.get();
    info!("switching from {:?} to {:?}", current, current.next());
    scene_assets.sdf = assets.load(current.asset_path());
    next_state.set(current.next());
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_sprite_bundle(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    commands.spawn((
        SpriteBundle {
            texture: scene_assets.sdf.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Sdf,
    ));
}

fn render_sdf(
    mut commands: Commands,
    scene_assets: Res<SceneAssets>,
    mut existing_sdf: Query<&mut Handle<Image>, With<Sdf>>,
) {
    if let Ok(mut image) = existing_sdf.get_single_mut() {
        *image = scene_assets.sdf.clone();
    }
}
// fn load_circle_cluttered(
//     mut scene_assets: ResMut<SceneAssets>,
//     // mut images: ResMut<Assets<Image>>,
//     mut asset_server: ResMut<AssetServer>,
// ) {
//     scene_assets.sdf = asset_server.load("imgs/circle_cluttered_sdf.png");
// }
//
// fn load_junction(
//     mut scene_assets: ResMut<SceneAssets>,
//     // mut images: ResMut<Assets<Image>>,
//     mut assets: ResMut<AssetServer>,
// ) {
//     scene_assets.sdf = assets.load("imgs/junction_sdf.png");
// }
// fn load_roundabout(
//     mut scene_assets: ResMut<SceneAssets>,
//     // mut images: ResMut<Assets<Image>>,
//     mut assets: ResMut<AssetServer>,
// ) {
//     scene_assets.sdf = assets.load("imgs/roundabout_sdf.png");
// }

// fn load(path: &'static str) -> impl FnMut(ResMut<SceneAssets>,
// ResMut<AssetServer>) {     move |mut scene_assets: ResMut<SceneAssets>,
// assets: ResMut<AssetServer>| {         scene_assets.sdf = assets.load(path);
//     }
// }

// fn load_very_cluttered
