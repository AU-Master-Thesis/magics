use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_egui::{
    egui::{
        self,
        epaint::Shadow,
        style::{HandleShape, Selection, WidgetVisuals, Widgets},
        Color32, Rounding, Stroke, Style, Visuals,
    },
    EguiContexts,
};
use bevy_infinite_grid::InfiniteGridSettings;
use catppuccin::{Colour, Flavour};

use crate::factorgraph::{Factor, Line, Variable};

/// Catppuccin **Bevy** theme wrapper
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

pub trait ColourExt {
    fn lightness(&self) -> f32;
}

impl ColourExt for Colour {
    fn lightness(&self) -> f32 {
        let (r, g, b) = Into::<(u8, u8, u8)>::into(*self);
        let average = (r as u16 + g as u16 + b as u16) / 3;
        average as f32 / 255.0
    }
}

pub trait CatppuccinThemeVisualsExt {
    fn catppuccin_light() -> Visuals {
        Self::catppuccin_flavour(Flavour::Latte)
    }
    fn catppuccin_dark() -> Visuals {
        Self::catppuccin_flavour(Flavour::Macchiato)
    }
    fn catppuccin_flavour(flavour: Flavour) -> Visuals;
}

pub trait CatppuccinThemeWidgetsExt {
    fn catppuccin_light() -> Widgets {
        Self::catppuccin_flavour(Flavour::Latte)
    }
    fn catppuccin_dark() -> Widgets {
        Self::catppuccin_flavour(Flavour::Macchiato)
    }
    fn catppuccin_flavour(flavour: Flavour) -> Widgets;
}

pub trait CatppuccinThemeSelectionExt {
    fn catppuccin_light() -> Selection {
        Self::catppuccin_flavour(Flavour::Latte)
    }
    fn catppuccin_dark() -> Selection {
        Self::catppuccin_flavour(Flavour::Macchiato)
    }
    fn catppuccin_flavour(flavour: Flavour) -> Selection;
}

pub trait FromCatppuccinColourExt {
    fn from_catppuccin_colour(colour: catppuccin::Colour) -> Color32;
    fn from_catppuccin_colour_ref(colour: &catppuccin::Colour) -> Color32 {
        Self::from_catppuccin_colour(*colour)
    }
    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Color32;
}

impl FromCatppuccinColourExt for Color32 {
    fn from_catppuccin_colour(colour: catppuccin::Colour) -> Color32 {
        Color32::from_rgb(colour.0, colour.1, colour.2)
    }
    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Color32 {
        let (r, g, b) = colour.into();
        Color32::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8)
    }
}

impl CatppuccinThemeVisualsExt for Visuals {
    fn catppuccin_flavour(flavour: Flavour) -> Visuals {
        let is_dark = flavour.base().lightness() < 0.5;
        Visuals {
            dark_mode: is_dark,
            override_text_color: Some(Color32::from_catppuccin_colour(flavour.text())),
            widgets: Widgets::catppuccin_flavour(flavour),
            selection: Selection::catppuccin_flavour(flavour),
            // hyperlink_color: Color32::from_rgb(90, 170, 255),
            hyperlink_color: Color32::from_catppuccin_colour(flavour.blue()),
            faint_bg_color: Color32::from_catppuccin_colour(flavour.mantle()), // visible, but barely so
            // extreme_bg_color: Color32::from_gray(10), // e.g. TextEdit background
            extreme_bg_color: Color32::from_catppuccin_colour(flavour.crust()), // e.g. TextEdit background
            // code_bg_color: Color32::from_gray(64),
            code_bg_color: Color32::from_catppuccin_colour(flavour.mantle()),
            // warn_fg_color: Color32::from_rgb(255, 143, 0), // orange
            warn_fg_color: Color32::from_catppuccin_colour(flavour.yellow()),
            error_fg_color: Color32::from_catppuccin_colour(flavour.red()),

            window_rounding: Rounding::same(10.0),
            window_shadow: if is_dark {
                Shadow::big_dark()
            } else {
                Shadow::big_light()
            },
            // window_fill: Color32::from_gray(27),
            window_fill: Color32::from_catppuccin_colour(flavour.base()),
            window_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.crust())),

