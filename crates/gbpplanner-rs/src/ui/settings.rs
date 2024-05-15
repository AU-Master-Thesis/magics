use std::{path::Path, time::Duration};

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32, Context, RichText},
    EguiContext, EguiContexts,
};
use bevy_infinite_grid::InfiniteGrid;
use bevy_inspector_egui::{bevy_inspector, DefaultInspectorConfigPlugin};
use bevy_notify::ToastEvent;
use catppuccin::Colour;
use gbp_config::{Config, DrawSection, DrawSetting};
use gbp_linalg::Float;
use gbp_schedule::GbpScheduleAtTimestep;
use repeating_array::RepeatingArray;
use smol_str::SmolStr;
use struct_iterable::Iterable;
use strum::IntoEnumIterator;

use super::{custom, scale::ScaleUi, OccupiedScreenSpace, ToUiString, UiScaleType, UiState};
use crate::{
    environment::cursor::CursorCoordinates,
    factorgraph::prelude::FactorGraph,
    input::{
        screenshot::TakeScreenshot, ChangingBinding, DrawSettingsEvent, ExportFactorGraphAsGraphviz,
    },
    pause_play::PausePlay,
    planner::robot::RadioAntenna,
    simulation_loader::{SimulationId, SimulationManager},
    theme::{CatppuccinTheme, CycleTheme, FromCatppuccinColourExt},
};

/// **Bevy** `Plugin` to add the settings panel to the UI
#[derive(Default)]
pub struct SettingsPanelPlugin;

impl Plugin for SettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (install_egui_image_loaders,))
            .add_systems(Update, (ui_settings_exclusive,))
            .add_plugins(DefaultInspectorConfigPlugin);
    }
}

fn install_egui_image_loaders(mut egui_ctx: EguiContexts) {
    egui_extras::install_image_loaders(egui_ctx.ctx_mut());
}

