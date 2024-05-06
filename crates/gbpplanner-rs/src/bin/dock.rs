#![allow(missing_docs)]

use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui,
    egui::{Ui, WidgetText},
    EguiContexts, EguiPlugin,
};
use egui_dock::{DockArea, DockState, Style, TabViewer};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<MyTabs>()
        .init_resource::<Enabled>()
        .add_systems(Update, render)
        .add_systems(Update, toggle_enabled.run_if(input_just_pressed(KeyCode::Space)));

    app.run();
}

// First, let's pick a type that we'll use to attach some data to each tab.
// It can be any type.
type Tab = String;

// To define the contents and properties of individual tabs, we implement the
// `TabViewer` trait. Only three things are mandatory: the `Tab` associated
// type, and the `ui` and `title` methods. There are more methods in `TabViewer`
// which you can also override.
struct MyTabViewer;

impl TabViewer for MyTabViewer {
    // This associated type is used to attach some data to each tab.
    type Tab = Tab;

    // Returns the current `tab`'s title.
    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.as_str().into()
    }

    // Defines the contents of a given `tab`.
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.label(format!("Content of {tab}"));
    }
}

// Here is a simple example of how you can manage a `DockState` of your
// application.
#[derive(Resource)]
struct MyTabs {
    dock_state: DockState<Tab>,
}

impl Default for MyTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl MyTabs {
    pub fn new() -> Self {
        // Create a `DockState` with an initial tab "tab1" in the main `Surface`'s root
        // node.
        let tabs = ["tab1", "tab2", "tab3"].map(str::to_string).into_iter().collect();
        let dock_state = DockState::new(tabs);
        Self { dock_state }
    }

    fn ui(&mut self, ui: &mut Ui) {
        // Here we just display the `DockState` using a `DockArea`.
        // This is where egui handles rendering and all the integrations.
        //
        // We can specify a custom `Style` for the `DockArea`, or just inherit
        // all of it from egui.
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut MyTabViewer);
    }
}

#[derive(Debug, Resource, Default, Deref, DerefMut)]
pub struct Enabled(pub bool);

fn toggle_enabled(mut enabled: ResMut<Enabled>) {
    enabled.0 = !enabled.0;
}

fn render(
    mut egui_ctx: EguiContexts,
    mut tabs: ResMut<MyTabs>,
    enabled: Res<Enabled>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = primary_window.single();

    let _window = egui::Window::new("main window")
        .default_height(primary_window.height())
        .default_width(primary_window.width())
        .collapsible(false)
        .movable(false)
        .enabled(enabled.0)
        // .default_width(1080.0)
        // .default_height(780.0)
        .show(egui_ctx.ctx_mut(), |ui| {
            tabs.ui(ui);
        });
}

// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// enum Pane {
//     Settings,
//     Text(String),
// }
