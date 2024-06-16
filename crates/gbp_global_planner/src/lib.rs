//! Global path planning module

pub mod rrtstar;

use std::sync::Arc;

use bevy::{
    ecs::{component::Component, entity::Entity, system::Resource},
    math::Vec2,
    tasks::Task,
};
use delegate::delegate;
use derive_more::Index;
use parry2d::{
    na::{self, Isometry2, Vector2},
    query::intersection_test,
    shape,
};
use rand::{
    distributions::{Distribution, Uniform},
    RngCore,
};

/// **Bevy** [`Resource`] for storing an RRT* Tree
/// Simply a wrapper for [`rrt::rrtstar::Tree`]
#[allow(unused)]
#[derive(Debug, Resource, Default)]
pub struct RRTStarTree(rrt::rrtstar::Tree<f64, f32>);

/// **Bevy** [`Resource`] for storing a path
/// Simply a wrapper for a list of [`Vec2`] points
#[derive(Debug, Resource, Default, Index)]
pub struct Path(pub Vec<Vec2>);

impl Path {
    delegate! {
        to self.0 {
            #[call(len)]
            pub fn len(&self) -> usize;

            #[allow(unused)]
            #[call(clear)]
            pub fn clear(&mut self);

            #[allow(unused)]
            #[call(push)]
            pub fn push(&mut self, point: Vec2);

            #[allow(unused)]
            #[call(contains)]
            pub fn contains(&self, point: &Vec2) -> bool;
        }
    }

    pub fn assign(&mut self, path: Vec<Vec2>) {
        self.0 = path;
    }

    #[allow(unused)]
    fn euclidean_length(&self) -> f32 {
        let mut length = 0.0;
        for i in 0..self.len() - 1 {
            length += (self[i] - self[i + 1]).length();
        }
        length
    }
}

/// Possible pathfinding errors
#[derive(Debug)]
pub enum PathfindingError {
    ReachedMaxIterations,
}

/// **Bevy** [`Component`] for storing the pathfinding task
#[derive(Component, Debug)]
pub struct PathfindingTask(pub Task<Result<Path, PathfindingError>>);

/// **Bevy** [`Component`] for storing the pathfinding task
#[derive(Component, Debug)]
pub struct PathfindingTaskTree(pub Task<Result<Tree, PathfindingError>>);

/// **Bevy** marker [`Component`] for attaching a [`PathFindingTask`]
#[derive(Component, Debug)]
pub struct PathFinder;

/// A Collider element
#[derive(Clone)]
pub struct Collider {
    /// The **Bevy** [`Entity`] associated with the collider
    pub associated_mesh: Option<Entity>,
    /// Global translation and rotation of the collider
    pub isometry: Isometry2<f32>,
    /// The shape of the collider
    pub shape: Arc<dyn shape::Shape>,
}

impl Collider {
    #[inline]
    pub fn aabb(&self) -> parry2d::bounding_volume::Aabb {
        self.shape.compute_aabb(&self.isometry)
    }
}
/// **Bevy** [`Resource`] for storing a list of colliders
#[derive(Resource, Default, Clone)]
pub struct Colliders(Vec<Collider>);

impl Colliders {
    delegate! {
        to self.0 {
            #[call(iter)]
            pub fn iter(&self) -> impl Iterator<Item = &Collider>;

            #[call(len)]
            pub fn len(&self) -> usize;

            #[call(is_empty)]
            pub fn is_empty(&self) -> bool;

            #[call(clear)]
            pub fn clear(&mut self);
        }
    }

    pub fn push(
        &mut self,
        associated_mesh: Option<Entity>,
        position: Isometry2<f32>,
        shape: Arc<dyn shape::Shape>,
    ) {
        self.0.push(Collider {
            associated_mesh,
            isometry: position,
            shape,
        });
    }
}

struct CollisionProblem {
    colliders: Colliders,
    collision_checker: shape::Ball,
}

impl CollisionProblem {
    fn new(colliders: Colliders) -> Self {
        let ball = shape::Ball::new(0.1f32);
        Self {
            colliders,
            collision_checker: ball,
        }
    }

    fn with_collision_radius(mut self, radius: f32) -> Self {
        let ball = shape::Ball::new(radius);
        self.collision_checker = ball;
        self
    }

    fn is_feasible(&self, point: &[f64]) -> bool {
        // place the intersection ball at the point
        let ball_pos = Isometry2::new(Vector2::new(point[0] as f32, point[1] as f32), na::zero());

        let mut intersecting = false;

        for collider in self.colliders.iter() {
            let isometry = collider.isometry;
            let shape = &collider.shape;
            intersecting = intersection_test(
                &ball_pos,
                &self.collision_checker,
                &isometry,
                shape.as_ref(),
            )
            .expect("Correct shapes should have been given.");
            if intersecting {
                break;
            }
        }

        // return true if not intersecting
        !intersecting
    }

    fn random_sample(&self, mut rng: &mut dyn RngCore) -> Vec<f64> {
        let between = Uniform::new(-2000.0, 2000.0);
        // let mut rng = rng;
        vec![between.sample(&mut rng), between.sample(&mut rng)]
    }
}
