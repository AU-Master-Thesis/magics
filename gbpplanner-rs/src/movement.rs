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

// orbit flag
#[derive(Component, Debug)]
pub struct Orbit {
    // point to rotate around
    pub origin: Vec3,
    pub radius: f32,
}

impl Orbit {
    pub fn new(origin: Vec3, radius: f32) -> Self {
        Self { origin, radius }
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

#[derive(Bundle, Default)]
pub struct MovementBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
}

// impl Default for MovementBundle {
//     fn default() -> Self {
//         Self {
//             linear_movement: LinearMovementBundle::default(),
//             angular_movement: AngularMovementBundle::default(),
//         }
//     }
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

#[derive(Bundle, Default)]
pub struct MovingMeshBundle {
    pub linear_movement: LinearMovementBundle,
    pub angular_movement: AngularMovementBundle,
    pub model: PbrBundle,
}

// impl Default for MovingMeshBundle {
//     fn default() -> Self {
//         Self {
//             linear_movement: Default::default(),
//             angular_movement: Default::default(),
//             model: Default::default(),
//         }
//     }
// }

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

#[derive(Component)]
pub struct Local;

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

fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.value += acceleration.value * time.delta_seconds();
    }
}

#[allow(clippy::type_complexity)]
fn update_position(
    mut query: Query<(&Velocity, &mut Transform), (Without<Orbit>, Without<Local>)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        if velocity.value.abs_diff_eq(Vec3::ZERO, std::f32::EPSILON) {
            continue;
        }
        transform.translation += velocity.value * time.delta_seconds();
    }
}

#[allow(clippy::type_complexity)]
fn update_position_local(
    mut query: Query<(&Velocity, &mut Transform), (With<Local>, Without<Orbit>)>,
    time: Res<Time>,
) {
    for (velocity, mut transform) in query.iter_mut() {
        if velocity.value.abs_diff_eq(Vec3::ZERO, std::f32::EPSILON) {
            continue;
        }
        let mutation = transform.local_x() * velocity.value.x
            + transform.local_z() * velocity.value.z
            + transform.local_y() * velocity.value.y;

        transform.translation += mutation * time.delta_seconds();
    }
}

fn update_position_local_orbit(
    mut query: Query<(&mut Orbit, &Velocity, &mut Transform), With<Local>>,
    time: Res<Time>,
) {
    // translate both the orbit.origin point and the transform
    for (mut orbit, velocity, mut transform) in query.iter_mut() {
        if velocity.value.abs_diff_eq(Vec3::ZERO, std::f32::EPSILON) {
            continue;
        }

        let source_z_direction = if f32::abs(transform.forward().dot(Vec3::Y)) > 0.5 {
            transform.up()
        } else {
            transform.forward()
        };

        let z_direction =
            Vec3::new(source_z_direction.x, 0.0, source_z_direction.z).normalize_or_zero();

        // info!("velocity.value.y {:?}", velocity.value.y);

        let from_local_translation = (transform.left() * velocity.value.x
            + z_direction * velocity.value.z)
            * time.delta_seconds();

        // info!("from_local_translation.y {:?}", from_local_translation.y);

        let zoom_direction = transform.forward();

        transform.translation +=
            from_local_translation + zoom_direction * velocity.value.y * time.delta_seconds();
        orbit.origin += from_local_translation;
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

fn update_rotation(
    mut query: Query<(&AngularVelocity, &mut Transform), Without<Orbit>>,
    time: Res<Time>,
) {
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

fn update_rotation_orbit(
    mut query: Query<(&Orbit, &AngularVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (orbit, angular_velocity, mut transform) in query.iter_mut() {
        let yaw = Quat::from_axis_angle(Vec3::Y, angular_velocity.value.x * time.delta_seconds());
        let pitch = Quat::from_axis_angle(
            *transform.right(),
            -angular_velocity.value.y * time.delta_seconds(),
        );

        transform.rotate_around(orbit.origin, yaw * pitch);
        // transform.look_at(orbit.origin, Vec3::Z);
        // transform.translation = orbit.origin + q * (transform.translation - orbit.origin);
    }
}
