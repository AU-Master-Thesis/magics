use bevy::{prelude::*, window::WindowTheme};
use bevy_egui::{
    egui::{
        epaint::Shadow,
        style::{HandleShape, Selection, WidgetVisuals, Widgets},
        Color32, Rounding, Stroke, Visuals,
    },
    EguiContexts, EguiPlugin,
};
use bevy_infinite_grid::InfiniteGridSettings;
use catppuccin::{Colour, Flavour, FlavourColours};
use strum_macros::EnumIter;

use crate::{
    environment,
    // factorgraph::{Factor, Line, Variable},
    planner::{self, RobotTracker},
};

#[derive(Component, Debug)]
pub struct ColorAssociation {
    // pub color: Color,
    pub name: DisplayColour,
}

/// Catppuccin **Bevy** theme wrapper
#[derive(Resource, Debug)]
pub struct CatppuccinTheme {
    pub flavour: Flavour,
}

impl Default for CatppuccinTheme {
    fn default() -> Self {
        let flavour = match dark_light::detect() {
            dark_light::Mode::Dark | dark_light::Mode::Default => Flavour::Macchiato,
            dark_light::Mode::Light => Flavour::Latte,
        };
        Self { flavour }
    }
}

#[derive(EnumIter, Debug)]
pub enum DisplayColour {
    Rosewater,
    Flamingo,
    Pink,
    Mauve,
    Red,
    Maroon,
    Peach,
    Yellow,
    Green,
    Teal,
    Sky,
    Sapphire,
    Blue,
    Lavender,
}

/// macro to implement all colour getters on [`CatppuccinTheme`] itself
macro_rules! impl_colour_getters {
    ($($x:ident),+ $(,)?) => (
        $(
            #[allow(dead_code)]
            #[inline(always)]
            pub const fn $x(&self) -> Colour {
                self.flavour.$x()
            }
        )+
    );
}

/// macro to implement all [`StandardMaterial`] colour getters on
/// [`CatppuccinTheme`]
macro_rules! impl_material_getters {
    ($($x:ident),+ $(,)?) => (
        paste::paste!{$(
            #[allow(dead_code)]
            #[inline(always)]
            pub fn [<$x _material>](&self) -> StandardMaterial {
                StandardMaterial {
                    base_color: Color::from_catppuccin_colour(self.flavour.$x()),
                    ..Default::default()
                }
            }
        )+}
    );
}

impl CatppuccinTheme {
    impl_colour_getters!(
        rosewater, flamingo, pink, mauve, red, maroon, peach, yellow, green, teal, sky, sapphire, blue, lavender, text,
        subtext1, subtext0, overlay2, overlay1, overlay0, surface2, surface1, surface0, base, mantle, crust,
    );

    impl_material_getters!(
        rosewater, flamingo, pink, mauve, red, maroon, peach, yellow, green, teal, sky, sapphire, blue, lavender, text,
        subtext1, subtext0, overlay2, overlay1, overlay0, surface2, surface1, surface0, base, mantle, crust,
    );

    pub fn grid_colour(&self) -> Color {
        let colour = if self.is_dark() {
            self.flavour.crust()
        } else {
            self.flavour.text()
        };

        let (r, g, b) = colour.into();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Color::rgba_u8(r, g, b, (0.5 * 255.0) as u8)
    }

    /// Check if the theme is a "dark" theme
    #[inline(always)]
    pub fn is_dark(&self) -> bool {
        self.flavour.base().lightness() < 0.5
    }

    /// Get iterator over all colours:
    /// `[rosewater, flamingo, pink, mauve, red, maroon, peach, yellow, green,
    /// teal, sky, sapphire, blue, lavender]`
    #[inline]
    pub const fn colours(&self) -> FlavourColours {
        self.flavour.colours()
    }

    /// Iterator over colours for display
    /// The 'colourful' colours so to say
    /// i.e. not the text, subtext, overlay, surface, base, mantle, crust
    /// colours
    pub fn into_display_iter(self) -> std::array::IntoIter<Colour, 14> {
        [
            self.flavour.rosewater(),
            self.flavour.flamingo(),
            self.flavour.pink(),
            self.flavour.mauve(),
            self.flavour.red(),
            self.flavour.maroon(),
            self.flavour.peach(),
            self.flavour.yellow(),
            self.flavour.green(),
            self.flavour.teal(),
            self.flavour.sky(),
            self.flavour.sapphire(),
            self.flavour.blue(),
            self.flavour.lavender(),
        ]
        .into_iter()
    }

