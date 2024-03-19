use std::{collections::VecDeque, sync::OnceLock};

use bevy::prelude::*;
use rand::Rng;

use super::robot::VariableTimestepsResource;
use crate::{
    asset_loader::SceneAssets,
    config::{formation::Shape, Config, Formation, FormationGroup},
    planner::robot::RobotBundle,
    theme::CatppuccinTheme,
};

static OBSTACLE_IMAGE: OnceLock<Image> = OnceLock::new();

#[derive(Resource)]
pub struct Repeat {
    have_spawned: Vec<bool>,
}

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_repeat_resource)
            .add_systems(Update, (formation_handler, show_formations));
    }
}

/// Check the `FormationGroup` resource, and make a `Repeat` resource,
/// reflecting whether each formation has been spawned at least once yet.
fn init_repeat_resource(mut commands: Commands, formation_group: Res<FormationGroup>) {
    let have_spawned = vec![false; formation_group.formations.len()];
    commands.insert_resource(Repeat { have_spawned });
}

/// Spawn relevant formations at each time step according to the
/// `FormationGroup` resource.
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
    variable_timesteps: Res<VariableTimestepsResource>,
) {
    // only continue if the image has been loaded
    let Some(image) = image_assets.get(&scene_assets.obstacle_image_sdf) else {
        return;
    };

    let _ = OBSTACLE_IMAGE.get_or_init(|| image.clone());

    // extract all formations from config
    formation_group
        .formations
        .iter()
        .enumerate()
        .for_each(|(i, formation)| {
            if !repeat.have_spawned[i]
                && !formation.repeat
                && formation.delay.as_secs_f32() < time.elapsed_seconds()
            {
                // Spawn the formation
                repeat.have_spawned[i] = true;
                spawn_formation(
                    &mut commands,
                    formation,
                    &config,
                    OBSTACLE_IMAGE
                        .get()
                        .expect("obstacle image should be allocated and initialised"),
                    &variable_timesteps,
                    &scene_assets,
                );
            }
        });
}

fn spawn_formation(
    commands: &mut Commands,
    formation: &Formation,
    config: &Config,
    image: &'static Image,
    variable_timesteps: &Res<VariableTimestepsResource>,
    scene_assets: &Res<SceneAssets>,
) {
    let first_wp = formation
        .waypoints
        .first()
        .expect("Formation cannot have 0 waypoint entries");

    let initial_positions = random_positions_on_shape(
        &first_wp.shape,
        formation.robots,
        config.simulation.world_size,
    );

    // TODO: create mapped waypoints

    // let variable_material =
    // materials.add(Color::from_catppuccin_colour_with_alpha(
    //     catppuccin_theme.blue(),
    //     0.75,
    // ));

    // let variable_mesh = meshes.add(
    //     Sphere::new(0.3)
    //         .mesh()
    //         .ico(4)
    //         .expect("4 subdivisions is less than the maximum allowed of 80"),
    // );

    // let lookahead_horizon =
    for position in initial_positions {
        // TODO: Used the actual mapped waypoints from the formation
        // TODO: add velocity to wa
        let wp = {
            let mut wp = Vec4::ZERO;
            wp.x = position.x;
            wp.y = position.y;
            wp.z = config.robot.max_speed;
            wp
        };
        let waypoints = [wp, Vec4::ZERO].iter().cloned().collect::<VecDeque<_>>();
        // let waypoints = VecDeque::from(vec![position, Vec2::ZERO]);
        let mut entity = commands.spawn_empty();
        let robot_id = entity.id();

        entity.insert((
            RobotBundle::new(
                robot_id,
                waypoints,
                variable_timesteps.timesteps.as_slice(),
                config,
                image,
            )
            .expect(
                "Possible `RobotInitError`s should be avoided due to the formation input being \
                 validated.",
            ),
            PbrBundle {
                mesh: scene_assets.meshes.robot.clone(),
                material: scene_assets.materials.robot.clone(),
                transform: Transform::from_translation(Vec3::new(position.x, 0.5, position.y)),
                ..Default::default()
            },
        ));
    }
}

fn show_formations(gizmos: Gizmos, formation_group: Res<FormationGroup>) {
    for formation in formation_group.formations.iter() {
        // formation.
    }
}

fn random_positions_on_shape(shape: &Shape, amount: usize, world_size: f32) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let mut positions = Vec::with_capacity(amount);

    match shape {
        Shape::Line((start, end)) => {
            let start = Vec2::from(start);
            let end = Vec2::from(end);
            for _ in 0..amount {
                // lerp a random point between start and end
                // TODO: ensure no robots spawn atop each other
                let lerp_amount = rng.gen_range(0.0..1.0);
                let new_position = start.lerp(end, lerp_amount);
                positions.push(new_position);
            }
        }
        Shape::Circle { radius, center } => {
            for _ in 0..amount {
                // generate a random angle and distance from the center
                let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                // let distance = rng.gen_range(0.0..*radius);
                let distance = radius;
                let new_position = Vec2::new(
                    center.x + angle.cos() * distance,
                    center.y + angle.sin() * distance,
                );
                positions.push(new_position);
            }
        }
        Shape::Polygon(points) => {
            // TODO: implement
            // Something about making the line segments between the points and then
            // randomly selecting a point along the total length of the line segments
            // and then finding the point on the line segment that is that distance from the
            // start
            todo!();
        }
    }

    positions
        .into_iter()
        .map(|p| world_size * (p - 0.5))
        // .map(|p| world_size * (p))
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::formation::Point;

    fn f32_eq(a: f32, b: f32) -> bool {
        f32::abs(a - b) <= f32::EPSILON
    }

    #[test]
    fn circle() {
        let center = Point { x: 0.0, y: 0.0 };
        let radius = 1.0;
        let shape = Shape::Circle { radius, center };
        let n = 8;
        let positions = random_positions_on_shape(&shape, n, 100.0);
        assert!(!positions.is_empty());
        assert_eq!(positions.len(), n);

        let center = Vec2::from(center);
        for p in positions {
            let distance_from_center = center.distance(p);
            assert_eq!(radius, distance_from_center);
        }
    }
}
