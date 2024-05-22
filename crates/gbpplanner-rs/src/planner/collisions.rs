use std::{collections::HashMap, ops::Deref, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use gbp_global_planner::Colliders;
use parry2d::{
    bounding_volume::BoundingVolume,
    na::{Complex, Isometry, Unit},
};

use super::{robot::Ball, RobotConnections};
use crate::{
    // environment::map_generator::Colliders,
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

#[derive(Default)]
pub struct RobotCollisionsPlugin;

impl RobotCollisionsPlugin {
    pub const UPDATE_EVERY: Duration = Duration::from_millis(200);
}

impl Plugin for RobotCollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::RobotRobotCollisions>()
            .init_resource::<resources::RobotEnvironmentCollisions>()
            .add_event::<events::RobotRobotCollision>()
            .add_event::<events::RobotEnvironmentCollision>()
            .add_systems(
                Update,
                (
                    update_robot_robot_collisions.run_if(on_timer(Self::UPDATE_EVERY)),
                    render_robot_robot_collisions,
                    toggle_visibility_of_robot_robot_collisions,
                    // toggle_visibility_of_robot_robot_collisions
                    //     .run_if(input_just_pressed(KeyCode::F10)),
                    update_robot_environment_collisions.run_if(
                        on_timer(Self::UPDATE_EVERY).and_then(resource_exists::<Colliders>),
                    ),
                    render_robot_environment_collisions,
                    toggle_visibility_of_robot_environment_collisions,
                    on_obstacle_clicked_on,
                    record_aabb_when_two_robots_collide,
                    record_aabb_when_robot_collides_with_environment,
                    // toggle_visibility_of_robot_environment_collisions
                    //     .run_if(input_just_pressed(KeyCode::F10)),
                ),
            )
            .add_systems(
                PostUpdate,
                clear_robot_robot_collisions
                    .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
            );
    }
}

fn clear_robot_robot_collisions(mut robot_collisions: ResMut<resources::RobotRobotCollisions>) {
    robot_collisions.clear();
}

fn update_robot_robot_collisions(
    mut robot_collisions: ResMut<resources::RobotRobotCollisions>,
    robots: Query<(Entity, &Transform, &Ball), With<RobotConnections>>,
    // PERF: store bounding spheres in a Local<> vec to reuse the allocation between system calls
    mut aabbs: Local<
        Vec<(
            Entity,
            Isometry<f32, Unit<Complex<f32>>, 2>,
            parry2d::bounding_volume::BoundingSphere,
        )>,
    >,
    mut evw_robots_collided: EventWriter<events::RobotRobotCollision>,
) {
    aabbs.clear();

    let iter = robots.iter().map(|(entity, tf, ball)| {
        let position = parry2d::na::Isometry2::translation(tf.translation.x, tf.translation.z); // bevy uses xzy coordinates
        let bounding_volume = ball.bounding_sphere(&position);
        (entity, position, bounding_volume)
        // (entity, ball.aabb(&position))
    });

    aabbs.extend(iter);

    if aabbs.len() < 2 {
        // No collisions if there is less than two robots
        return;
    }

    for (r, c) in
        seq::upper_triangular_exclude_diagonal(aabbs.len().try_into().expect("more than one robot"))
            .expect("more than one robot")
    {
        let is_colliding = aabbs[r].2.intersects(&aabbs[c].2);
        // aabbs[r].1.intersection()
        let collision_status = robot_collisions.update(aabbs[r].0, aabbs[c].0, is_colliding);

        match collision_status {
            CollisionStatus::Hit => {
                // ehh...
                let r_ball = robots.get(aabbs[r].0).unwrap().2;
                let c_ball = robots.get(aabbs[c].0).unwrap().2;
                let r_aabb = r_ball.aabb(&aabbs[r].1);
                let c_aabb = c_ball.aabb(&aabbs[c].1);
                let intersection = r_aabb
                    .intersection(&c_aabb)
                    .expect("the robots just hit each other, so they intersect");

                // let intersection = aabbs[r]
                //     .1
                //     .intersection(&aabbs[c].1)
                //     .expect("the robots just hit each other, so they intersect");

                println!(
                    "send robot collided event with intersection: {:?}",
                    &intersection
                );
                evw_robots_collided.send(events::RobotRobotCollision {
                    robot_a: aabbs[c].0,
                    robot_b: aabbs[r].0,
                    intersection,
                });
            }
            _ => {}
        }
    }
}

pub mod resources {
    // use bevy::prelude::*;
    use super::*;

    #[derive(Resource)]
    pub struct RobotRobotCollisions {
        inner:      HashMap<(Entity, Entity), CollisionHistory>,
        collisions: usize,
    }

    impl RobotRobotCollisions {
        fn new() -> Self {
            Self {
                inner:      HashMap::new(),
                collisions: 0,
            }
        }