    pub const fn get_display_colour(&self, display_colour: &DisplayColour) -> Colour {
        match display_colour {
            DisplayColour::Rosewater => self.flavour.rosewater(),
            DisplayColour::Flamingo => self.flavour.flamingo(),
            DisplayColour::Pink => self.flavour.pink(),
            DisplayColour::Mauve => self.flavour.mauve(),
            DisplayColour::Red => self.flavour.red(),
            DisplayColour::Maroon => self.flavour.maroon(),
            DisplayColour::Peach => self.flavour.peach(),
            DisplayColour::Yellow => self.flavour.yellow(),
            DisplayColour::Green => self.flavour.green(),
            DisplayColour::Teal => self.flavour.teal(),
            DisplayColour::Sky => self.flavour.sky(),
            DisplayColour::Sapphire => self.flavour.sapphire(),
            DisplayColour::Blue => self.flavour.blue(),
            DisplayColour::Lavender => self.flavour.lavender(),
        }
    }

    // pub fn get_rosewater_material(&self) -> StandardMaterial {
    //     StandardMaterial {
    //         base_color: Color::from_catppuccin_colour(self.flavour.rosewater()),
    //         ..Default::default()
    //     }
    // }
}

pub trait ColourExt {
    fn lightness(&self) -> f32;
}

impl ColourExt for Colour {
    fn lightness(&self) -> f32 {
        let (r, g, b) = Into::<(u8, u8, u8)>::into(*self);
        let average = (u16::from(r) + u16::from(g) + u16::from(b)) / 3;
        f32::from(average) / 255.0
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
    fn from_catppuccin_colour_ref(colour: catppuccin::Colour) -> Color32 {
        Self::from_catppuccin_colour(colour)
    }
    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Color32;
}

impl FromCatppuccinColourExt for Color32 {
    fn from_catppuccin_colour(colour: catppuccin::Colour) -> Color32 {
        Self::from_rgb(colour.0, colour.1, colour.2)
    }

    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Color32 {
        let (r, g, b) = colour.into();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8)
    }
}

pub trait ColorFromCatppuccinColourExt {
    fn from_catppuccin_colour(colour: catppuccin::Colour) -> Color;
    fn from_catppuccin_colour_ref(colour: catppuccin::Colour) -> Color {
        Color::from_catppuccin_colour(colour)
    }
    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Color;
}

impl ColorFromCatppuccinColourExt for Color {
    fn from_catppuccin_colour(colour: catppuccin::Colour) -> Self {
        let (r, g, b) = colour.into();
        Self::rgb_u8(r, g, b)
    }

    fn from_catppuccin_colour_with_alpha(colour: catppuccin::Colour, alpha: f32) -> Self {
        let (r, g, b) = colour.into();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self::rgba_u8(r, g, b, (alpha * 255.0) as u8)
    }
}

