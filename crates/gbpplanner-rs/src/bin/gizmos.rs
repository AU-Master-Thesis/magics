#![allow(dead_code, unused_macros, missing_docs)]

use bevy::{prelude::*, window::WindowResized};
// use std::f32::consts::{PI, TAU};

fn main() {
    let (width, height) = (1080.0, 780.0);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (width, height).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(WindowResolution { width, height })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                on_resize_system,
                // draw_cartesian_coordinate_system,
                draw,
            ),
        )
        .run();
}

#[derive(Resource)]
struct WindowResolution {
    width:  f32,
    height: f32,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn draw_cartesian_coordinate_system(mut gizmos: Gizmos, window_resolution: Res<WindowResolution>) {
    // let scale = 100.0;
    gizmos.line_2d(
        -window_resolution.width * Vec2::X,
        window_resolution.width * Vec2::X,
        Color::RED,
    );
    gizmos.line_2d(
        -window_resolution.height * Vec2::Y,
        window_resolution.height * Vec2::Y,
        Color::GREEN,
    );
}

struct Line {
    start: Vec2,
    end:   Vec2,
}

macro_rules! line {
    ($x1:expr, $y1:expr => $x2:expr, $y2:expr) => {
        Line {
            start: Vec2::new($x1, $y1),
            end:   Vec2::new($x2, $y2),
        }
    };
}

struct LineSegment3d {
    start: Vec3,
    end:   Vec3,
}

macro_rules! line3d {
    ($x1:expr, $y1:expr => $x2:expr, $y2:expr) => {
        LineSegment3d {
            start: Vec3::new($x1, 0.0, $y1),
            end:   Vec3::new($x2, 0.0, $y2),
        }
    };
}

fn draw(
    mut gizmos: Gizmos,
    //  time: Res<Time>
) {
    // gizmos.arrow_2d(Vec2::ZERO, Vec2::X, Color::GREEN);
    let mut draw_line = |line: &Line| {
        gizmos.line_2d(line.start, line.end, Color::WHITE);
    };

    let line0 = line!(35.0, 35. => 20.0, 50.0);
    let line1 = line!(140.0, 150.0 => 140., 190.);
    let line2 = line!(300.0, 20.0 => 320., 35.);
    draw_line(&line0);
    draw_line(&line1);
    draw_line(&line2);

    gizmos.line_2d(line0.start, line1.start, Color::BLUE);
    gizmos.line_2d(line1.start, line2.start, Color::BLUE);

    gizmos.line_2d(line0.end, line1.end, Color::BLUE);
    gizmos.line_2d(line1.end, line2.end, Color::BLUE);

    let mut draw_line3d = |line: &LineSegment3d| {
        gizmos.line(line.start, line.end, Color::YELLOW);
    };

    let line3d0 = line3d!(35.0, 35. => 20.0, 50.0);
    let line3d1 = line3d!(140.0, 150.0 => 140., 190.);
    let line3d2 = line3d!(300.0, 20.0 => 320., 35.);

    draw_line3d(&line3d0);
    draw_line3d(&line3d1);
    draw_line3d(&line3d2);

    gizmos.linestrip_2d([Vec2::ZERO, Vec2::X, Vec2::Y], Color::GREEN);

    // gizmos.line_gradient_2d(, , , )
}

/// This system shows how to respond to a window being resized.
/// Whenever the window is resized, the text will update with the new
/// resolution.
fn on_resize_system(mut resize_reader: EventReader<WindowResized>, mut window_resolution: ResMut<WindowResolution>) {
    for e in resize_reader.read() {
        window_resolution.width = e.width;
        window_resolution.height = e.height;
        // info!("window resolution changed: ({}, {})", e.width, e.height);
    }
}
