use bevy::prelude::*;
use bevy::window::WindowTheme;
use catppuccin::Flavour;

/// Catppuccin Bevy theme wrapper
#[derive(Resource, Debug)]
pub struct CattpuccinTheme {
    pub flavour: Flavour,
}

impl Default for CattpuccinTheme {
    fn default() -> Self {
        Self {
            flavour: Flavour::Macchiato,
        }
    }
}

impl CattpuccinTheme {
    pub fn new(flavour: Flavour) -> Self {
        Self { flavour }
    }
}

/// Theme event
#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeEvent;

/// Theming plugin
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThemeEvent>()
            .init_resource::<CattpuccinTheme>()
            .add_systems(
                Startup,
                set_theme(WindowTheme::Dark).run_if(theme_is_not_set),
            )
            .add_systems(Update, toggle_theme);
    }
}

fn theme_is_not_set(windows: Query<&Window>) -> bool {
    let window = windows.single();
    window.window_theme.is_none()
}

fn set_theme(theme: WindowTheme) -> impl FnMut(Query<&mut Window>) {
    move |mut windows: Query<&mut Window>| {
        let mut window = windows.single_mut();
        window.window_theme = Some(theme);
    }
}

fn toggle_theme(
    mut windows: Query<&mut Window>,
    mut theme_event: EventReader<ThemeEvent>,
    mut catppuccin_theme: ResMut<CattpuccinTheme>,
) {
    let mut window = windows.single_mut();
    info!("cattuccin_theme: {:?}", catppuccin_theme);
    for _ in theme_event.read() {
        if let Some(current_theme) = window.window_theme {
            window.window_theme = match current_theme {
                WindowTheme::Light => {
                    info!("Switching WindowTheme: Light -> Dark");
                    catppuccin_theme.flavour = match catppuccin_theme.flavour {
                        Flavour::Latte => Flavour::Macchiato,
                        _ => Flavour::Latte,
                    };
                    Some(WindowTheme::Dark)
                }
                WindowTheme::Dark => {
                    info!("Switching WindowTheme: Dark -> Light");
                    catppuccin_theme.flavour = match catppuccin_theme.flavour {
                        Flavour::Latte => Flavour::Macchiato,
                        _ => Flavour::Latte,
                    };
                    Some(WindowTheme::Light)
                }
            };
        }
    }
}