impl CatppuccinThemeVisualsExt for Visuals {
    fn catppuccin_flavour(flavour: Flavour) -> Visuals {
        let is_dark = flavour.base().lightness() < 0.5;
        Self {
            dark_mode: is_dark,
            override_text_color: Some(Color32::from_catppuccin_colour(flavour.text())),
            widgets: Widgets::catppuccin_flavour(flavour),
            selection: Selection::catppuccin_flavour(flavour),
            // hyperlink_color: Color32::from_rgb(90, 170, 255),
            hyperlink_color: Color32::from_catppuccin_colour(flavour.blue()),
            faint_bg_color: Color32::from_catppuccin_colour(flavour.mantle()), /* visible, but
                                                                                * barely so */
            // extreme_bg_color: Color32::from_gray(10), // e.g. TextEdit background
            extreme_bg_color: Color32::from_catppuccin_colour(flavour.crust()), /* e.g. TextEdit
                                                                                 * background */
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
            clip_rect_margin: 3.0, /* should be at least half the size of the widest frame stroke
                                    * + max WidgetVisuals::expansion */
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
                bg_fill:      Color32::from_catppuccin_colour(flavour.surface0()),
                bg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.surface1())), /* separators,
                                                                                                      * indentation
                                                                                                      * lines */
                fg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.text())), /* normal text
                                                                                                  * color */
                rounding:     Rounding::same(5.0),
                expansion:    0.0,
            },
            inactive: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill:      Color32::from_catppuccin_colour(flavour.surface1()),
                bg_stroke:    Stroke::default(), // default = 0 width stroke
                fg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.subtext1())), // button text
                rounding:     Rounding::same(5.0),
                expansion:    0.0,
            },
            hovered: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface2()),
                bg_fill:      Color32::from_catppuccin_colour(flavour.surface2()),
                bg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.overlay0())), /* e.g. hover over window edge or button */
                fg_stroke:    Stroke::new(1.5, Color32::from_catppuccin_colour(flavour.overlay1())),
                rounding:     Rounding::same(7.0),
                expansion:    2.0,
            },
            active: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill:      Color32::from_catppuccin_colour(flavour.surface1()),
                bg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.lavender())),
                fg_stroke:    Stroke::new(2.0, Color32::from_catppuccin_colour(flavour.lavender())),
                rounding:     Rounding::same(7.0),
                expansion:    2.0,
            },
            open: WidgetVisuals {
                weak_bg_fill: Color32::from_catppuccin_colour(flavour.surface1()),
                bg_fill:      Color32::from_catppuccin_colour(flavour.surface0()),
                bg_stroke:    Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.surface1())),
                fg_stroke:    Stroke::new(1.5, Color32::from_catppuccin_colour(flavour.overlay1())),
                rounding:     Rounding::same(5.0),
                expansion:    0.0,
            },
        }
    }
}

impl CatppuccinThemeSelectionExt for Selection {
    fn catppuccin_flavour(flavour: Flavour) -> Selection {
        Self {
            bg_fill: Color32::from_catppuccin_colour(flavour.lavender()),
            stroke:  Stroke::new(1.0, Color32::from_catppuccin_colour(flavour.blue())),
        }
    }
}

/// Signal that theme should be toggled
#[derive(Event, Debug, Copy, Clone)]
pub struct CycleTheme(pub Flavour);

impl Default for CycleTheme {
    fn default() -> Self {
        Self(Flavour::Macchiato)
    }
}

/// Signal after theme has been toggled
#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeChanged;

/// Theming plugin
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        let window_theme = match dark_light::detect() {
            dark_light::Mode::Dark | dark_light::Mode::Default => WindowTheme::Dark,
            dark_light::Mode::Light => WindowTheme::Light,
        };

        info!(
            "based on OS light/dark theme preference, setting window theme to: {:?}",
            window_theme
        );

        app.add_event::<CycleTheme>()
            .add_event::<ThemeChanged>()
            .init_resource::<CatppuccinTheme>()
            .add_systems(
                Startup,
                // init_window_theme(WindowTheme::Dark).run_if(theme_is_not_initialised),
                init_window_theme(window_theme),
            )
            .add_systems(
                Update,
                (
                    change_theme,
                    handle_clear_color,
                    handle_infinite_grid,
                    handle_variables,
                    // handle_factors,
                    // handle_lines,
                    handle_robots,
                    handle_waypoints,
                    // handle_variable_visualisers,
                    handle_obstacles,
                ),
            );

        if app.is_plugin_added::<EguiPlugin>() {
            app.add_systems(Update, handle_egui);
        }
    }
}

/// **Bevy** run criteria, checking if the window theme has been set
fn theme_is_not_initialised(windows: Query<&Window>) -> bool {
    let window = windows.single();
    window.window_theme.is_none()
}

/// **Bevy** `Startup` system to set the window theme
/// Run criteria: `theme_is_not_initialised`
/// Only used to set the window theme if it wasn't possible to detect from the
/// system
fn init_window_theme(theme: WindowTheme) -> impl FnMut(Query<&mut Window>) {
    move |mut windows: Query<&mut Window>| {
        let mut window = windows.single_mut();
        window.window_theme = Some(theme);
    }
}

