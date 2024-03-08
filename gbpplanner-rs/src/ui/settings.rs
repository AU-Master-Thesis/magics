use std::path::PathBuf;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32, Context},
    EguiContext, EguiContexts, EguiSettings,
};

use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};
use struct_iterable::Iterable;
use strum::IntoEnumIterator;

use crate::{
    config::{Config, DrawSection, DrawSetting},
    environment::cursor::CursorCoordinates,
    theme::{CatppuccinTheme, FromCatppuccinColourExt, ThemeEvent},
};

use super::{custom, OccupiedScreenSpace, ToDisplayString, UiScaleType, UiState};

/// **Bevy** `Plugin` to add the settings panel to the UI
pub struct SettingsPanelPlugin;

impl Plugin for SettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiScaleEvent>()
            .add_event::<EnvironmentEvent>()
            .add_event::<ExportGraphEvent>()
            .add_event::<DrawSettingsEvent>()
            .add_systems(Startup, install_egui_image_loaders)
            .add_systems(Update, (ui_settings_exclusive, scale_ui))
            .add_plugins(DefaultInspectorConfigPlugin);
    }
}

/// Simple **Bevy** trigger `Event`
/// Write to this event whenever you want to toggle the environment
#[derive(Event, Debug, Copy, Clone)]
pub struct EnvironmentEvent;

/// Simple **Bevy** trigger `Event`
/// Write to this event whenever you want the UI scale to update
#[derive(Event, Debug, Copy, Clone)]
pub struct UiScaleEvent;

/// Simple **Bevy** trigger `Event`
/// Write to this event whenever you want to export the graph to a `.dot` file
#[derive(Event, Debug, Copy, Clone)]
pub struct ExportGraphEvent;

/// **Bevy** `Event` for the draw settings
/// This event is triggered when a draw setting is toggled
#[derive(Event, Debug, Clone)]
pub struct DrawSettingsEvent {
    pub setting: DrawSetting,
    pub value: bool,
}

fn install_egui_image_loaders(mut egui_ctx: EguiContexts) {
    egui_extras::install_image_loaders(egui_ctx.ctx_mut());
}

impl ToDisplayString for catppuccin::Flavour {
    fn to_display_string(&self) -> String {
        match self {
            catppuccin::Flavour::Frappe => "Frappe".to_string(),
            catppuccin::Flavour::Latte => "Latte".to_string(),
            catppuccin::Flavour::Macchiato => "Macchiato".to_string(),
            catppuccin::Flavour::Mocha => "Mocha".to_string(),
        }
    }
}

fn ui_settings_exclusive(world: &mut World) {
    // query for all the things ui_settings_panel needs
    let Ok(contexts) = world
        .query_filtered::<&EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };

    let mut egui_context = contexts.clone();

    world.resource_scope(|world, ui_state: Mut<UiState>| {
        world.resource_scope(|world, config: Mut<Config>| {
            world.resource_scope(|world, occupied_screen_space: Mut<OccupiedScreenSpace>| {
                world.resource_scope(|world, catppuccin_theme: Mut<CatppuccinTheme>| {
                    world.resource_scope(|world, cursor_coordinates: Mut<CursorCoordinates>| {
                        ui_settings_panel(
                            egui_context.get_mut(),
                            ui_state,
                            config,
                            occupied_screen_space,
                            cursor_coordinates,
                            catppuccin_theme,
                            world,
                        );
                    });
                });
            });
        });
    });
}

