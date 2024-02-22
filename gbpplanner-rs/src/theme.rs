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

/// Signal that theme should be toggled
#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeEvent;

/// Signal after theme has been toggled
#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeToggledEvent;

/// Theming plugin
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ThemeEvent>()
            .add_event::<ThemeToggledEvent>()
            .init_resource::<CattpuccinTheme>()
            .add_systems(
                Startup,
                set_theme(WindowTheme::Dark).run_if(theme_is_not_set),
            )
            .add_systems(Update, (toggle_theme, handle_clear_color, test));
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
    mut theme_toggled_event: EventWriter<ThemeToggledEvent>,
) {
    let mut window = windows.single_mut();
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
            theme_toggled_event.send(ThemeToggledEvent);
        }
    }
}

fn handle_clear_color(
    mut clear_color: ResMut<ClearColor>,
    catppuccin_theme: Res<CattpuccinTheme>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
) {
    for _ in theme_toggled_event.read() {
        let (r, g, b) = catppuccin_theme.flavour.crust().into();
        *clear_color = ClearColor(Color::rgb_u8(r, g, b));
    }
}

fn test(mut theme_toggled_event: EventReader<ThemeToggledEvent>) {
    for _ in theme_toggled_event.read() {
        info!("TEST");
    }
}