/// **Bevy** `Update` system to change the theme
/// Reads `catppuccin::Flavour` from `ThemeEvent` to change the theme
/// Emits a `ThemeChangedEvent` after the theme has been changed, to be used by
/// other systems that actually change the colours
fn change_theme(
    mut windows: Query<&mut Window>,
    mut theme_event_reader: EventReader<CycleTheme>,
    mut theme: ResMut<CatppuccinTheme>,
    mut theme_toggled_event: EventWriter<ThemeChanged>,
    // mut contexts: EguiContexts,
) {
    let mut window = windows.single_mut();
    for CycleTheme(new_flavour) in theme_event_reader.read() {
        let new_window_theme = match new_flavour {
            Flavour::Latte | Flavour::Frappe => WindowTheme::Light,
            Flavour::Macchiato | Flavour::Mocha => WindowTheme::Dark,
        };
        info!("switching theme {:?} -> {:?}", theme.flavour, new_flavour);
        window.window_theme = Some(new_window_theme);
        // contexts
        //     .ctx_mut()
        //     .style_mut(|style| style.visuals =
        // Visuals::catppuccin_flavour(*new_flavour));
        theme.flavour = *new_flavour;
        theme_toggled_event.send(ThemeChanged);
    }
}

fn handle_egui(
    mut egui_contexts: EguiContexts,
    theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
) {
    for _ in theme_changed_event.read() {
        egui_contexts
            .ctx_mut()
            .style_mut(|style| style.visuals = Visuals::catppuccin_flavour(theme.flavour));
    }
}

/// **Bevy** `Update` system to handle the clear colour theme change
/// Reads `ThemeChangedEvent` to know when to change the clear colour
fn handle_clear_color(
    mut clear_color: ResMut<ClearColor>,
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
) {
    for _ in theme_changed_event.read() {
        *clear_color = ClearColor(Color::from_catppuccin_colour(catppuccin_theme.flavour.base()));
    }
}

/// **Bevy** `Update` system to handle the infinite grid theme change
/// Reads `ThemeChangedEvent` to know when to change the infinite grid theme
fn handle_infinite_grid(
    theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
    mut infinite_grid_settings: Query<&mut InfiniteGridSettings>,
) {
    for _ in theme_changed_event.read() {
        if let Ok(mut settings) = infinite_grid_settings.get_single_mut() {
            let grid_colour = theme.grid_colour();
            settings.major_line_color = grid_colour.with_a(0.5);
            settings.minor_line_color = grid_colour.with_a(0.25);
            settings.x_axis_color = Color::from_catppuccin_colour_with_alpha(theme.flavour.red(), 0.1);
            settings.z_axis_color = Color::from_catppuccin_colour_with_alpha(theme.flavour.blue(), 0.1);
        }
    }
}

// /// **Bevy** `Update` system to handle the variable theme change
// /// Reads `ThemeChangedEvent` to know when to change the variable colour
// fn handle_variables(
//     catppuccin_theme: Res<CatppuccinTheme>,
//     mut theme_changed_event: EventReader<ThemeChangedEvent>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut query_variable: Query<&mut Handle<StandardMaterial>, With<Variable>>,
// ) {
//     for _ in theme_changed_event.read() {
//         for handle in query_variable.iter_mut() {
//             if let Some(material) = materials.get_mut(handle.clone()) {
//                 material.base_color =
//
// Color::from_catppuccin_colour_with_alpha(catppuccin_theme.flavour.blue(),
// 0.75);             }
//         }
//     }
// }

// /// **Bevy** `Update` system to handle the theme change for
// `VariableVisualiser` /// Reads `ThemeChangedEvent` to know when to change the
// variable colour fn handle_variable_visualisers(
//     catppuccin_theme: Res<CatppuccinTheme>,
//     mut theme_changed_event: EventReader<ThemeChangedEvent>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut query_variable: Query<&mut Handle<StandardMaterial>,
// With<planner::VariableVisualiser>>, ) {
//     for _ in theme_changed_event.read() {
//         for handle in query_variable.iter_mut() {
//             if let Some(material) = materials.get_mut(handle.clone()) {
//                 material.base_color =
//
// Color::from_catppuccin_colour_with_alpha(catppuccin_theme.flavour.blue(),
// 0.75);             }
//         }
//     }
// }

// /// **Bevy** `Update` system to handle the factor theme change
// /// Reads `ThemeChangedEvent` to know when to change the factor colour
// fn handle_factors(
//     catppuccin_theme: Res<CatppuccinTheme>,
//     mut theme_changed_event: EventReader<ThemeChangedEvent>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut query_factor: Query<&mut Handle<StandardMaterial>, With<Factor>>,
// ) {
//     for _ in theme_changed_event.read() {
//         for handle in query_factor.iter_mut() {
//             if let Some(material) = materials.get_mut(handle.clone()) {
//                 material.base_color =
// Color::from_catppuccin_colour_with_alpha(
// catppuccin_theme.flavour.green(),                     0.75,
//                 );
//             }
//         }
//     }
// }

