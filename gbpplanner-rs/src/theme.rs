use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_infinite_grid::InfiniteGridSettings;
use catppuccin::Flavour;

use crate::factorgraph::{Factor, Line, Variable};

/// Catppuccin Bevy theme wrapper
#[derive(Resource, Debug)]
pub struct CatppuccinTheme {
    pub flavour: Flavour,
}

impl Default for CatppuccinTheme {
    fn default() -> Self {
        Self {
            flavour: Flavour::Macchiato,
        }
    }
}

impl CatppuccinTheme {
    pub fn grid_colour(&self, windows: Query<&Window>) -> Color {
        let window = windows
            .get_single()
            .expect("There should be exactly one window");
        let colour = match window.window_theme {
            Some(WindowTheme::Light) => self.flavour.text(),
            Some(WindowTheme::Dark) => self.flavour.crust(),
            None => self.flavour.text(),
        };

        let (r, g, b) = colour.into();
        Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
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
            .init_resource::<CatppuccinTheme>()
            .add_systems(
                Startup,
                set_theme(WindowTheme::Dark).run_if(theme_is_not_set),
            )
            .add_systems(
                Update,
                (
                    toggle_theme,
                    handle_clear_color,
                    handle_infinite_grid,
                    handle_variables,
                    handle_factors,
                    handle_lines,
                ),
            );
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
    mut catppuccin_theme: ResMut<CatppuccinTheme>,
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
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
) {
    for _ in theme_toggled_event.read() {
        let (r, g, b) = catppuccin_theme.flavour.crust().into();
        *clear_color = ClearColor(Color::rgb_u8(r, g, b));
    }
}

fn handle_infinite_grid(
    catppuccin_theme: Res<CatppuccinTheme>,
    windows: Query<&Window>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
    mut query_infinite_grid: Query<&mut InfiniteGridSettings>,
) {
    let grid_colour = catppuccin_theme.grid_colour(windows);
    for _ in theme_toggled_event.read() {
        if let Ok(mut settings) = query_infinite_grid.get_single_mut() {
            settings.major_line_color = grid_colour.with_a(0.5);
            settings.minor_line_color = grid_colour.with_a(0.25);
            let (r, g, b) = catppuccin_theme.flavour.maroon().into();
            settings.x_axis_color = Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8);
            let (r, g, b) = catppuccin_theme.flavour.blue().into();
            settings.z_axis_color = Color::rgba_u8(r, g, b, (0.1 * 255.0) as u8);
        }
    }
}

fn handle_variables(
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_variable: Query<&mut Handle<StandardMaterial>, With<Variable>>,
) {
    for _ in theme_toggled_event.read() {
        for handle in query_variable.iter_mut() {
            if let Some(material) = materials.get_mut(handle.clone()) {
                let alpha = 0.75;
                material.base_color = {
                    let (r, g, b) = catppuccin_theme.flavour.blue().into();
                    Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
                };
            }
        }
    }
}

fn handle_factors(
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_factor: Query<&mut Handle<StandardMaterial>, With<Factor>>,
) {
    for _ in theme_toggled_event.read() {
        for handle in query_factor.iter_mut() {
            if let Some(material) = materials.get_mut(handle.clone()) {
                let alpha = 0.75;
                material.base_color = {
                    let (r, g, b) = catppuccin_theme.flavour.green().into();
                    Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
                };
            }
        }
    }
}

fn handle_lines(
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_toggled_event: EventReader<ThemeToggledEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_line: Query<&mut Handle<StandardMaterial>, With<Line>>,
) {
    for _ in theme_toggled_event.read() {
        for handle in query_line.iter_mut() {
            if let Some(material) = materials.get_mut(handle.clone()) {
                let alpha = 0.75;
                material.base_color = {
                    let (r, g, b) = catppuccin_theme.flavour.text().into();
                    Color::rgba_u8(r, g, b, (alpha * 255.0) as u8)
                };
            }
        }
    }
}
