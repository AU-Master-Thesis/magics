use std::{collections::VecDeque, num::NonZeroUsize, sync::OnceLock};

use bevy::prelude::*;
use rand::Rng;
use typed_floats::StrictlyPositiveFinite;

use super::robot::VariableTimestepsResource;
use crate::{
    asset_loader::SceneAssets,
    config::{formation::Shape, Config, Formation, FormationGroup},
    percentage::Percentage,
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

fn percentage_to_world_size(percentage: Percentage, world_size: f32) -> f32 {
    todo!()
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

    dbg!(&formation);
    let mut rng = rand::thread_rng();
    let lerp_amounts = match &first_wp.shape {
        Shape::Line((start, end)) => randomly_place_nonoverlapping_circles_along_line_segment(
            Vec2::from(start),
            Vec2::from(end),
            formation.robots,
            // NonZeroUsize::new(formation.robots).expect("robots is not zero"),
            config.robot.radius,
            NonZeroUsize::new(100).expect("100 is not zero"),
            &mut rng,
        )
        .expect("Could place non-overlapping circles along line segment"),
        _ => unimplemented!(),
    };

    let initial_position_of_each_robot = map_positions(&first_wp.shape, &lerp_amounts);
    let mut positions_of_each_robot = vec![initial_position_of_each_robot];
    positions_of_each_robot.extend(
        formation
            .waypoints
            .iter()
            .skip(1)
            .map(|wp| map_positions(&wp.shape, &lerp_amounts)),
    );

    let max_speed = config.robot.max_speed.get();
    for positions in positions_of_each_robot {
        // TODO: add velocity to wa
        // let wp = Vec4::new(position.x, position.y, config.robot.max_speed, 0.0);

        let waypoints = positions
            .iter()
            .map(|p| Vec4::new(p.x, p.y, max_speed, 0.0))
            .collect::<VecDeque<_>>();
        // let waypoints = [wp, Vec4::ZERO].iter().cloned().collect::<VecDeque<_>>();
        // let waypoints = VecDeque::from(vec![position, Vec2::ZERO]);
        let mut entity = commands.spawn_empty();
        let robot_id = entity.id();

        let initial_position = positions.first().expect("positions is not empty");
        let initial_translation = Vec3::new(initial_position.x, 0.5, initial_position.y);

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
                transform: Transform::from_translation(initial_translation),
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

fn map_positions(shape: &Shape, lerp_amounts: &[f32]) -> Vec<Vec2> {
    match shape {
        Shape::Line((start, end)) => {
            let start = Vec2::from(start);
            let end = Vec2::from(end);
            lerp_amounts
                .iter()
                .map(|&lerp_amount| start.lerp(end, lerp_amount))
                .collect()
        }
        _ => unimplemented!(),
    }
}

#[derive(Debug, Clone, Copy)]
struct LineSegment {
    from: Vec2,
    to:   Vec2,
}

impl LineSegment {
    fn new(from: Vec2, to: Vec2) -> Self {
        Self { from, to }
    }

    fn length(&self) -> f32 {
        self.from.distance(self.to)
    }
}

fn randomly_place_nonoverlapping_circles_along_line_segment(
    from: Vec2,
    to: Vec2,
    num_circles: NonZeroUsize,
    radius: StrictlyPositiveFinite<f32>,
    max_attempts: NonZeroUsize,
    rng: &mut impl Rng,
) -> Option<Vec<f32>> {
    let num_circles = num_circles.get();
    let max_attempts = max_attempts.get();
    // let mut rng = rand::thread_rng();
    let mut lerp_amounts: Vec<f32> = Vec::with_capacity(num_circles);
    let mut placed: Vec<Vec2> = Vec::with_capacity(num_circles);

    let diameter = radius.get() * 2.0;

    for _ in 0..max_attempts {
        placed.clear();
        lerp_amounts.clear();

        for _ in 0..num_circles {
            let lerp_amount = rng.gen_range(0.0..1.0);
            let new_position = from.lerp(to, lerp_amount);

            let valid = placed.iter().all(|&p| new_position.distance(p) >= diameter);

            if valid {
                lerp_amounts.push(lerp_amount);
                placed.push(new_position);
                if placed.len() == num_circles {
                    return Some(lerp_amounts);
                }
            }
        }
    }

    None
}

fn random_positions_on_shape(shape: &Shape, amount: usize, world_size: f32) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    // let mut positions = Vec::with_capacity(amount);

    match shape {
        Shape::Line((start, end)) => {
            // let start = Vec2::from(start);
            // let end = Vec2::from(end);
            (0..amount).map(|_| {
                rng.gen_range(0.0..1.0)
            }).collect()
            // for _ in 0..amount {
            //     // lerp a random point between start and end
            //     // TODO: ensure no robots spawn atop each other
            //     let lerp_amount = rng.gen_range(0.0..1.0);
            // }
        },
        _ => unimplemented!()
        // Shape::Circle { radius, center } => {
        //     for _ in 0..amount {
        //         // generate a random angle and distance from the center
        //         let angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
        //         // let distance = rng.gen_range(0.0..*radius);
        //         let distance = radius;
        //         let new_position = Vec2::new(
        //             center.x + angle.cos() * distance,
        //             center.y + angle.sin() * distance,
        //         );
        //         positions.push(new_position);
        //     }
        // }
        // Shape::Polygon(points) => {
        //     // TODO: implement
        //     // Something about making the line segments between the points and then
        //     // randomly selecting a point along the total length of the line segments
        //     // and then finding the point on the line segment that is that distance from the
        //     // start
        //     todo!();
        // }
    }

    // positions
    //     .into_iter()
    //     .map(|p| world_size * (p - 0.5))
    //     // .map(|p| world_size * (p))
    //     .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::formation::Point;

    fn f32_eq(a: f32, b: f32) -> bool {
        f32::abs(a - b) <= f32::EPSILON
    }

    // #[test]
    // fn circle() {
    //     let center = Point { x: 0.0, y: 0.0 };
    //     let radius = 1.0.try_into().unwrap();
    //     let shape = Shape::Circle { radius, center };
    //     let n = 8;
    //     let positions = random_positions_on_shape(&shape, n, 100.0);
    //     assert!(!positions.is_empty());
    //     assert_eq!(positions.len(), n);
    //
    //     let center = Vec2::from(center);
    //     for p in positions {
    //         let distance_from_center = center.distance(p);
    //         assert_eq!(radius, distance_from_center);
    //     }
    // }
}
