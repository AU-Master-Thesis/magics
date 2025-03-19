use bevy::prelude::*;

use super::camera::MainCamera;
use crate::asset_loader::AssetLibrary;

/// Plugin for handling cursor interaction with the ground plane
pub struct CursorToGroundPlugin;

impl Plugin for CursorToGroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorCoordinates>()
            .add_systems(Startup, spawn_invisible_ground_plane)
            .add_systems(Update, cursor_to_ground_plane);
    }
}

/// Resource storing the position of the mouse cursor on the 3D ground plane
#[derive(Resource, Default)]
pub struct CursorCoordinates {
    /// Global (world-space) coordinates
    global: Vec3,
    /// Local (relative to the ground plane) coordinates
    local: Vec2,
}

impl CursorCoordinates {
    /// Get the global (world-space) coordinates of the cursor
    pub const fn global(&self) -> Vec3 {
        self.global
    }

    /// Get the local (relative to the ground plane) coordinates of the cursor
    pub const fn local(&self) -> Vec2 {
        self.local
    }
}

/// Component marker for the invisible ground plane
#[derive(Component)]
struct InvisibleGroundPlane;

/// System to spawn an invisible ground plane for cursor interaction
fn spawn_invisible_ground_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_library: Option<Res<AssetLibrary>>,
) {
    // Create a basic plane mesh if not available in asset library
    let mesh = if let Some(asset_lib) = asset_library {
        asset_lib.get_mesh("ground_plane")
            .cloned()
            .unwrap_or_else(|| meshes.add(shape::Plane::from_size(1000.0).into()))
    } else {
        meshes.add(shape::Plane::from_size(1000.0).into())
    };
    
    // Create a transparent material for the invisible plane
    let material = materials.add(StandardMaterial {
        base_color: Color::rgba(1.0, 1.0, 1.0, 0.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    
    commands.spawn((
        InvisibleGroundPlane,
        PbrBundle {
            transform: Transform::default(),
            mesh,
            material,
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

/// System to update cursor coordinates based on intersection with ground plane
fn cursor_to_ground_plane(
    mut ground_coords: ResMut<CursorCoordinates>,
    // Query to get the window (so we can read the current cursor position)
    q_window: Query<&Window>,
    // Query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    // Query to get ground plane's transform
    q_plane: Query<&GlobalTransform, With<InvisibleGroundPlane>>,
) {
    // Get the camera info and transform
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };

    // Get the ground plane's transform
    let Ok(ground_transform) = q_plane.get_single() else {
        return;
    };

    // Get the primary window
    let Ok(window) = q_window.get_single() else {
        return;
    };

    // Check if the cursor is inside the window and get its position
    let Some(cursor_position) = window.cursor_position() else {
        // If the cursor is not inside the window, we can't do anything
        return;
    };

    // Mathematically, we can represent the ground as an infinite flat plane.
    // We need a point (to position the plane) and a normal vector (the "up" direction)
    let plane_origin = ground_transform.translation();
    let plane = Plane3d::new(ground_transform.up());

    // Get a ray pointing from the viewport (screen) into the world
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Do a ray-plane intersection test, giving us the distance to the ground
    let Some(distance) = ray.intersect_plane(plane_origin, plane) else {
        // If the ray does not intersect the ground plane, we can't do anything
        return;
    };

    // Use the distance to compute the actual point on the ground in world-space
    let global_cursor = ray.get_point(distance);
    ground_coords.global = global_cursor;

    // To compute the local coordinates, we need the inverse of the plane's transform
    let inverse_transform_matrix = ground_transform.compute_matrix().inverse();
    let local_cursor = inverse_transform_matrix.transform_point3(global_cursor);

    // We can discard the Y coordinate, because it should always be zero
    // (our point is supposed to be on the plane)
    ground_coords.local = local_cursor.xz();
}