impl ToUiString for catppuccin::Flavour {
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
                            // world.resource_scope(|world, pause_play: Mut<State<PausedState>>| {
                            world.resource_scope(|world, time: Mut<Time<Virtual>>| {
                                world.resource_scope(|world, time_fixed: Mut<Time<Fixed>>| {
                                    world.resource_scope(
                                        |world, simulation_manager: Mut<SimulationManager>| {
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
                                                        // pause_play,
                                                        time,
                                                        time_fixed,
                                                        config_store,
                                                        simulation_manager,
                                                    );
                                                },
                                            );
                                        },
                                    );
                                });
                            });
                            // });
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
    theme: Mut<CatppuccinTheme>,
    world: &mut World,
    _currently_changing: Mut<ChangingBinding>,
    // pause_state: Mut<State<PausedState>>,
    mut time_virtual: Mut<Time<Virtual>>,
    mut time_fixed: Mut<Time<Fixed>>,
    mut config_store: Mut<GizmoConfigStore>,
    mut simulation_manager: Mut<SimulationManager>,
) {
    let mut title_colors = TitleColors::new([
        theme.green(),
        theme.blue(),
        theme.mauve(),
        theme.maroon(),
        theme.lavender(),
    ]);

    // let panel_resizable = false;

    let width = 400.0;
    let right_panel = egui::SidePanel::right("Settings Panel")
        .default_width(width)
        // .max_width(width)
        // .exact_width(width)
        .resizable(false)
        // .show(ctx, |ui| {
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
                            ui.menu_button(theme.flavour.to_display_string(), |ui| {
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

                        // ui.spacing_mut().slider_width = ui.available_size_before_wrap().x;
                        // ui.spacing_mut().slider_width = ui.available_width();
                        ui.spacing_mut().slider_width =
                            ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let enabled = matches!(ui_state.scale_type, UiScaleType::Custom);
                        let slider = egui::Slider::new(
                                &mut ui_state.scale_percent,
                                UiState::VALID_SCALE_INTERVAL,
                            )

                            .suffix("%")
                            .show_value(true);

                        let slider_response = ui.add_enabled(
                            enabled,
                            slider
                        );
                        // Only trigger ui scale update when the slider is released or lost focus
                        // otherwise it would be imposssible to drag the slider while the ui is
                        // scaling
                        #[allow(clippy::cast_precision_loss)]
                        if slider_response.drag_released() || slider_response.lost_focus() {
                        // if slider_response.changed() {
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


                    {
                        custom::subheading(ui, "GBP",
                            Some(Color32::from_catppuccin_colour(
                                title_colors.next_or_first(),
                            )),
                        );

                        ui.label("Iterations Per Timestep");
                        ui.separator();
                        // ui.add_space(2.5);

                        custom::grid("iterations_per_timestep_grid", 2).show(ui, |ui| {
                            ui.label("Internal");
                            let mut text = config.gbp.iteration_schedule.internal.to_string();

                            let te_output = egui::TextEdit::singleline(&mut text)
                                .char_limit(3)
                                .interactive(time_virtual.is_paused())
                                .show(ui);

                            // if te_output.response.lost_focus() && te_output.response.changed() {
                            if  te_output.response.changed() {
                                if let Ok(x) = text.parse::<usize>() {
                                    config.gbp.iteration_schedule.internal = x;
                                } else if text.is_empty() {
                                    config.gbp.iteration_schedule.internal = 0;
                                }
                                else {
                                    error!("failed to parse {} as usize", text);
                                }
                            }
                            ui.end_row();

                            ui.label("External");
                            let mut text = config.gbp.iteration_schedule.external.to_string();
                            let te_output = egui::TextEdit::singleline(&mut text)
                                .char_limit(3)
                                .interactive(time_virtual.is_paused())
                                // .cursor_at_end(true)
                                .show(ui);

                            if  te_output.response.changed() {
                                if let Ok(x) = text.parse::<usize>() {
                                    config.gbp.iteration_schedule.external = x;
                                } else if text.is_empty() {
                                    config.gbp.iteration_schedule.external = 0;
                                } else {
                                    error!("failed to parse {} as usize", text);
                                }
                            }
                            ui.end_row();
                        });


                        // TODO: very ugly, but it works
                            {
                                let n = config.gbp.iteration_schedule.internal.max(config.gbp.iteration_schedule.external);

                                let schedule_config = gbp_schedule::GbpScheduleConfig {
                                    internal: config.gbp.iteration_schedule.internal as u8,
                                    external: config.gbp.iteration_schedule.external as u8,
                                };
                                // let size = ui.available_size();
                                let max_rect = ui.max_rect();
                                // dbg!((&max_rect, &size));

                                // let painter_rect = egui::Rect {min: egui::Pos2::new(max_rect.left(), ui.cursor().top()), max: egui::Pos2::new(max_rect.right(), ui.cursor().top() + 30.0)};
                                // let (response, painter) = ui.allocate_painter(painter_rect, egui::Sense::hover());

                                // let clip_rect = ui.clip_rect();
                                let painter = ui.painter();
                                // let painter = ui.painter_at(max_rect);
                                let schedule = config.gbp.iteration_schedule.schedule.get(schedule_config);

                                let margin_x = 5.0;
                                let margin_y = 10.0;
                                let stroke_width = 2.0;
                                let line_gap = 5.0;
                                let line_height = 5.0;

                                let max_x = max_rect.width() - 2.0 * margin_x;

                                let inbetween_width_max = 10.0;
                                let inbetween_width_percentage = 0.2;

                                let mut cell_width = if n <= 1 { max_x } else { max_x * (1.0 - inbetween_width_percentage) / n as f32 };
                                let mut inbetween_width = if n <= 1 { 0.0 } else { max_x * inbetween_width_percentage / (n - 1) as f32 };
                                if inbetween_width > inbetween_width_max {
                                    inbetween_width = inbetween_width_max;
                                    cell_width = (max_x - (inbetween_width * (n - 1) as f32)) / n as f32;
                                }

                                let start_x = max_rect.left() + margin_x;
                                let start_y = ui.cursor().top() + margin_y;

                                let mut x = start_x;
                                let mut y = start_y;

                                for GbpScheduleAtTimestep { internal, external } in schedule {
                                    let internal_color = if internal {
                                        Color32::from_catppuccin_colour(theme.sky())
                                    } else {
                                        Color32::from_catppuccin_colour(theme.overlay0())
                                    };
                                    let external_color = if external {
                                        Color32::from_catppuccin_colour(theme.maroon())
                                    } else {
                                        Color32::from_catppuccin_colour(theme.overlay0())
                                    };
                                    // let external_color = if external { Color32::RED } else { Color32::GRAY };

                                    let start_pos = egui::Pos2::new(x, y);
                                    let end_pos = egui::Pos2::new(x + cell_width, y);
                                    painter.line_segment(
                                        [start_pos, end_pos],
                                        egui::Stroke::new(stroke_width, internal_color),
                                    );
                                    y += line_height + line_gap;

                                    let start_pos = egui::Pos2::new(x, y);
                                    let end_pos = egui::Pos2::new(x + cell_width, y);
                                    painter.line_segment(
                                        [start_pos, end_pos],
                                        egui::Stroke::new(stroke_width, external_color),
                                    );

                                    x += cell_width + inbetween_width;
                                    y = start_y;
                                }

                                // ui.add_space(100.0);
                                ui.add_visible(false, egui::Label::new("ghost"));
                                // ui.end_row();
                                // ui.add_space(100.0);
                            }

                        ui.add_space(20.0);

                        custom::grid("select_gbp_schedule_grid", 2).show(ui, |ui| {
                            ui.label("Schedule");
                            ui.vertical_centered_justified(|ui| {
                                let current: &'static str = config.gbp.iteration_schedule.schedule.into();
                                ui.menu_button(current, |ui| {
                                    for schedule in gbp_config::GbpIterationScheduleKind::iter() {
                                        ui.vertical_centered_justified(|ui| {
                                            let text: &'static str = schedule.into();
                                            if ui.button(text).clicked() {
                                                let new_schedule = schedule.into();
                                                config.gbp.iteration_schedule.schedule = new_schedule;
                                                world.send_event::<crate::planner::robot::GbpScheduleChanged>(config.gbp.iteration_schedule.into());
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                })
                            });

                            ui.end_row();
                        });

                        // ui.separator();
                        //
                        // custom::grid("factors_grid", 2).show(ui, |ui| {
                        //     ui.label("Obstacle");
                        //
                        //     custom::float_right(ui, |ui| {
                        //         let mut obstacle: bool = true;
                        //         if custom::toggle_ui(ui, &mut obstacle).clicked() {
                        //             error!("todo");
                        //         }
                        //     });
                        //     ui.end_row();
                        //
                        //     ui.label("Pose");
                        //     custom::float_right(ui, |ui| {
                        //         let mut toggle: bool = true;
                        //         if custom::toggle_ui(ui, &mut toggle).clicked() {
                        //             error!("todo");
                        //         }
                        //     });
                        //     ui.end_row();
                        //
                        //     ui.label("Dynamic");
                        //     custom::float_right(ui, |ui| {
                        //         let mut toggle: bool = true;
                        //         if custom::toggle_ui(ui, &mut toggle).clicked() {
                        //             error!("todo");
                        //         }
                        //     });
                        //     ui.end_row();
                        //
                        //     ui.label("Interrobot");
                        //     custom::float_right(ui, |ui| {
                        //         let mut toggle: bool = true;
                        //         if custom::toggle_ui(ui, &mut toggle).clicked() {
                        //             error!("todo");
                        //         }
                        //     });
                        //     ui.end_row();
                        //
                        // });
                        //
                        // ui.add_space(2.5);

                        custom::grid("gbp_grid", 2).show(ui, |ui| {
                            ui.label("Safety Distance");
                            ui.spacing_mut().slider_width = ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                            let mut safety_dist_multiplier = config.robot.inter_robot_safety_distance_multiplier.get();
                            // let mut available_size = ui.available_size();
                            // available_size.x += 10.0;
                            // let slider_response = ui.add_sized(available_size,
                               let slider_response = ui.add_enabled(
                                   time_virtual.is_paused(),
                                      egui::Slider::new(&mut safety_dist_multiplier, 1.0..=10.0)
                                    .suffix("r")
                                    // .text(" * radius")
                                    .fixed_decimals(1)
                                    .trailing_fill(true));
                            if slider_response.enabled() && slider_response.changed() {
                                config.robot.inter_robot_safety_distance_multiplier = safety_dist_multiplier.try_into().expect("slider range set to [0.1, 10.0]");

                                let mut query = world.query::<&mut FactorGraph>();
                                for mut factorgraph in query.iter_mut(world) {
                                    factorgraph.update_inter_robot_safety_distance_multiplier(Float::from(config.robot.inter_robot_safety_distance_multiplier.get()).try_into().expect("> 0.0"));
                                }
                            }

                            ui.end_row();

                            ui.label("Max Speed");
                            // slider for robot max speed  in (0.0, 10.]
                            // ui.spacing_mut().slider_width = ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING - 16.0);
                            ui.spacing_mut().slider_width = ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);

                            let mut max_speed = config.robot.max_speed.get();
                            // let mut available_size = ui.available_size();
                            // available_size.x += 10.0;
                            // let slider_response = ui.add_sized(available_size,
                               let slider_response = ui.add_enabled(
                                   time_virtual.is_paused(),
                                                                      egui::Slider::new(&mut max_speed, 0.1..=100.0)

                                    .suffix("m/s")
                                    .fixed_decimals(1)
                                    .trailing_fill(true));
                            if slider_response.enabled() && slider_response.changed() {
                                config.robot.max_speed = max_speed.try_into().expect("slider range set to [0.1, 10.0]");
                            }

                            ui.end_row();
                        });
                    }

                    custom::subheading(ui, "Communication",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );


                    custom::grid("communication_grid", 2).show(ui,|ui| {
                        ui.label("Radius");
                        // slider for communication radius in (0.0, 50.]
                        ui.spacing_mut().slider_width =
                            ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let mut comms_radius = config.robot.communication.radius.get();
                        let slider_response = ui.add(
                            egui::Slider::new(&mut comms_radius, 0.1..=50.0)
                                .suffix("m")
                                .fixed_decimals(1)
                                .trailing_fill(true)

                        );
                        if slider_response.changed() {
                            config.robot.communication.radius = comms_radius.try_into().expect("slider range set to [0.1, 50.0]");
                            // TODO: this should not be done with a query here, but there is not
                            // much time left.
                            let mut query = world.query::<&mut RadioAntenna>();
                            for mut antenna in query.iter_mut(world) {
                                antenna.radius = comms_radius;
                            }
                        }
                        ui.end_row();
                        // Slider for communication failure rate (probability) in [0.0, 1.0]
                        ui.label("Failure");
                        ui.spacing_mut().slider_width = ui.available_width()  - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let mut failure_rate = config.robot.communication.failure_rate;
                        let slider_response = ui.add(
                            egui::Slider::new(&mut failure_rate, 0.0..=1.0)
                                .suffix("%")
                                .fixed_decimals(2)
                                .trailing_fill(true)
                        );
                        if slider_response.changed() {
                            config.robot.communication.failure_rate = failure_rate;
                        }
                        ui.end_row();
                    });


                    custom::subheading(
                        ui,
                        "Draw",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );

                    custom::grid("draw_special_grid", 4).show(ui, |ui| {
                        custom::fill_x(ui, |ui| {
                            if ui.button("None").clicked() {
                                let events = config.visualisation.draw
                                    .iter()
                                    .filter_map(|(name, _value)| name.parse::<DrawSetting>().ok())
                                    .map(|setting| DrawSettingsEvent {setting, draw: false} );
                                world.send_event_batch(events);

                                config.visualisation.draw = DrawSection::all_disabled();
                            }
                        });
                        custom::fill_x(ui, |ui| {
                            if ui.button("All").clicked() {
                                let events = config.visualisation.draw
                                    .iter()
                                    .filter_map(|(name, _value)| name.parse::<DrawSetting>().ok())
                                    .map(|setting| DrawSettingsEvent {setting, draw: true} );
                                world.send_event_batch(events);

                                config.visualisation.draw = DrawSection::all_enabled();
                            }
                        });
                        custom::fill_x(ui, |ui| {
                            if ui.button("Flip").clicked() {
                                let events = config.visualisation.draw
                                    .iter()
                                    .filter_map(|(name, value)|{
                                        if let (Ok(setting), Some(value)) = (name.parse::<DrawSetting>(), value.downcast_ref::<bool>()) {
                                            Some(DrawSettingsEvent {setting, draw: !value })
                                        } else {
                                            None
                                        }
                                    });
                                world.send_event_batch(events);

                                config.visualisation.draw.flip_all();
                            }
                        });
                        custom::fill_x(ui, |ui| {
                            if ui.button("Reset").clicked() {
                                let unmodified_draw_section = simulation_manager.active().map(|sim| &sim.config.visualisation.draw).unwrap();
                                let events = unmodified_draw_section
                                    .iter()
                                    .filter_map(|(name, value)|{
                                        if let (Ok(setting), Some(value)) = (name.parse::<DrawSetting>(), value.downcast_ref::<bool>()) {
                                            Some(DrawSettingsEvent {setting, draw: *value })
                                        } else {
                                            None
                                        }
                                    });
                                world.send_event_batch(events);
                                config.visualisation.draw = *unmodified_draw_section;
                            }
                        });

                        ui.end_row();
                    });

                    ui.add_space(2.5);
                    ui.separator();

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
                                    println!("name: {}", name);
                                    if let Ok(setting_kind) = name.parse::<DrawSetting>() {
                                        let event = DrawSettingsEvent {
                                            setting: setting_kind,
                                            draw:    *setting,
                                        };
                                        world.send_event::<DrawSettingsEvent>(event);
                                    } else {
                                        error!("Failed to parse into a `DrawSection`: {}", name);
                                    }
                                    //
                                    // let setting_kind: DrawSetting = name.parse().expect(
                                    //     "the ui strings are generated from the enum, so parse \
                                    //      should not fail",
                                    // );
                                    // let event = DrawSettingsEvent {
                                    //     setting: setting_kind,
                                    //     draw:    *setting,
                                    // };
                                    // world.send_event::<DrawSettingsEvent>(event);
                                }
                            });
                            ui.end_row();
                        }

                    });

                    ui.add_space(2.5);
                    ui.add(egui::Separator::default().shrink(20.0));
                    custom::grid("special_draw_grid", 2).show(ui, |ui| {
                        // GIZMOS
                        let (gizmo_config, _) =
                            config_store.config_mut::<DefaultGizmoConfigGroup>();
                        ui.label("Gizmos");
                        custom::float_right(ui, |ui| {
                            custom::toggle_ui(ui, &mut gizmo_config.enabled);
                        });

                        ui.end_row();

                        // INFINITE GRID
                        let mut query = world.query::<(&mut Visibility, &InfiniteGrid)>();
                        if let Ok((mut visibility, infinite_grid)) = query.get_single_mut(world) {
                            ui.label("Infinite Grid");
                            custom::float_right(ui, |ui| {
                                let mut visible = Visibility::Hidden != *visibility;
                                let response = custom::toggle_ui(ui, &mut visible);
                                if response.changed() {
                                    *visibility = if visible { Visibility::Visible } else { Visibility::Hidden };
                                }
                            });
                        }
                    });


                    custom::subheading(
                        ui,
                        "Simulation",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );
                    custom::grid("simulation_settings_grid", 2).show(ui, |ui| {
                        ui.label("Active Simulation");
                        custom::grid("simulation_settings_grid", 2).show(ui, |ui| {

                            ui.centered_and_justified(|ui| {
                                if ui.button("󰑓").on_hover_text("Reload the active simulation").clicked() {
                                    simulation_manager.reload();
                                }
                            });

                            // Combo box of available simulations
                            ui.vertical_centered_justified(|ui| {
                                ui.menu_button(simulation_manager.active_name().map(ToString::to_string).unwrap_or(format!("N/A")), |ui| {
                                    #[allow(clippy::needless_collect)] // clippy is wrong about
                                    // this one
                                    for (id, sim) in simulation_manager.ids_and_names().collect::<Vec<(SimulationId, SmolStr)>>()  {
                                        ui.vertical_centered_justified(|ui| {
                                            let name: String = sim.into();
                                            if ui.button(name).clicked() {
                                                simulation_manager.load(id);
                                                ui.close_menu();
                                            }
                                        });
                                    }
                                });
                            });
                            ui.end_row();
                        });

                        ui.end_row();

                        ui.label("Simulation Time");

                        custom::rect_label(
                            ui,
                            format!(
                                "{:.2} / {:.2} s",
                                time_virtual.elapsed_seconds(),
                                config.simulation.max_time.get()
                            ),
                            None,
                        );
                        ui.end_row();

                        ui.label("Δt");
                        let dt = time_fixed.delta_seconds();
                        let hz = (1.0 / dt).ceil() as u32;
                        custom::rect_label(ui, format!("{:.4} s = {} Hz", dt, hz), None);
                        ui.end_row();

                        // slider for simulation time between 0 and 100
                        ui.label("Simulation Speed");
                        // slider for simulation speed (time scale) between 0.1 and 10
                        ui.spacing_mut().slider_width =
                            ui.available_width() - (custom::SLIDER_EXTRA_WIDE + custom::SPACING);
                        let slider_response = ui.add(
                            egui::Slider::new(&mut config.simulation.time_scale.get(), 0.1..=5.0)
                                .suffix("x")
                                .trailing_fill(true)
                                .show_value(true),
                        );
                        if slider_response.drag_released() || slider_response.lost_focus() {
                            info!("time scale changed: {}", config.simulation.time_scale);
                            time_virtual.set_relative_speed(config.simulation.time_scale.get());
                        }
                        ui.end_row();

                        ui.label("Manual Controls");

                        custom::grid("manual_controls_settings_grid", 2).show(ui, |ui| {
                            // step forward button
                            // ui.add_enabled_ui(!pause_state.is_paused(), |ui| {
                            ui.add_enabled_ui(!time_virtual.is_paused(), |ui| {
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
                            // let pause_play_text = if pause_state.is_paused() {
                            let pause_play_text = if time_virtual.is_paused() {
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


                        });
                        ui.end_row();

                        ui.label("Timesteps per Step");

                        let mut text = config.manual.timesteps_per_step.to_string();

                        let te_output = egui::TextEdit::singleline(&mut text)
                            .char_limit(3)
                            .interactive(time_virtual.is_paused())
                            .show(ui);

                        if te_output.response.changed() {
                            match text.parse::<usize>() {
                                Ok(x) if x >= 1 => {
                                    config.manual.timesteps_per_step = x.try_into().unwrap();

                                },
                                _ => {
                                    error!("failed to parse {} as usize", text);
                                },
                            }
                        }
                    });

                    custom::subheading(
                        ui,
                        "Export",
                        Some(Color32::from_catppuccin_colour(
                            title_colors.next_or_first(),
                        )),
                    );

                    custom::grid("export_grid", 3).show(ui, |ui| {
                        // GRAPHVIZ EXPORT TOGGLE
                        ui.label("Graphviz");
                        custom::fill_x(ui, |ui| {
                            if ui.button("Export").clicked() {
                                world.send_event::<ExportFactorGraphAsGraphviz>(ExportFactorGraphAsGraphviz);
                            }
                        });
                        custom::fill_x(ui, |ui| {
                            if ui.button("Open").clicked() {

                                if cfg!(target_arch = "wasm32") {
                                    world.send_event::<ToastEvent>(ToastEvent::warning("Not supported on wasm32"));
                                } else {
                                    let png_output_path = Path::new("factorgraphs.png");

                                    if !png_output_path.exists() {
                                        world.send_event::<ToastEvent>(ToastEvent::warning("No factorgraph has been exported yet"));
                                    } else {
                                        if let Err(err) = open::that_detached(png_output_path) {
                                            let err_msg = format!("Failed to open {}: {}", png_output_path.display(), err);
                                            error!(err_msg);
                                            world.send_event::<ToastEvent>(ToastEvent::error(err_msg));
                                        }
                                    }
                                }
                            }
                        });

                        ui.end_row();

                        ui.label("Metrics");
                        custom::fill_x(ui, |ui| {
                            if ui.button("Export").clicked() {
                                use crate::export::events::Export;
                                world.send_event::<Export>(Export {
                                    toast: true,
                                    ..Default::default()
                                });
                            }
                        });

                        custom::fill_x(ui, |ui| {
                            if ui.button("Open").clicked() {
                                if cfg!(target_arch = "wasm32") {
                                    world.send_event::<ToastEvent>(ToastEvent::warning("Not supported on wasm32"));
                                } else {
                                    use crate::export::events::OpenLatestExport;
                                    world.send_event::<OpenLatestExport>(OpenLatestExport);
                                }
                            }
                        });
                    });

                    ui.add_space(10.0);

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
