//! Movement components and systems for the Magics planner UI
//!
//! This module provides movement functionality including velocity, acceleration,
//! position updates, rotation, and movement bundles.

use bevy::prelude::*;

/// Component for entity velocity
#[derive(Component, Debug, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

/// Component for entity acceleration
#[derive(Component, Debug)]
pub struct Acceleration {
    pub value: Vec3,
}

impl Acceleration {
    pub const fn new(value: Vec3) -> Self {
        Self { value }
    }
}

/// Component for entity angular velocity
#[derive(Component, Debug)]
pub struct AngularVelocity {
    pub value: Vec3,
}

impl AngularVelocity {
    pub const fn new(value: Vec3) -> Self {
        Self { value }
    }
}

/// Component for entity angular acceleration
#[derive(Component, Debug)]
pub struct AngularAcceleration {
    pub value: Vec3,
}

impl AngularAcceleration {
    pub const fn new(value: Vec3) -> Self {
        Self { value }
    }
}

/// Component for orbit movement
#[derive(Component, Debug)]
pub struct Orbit {
    /// Point to rotate around
    pub origin: Vec3,
    /// Orbit radius
    pub radius: f32,
}

impl Orbit {
    pub const fn new(origin: Vec3, radius: f32) -> Self {
        Self { origin, radius }
    }
}

/// Bundle for linear movement components
#[derive(Bundle)]
pub struct LinearMovementBundle {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

impl Default for LinearMovementBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity(Vec3::ZERO),
            acceleration: Acceleration::new(Vec3::ZERO),
        }
    }
}

/// Bundle for angular movement components
#[derive(Bundle)]
pub struct AngularMovementBundle {
    pub angular_velocity: AngularVelocity,
    pub angular_acceleration: AngularAcceleration,
}

impl Default for AngularMovementBundle {
    fn default() -> Self {
        Self {
            angular_velocity: AngularVelocity::new(Vec3::ZERO),
            angular_acceleration: AngularAcceleration::new(Vec3::ZERO),
        }
    }
}

/// Bundle for all movement components
#[derive(Bundle, Default)]
pub struct MovementBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
}

/// Bundle for moving objects with a 3D model
#[derive(Bundle, Default)]
pub struct MovingObjectBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
    pub model: SceneBundle,
}

/// Bundle for moving objects with a mesh
#[derive(Bundle, Default)]
pub struct MovingMeshBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
    pub model: PbrBundle,
}

/// Bundle for orbit movement
#[derive(Bundle)]
pub struct OrbitMovementBundle {
    pub angular_movement: AngularMovementBundle,
    pub orbit: Orbit,
}

impl Default for OrbitMovementBundle {
    fn default() -> Self {
        Self {
            angular_movement: AngularMovementBundle::default(),
            orbit: Orbit::new(Vec3::ZERO, 10.0),
        }
    }
}

/// Component marker for local movement
#[derive(Component)]
pub struct Local;

/// Plugin for movement functionality
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_velocity,
                update_position,
                update_position_local,
                update_position_local_orbit,
                update_angular_velocity,
                update_rotation,
                update_rotation_orbit,
            ),
        );
    }
}

/// Update velocity based on acceleration
fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in &mut query {
        velocity.0 += acceleration.value * time.delta_seconds();
    }
}

/// Update position based on velocity for global entities
#[allow(clippy::type_complexity)]
fn update_position(
    mut query: Query<(&Velocity, &mut Transform), (Without<Orbit>, Without<Local>)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in &mut query {
        if velocity.abs_diff_eq(Vec3::ZERO, f32::EPSILON) {
            continue;
        }
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

/// Update position based on velocity for local entities (no orbit)
#[allow(clippy::type_complexity)]
fn update_position_local(
    mut query: Query<(&Velocity, &mut Transform), (With<Local>, Without<Orbit>)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in &mut query {
        if velocity.abs_diff_eq(Vec3::ZERO, f32::EPSILON) {
            continue;
        }
        let mutation = transform.local_x() * velocity.x
            + transform.local_z() * velocity.z
            + transform.local_y() * velocity.y;

        transform.translation += mutation * time.delta_seconds();
    }
}

/// Update position based on velocity for local entities with orbit
fn update_position_local_orbit(
    mut query: Query<(&mut Orbit, &Velocity, &mut Transform), With<Local>>,
    time: Res<Time>,
) {
    // translate both the orbit.origin point and the transform
    for (mut orbit, velocity, mut transform) in &mut query {
        if velocity.abs_diff_eq(Vec3::ZERO, f32::EPSILON) {
            continue;
        }

        let source_z_direction = if f32::abs(transform.forward().dot(Vec3::Y)) > 0.5 {
            transform.up()
        } else {
            transform.forward()
        };

        let z_direction =
            Vec3::new(source_z_direction.x, 0.0, source_z_direction.z).normalize_or_zero();

        let from_local_translation =
            (transform.left() * velocity.x + z_direction * velocity.z) * time.delta_seconds();

        let zoom_direction = transform.forward();

        transform.translation +=
            from_local_translation + zoom_direction * velocity.y * time.delta_seconds();
        orbit.origin += from_local_translation;
    }
}

/// Update angular velocity based on angular acceleration
fn update_angular_velocity(
    mut query: Query<(&AngularAcceleration, &mut AngularVelocity)>,
    time: Res<Time>,
) {
    for (angular_acceleration, mut angular_velocity) in &mut query {
        angular_velocity.value += angular_acceleration.value * time.delta_seconds();
    }
}

/// Update rotation based on angular velocity for non-orbit entities
fn update_rotation(
    mut query: Query<(&AngularVelocity, &mut Transform), Without<Orbit>>,
    time: Res<Time>,
) {
    for (angular_velocity, mut transform) in &mut query {
        let q = Quat::from_euler(
            EulerRot::XYZ,
            angular_velocity.value.x * time.delta_seconds(),
            angular_velocity.value.y * time.delta_seconds(),
            angular_velocity.value.z * time.delta_seconds(),
        );
        transform.rotation = q * transform.rotation;
    }
}

/// Update rotation based on angular velocity for orbit entities
fn update_rotation_orbit(
    mut query: Query<(&Orbit, &AngularVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (orbit, angular_velocity, mut transform) in &mut query {
        if !transform.is_finite() {
            continue;
        }
        
        let yaw = Quat::from_axis_angle(Vec3::Y, -angular_velocity.value.x * time.delta_seconds());

        let pitch = Quat::from_axis_angle(
            *transform.right(),
            -angular_velocity.value.y * time.delta_seconds(),
        );

        transform.rotate_around(orbit.origin, yaw * pitch);
    }
}
