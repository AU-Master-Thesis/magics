use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
pub struct ShapesPlugin {
    colors: Colors,
}

impl ShapesPlugin {
    pub fn new(colors_arr: [Color; 4]) -> Self {
        Self {
            colors: Colors(colors_arr),
        }
    }
}

impl Plugin for ShapesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.colors)
            // .add_systems(Startup, (setup, set_colors).chain());
            .add_systems(Startup, setup)
            .add_systems(Update, set_colors);
    }
}

#[derive(Resource, Clone, Copy)]
struct Colors([Color; 4]);

fn set_colors(
    colors: Res<Colors>,
    mut query: Query<(&Transform, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Inside set_colors");
    for (i, (_, handle)) in query.iter_mut().enumerate() {
        // let material = materials.get_mut(handle).unwrap();
        if let Some(material) = materials.get_mut(handle) {
            info!("Setting color to {:?}", colors.0[i]);
            let color = colors.0[i];
            material.color = color;
        }
    }
}

// fn set_colors(query: Query<Entity>) {
//     info!("Inside set_colors");
// }

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(50.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
        ..default()
    });

    // Rectangle
    // commands.spawn(SpriteBundle {
    //     sprite: Sprite {
    //         color: Color::rgb(0.25, 0.25, 0.75),
    //         custom_size: Some(Vec2::new(50.0, 100.0)),
    //         ..default()
    //     },
    //     transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
    //     ..default()
    // });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 100.)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::rgb(0.25, 0.25, 0.75))),
        transform: Transform::from_translation(Vec3::new(-50., 0., 0.)),
        ..default()
    });

    // Quad
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(Vec2::new(50., 100.)).into())
            .into(),
        material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
        transform: Transform::from_translation(Vec3::new(50., 0., 0.)),
        ..default()
    });

    // Hexagon
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::RegularPolygon::new(50., 6).into()).into(),
        material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
        transform: Transform::from_translation(Vec3::new(150., 0., 0.)),
        ..default()
    });
}
