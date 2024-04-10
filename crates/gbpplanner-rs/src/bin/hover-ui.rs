#![allow(missing_docs)]

use bevy::{prelude::*, window::PrimaryWindow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move_hover_window);

    app.run();
    Ok(())
}

// #[inline]
// fn px(logical_pixels: f32) -> Val {
//     Val::Px(logical_pixels)
// }

trait ValExt {
    fn px(self) -> Val;
    fn percent(self) -> Val;
}

impl ValExt for f32 {
    fn px(self) -> Val {
        Val::Px(self)
    }

    fn percent(self) -> Val {
        if !(0.0..=100.0).contains(&self) {
            panic!("value outside interval [0.0, 100.0]");
        }
        Val::Percent(self)
    }
}

impl ValExt for i32 {
    fn px(self) -> Val {
        Val::Px(self as f32)
    }

    fn percent(self) -> Val {
        if !(0..=100).contains(&self) {
            panic!("value outside interval [0, 100]");
        }

        Val::Percent(self as f32)
    }
}

fn relative_cursor_position_in_window(window: &Window) -> Option<Vec2> {
    let cursor_position = window.cursor_position()?;
    let relative_cursor_position = Vec2::new(
        cursor_position.x / window.width(),
        cursor_position.y / window.height(),
    );

    Some(relative_cursor_position)
}

#[derive(Component)]
pub struct HoverWindow;

fn setup(mut commands: Commands, query_window: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = query_window.get_single() else {
        error!("no primary window");
        return;
    };

    let Some(relative_cursor_position) = relative_cursor_position_in_window(window) else {
        warn!("cursor position is outside primary window");
        return;
    };

    commands.spawn(Camera2dBundle::default());

    let root = commands
        .spawn((HoverWindow, NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: 5.0.percent(),
                // top: percent(5.0),
                left: 5.0.percent(),

                width: 200.px(),
                height: 400.px(),
                ..default()
            },
            background_color: Color::RED.into(),
            border_color: Color::GREEN.into(),
            ..default()
        }))
        .id();

    // all the different combinations of border edges
    // these correspond to the labels above
    let borders = [
        UiRect::default(),
        UiRect::all(Val::Px(10.)),
        UiRect::left(Val::Px(10.)),
        UiRect::right(Val::Px(10.)),
        UiRect::top(Val::Px(10.)),
        UiRect::bottom(Val::Px(10.)),
        UiRect::horizontal(Val::Px(10.)),
        UiRect::vertical(Val::Px(10.)),
        UiRect {
            left: Val::Px(10.),
            top: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            left: Val::Px(10.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            right: Val::Px(10.),
            top: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            right: Val::Px(10.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            right: Val::Px(10.),
            top: Val::Px(10.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            left: Val::Px(10.),
            top: Val::Px(10.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            left: Val::Px(10.),
            right: Val::Px(10.),
            top: Val::Px(10.),
            ..Default::default()
        },
        UiRect {
            left: Val::Px(10.),
            right: Val::Px(10.),
            bottom: Val::Px(10.),
            ..Default::default()
        },
    ];

    let title = commands
        .spawn(TextBundle::from_section("variable", TextStyle {
            font_size: 10.0,
            ..Default::default()
        }))
        .id();

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .push_children(&[title])
        .id();

    commands.entity(root).add_child(container);
}

fn primary_window_exists(query_window: Query<&Window, With<PrimaryWindow>>) -> bool {
    query_window.get_single().is_ok()
}

fn move_hover_window(
    mut query_hover_window: Query<&mut Transform, With<HoverWindow>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = query_window.single();

    let Some(relative_cursor_position) = relative_cursor_position_in_window(window) else {
        warn!("cursor outside primary window");
        return;
    };

    let mut transform = query_hover_window.single_mut();

    let x = relative_cursor_position.x * window.width();
    let y = relative_cursor_position.y * window.height();

    info!("x = {}, y = {}", x, y);

    transform.translation = Vec3::new(x, 0.0, y);
}
