use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Velocity {
    pub value: Vec3,
}

impl Velocity {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct Acceleration {
    pub value: Vec3,
}

impl Acceleration {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct AngularVelocity {
    pub value: Vec3,
}

impl AngularVelocity {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Component, Debug)]
pub struct AngularAcceleration {
    pub value: Vec3,
}

impl AngularAcceleration {
    pub fn new(value: Vec3) -> Self {
        Self { value }
    }
}

#[derive(Bundle)]
pub struct LinearMovementBundle {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

impl Default for LinearMovementBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity::new(Vec3::ZERO),
            acceleration: Acceleration::new(Vec3::ZERO),
        }
    }
}

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

// #[derive(Bundle)]
// pub struct MovingObjectBundle {
//     pub velocity: Velocity,
//     pub acceleration: Acceleration,
//     pub angular_velocity: AngularVelocity,
//     pub angular_acceleration: AngularAcceleration,
//     pub model: SceneBundle,
// }

#[derive(Bundle)]
pub struct MovingObjectBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
    pub model: SceneBundle,
}

impl Default for MovingObjectBundle {
    fn default() -> Self {
        Self {
            linear_movement: LinearMovementBundle::default(),
            angular_movement: AngularMovementBundle::default(),
            model: SceneBundle::default(),
        }
    }
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_velocity,
                update_position,
                update_angular_velocity,
                update_rotation,
            ),
        );
    }
}

fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.value += acceleration.value * time.delta_seconds();
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.value * time.delta_seconds();
    }
}

fn update_angular_velocity(
    mut query: Query<(&AngularAcceleration, &mut AngularVelocity)>,
    time: Res<Time>,
) {
    for (angular_acceleration, mut angular_velocity) in query.iter_mut() {
        angular_velocity.value += angular_acceleration.value * time.delta_seconds();
    }
}

fn update_rotation(mut query: Query<(&AngularVelocity, &mut Transform)>, time: Res<Time>) {
    for (angular_velocity, mut transform) in query.iter_mut() {
        let q = Quat::from_euler(
            EulerRot::XYZ,
            angular_velocity.value.x * time.delta_seconds(),
            angular_velocity.value.y * time.delta_seconds(),
            angular_velocity.value.z * time.delta_seconds(),
        );
        transform.rotation = q * transform.rotation;
    }
}
