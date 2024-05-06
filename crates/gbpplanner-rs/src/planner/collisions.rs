use std::{collections::HashMap, time::Duration};

use bevy::{prelude::*, time::common_conditions::on_timer};
use parry2d::bounding_volume::BoundingVolume;

use super::{robot::Ball, RobotState};
use crate::simulation_loader::{LoadSimulation, ReloadSimulation};

#[derive(Default)]
pub struct RobotCollisionsPlugin;

impl RobotCollisionsPlugin {
    pub const UPDATE_EVERY: Duration = Duration::from_millis(250);
}

impl Plugin for RobotCollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RobotCollisions>()
            .add_systems(
                Update,
                update_robot_collisions.run_if(on_timer(Self::UPDATE_EVERY)),
            )
            .add_systems(
                PostUpdate,
                clear_robot_collisions
                    .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
            );
    }
}

fn clear_robot_collisions(mut robot_collisions: ResMut<RobotCollisions>) {
    robot_collisions.clear();
}

fn update_robot_collisions(
    mut robot_collisions: ResMut<RobotCollisions>,
    robots: Query<(Entity, &Transform, &Ball), With<RobotState>>,
    mut aabbs: Local<Vec<(Entity, parry2d::bounding_volume::Aabb)>>,
) {
    aabbs.clear();

    let iter = robots.iter().map(|(entity, tf, ball)| {
        let position = parry2d::na::Isometry2::translation(tf.translation.x, tf.translation.z); // bevy uses xzy coordinates
        (entity, ball.aabb(&position))
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
        let is_colliding = aabbs[r].1.intersects(&aabbs[c].1);
        robot_collisions.update(aabbs[r].0, aabbs[c].0, is_colliding);
    }
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
}

impl CollisionHistory {
    fn new() -> Self {
        Self {
            times: 0,
            state: CollisionState::Free,
        }
    }

    fn update(&mut self, is_colliding: bool) {
        match self.state {
            CollisionState::Colliding if is_colliding => {}
            CollisionState::Colliding if !is_colliding => {
                self.state = CollisionState::Free;
            }
            CollisionState::Free if !is_colliding => {}
            CollisionState::Free if is_colliding => {
                self.state = CollisionState::Colliding;
                self.times += 1;
            }

            _ => unreachable!(),
        }
    }

    fn collisions(&self) -> usize {
        self.times
    }
}

#[derive(Resource)]
pub struct RobotCollisions {
    // inner: HashMap<(usize, usize), CollisionHistory>,
    inner: HashMap<(Entity, Entity), CollisionHistory>,
}

impl RobotCollisions {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    fn update(&mut self, e1: Entity, e2: Entity, is_colliding: bool) {
        let entry = self
            .inner
            .entry((e1, e2))
            .or_insert(CollisionHistory::new());
        entry.update(is_colliding);
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

    /// PERF: cache the count un `update()`
    pub fn collisions(&self) -> usize {
        self.inner.values().map(|c| c.collisions()).sum::<usize>()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }
}

impl Default for RobotCollisions {
    fn default() -> Self {
        Self::new()
    }
}