/// **Bevy** `Update` system to display the `egui` settings panel
#[allow(clippy::too_many_arguments)]
fn ui_settings_panel(
    contexts: &mut Context,
    mut ui_state: Mut<UiState>,
    mut config: Mut<Config>,
    mut occupied_screen_space: Mut<OccupiedScreenSpace>,
    cursor_coordinates: Mut<CursorCoordinates>,
    catppuccin_theme: Mut<CatppuccinTheme>,
    world: &mut World,
) {
    // let ctx = contexts.ctx_mut();
    // let ctx = contexts.get_mut();
    let ctx = contexts;

    let right_panel = egui::SidePanel::right("Settings Panel")
        .default_width(200.0)
        .resizable(true)
        .show_animated(ctx, ui_state.right_panel, |ui| {
            ui.add_space(10.0);
            ui.heading("Settings");
            ui.add_space(5.0);
            ui.separator();

            egui::ScrollArea::vertical()
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    custom::subheading(ui, "General", Some(Color32::from_catppuccin_colour(catppuccin_theme.flavour.green())));
                    egui::Grid::new("cool_grid")
                        .num_columns(2)
                        .min_col_width(100.0)
                        .striped(false)
                        .spacing((10.0, 10.0))
                        .show(ui, |ui| {
                            // THEME SELECTOR
                            ui.label("Theme");
                            ui.vertical_centered_justified(|ui| {
                                ui.menu_button(
                                    catppuccin_theme.flavour.to_display_string(),
                                    |ui| {
                                        for flavour in &[
                                            catppuccin::Flavour::Frappe,
                                            catppuccin::Flavour::Latte,
                                            catppuccin::Flavour::Macchiato,
                                            catppuccin::Flavour::Mocha,
                                        ] {
                                            ui.vertical_centered_justified(|ui| {
                                                if ui.button(flavour.to_display_string()).clicked()
                                                {
                                                    world.send_event::<ThemeEvent>(ThemeEvent(*flavour));
                                                    // theme_event.send(ThemeEvent(*flavour));
                                                    ui.close_menu();
                                                }
                                            });
                                        }
                                    },
                                );
                            });
                            ui.end_row();

                            // UI SCALING TYPE SELECTOR
                            ui.label("Scale Type");
                            ui.vertical_centered_justified(|ui| {
                                ui.menu_button(ui_state.scale_type.to_display_string(), |ui| {
                                    for scale in UiScaleType::iter() {
                                        ui.vertical_centered_justified(|ui| {
                                            if ui.button(scale.to_display_string()).clicked() {
                                                ui_state.scale_type = scale;
                                                world.send_event::<UiScaleEvent>(UiScaleEvent);
                                                // scale_event.send(UiScaleEvent);
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                })
                            });
                            ui.end_row();

                            // UI SCALING SLIDER
                            ui.add_enabled_ui(
                                matches!(ui_state.scale_type, UiScaleType::Custom),
                                |ui| {
                                    ui.label("Custom Scale");
                                },
                            );
                            let slider_response = ui.add_enabled(
                                matches!(ui_state.scale_type, UiScaleType::Custom),
                                egui::Slider::new(&mut ui_state.scale_percent, 50..=200)
                                    .text("%")
                                    .show_value(true),
                            );
                            // Only trigger ui scale update when the slider is released or lost focus
                            // otherwise it would be imposssible to drag the slider while the ui is scaling
                            if slider_response.drag_released() || slider_response.lost_focus() {
                                world.send_event::<UiScaleEvent>(UiScaleEvent);
                                // scale_event.send(UiScaleEvent);
                            }
                            ui.end_row();
                        });
                    custom::subheading(ui, "Draw", Some(Color32::from_catppuccin_colour(catppuccin_theme.flavour.blue())));
                    egui::CollapsingHeader::new("").default_open(true).show(ui, |ui| {
                        egui::Grid::new("draw_grid")
                            .num_columns(2)
                            .min_col_width(100.0)
                            .striped(false)
                            .spacing((10.0, 10.0))
                            .show(ui, |ui| {
                                // CONFIG DRAW SECTION
                                // This should add a toggle for each draw setting in the config
                                // Should be 4 toggles
                                for (name, _) in config.visualisation.draw.clone().iter() {
                                    ui.label(DrawSection::to_display_string(name));
                                    let setting = config.visualisation.draw.get_field_mut::<bool>(name)
                                        .expect("Since I am iterating over the fields, I should be able to get the field");
                                    custom::float_right(ui, |ui| {
                                        if custom::toggle_ui(ui, setting).clicked() {
                                            let setting_kind = DrawSetting::from_str(name).expect("The name of the draw section should be a valid DrawSection");
                                            let event = DrawSettingsEvent { setting: setting_kind, value: *setting };
                                            world.send_event::<DrawSettingsEvent>(event);
                                            // draw_setting_event.send(event);
                                        }
                                    });
                                    ui.end_row();
                                }
                            });
                        });
                        custom::subheading(ui, "Export", Some(Color32::from_catppuccin_colour(catppuccin_theme.flavour.mauve())));

                        let png_output_path = PathBuf::from("../../../factorgraphs").with_extension("png");

                        custom::grid(ui, "export_grid", 3, |ui| {
                            // GRAPHVIZ EXPORT TOGGLE
                            ui.label("Graphviz");
                            custom::fill_x(ui, |ui| {
                                if ui.button("Export").clicked() {
                                    world.send_event::<ExportGraphEvent>(ExportGraphEvent);
                                    // export_graph_event.send(ExportGraphEvent);
                                }
                            });
                            custom::fill_x(ui, |ui| {
                                if ui.button("Open").clicked() {
                                    let _ = open::that(&png_output_path)
                                        .inspect_err(|e| error!("failed to open ./{:?}: {e}", png_output_path));
                                }
                            });
                        });

                        ui.add_space(10.0);
                        // ui.add(egui::Image::new(egui::include_image!("../../../factorgraphs.png")));
                        // ui.add_space(10.0);

                        // INSPECTOR
                        custom::subheading(ui, "Inspector", Some(Color32::from_catppuccin_colour(catppuccin_theme.flavour.maroon())));
                        custom::grid(ui, "inspector_grid", 3, |ui| {
                            ui.label("Cursor");
                            ui.label(egui::RichText::new(format!("x: {:7.2}", cursor_coordinates.local().x)).monospace());
                            ui.label(egui::RichText::new(format!("y: {:7.2}", cursor_coordinates.local().y)).monospace());
                        });
                        ui.collapsing("Entities", |ui| {
                            bevy_inspector::ui_for_world_entities(world, ui);
                        });
                        ui.collapsing("Resources", |ui| {
                            bevy_inspector::ui_for_resources(world, ui);
                        });
                        ui.collapsing("Assets", |ui| {
                            bevy_inspector::ui_for_all_assets(world, ui);
                        });
                    });
        });

    occupied_screen_space.right = right_panel
        .map(|ref inner| inner.response.rect.width())
        .unwrap_or(0.0);
}

fn scale_ui(
    mut egui_settings: ResMut<EguiSettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale_event_reader: EventReader<UiScaleEvent>,
    ui_state: Res<UiState>,
) {
    for _ in scale_event_reader.read() {
        if let Ok(window) = windows.get_single() {
            let scale_factor = match ui_state.scale_type {
                UiScaleType::None => 1.0,
                UiScaleType::Custom => ui_state.scale_percent as f32 / 100.0,
                UiScaleType::Window => 1.0 / window.scale_factor() as f32,
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}