        pub(super) fn update(
            &mut self,
            e1: Entity,
            e2: Entity,
            is_colliding: bool,
        ) -> CollisionStatus {
            let entry = self
                .inner
                .entry((e1, e2))
                .or_insert(CollisionHistory::new());
            let collision_status = entry.update(is_colliding);
            if let CollisionStatus::Hit = collision_status {
                self.collisions += 1;
            }

            collision_status
        }

        pub fn get(&self, entity: Entity) -> Option<usize> {
            self.inner
                .iter()
                .filter_map(|((e1, e2), c)| {
                    if *e1 == entity {
                        Some(c.collisions())
                    } else if *e2 == entity {
                        Some(c.collisions())
                    } else {
                        None
                    }
                })
                .sum::<usize>()
                .into()
        }

        pub fn num_collisions(&self) -> usize {
            self.collisions
            // self.inner.values().map(|c| c.collisions()).sum::<usize>()
        }

        pub(super) fn clear(&mut self) {
            self.inner.clear();
        }

        pub(super) fn record_collision(&mut self, event: &events::RobotRobotCollision) {
            if let Some(entry) = self.inner.get_mut(&(event.robot_a, event.robot_b)) {
                entry.aabbs.push(event.intersection.clone());
            } else if let Some(entry) = self.inner.get_mut(&(event.robot_b, event.robot_a)) {
                entry.aabbs.push(event.intersection.clone());
            }
        }

        pub fn collisions(
            &self,
        ) -> impl '_ + Iterator<Item = ((Entity, Entity), &[parry2d::bounding_volume::Aabb])>
// TODO:create iterator struct for this
        {
            self.inner
                .iter()
                .map(|(k, v)| (*k, v.aabbs.as_slice()))
                .filter(|(_, v)| v.len() > 0)
        }
    }

    impl Default for RobotRobotCollisions {
        fn default() -> Self {
            Self::new()
        }
    }

    #[derive(Resource)]
    pub struct RobotEnvironmentCollisions {
        inner:      HashMap<(Entity, Entity), CollisionHistory>,
        collisions: usize,
    }

    impl RobotEnvironmentCollisions {
        fn new() -> Self {
            Self {
                inner:      HashMap::new(),
                collisions: 0,
            }
        }

        pub(super) fn update(
            &mut self,
            e1: Entity,
            e2: Entity,
            is_colliding: bool,
        ) -> CollisionStatus {
            let entry = self
                .inner
                .entry((e1, e2))
                .or_insert(CollisionHistory::new());
            let collision_status = entry.update(is_colliding);
            if let CollisionStatus::Hit = collision_status {
                self.collisions += 1;
            }

            collision_status
        }

        pub fn get(&self, entity: Entity) -> Option<usize> {
            self.inner
                .iter()
                .filter_map(|((e1, e2), c)| {
                    if *e1 == entity {
                        Some(c.collisions())
                    } else if *e2 == entity {
                        Some(c.collisions())
                    } else {
                        None
                    }
                })
                .sum::<usize>()
                .into()
        }

        pub fn num_collisions(&self) -> usize {
            self.collisions
            // self.inner.values().map(|c| c.collisions()).sum::<usize>()
        }

        pub fn collisions(
            &self,
        ) -> impl '_ + Iterator<Item = ((Entity, Entity), &[parry2d::bounding_volume::Aabb])> // TODO:
        // create iterator struct for this
        {
            self.inner
                .iter()
                .map(|(k, v)| (*k, v.aabbs.as_slice()))
                .filter(|(_, v)| v.len() > 0)
        }

        fn clear(&mut self) {
            self.inner.clear();
        }

        pub fn robots_collided_with(&self, obstacle_entity: Entity) -> Option<Vec<Entity>> {
            let mut exists = false;
            for (_, entity) in self.inner.keys() {
                if *entity == obstacle_entity {
                    exists = true;
                    break;
                }
            }

            if !exists {
                return None;
            }

            let robot_entities = self
                .inner
                .iter()
                .filter_map(|((e1, e2), history)| {
                    if *e2 == obstacle_entity && history.times > 0 {
                        Some(*e1)
                    } else {
                        None
                    }
                })
                .collect();

            Some(robot_entities)
        }

        pub(super) fn record_collision(&mut self, event: &events::RobotEnvironmentCollision) {
            if let Some(entry) = self.inner.get_mut(&(event.robot, event.obstacle)) {
                entry.aabbs.push(event.intersection.clone());
            }
        }
    }

    impl Default for RobotEnvironmentCollisions {
        fn default() -> Self {
            Self::new()
        }
    }
}

pub mod events {
    use bevy::prelude::*;

    #[derive(Debug, Event)]
    pub struct RobotEnvironmentCollision {
        pub robot: Entity,
        pub obstacle: Entity,
        pub intersection: parry2d::bounding_volume::Aabb,
    }