            menu_rounding: Rounding::same(6.0),

            // panel_fill: Color32::from_gray(27),
            panel_fill: Color32::from_catppuccin_colour(flavour.base()),

            popup_shadow: if is_dark {
                Shadow::small_dark()
            } else {
                Shadow::small_light()
            },
            resize_corner_size: 12.0,
            text_cursor: Stroke::new(2.0, Color32::from_catppuccin_colour(flavour.lavender())),
            text_cursor_preview: false,
            clip_rect_margin: 3.0, // should be at least half the size of the widest frame stroke + max WidgetVisuals::expansion
            button_frame: true,
            collapsing_header_frame: false,
            indent_has_left_vline: true,

            striped: true,

            slider_trailing_fill: false,
            handle_shape: HandleShape::Circle,

            interact_cursor: None,

            image_loading_spinners: true,
            ..Default::default()
        }
    }
}

impl CatppuccinThemeWidgetsExt for Widgets {
    fn catppuccin_flavour(flavour: Flavour) -> Self {
        Self {
            noninteractive: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface0()),
                bg_fill: Color32::from_catppuccin_colour(flavour.surface0()),
                bg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.surface1())), // separators, indentation lines
                fg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.text())), // normal text color
                rounding: Rounding::same(5.0),
                expansion: 0.0,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_stroke: Default::default(), // default = 0 width stroke
                fg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.subtext1())), // button text
                rounding: Rounding::same(5.0),
                expansion: 0.0,
            },
            hovered: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface2()),
                bg_fill: Color32::from_catppuccin_colour(flavour.surface2()),
                bg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.overlay0())), // e.g. hover over window edge or button
                fg_stroke: Stroke::new(1.5, Color32::from_catppuccin_colour(flavour.overlay1())),
                rounding: Rounding::same(8.0),
                expansion: 3.0,
            },
            active: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.lavender())),
                fg_stroke: Stroke::new(2.0, Color32::from_catppuccin_colour(flavour.lavender())),
                rounding: Rounding::same(8.0),
                expansion: 3.0,
            },
            open: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill: Color32::from_catppuccin_colour(flavour.surface0()),
                bg_stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.surface1())),
                fg_stroke: Stroke::new(1.5, Color32::from_catppuccin_colour(flavour.overlay1())),
                rounding: Rounding::same(5.0),
                expansion: 0.0,
            },
        }
    }
}

impl CatppuccinThemeSelectionExt for Selection {
    fn catppuccin_flavour(flavour: Flavour) -> Selection {
        Self {
            bg_fill: Color32::from_catppuccin_colour(flavour.lavender()),
            stroke: Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.blue())),
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
    mut contexts: EguiContexts,
) {
    let mut window = windows.single_mut();
    for _ in theme_event.read() {
        if let Some(current_theme) = window.window_theme {
            let (window_theme, egui_style) = match current_theme {
                WindowTheme::Light => {
                    info!("Switching WindowTheme: Light -> Dark");
                    catppuccin_theme.flavour = match catppuccin_theme.flavour {
                        Flavour::Latte => Flavour::Macchiato,
                        _ => Flavour::Latte,
                    };

                    (Some(WindowTheme::Dark), Visuals::catppuccin_dark())
                }
                WindowTheme::Dark => {
                    info!("Switching WindowTheme: Dark -> Light");
                    catppuccin_theme.flavour = match catppuccin_theme.flavour {
                        Flavour::Latte => Flavour::Macchiato,
                        _ => Flavour::Latte,
                    };

                    (Some(WindowTheme::Light), Visuals::catppuccin_light())
                }
            };
            window.window_theme = window_theme;
            contexts
                .ctx_mut()
                .style_mut(|style| style.visuals = egui_style);
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
        let (r, g, b) = catppuccin_theme.flavour.base().into();
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