// /// **Bevy** `Update` system to handle the line theme change
// /// Reads `ThemeChangedEvent` to know when to change the line colour
// fn handle_lines(
//     catppuccin_theme: Res<CatppuccinTheme>,
//     mut theme_changed_event: EventReader<ThemeChangedEvent>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
//     mut query_line: Query<&mut Handle<StandardMaterial>, With<Line>>,
// ) {
//     for _ in theme_changed_event.read() {
//         for handle in query_line.iter_mut() {
//             if let Some(material) = materials.get_mut(handle.clone()) {
//                 material.base_color =
//
// Color::from_catppuccin_colour_with_alpha(catppuccin_theme.flavour.text(),
// 0.75);             }
//         }
//     }
// }

/// **Bevy** [`Update`] system to handle theme change for Robots
/// Reads [`ThemeChangedEvent`] to know when to change the robot colour
fn handle_robots(
    theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_robot: Query<(&mut Handle<StandardMaterial>, &ColorAssociation), With<planner::RobotState>>,
) {
    for _ in theme_changed_event.read() {
        for (handle, color_association) in &mut query_robot {
            if let Some(material) = materials.get_mut(handle.clone()) {
                material.base_color =
                    Color::from_catppuccin_colour_with_alpha(theme.get_display_colour(&color_association.name), 0.75);
            }
        }
    }
}

/// **Bevy** [`Update`] system to handle the theme change for waypoints
/// Reads [`ThemeChangedEvent`] to know when to change the waypoint colour
/// Queries all [`StandardMaterial`] handles with [`Waypoint`] components
fn handle_waypoints(
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_waypoint: Query<&mut Handle<StandardMaterial>, With<planner::WaypointVisualiser>>,
) {
    for _ in theme_changed_event.read() {
        for handle in &mut query_waypoint {
            if let Some(material) = materials.get_mut(handle.clone()) {
                material.base_color = Color::from_catppuccin_colour_with_alpha(catppuccin_theme.flavour.maroon(), 0.75);
            }
        }
    }
}

/// **Bevy** [`Update`] system to handle the theme change for variables
/// Reads [`ThemeChangedEvent`] to know when to change the variable colour
/// Queries all [`StandardMaterial`] handles with [`VariableVisualiser`] and
/// [`RobotTracker`] components The [`RobotTracker`] component is used to query
/// for the robot's [`ColorAssociation`] to get the correct colour
fn handle_variables(
    theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_variable: Query<(&mut Handle<StandardMaterial>, &RobotTracker), With<planner::VariableVisualiser>>,
    query_robot: Query<(Entity, &ColorAssociation), With<planner::RobotState>>,
) {
    for _ in theme_changed_event.read() {
        for (handle, robot_tracker) in &mut query_variable {
            if let Some(material) = materials.get_mut(handle.clone()) {
                // query to get the robot's `ColorAssociation`
                if let Ok((_, color_association)) = query_robot.get(robot_tracker.robot_id) {
                    material.base_color = Color::from_catppuccin_colour_with_alpha(
                        theme.get_display_colour(&color_association.name),
                        0.75,
                    );
                }
            }
        }
    }
}

/// **Bevy** [`Update`] system to handle the theme change for obstacles
/// Reads [`ThemeChangedEvent`] to know when to change the obstacle colour
/// Queries all [`StandardMaterial`] handles with [`MapCell`] components
fn handle_obstacles(
    catppuccin_theme: Res<CatppuccinTheme>,
    mut theme_changed_event: EventReader<ThemeChanged>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_obstacle: Query<&mut Handle<StandardMaterial>, With<environment::ObstacleMarker>>,
) {
    for _ in theme_changed_event.read() {
        for handle in &mut query_obstacle {
            if let Some(material) = materials.get_mut(handle.clone()) {
                material.base_color = Color::from_catppuccin_colour(catppuccin_theme.flavour.text());
            }
        }
    }
}

// /// **Bevy** [`Resource`] for Catppuccin theme assets
// /// Contains base-materials for all the colours in the Catppuccin theme
// #[derive(Debug, Default, Resource)]
// pub struct CatppuccinThemeAssets {}