    #[derive(Event)]
    pub struct RobotRobotCollision {
        pub robot_a:      Entity,
        pub robot_b:      Entity,
        pub intersection: parry2d::bounding_volume::Aabb,
    }
}

fn update_robot_environment_collisions(
    env_colliders: Res<Colliders>,
    robots: Query<(Entity, &Transform, &Ball), With<RobotConnections>>,
    mut robot_environment_collisions: ResMut<resources::RobotEnvironmentCollisions>,
    mut aabbs: Local<
        Vec<(
            Entity,
            Isometry<f32, Unit<Complex<f32>>, 2>,
            parry2d::bounding_volume::BoundingSphere,
        )>,
    >,
    mut evw_robot_environment_collision: EventWriter<events::RobotEnvironmentCollision>,
) {
    // aabbs.clear();
    //
    // let iter = robots.iter().map(|(entity, tf, ball)| {
    //     let position = parry2d::na::Isometry2::translation(tf.translation.x,
    // tf.translation.z); // bevy uses xzy coordinates
    // // (entity, ball.aabb(&position))     (entity, position,
    // ball.bounding_sphere(&position)) });
    //
    // aabbs.extend(iter);
    //
    // if aabbs.is_empty() {
    //     // No collisions if there are no robots
    //     return;
    // }

    // println!("#env colliders: {}", env_colliders.len());

    // check every robot aabb against every environment aabb

    for (robot_id, tf, ball) in &robots {
        let robot_pos = parry2d::na::Isometry2::translation(tf.translation.x, tf.translation.z);

        for env_collider in env_colliders.iter() {
            let is_colliding: bool = parry2d::query::intersection_test(
                &env_collider.isometry,
                env_collider.shape.as_ref(),
                &robot_pos,
                ball.deref(),
            )
            .expect("used shapes are supported");
            let env_mesh_id = env_collider
                .associated_mesh
                .expect("Environment collider should have an associated mesh.");
            let collision_status =
                robot_environment_collisions.update(robot_id, env_mesh_id, is_colliding);

            match collision_status {
                CollisionStatus::Hit => {
                    let robot_aabb = ball.aabb(&robot_pos);
                    let env_aabb = env_collider.aabb();
                    let intersection = robot_aabb.intersection(&env_aabb).unwrap();
                    evw_robot_environment_collision.send(events::RobotEnvironmentCollision {
                        robot: robot_id,
                        obstacle: env_mesh_id,
                        intersection,
                    });

                    warn!(
                        "robot {:?} collided with environment collision event with intersection: \
                         {:?}",
                        &robot_id, &intersection
                    );
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollisionStatus {
    Hit,
    Colliding,
    End,
    Free,
}

#[derive(Debug, Default)]
enum CollisionState {
    Colliding,
    #[default]
    Free,
}

struct CollisionHistory {
    /// How many times a collision has happened between two robots
    times: usize,
    /// The current state of the collision
    state: CollisionState,
    pub(super) aabbs: Vec<parry2d::bounding_volume::Aabb>,
}

impl CollisionHistory {
    fn new() -> Self {
        Self {
            times: 0,
            state: CollisionState::Free,
            aabbs: Vec::new(),
        }
    }

    fn update(&mut self, is_colliding: bool) -> CollisionStatus {
        match self.state {
            CollisionState::Colliding if is_colliding => CollisionStatus::Colliding,
            CollisionState::Colliding if !is_colliding => {
                self.state = CollisionState::Free;
                CollisionStatus::End
            }
            CollisionState::Free if !is_colliding => CollisionStatus::Free,
            CollisionState::Free if is_colliding => {
                self.state = CollisionState::Colliding;
                self.times += 1;
                CollisionStatus::Hit
            }

            _ => unreachable!(),
        }
    }

    fn collisions(&self) -> usize {
        self.times
    }
}

/// Marker components
mod markers {
    use bevy::ecs::component::Component;

    #[derive(Component)]
    pub struct RobotRobotCollision;

    #[derive(Component)]
    pub struct RobotEnvironmentCollision;
}

fn render_robot_robot_collisions(
    mut commands: Commands,
    mut evr_robots_collided: EventReader<events::RobotRobotCollision>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<gbp_config::Config>,
) {
    for event in evr_robots_collided.read() {
        let material = materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        });

        let aabb = &event.intersection;
        let center = Vec3::new(aabb.mins.x + aabb.maxs.x, 0.0, aabb.mins.y + aabb.maxs.y) / 2.0;
        // let half_size = Vec3::new(aabb.maxs.x - aabb.mins.x, 0.0, aabb.maxs.y -
        // aabb.mins.y) / 2.0;
        let cuboid = Cuboid::from_size(Vec3::new(
            aabb.maxs.x - aabb.mins.x,
            -2.0,
            aabb.maxs.y - aabb.mins.y,
        ));

        let initial_visibility = if config.visualisation.draw.robot_robot_collisions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let mesh = meshes.add(cuboid);
        commands.spawn((
            PbrBundle {
                mesh,
                visibility: initial_visibility,
                material,
                transform: Transform::from_translation(center),
                ..default()
            },
            crate::simulation_loader::Reloadable,
            markers::RobotRobotCollision,
        ));
    }
}

fn toggle_visibility_of_robot_robot_collisions(
    mut q_robot_collisions: Query<&mut Visibility, With<markers::RobotRobotCollision>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        if matches!(event.setting, gbp_config::DrawSetting::RobotRobotCollisions) {
            for mut visibility in &mut q_robot_collisions {
                *visibility = if *visibility == Visibility::Visible {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

fn render_robot_environment_collisions(
    mut commands: Commands,
    mut evr_robots_collided: EventReader<events::RobotEnvironmentCollision>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    config: Res<gbp_config::Config>,
    // mut q_obstacles: Query<&mut Handle<StandardMaterial>,
    // With<crate::environment::ObstacleMarker>>,
) {
    for event in evr_robots_collided.read() {
        let material = materials.add(StandardMaterial {
            base_color: Color::RED,
            ..default()
        });

        // if let Ok(mut material) = q_obstacles.get_mut(event.obstacle) {
        //     println!("changing material of {:?}", event.obstacle);
        //     *material = material.clone();
        // }

        let aabb = &event.intersection;
        let center = Vec3::new(aabb.mins.x + aabb.maxs.x, 0.0, aabb.mins.y + aabb.maxs.y) / 2.0;
        // let half_size = Vec3::new(aabb.maxs.x - aabb.mins.x, 0.0, aabb.maxs.y -
        // aabb.mins.y) / 2.0;
        let cuboid = Cuboid::from_size(Vec3::new(
            aabb.maxs.x - aabb.mins.x,
            2.0,
            aabb.maxs.y - aabb.mins.y,
        ));

        let initial_visibility = if config.visualisation.draw.robot_environment_collisions {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        let mesh = meshes.add(cuboid);
        commands.spawn((
            PbrBundle {
                mesh,
                material,
                visibility: initial_visibility,
                transform: Transform::from_translation(center),
                ..default()
            },
            crate::simulation_loader::Reloadable,
            markers::RobotEnvironmentCollision,
        ));
    }
}

fn toggle_visibility_of_robot_environment_collisions(
    mut q_robot_collisions: Query<&mut Visibility, With<markers::RobotEnvironmentCollision>>,
    mut evr_draw_settings: EventReader<crate::input::DrawSettingsEvent>,
) {
    for event in evr_draw_settings.read() {
        // println!("draw setting event: {:?}", event);
        if matches!(
            event.setting,
            gbp_config::DrawSetting::RobotEnvironmentCollisions
        ) {
            for mut visibility in &mut q_robot_collisions {
                *visibility = if *visibility == Visibility::Visible {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
        }
    }
}

fn on_obstacle_clicked_on(
    mut evr_obstacle_clicked_on: EventReader<
        crate::environment::map_generator::events::ObstacleClickedOn,
    >,
    robot_environment_collisions: Res<resources::RobotEnvironmentCollisions>,
) {
    use colored::Colorize;
    for event in evr_obstacle_clicked_on.read() {
        // print all the robots that have hit the obstacle
        let obstacle_entity: Entity = event.0;
        println!("obstacle: {:?}", obstacle_entity);
        if let Some(robot_entities) =
            robot_environment_collisions.robots_collided_with(obstacle_entity)
        {
            println!("collisions: {}", robot_entities.len());
            for robot_entity in robot_entities {
                println!("  robot: {}", format!("{:?}", robot_entity).red());
            }
        }

        robot_environment_collisions
            .collisions()
            .filter(|((_, obstacle), aabbs)| *obstacle == obstacle_entity && !aabbs.is_empty())
            .for_each(|((robot, _), aabbs)| {
                println!("  robot: {}", format!("{:?}", robot).red());
                for aabb in aabbs {
                    println!("    {:?}", aabb);
                }
            });
    }
}

fn record_aabb_when_two_robots_collide(
    mut evr_robots_collided: EventReader<events::RobotRobotCollision>,
    mut robot_collisions: ResMut<resources::RobotRobotCollisions>,
) {
    for event in evr_robots_collided.read() {
        robot_collisions.record_collision(event);
    }
}

fn record_aabb_when_robot_collides_with_environment(
    mut evr_robots_collided: EventReader<events::RobotEnvironmentCollision>,
    mut robot_collisions: ResMut<resources::RobotEnvironmentCollisions>,
) {
    for event in evr_robots_collided.read() {
        robot_collisions.record_collision(event);
    }
}
