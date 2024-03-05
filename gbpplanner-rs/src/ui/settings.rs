use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Color32, RichText},
    EguiContexts,
};

use crate::theme::{CatppuccinTheme, ThemeEvent};

use super::{OccupiedScreenSpace, ToDisplayString, UiState};

pub struct SettingsPanelPlugin;

impl Plugin for SettingsPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (ui_settings_panel,));
    }
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

/// **Bevy** `Update` system to display the `egui` settings panel
fn ui_settings_panel(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut theme_event: EventWriter<ThemeEvent>,
    catppuccin_theme: Res<CatppuccinTheme>,
) {
    let ctx = contexts.ctx_mut();

    let right_panel = egui::SidePanel::right("Settings Panel")
        .default_width(200.0)
        .resizable(false)
        .show_animated(ctx, ui_state.right_panel, |ui| {
            ui.add_space(10.0);
            ui.heading("Settings");
            ui.add_space(5.0);
            ui.separator();

            egui::ScrollArea::vertical()
                .drag_to_scroll(true)
                .show(ui, |ui| {
                    ui.add_space(10.0);
                    egui::Grid::new("cool_grid")
                        .num_columns(2)
                        .min_col_width(100.0)
                        .striped(false)
                        .spacing((10.0, 10.0))
                        .show(ui, |ui| {
                            // toggle_ui(ui, &mut theme);
                            ui.label("Select Theme:");
                            ui.vertical_centered_justified(|ui| {
                                ui.menu_button(
                                    catppuccin_theme.flavour.to_display_string(),
                                    |ui| {
                                        ui.set_width(100.0);
                                        for flavour in &[
                                            catppuccin::Flavour::Frappe,
                                            catppuccin::Flavour::Latte,
                                            catppuccin::Flavour::Macchiato,
                                            catppuccin::Flavour::Mocha,
                                        ] {
                                            ui.vertical_centered_justified(|ui| {
                                                if ui.button(flavour.to_display_string()).clicked()
                                                {
                                                    theme_event.send(ThemeEvent(*flavour));
                                                    ui.close_menu();
                                                }
                                            });
                                        }
                                    },
                                );
                            });
                            ui.end_row();
                            ui.label("New row");
                        });
                });
        });

    occupied_screen_space.right = right_panel
        .map(|ref inner| inner.response.rect.width())
        .unwrap_or(0.0);
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // Widget code can be broken up in four steps:
    //  1. Decide a size for the widget
    //  2. Allocate space for it
    //  3. Handle interactions with the widget (if any)
    //  4. Paint the widget

    // 1. Deciding widget size:
    // You can query the `ui` how much space is available,
    // but in this example we have a fixed size widget based on the height of a standard button:
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);

    // 2. Allocating space:
    // This is where we get a region of the screen assigned.
    // We also tell the Ui to sense clicks in the allocated region.
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    // 3. Interact: Time to check for clicks!
    if response.clicked() {
        *on = !*on;
        response.mark_changed(); // report back that the value changed
    }

    // Attach some meta-data to the response which can be used by screen readers:
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));

    // 4. Paint!
    // Make sure we need to paint:
    if ui.is_rect_visible(rect) {
        // Let's ask for a simple animation from egui.
        // egui keeps track of changes in the boolean associated with the id and
        // returns an animated value in the 0-1 range for how much "on" we are.
        let how_on = ui.ctx().animate_bool(response.id, *on);
        // We will follow the current style by asking
        // "how should something that is being interacted with be painted?".
        // This will, for instance, give us different colors when the widget is hovered or clicked.
        let visuals = ui.style().interact_selectable(&response, *on);
        // All coordinates are in absolute screen coordinates so we use `rect` to place the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        // Paint the circle, animating it from left to right with `how_on`:
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter()
            .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    // All done! Return the interaction response so the user can check what happened
    // (hovered, clicked, ...) and maybe show a tooltip:
    response
}
