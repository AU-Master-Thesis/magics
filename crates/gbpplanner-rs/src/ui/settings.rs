use std::time::Duration;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32, Context, RichText},
    EguiContext, EguiContexts,
};
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};
use catppuccin::Colour;
use repeating_array::RepeatingArray;
use struct_iterable::Iterable;
use strum::IntoEnumIterator;

use super::{custom, scale::ScaleUi, OccupiedScreenSpace, ToDisplayString, UiScaleType, UiState};
use crate::{
    config::{Config, DrawSection, DrawSetting},
    environment::cursor::CursorCoordinates,
    input::{screenshot::TakeScreenshot, ChangingBinding, DrawSettingsEvent, ExportGraphEvent},
    pause_play::{PausePlay, PausedState},
    theme::{CatppuccinTheme, CycleTheme, FromCatppuccinColourExt},
};

/// **Bevy** `Plugin` to add the settings panel to the UI
pub struct SettingsPanelPlugin;

impl Plugin for SettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (install_egui_image_loaders,))
            .add_systems(
                Update,
                (
                    ui_settings_exclusive,
                    // ui_settings_exclusive.run_if(show_right_panel),
                ),
            )
            .add_plugins(DefaultInspectorConfigPlugin);
    }
}

// fn show_right_panel(ui: Res<UiState>) -> bool {
//     ui.right_panel_visible
// }

fn install_egui_image_loaders(mut egui_ctx: EguiContexts) {
    egui_extras::install_image_loaders(egui_ctx.ctx_mut());
}

impl ToDisplayString for catppuccin::Flavour {
    fn to_display_string(&self) -> String {
        match self {
            Self::Frappe => "Frappe".to_string(),
            Self::Latte => "Latte".to_string(),
            Self::Macchiato => "Macchiato".to_string(),
            Self::Mocha => "Mocha".to_string(),
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
                        world.resource_scope(|world, currently_changing: Mut<ChangingBinding>| {
                            world.resource_scope(|world, pause_play: Mut<State<PausedState>>| {
                                world.resource_scope(|world, time: Mut<Time<Virtual>>| {
                                    world.resource_scope(|world, time_fixed: Mut<Time<Fixed>>| {
                                        world.resource_scope(
                                            |world, config_store: Mut<GizmoConfigStore>| {
                                                ui_settings_panel(
                                                    egui_context.get_mut(),
                                                    ui_state,
                                                    config,
                                                    occupied_screen_space,
                                                    cursor_coordinates,
                                                    catppuccin_theme,
                                                    world,
                                                    currently_changing,
                                                    pause_play,
                                                    time,
                                                    time_fixed,
                                                    config_store,
                                                );
                                            },
                                        );
                                    });
                                });
                            });
                        });
                    });
                });
            });
        });
    });
}

type TitleColors = RepeatingArray<Colour, 5>;

