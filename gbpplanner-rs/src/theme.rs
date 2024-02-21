use bevy::prelude::*;
use bevy::window::WindowTheme;
use catppuccin::Flavour;

/// Theme event
#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeEvent;

/// Theming plugin
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThemeEvent>()
            .add_systems(Update, toggle_theme);
    }
}

fn toggle_theme(
    mut windows: Query<&mut Window>,
    mut theme_event: EventReader<ThemeEvent>,
) {
    let mut window = windows.single_mut();
    for _ in theme_event.read() {
        if let Some(current_theme) = window.window_theme {
            window.window_theme = match current_theme {
                WindowTheme::Light => Some(WindowTheme::Dark),
                WindowTheme::Dark => Some(WindowTheme::Light),
            };
        }
    }
    // if input.just_pressed(KeyCode::F) {
    //     let mut window = windows.single_mut();

    //     if let Some(current_theme) = window.window_theme {
    //         window.window_theme = match current_theme {
    //             WindowTheme::Light => Some(WindowTheme::Dark),
    //             WindowTheme::Dark => Some(WindowTheme::Light),
    //         };
    //     }
    // }
}
