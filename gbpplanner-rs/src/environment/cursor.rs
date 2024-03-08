use bevy::{prelude::*, window::PrimaryWindow};

use crate::asset_loader::SceneAssets;

use super::camera::MainCamera;

pub struct CursorToGroundPlugin;

impl Plugin for CursorToGroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorCoordinates>()
            .add_systems(Startup, init_ground_plane)
            .add_systems(Update, cursor_to_ground_plane);
    }
}

// Everything below this is taken from
// https://bevy-cheatbook.github.io/cookbook/cursor2world.html
// Only with minor modifications

/// Here we will store the position of the mouse cursor on the 3D ground plane.
#[derive(Resource, Default)]
pub struct CursorCoordinates {
    // Global (world-space) coordinates
    global: Vec3,
    // Local (relative to the ground plane) coordinates
    local: Vec2,
}

impl CursorCoordinates {
    /// Get the global (world-space) coordinates of the cursor
    pub fn global(&self) -> Vec3 {
        self.global
    }

    /// Get the local (relative to the ground plane) coordinates of the cursor
    pub fn local(&self) -> Vec2 {
        self.local
    }
}

/// Used to help identify our ground plane
#[derive(Component)]
struct InvisibleGroundPlane;

fn init_ground_plane(mut commands: Commands, scene_assets: Res<SceneAssets>) {
    // Spawn the invisible ground plane
    commands.spawn((
        InvisibleGroundPlane,
        PbrBundle {
            transform: Transform::default(),
            mesh: scene_assets.meshes.plane.clone(),
            //.with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            material: scene_assets.materials.waypoint.clone(),
            ..default()
        },
    ));
}

fn cursor_to_ground_plane(
    mut ground_coords: ResMut<CursorCoordinates>,
    // Query to get the window (so we can read the current cursor position)
    // (we will only work with the primary window)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // Query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    // Query to get ground plane's transform
    q_plane: Query<&GlobalTransform, With<InvisibleGroundPlane>>,
) {
    // Get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // Ditto for the ground plane's transform
    let ground_transform = q_plane.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // Check if the cursor is inside the window and get its position
    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    // Mathematically, we can represent the ground as an infinite flat plane.
    // To do that, we need a point (to position the plane) and a normal vector
    // (the "up" direction, perpendicular to the ground plane).

    // We can get the correct values from the ground entity's GlobalTransform
    let plane_origin = ground_transform.translation();
    let plane = Plane3d::new(ground_transform.up());

    // Ask Bevy to give us a ray pointing from the viewport (screen) into the world
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    // Do a ray-plane intersection test, giving us the distance to the ground
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        // If the ray does not intersect the ground
        // (the camera is not looking towards the ground), we can't do anything
        return;
    };

    // Use the distance to compute the actual point on the ground in world-space
    let global_cursor = ray.get_point(distance);

    ground_coords.global = global_cursor;

    // Uo compute the local coordinates, we need the inverse of the plane's transform
    let inverse_transform_matrix = ground_transform.compute_matrix().inverse();
    let local_cursor = inverse_transform_matrix.transform_point3(global_cursor);

    // We can discard the Y coordinate, because it should always be zero
    // (our point is supposed to be on the plane)
    ground_coords.local = local_cursor.xz();
}
