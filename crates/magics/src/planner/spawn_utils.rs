//! Utility functions for spawning robot entities.

use std::time::Duration;

use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, On, PickableBundle, Pointer};
use bevy_prng::WyRand;
use bevy_rand::prelude::{ForkableRng, GlobalEntropy};
use gbp_config::geometry::Shape; // Use external crate path
use gbp_config::{
    formation::{PlanningStrategy, ReachedWhen},
    Config,
};
use gbp_linalg::prelude::*;
use min_len_vec::{one_or_more, two_or_more, OneOrMore, TwoOrMore};
use rand::seq::IteratorRandom;
use rand::Rng; // Added import
use strum::IntoEnumIterator;

use crate::simulation_loader::SharedSdfImage;
use crate::{
    api::state::FactorWeights as ApiFactorWeights, // Keep alias for clarity
    environment::FollowCameraMe,
    factorgraph::factorgraph::FactorGraph,
    goal_area,
    movement::Velocity,
    planner::{
        robot::{
            Ball, FinishedPath, GbpIterationSchedule, Mission, RadioAntenna, Radius, RobotBundle,
            RobotConnections, RobotDespawned, RobotSpawned, Route, StateVector, VariableTimesteps,
            T0,
        },
        spawner::{RobotClickedOn, WaypointCreated},
        tracking::{PositionTracker, VelocityTracker},
    },
    simulation_loader::{Reloadable, Sdf, SdfImage},
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
    utils, /* Added utils for get_variable_timesteps
            * gbp_config::geometry::Shape, // Removed duplicate import */
};

/// Places a single robot randomly within the given shape, avoiding collisions
/// with already placed robots. Returns `Some(Vec2)` with the valid position if
/// successful within `max_attempts`, otherwise `None`.
pub fn place_single_robot<R: Rng + ?Sized>(
    // Made public
    shape: &Shape,
    world_dims: &gbp_config::formation::WorldDimensions,
    robot_radius: f32,
    placed_robots: &[(Vec2, f32)], // List of (position, radius) of already placed robots
    max_attempts: usize,
    rng: &mut R,
) -> Option<Vec2> {
    for _ in 0..max_attempts {
        if let Some(candidate_pos) = shape.get_random_point(world_dims, rng) {
            let collision_free = placed_robots.iter().all(|(other_pos, other_radius)| {
                candidate_pos.distance(*other_pos) >= robot_radius + *other_radius
            });

            if collision_free {
                return Some(candidate_pos);
            }
        } else {
            // Shape does not support random point generation (e.g., Polygon)
            // or failed internally. We cannot place the robot.
            error!(
                "Failed to generate a random point within the provided shape: {:?}",
                shape
            );
            return None;
        }
    }
    // Failed to find a collision-free spot after max_attempts
    warn!(
        "Failed to place robot collision-free after {} attempts within shape: {:?}",
        max_attempts, shape
    );
    None
}

/// Helper function to spawn a single robot entity with all necessary
/// components.
#[allow(clippy::too_many_arguments)]
pub fn spawn_robot(
    commands: &mut Commands,
    config: &Config,
    env_config: &gbp_environment::Environment,
    sdf: SharedSdfImage,
    prng: &mut GlobalEntropy<WyRand>,
    materials: &mut Assets<StandardMaterial>,
    mesh_assets: &mut Assets<Mesh>,
    theme: &CatppuccinTheme,
    time_fixed: &Time<Fixed>,
    evw_robot_spawned: &mut EventWriter<RobotSpawned>,
    evw_waypoint_created: &mut EventWriter<WaypointCreated>,
    // Robot specific parameters
    initial_state_vec: StateVector,
    waypoints_for_mission: TwoOrMore<StateVector>,
    radius: f32,
    planning_strategy: PlanningStrategy,
    target_speed: f32,
    waypoint_reached_when_intersects: ReachedWhen,
    finished_when_intersects: ReachedWhen,
    initial_custom_weights: Option<ApiFactorWeights>, /* Note: Handling this requires caller to
                                                       * queue WeightUpdate */
) -> Entity {
    // Timesteps
    let divisor: f32 = (radius / 2.0 / target_speed).max(f32::EPSILON);
    let lookahead_horizon: u32 = (config.robot.planning_horizon.get() / divisor).round() as u32;
    let lookahead_multiple = config.gbp.lookahead_multiple as u32;
    let variable_timesteps = utils::get_variable_timesteps(lookahead_horizon, lookahead_multiple);

    // --- Prepare Bundles ---
    let mut entity_commands = commands.spawn_empty();
    let new_entity = entity_commands.id();

    // Create RobotBundle
    let robot_bundle = RobotBundle::new(
        new_entity,
        initial_state_vec,
        variable_timesteps.as_slice(),
        config,
        env_config,
        radius,
        sdf, 
        time_fixed.elapsed().as_secs_f64(),
        waypoints_for_mission.clone(), // Clone waypoints for the bundle
        planning_strategy,
        waypoint_reached_when_intersects,
        finished_when_intersects,
    );

    // Visuals
    let initial_pos: Vec2 = initial_state_vec.position();
    let initial_translation = Vec3::new(initial_pos.x, -1.5, initial_pos.y);
    let random_color = DisplayColour::iter()
        .choose(prng)
        .expect("there is more than 0 colors");
    let material = materials.add(StandardMaterial {
        base_color: Color::from_catppuccin_colour_ref(theme.get_display_colour(&random_color)),
        ..Default::default()
    });
    let mesh = mesh_assets.add(
        Sphere::new(radius)
            .mesh()
            .ico(2)
            .expect("Subdivision level is valid"),
    );
    let pbr_bundle = PbrBundle {
        mesh,
        material,
        transform: Transform::from_translation(initial_translation),
        visibility: Visibility::Visible,
        ..Default::default()
    };

    // Get goal position before moving robot_bundle
    let goal_position_for_event = robot_bundle
        .mission
        .taskpoints
        .last()
        .map(|wp| wp.position());

    // --- Insert Components ---
    entity_commands.insert((
        robot_bundle, // robot_bundle is moved here
        pbr_bundle,
        prng.fork_rng(),
        Reloadable,
        PositionTracker::new(10000, Duration::from_millis(100)),
        VelocityTracker::new(10000, Duration::from_millis(100)),
        PickableBundle::default(),
        On::<Pointer<Click>>::send_event::<RobotClickedOn>(),
        ColorAssociation { name: random_color },
        FollowCameraMe::new(0.0, 30.0, 0.0),
        goal_area::components::Collider(Box::new(parry2d::shape::Ball::new(radius))),
    ));

    // --- Post-Spawn Actions ---

    // Send WaypointCreated event for the goal
    if let Some(goal_pos) = goal_position_for_event {
        evw_waypoint_created.send(WaypointCreated {
            for_robot: new_entity,
            position:  goal_pos,
        });
    }

    // Send RobotSpawned event
    evw_robot_spawned.send(RobotSpawned(new_entity));

    // Note: Custom weights must be handled by the caller by queuing a WeightUpdate

    info!("Spawned new robot entity {:?}", new_entity);

    new_entity
}
