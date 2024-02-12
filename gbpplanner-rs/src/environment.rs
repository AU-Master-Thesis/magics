use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use catppuccin::Flavour;

pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let (r, g, b) = Flavour::Macchiato.base().into();
        app.insert_resource(ClearColor(Color::rgb_u8(r, g, b)))
            .insert_resource(AmbientLight {
                color: Color::default(),
                brightness: 0.5,
            })
            .add_plugins(InfiniteGridPlugin)
            .add_systems(Startup, build_environment);
    }
}

fn build_environment(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            // shadow_color: None,
            major_line_color: {
                let (r, g, b) = Flavour::Macchiato.crust().into();
                Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            minor_line_color: {
                let (r, g, b) = Flavour::Macchiato.crust().into();
                Color::rgba_u8(r, g, b, (0.25 * 255.0) as u8)
            },
            x_axis_color: {
                let (r, g, b) = Flavour::Macchiato.maroon().into();
                Color::rgb_u8(r, g, b)
                // Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            z_axis_color: {
                let (r, g, b) = Flavour::Macchiato.blue().into();
                Color::rgb_u8(r, g, b)
                // Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
            },
            ..default()
        },
        ..default()
    });

    // commands.spawn(DirectionalLightBundle {
    //     transform: Transform::from_translation(Vec3::X * 15.0 + Vec3::Z * 20.0)
    //         .looking_at(Vec3::ZERO, Vec3::Z),
    //     ..default()
    // });

    // let mat = standard_materials.add(StandardMaterial::default());

    // // cube
    // commands.spawn(PbrBundle {
    //     material: mat.clone(),
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     transform: Transform {
    //         translation: Vec3::new(3., 4., 0.),
    //         rotation: Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize()),
    //         scale: Vec3::splat(1.5),
    //     },
    //     ..default()
    // });

    // commands.spawn(PbrBundle {
    //     material: mat.clone(),
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
    //     transform: Transform::from_xyz(0.0, 2.0, 0.0),
    //     ..default()
    // });

    // read image, and display as height-map
    // black is height of 1, white is height of 0
}
