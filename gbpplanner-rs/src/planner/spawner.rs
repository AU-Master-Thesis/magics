use std::{collections::VecDeque, sync::OnceLock};

use bevy::prelude::*;
use rand::Rng;

use crate::{
    asset_loader::SceneAssets,
    config::{formation::Shape, Config, Formation, FormationGroup},
    planner::robot::RobotBundle,
    theme::CatppuccinTheme,
    utils::get_variable_timesteps,
};

// pub static IMAGE: OnceLock<Image> = OnceLock::new();
static IMAGE: OnceLock<Image> = OnceLock::new();

#[derive(Resource)]
pub struct Repeat {
    have_spawned: Vec<bool>,
}

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_repeat_resource)
            .add_systems(Update, formation_handler);
    }
}

/// Check the `FormationGroup` resource, and make a `Repeat` resource,
/// reflecting whether each formation has been spawned at least once yet.
fn init_repeat_resource(mut commands: Commands, formation_group: Res<FormationGroup>) {
    let have_spawned = vec![false; formation_group.formations.len()];
    commands.insert_resource(Repeat { have_spawned });
}

// fn init_obstacle_sdf(
//     mut commands: Commands,
//     scene_assets: Res<SceneAssets>,
//     mut image_assets: ResMut<Assets<Image>>,
// ) {
//     // only continue if the image has been loaded
//     let Some(image) = image_assets.get(&scene_assets.obstacle_image_sdf) else {
//         return;
//     };

//     let _ = IMAGE.get_or_init(|| image.clone());
// }

/// Spawn relevant formations at each time step according to the `FormationGroup` resource.
fn formation_handler(
    mut commands: Commands,
    formation_group: Res<FormationGroup>,
    mut repeat: ResMut<Repeat>,
    time: Res<Time>,
    config: Res<Config>,
    scene_assets: Res<SceneAssets>,
    image_assets: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    // only continue if the image has been loaded
    let Some(image) = image_assets.get(&scene_assets.obstacle_image_sdf) else {
        return;
    };

    let _ = IMAGE.get_or_init(|| image.clone());
    // let image = Box::leak(Box::new(image.clone()));
    // let _ = IMAGE.get_or_init(|| image.clone());

    // let image = Box::leak(Box::new(image.clone()));

    // extract all formations from config
    formation_group
        .formations
        .iter()
        .enumerate()
        .for_each(|(i, formation)| {
            if !repeat.have_spawned[i]
                && !formation.repeat
                && formation.time < time.elapsed_seconds()
            {
                // Spawn the formation
                repeat.have_spawned[i] = true;
                spawn_formation(
                    &mut commands,
                    formation,
                    &config,
                    &IMAGE.get().expect("Image should be initialised"),
                    &mut materials,
                    &mut meshes,
                    &catppuccin_theme,
                );
            }
        });
}

fn spawn_formation(
    commands: &mut Commands,
    formation: &Formation,
    config: &Config,
    image: &'static Image,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    catppuccin_theme: &Res<CatppuccinTheme>,
) {
    // // only continue if the image has been loaded
    // let Some(image) = image_assets.get(&scene_assets.obstacle_image_sdf) else {
    //     return;
    // };

    // spawn the formation
    let first_wp = formation
        .waypoints
        .first()
        .expect("Formation cannot have 0 waypoint entries");

    let initial_positions = random_position_on_shape(&first_wp.shape, formation.robots);

    // TODO: create mapped waypoints

    let alpha = 0.75;
    let variable_material = materials.add({
        let (r, g, b) = catppuccin_theme.flavour.blue().into();
        Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
    });

    let variable_mesh = meshes.add(
        bevy::math::primitives::Sphere::new(0.3)
            .mesh()
            .ico(4)
            .expect("4 subdivisions is less than the maximum allowed of 80"),
    );
    // let variable_mesh = meshes.add(
    //     shape::Icosphere {
    //         radius: 0.3,
    //         subdivisions: 4,
    //     }
    //     .try_into()
    //     .unwrap(),
    // );

    for position in initial_positions.clone().iter() {
        commands.spawn((
            RobotBundle::new(
                // TODO: Used tha actual mapped waypoints from the formation
                VecDeque::from(vec![*position, *position + Vec2::new(1.0, 1.0)]),
                // TODO: calculate variable timesteps in `Startup` stage and store in a resource
                &get_variable_timesteps(
                    (config.robot.planning_horizon / config.simulation.t0) as u32,
                    config.gbp.lookahead_multiple as u32,
                ),
                &config,
                image,
            )
            .expect("Possible `RobotInitError`s should be avoided due to the formation input being validated."),
            PbrBundle {
                mesh: variable_mesh.clone(),
                material: variable_material.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, 0.5, position.y)),
                ..Default::default()
            },
        ));
    }
}

fn random_position_on_shape(shape: &Shape, amount: usize) -> Vec<Vec2> {
    match shape {
        Shape::Line((start, end)) => {
            let mut rng = rand::thread_rng();
            let mut positions = Vec::with_capacity(amount);
            for _ in 0..amount {
                // lerp a random point between start and end
                let lerp_amount = rng.gen_range(0.0..1.0);
                let new_position = Vec2::from(start).lerp(Vec2::from(end), lerp_amount);
                positions.push(new_position);
            }
            positions
        }
        // Shape::Circle(center, radius) => {
        //     let mut rng = rand::thread_rng();
        //     let mut positions = Vec::with_capacity(amount);
        //     for _ in 0..amount {
        //         // generate a random angle and distance from the center
        //         let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        //         let distance = rng.gen_range(0.0..radius);
        //         let new_position = Vec2::new(
        //             center.x + angle.cos() * distance,
        //             center.y + angle.sin() * distance,
        //         );
        //         positions.push(new_position);
        //     }
        //     positions
        // }
        Shape::Circle { radius, center } => {
            let mut rng = rand::thread_rng();
            let mut positions = Vec::with_capacity(amount);
            for _ in 0..amount {
                // generate a random angle and distance from the center
                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let distance = rng.gen_range(0.0..*radius);
                let new_position = Vec2::new(
                    center.x + angle.cos() * distance,
                    center.y + angle.sin() * distance,
                );
                positions.push(new_position);
            }
            positions
        }
        Shape::Polygon(points) => {
            // TODO: implement
            // Something about making the line segments between the points and then
            // randomly selecting a point along the total length of the line segments
            // and then finding the point on the line segment that is that distance from the start
            todo!();
        }
    }
}
