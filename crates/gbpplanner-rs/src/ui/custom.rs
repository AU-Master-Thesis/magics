#![warn(missing_docs)]

use bevy_egui::egui::{
    self, Align, Align2, Color32, Direction, FontId, Grid, InnerResponse, Layout, RichText, Ui, Vec2b,
};
use egui_extras::{Column, TableBuilder};

/// A simple function to float a widget to the right
pub fn float_right<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.with_layout(Layout::right_to_left(Align::Center), add_contents)
}

/// A simple function to float a widget to the left
pub fn float_left<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.with_layout(Layout::left_to_right(Align::Center), add_contents)
}

/// A simple function to make a widget fill the available space in x
pub fn fill_x<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.with_layout(Layout::centered_and_justified(Direction::TopDown), add_contents)
}

// /// A function to fill in both x and y
// pub fn fill<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) ->
// InnerResponse<R> {     ui.vertical_centered_justified(add_contents)
// }

// A function to simplify vertically centering a widget
pub fn center_y<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    ui.with_layout(Layout::left_to_right(Align::Center), add_contents)
}

/// A separator with color and space before and after
pub fn separator(ui: &mut Ui, color: Option<Color32>, space_before: Option<f32>, space_after: Option<f32>) {
    ui.add_space(space_before.unwrap_or(0.0));
    let before = ui.visuals().widgets.noninteractive.bg_stroke.color;
    ui.visuals_mut().widgets.noninteractive.bg_stroke.color = color.unwrap_or(before);
    ui.separator();
    ui.visuals_mut().widgets.noninteractive.bg_stroke.color = before;
    ui.add_space(space_after.unwrap_or(0.0));
}

/// A heading with color and a separator after
pub fn heading(ui: &mut Ui, text: &str, color: Option<Color32>) {
    ui.add_space(10.0);
    ui.heading(RichText::new(text).color(color.unwrap_or_else(|| ui.visuals().text_color())));
    separator(ui, color, Some(5.0), Some(0.0));
}

/// A subheading with color and a separator after
pub fn subheading(ui: &mut Ui, text: &str, color: Option<Color32>) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(text)
            .size(16.0)
            .color(color.unwrap_or_else(|| ui.visuals().text_color())),
    );
    separator(ui, color, None, Some(5.0));
}

pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    // Widget code can be broken up in four steps:
    //  1. Decide a size for the widget
    //  2. Allocate space for it
    //  3. Handle interactions with the widget (if any)
    //  4. Paint the widget

    // 1. Deciding widget size:
    // You can query the `ui` how much space is available,
    // but in this example we have a fixed size widget based on the height of a
    // standard button: let available_x = ui.available_width();
    // let desired_y = ui.spacing().interact_size.y;
    // let desired_size = egui::vec2(available_x, desired_y);
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
        // This will, for instance, give us different colors when the widget is hovered
        // or clicked.
        let visuals = ui.style().interact_selectable(&response, *on);
        // All coordinates are in absolute screen coordinates so we use `rect` to place
        // the elements.
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
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

/// Label with a rounded rectangle background
pub fn rect_label(ui: &mut egui::Ui, text: String, interact: Option<egui::Sense>) -> egui::Response {
    let available_width = ui.available_width();
    let desired_y = ui.spacing().interact_size.y;

    let desired_size = egui::vec2(available_width, desired_y);

    let interaction = interact.unwrap_or_else(egui::Sense::focusable_noninteractive);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, interaction);

    if response.clicked() {
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let visuals = if interact.is_none() {
            ui.style().visuals.widgets.noninteractive
        } else {
            ui.style().interact_selectable(&response, false)
        };
        // let visuals = ui.style().interact_selectable(&response, false);

        let rect = rect.expand(visuals.expansion);
        let radius = visuals.rounding;

        ui.painter().rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            text,
            FontId::default(),
            visuals.text_color(),
        );
    }

    response
}

pub fn grid(
    // ui: &mut egui::Ui,
    name: &str,
    cols: usize,
    // add_contents: impl FnOnce(&mut Ui) -> R,
) -> Grid {
    Grid::new(name)
        .num_columns(cols)
        .min_col_width(100.0)
        .striped(false)
        .spacing((10.0, 10.0))
    // .show(ui, add_contents)
}

pub const ROW_HEIGHT: f32 = 20.0;
pub const BINDING_ROW_HEIGHT: f32 = 35.0;
pub const FIRST_COL_WIDTH: f32 = 200.0;
pub const BINDING_COL_WIDTH: f32 = 100.0;
pub const SPACING: f32 = 5.0;
pub const SLIDER_EXTRA: f32 = 45.0;
pub const SLIDER_EXTRA_WIDE: f32 = 65.0;

pub fn binding_table(ui: &mut Ui) -> TableBuilder<'_> {
    TableBuilder::new(ui)
        .column(Column::exact(FIRST_COL_WIDTH))
        .columns(Column::exact(BINDING_COL_WIDTH), 2)
        .vscroll(false)
        .auto_shrink(Vec2b::new(false, true))
}

pub fn sens_table(ui: &mut Ui) -> TableBuilder<'_> {
    TableBuilder::new(ui)
        .column(Column::exact(FIRST_COL_WIDTH))
        .column(Column::exact(BINDING_COL_WIDTH * 2.0))
        .vscroll(false)
        .auto_shrink(Vec2b::new(false, true))
        .striped(false)
}