/// **Bevy** `Update` system to display the `egui` settings panel
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn ui_settings_panel(
    // ctx: &mut Context,
    ctx: &Context,
    mut ui_state: Mut<UiState>,
    mut config: Mut<Config>,
    mut occupied_screen_space: Mut<OccupiedScreenSpace>,
    cursor_coordinates: Mut<CursorCoordinates>,
    catppuccin_theme: Mut<CatppuccinTheme>,
    world: &mut World,
    _currently_changing: Mut<ChangingBinding>,
    pause_state: Mut<State<PausedState>>,
    mut time_virtual: Mut<Time<Virtual>>,
    mut time_fixed: Mut<Time<Fixed>>,
    mut config_store: Mut<GizmoConfigStore>,
) {
    let mut title_colors = TitleColors::new([
        catppuccin_theme.green(),
        catppuccin_theme.blue(),
        catppuccin_theme.mauve(),
        catppuccin_theme.maroon(),
        catppuccin_theme.lavender(),
    ]);

    let panel_resizable = false;

    let top_panel = egui::TopBottomPanel::top("Top Panel")
        .default_height(100.0)
        .resizable(panel_resizable)
        .show_animated(ctx, ui_state.top_panel_visible, |ui| {
            ui.strong("top panel");
            // #[derive(PartialEq, Eq)]
            // enum Enum {
            //     First,
            //     Second,
            //     Third,
            // }
            // let mut selected = "foo".to_string();
            // egui::ComboBox::from_label("Select one!")
            //     .selected_text(format!("{:?}", selected))
            //     .show_ui(ui, |ui| {
            //         ui.selectable_value(&mut selected, Enum::First, "First");
            //         ui.selectable_value(&mut selected, Enum::Second,
            // "Second");         ui.selectable_value(&mut selected,
            // Enum::Third, "Third");     });
        });

    occupied_screen_space.top = top_panel.map_or(0.0, |ref inner| inner.response.rect.width());

    let right_panel = egui::SidePanel::right("Settings Panel")
        .default_width(200.0)
        .resizable(panel_resizable)
        .show_animated(ctx, ui_state.right_panel_visible, |ui| {
            ui_state.mouse_over.right_panel = ui.rect_contains_pointer(ui.max_rect())
                && config.interaction.ui_focus_cancels_inputs;

            custom::heading(ui, "Settings", None);

            egui::ScrollArea::vertical()
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    custom::subheading(
                        ui,
                        "General",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    custom::grid("settings_general_grid", 2).show(ui, |ui| {
                        // THEME SELECTOR
                        ui.label("Theme");
                        ui.vertical_centered_justified(|ui| {
                            ui.menu_button(catppuccin_theme.flavour.to_display_string(), |ui| {
                                for flavour in &[
                                    catppuccin::Flavour::Frappe,
                                    catppuccin::Flavour::Latte,
                                    catppuccin::Flavour::Macchiato,
                                    catppuccin::Flavour::Mocha,
                                ] {
                                    ui.vertical_centered_justified(|ui| {
                                        if ui.button(flavour.to_display_string()).clicked() {
                                            world.send_event::<CycleTheme>(CycleTheme(*flavour));
                                            // theme_event.send(ThemeEvent(*flavour));
                                            ui.close_menu();
                                        }
                                    });
                                }
                            });
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
                                            // world.send_event::<UiScaleEvent>(UiScaleEvent);
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

                        ui.spacing_mut().slider_width =
                            ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let slider_response = ui.add_enabled(
                            matches!(ui_state.scale_type, UiScaleType::Custom),
                            // egui::Slider::new(&mut ui_state.scale_percent, 50..=200)
                            egui::Slider::new(
                                &mut ui_state.scale_percent,
                                UiState::VALID_SCALE_INTERVAL,
                            )
                            .text("%")
                            .show_value(true),
                        );
                        // Only trigger ui scale update when the slider is released or lost focus
                        // otherwise it would be imposssible to drag the slider while the ui is
                        // scaling
                        #[allow(clippy::cast_precision_loss)]
                        if slider_response.drag_released() || slider_response.lost_focus() {
                            world.send_event::<ScaleUi>(ScaleUi::Set(
                                ui_state.scale_percent as f32 / 100.0,
                            ));
                        }

                        ui.end_row();

                        ui.label("Take Screenhot");
                        custom::fill_x(ui, |ui| {
                            if ui.button("").clicked() {
                                world.send_event::<TakeScreenshot>(TakeScreenshot::default());
                            }
                        });
                        ui.end_row();
                    });

                    custom::subheading(
                        ui,
                        "Draw",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    // egui::CollapsingHeader::new("").default_open(true).show(ui, |ui| {
                    custom::grid("draw_grid", 2).show(ui, |ui| {
                        // CONFIG DRAW SECTION
                        // This should add a toggle for each draw setting in the config
                        // Should be 4 toggles
                        for (name, _) in config.visualisation.draw.clone().iter() {
                            ui.label(DrawSection::to_display_string(name));
                            let setting = config
                                .visualisation
                                .draw
                                .get_field_mut::<bool>(name)
                                .expect(
                                    "Since I am iterating over the fields, I should be able to \
                                     get the field",
                                );
                            custom::float_right(ui, |ui| {
                                if custom::toggle_ui(ui, setting).clicked() {
                                    let setting_kind: DrawSetting = name.parse().expect(
                                        "the ui strings are generated from the enum, so parse \
                                         should not fail",
                                    );
                                    // let setting_kind = DrawSetting::from_str(name).expect(
                                    //     "The name of the draw section should be a valid \
                                    //      DrawSection",
                                    // );
                                    let event = DrawSettingsEvent {
                                        setting: setting_kind,
                                        draw:    *setting,
                                    };
                                    world.send_event::<DrawSettingsEvent>(event);
                                }
                            });
                            ui.end_row();
                        }

                        // GIZMOS
                        let (gizmo_config, _) =
                            config_store.config_mut::<DefaultGizmoConfigGroup>();
                        ui.label("Gizmos");
                        custom::float_right(ui, |ui| {
                            custom::toggle_ui(ui, &mut gizmo_config.enabled);
                        });
                    });

                    custom::subheading(
                        ui,
                        "Simulation",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    custom::grid("simulation_settings_grid", 2).show(ui, |ui| {
                        ui.label("Simulation Time");
                        // let progress =
                        //     time_fixed.elapsed_seconds() / config.simulation.max_time.get();
                        // let progressbar =
                        //     egui::widgets::ProgressBar::new(progress).fill(Color32::RED);
                        // ui.add(progressbar);

                        custom::rect_label(
                            ui,
                            format!(
                                "{:.2} / {:.2}",
                                // time_fixed.elapsed_seconds(),
                                time_virtual.elapsed_seconds(),
                                config.simulation.max_time.get()
                            ),
                            None,
                        );
                        ui.end_row();

                        // slider for simulation time between 0 and 100
                        ui.label("Simulation Speed");
                        // slider for simulation speed (time scale) between 0.1 and 10
                        ui.spacing_mut().slider_width =
                            ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let slider_response = ui.add(
                            egui::Slider::new(&mut config.simulation.time_scale.get(), 0.1..=5.0)
                                .text("x")
                                .show_value(true),
                        );
                        if slider_response.drag_released() || slider_response.lost_focus() {
                            // time.set_time_scale(config.simulation.time_scale);
                            info!("time scale changed: {}", config.simulation.time_scale);
                            time_virtual.set_relative_speed(config.simulation.time_scale.get());
                        }
                        ui.end_row();

                        ui.label("Manual Controls");

                        custom::grid("manual_controls_settings_grid", 2).show(ui, |ui| {
                            // step forward button
                            ui.add_enabled_ui(!pause_state.is_paused(), |ui| {
                                custom::fill_x(ui, |ui| {
                                    if ui
                                        .button(RichText::new("󰒭").size(25.0))
                                        .on_hover_text("Step forward one step in the simulation")
                                        .clicked()
                                    {
                                        #[allow(
                                            clippy::cast_precision_loss,
                                            clippy::cast_possible_truncation
                                        )]
                                        let step_size = config.simulation.manual_step_factor as f32
                                            / config.simulation.hz as f32;
                                        time_fixed.advance_by(Duration::from_secs_f32(step_size));
                                    }
                                });
                            });
                            // pause/play button
                            let pause_play_text = if pause_state.is_paused() {
                                ""
                            } else {
                                ""
                            };
                            custom::fill_x(ui, |ui| {
                                if ui
                                    .button(pause_play_text)
                                    .on_hover_text("Play or pause the simulation")
                                    .clicked()
                                {
                                    world.send_event::<PausePlay>(PausePlay::Toggle);
                                }
                            });

                            // ui.end_row();
                        });
                        // ui.end_row();
                    });

                    custom::subheading(
                        ui,
                        "Export",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );

                    // let png_output_path = PathBuf::from("./factorgraphs").with_extension("png");

                    custom::grid("export_grid", 3).show(ui, |ui| {
                        // GRAPHVIZ EXPORT TOGGLE
                        ui.label("Graphviz");
                        custom::fill_x(ui, |ui| {
                            if ui.button("Export").clicked() {
                                world.send_event::<ExportGraphEvent>(ExportGraphEvent);
                            }
                        });
                        custom::fill_x(ui, |ui| {
                            if ui.button("Open").clicked() {
                                // #[cfg(not(target_os =
                                // "wasm32-unknown-unknown"))]
                                // let _ = open::that(&png_output_path).
                                // inspect_err(|e| {
                                //     // TODO: show a popup with the error
                                //     // create a notification system, that can
                                // be send events with
                                //     // notifications to show

                                //     // let popup_id =
                                // ui.make_persistent_id("my_unique_id");
                                //     // let above = egui::AboveOrBelow::Above;
                                //     // egui::popup_above_or_below_widget(ui,
                                // popup_id,
                                //     // widget_response, above_or_below,
                                // add_contents)
                                //     error!("failed to open ./{:?}: {e}",
                                // png_output_path)
                                // });
                            }
                        });
                    });

                    ui.add_space(10.0);
                    // ui.add(egui::Image::new(egui::include_image!("../../../factorgraphs.png")));
                    // ui.add_space(10.0);

                    // INSPECTOR
                    custom::subheading(
                        ui,
                        "Inspector",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    custom::grid("inspector_grid", 3).show(ui, |ui| {
                        ui.label("Cursor");
                        // y coordinate
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label("x:");
                            let x_coordinate = format!("{:7.2}", cursor_coordinates.local().x);
                            if custom::rect_label(ui, x_coordinate.clone(), None).clicked() {
                                ui.output_mut(|o| {
                                    // this will only work if `interact =
                                    // Some(egui::Sense::click())` or similar
                                    o.copied_text = x_coordinate.to_string();
                                });
                            }
                        });
                        // x coordinate
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label("y:");
                            let y_coordinate = format!("{:7.2}", cursor_coordinates.local().y);
                            if custom::rect_label(ui, y_coordinate.clone(), None).clicked() {
                                ui.output_mut(|o| {
                                    // this will only work if `interact =
                                    // Some(egui::Sense::click())` or similar
                                    o.copied_text = y_coordinate.to_string();
                                });
                            }
                        });
                        // custom::rect_label(ui, format!("y: {:7.2}",
                        // cursor_coordinates.local().y));
                    });

                    ui.add_space(2.5);

                    ui.separator();
                    ui.collapsing("Entities", |ui| {
                        bevy_inspector::ui_for_world_entities(world, ui);
                    });
                    ui.collapsing("Resources", |ui| {
                        bevy_inspector::ui_for_resources(world, ui);
                    });
                    ui.collapsing("Assets", |ui| {
                        bevy_inspector::ui_for_all_assets(world, ui);
                    });

                    custom::subheading(
                        ui,
                        "Other",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    custom::grid("other_grid", 2).show(ui, |ui| {
                        ui.label("UI Focus Cancels Inputs");
                        custom::float_right(ui, |ui| {
                            custom::toggle_ui(ui, &mut config.interaction.ui_focus_cancels_inputs)
                        });
                    });

                    ui.add_space(10.0);
                });
        });

    occupied_screen_space.right = right_panel.map_or(0.0, |ref inner| inner.response.rect.width());
}
