//! bevy_notify
//!
//! `bevy_notify` is bevy plugin wrapping [egui-notify](https://crates.io/crates/egui-notify)
//!
//! # Examples
//! ```rust
//! use bevy::prelude::*;
//! use bevy::input::common_conditions::input_just_pressed
//! use bevy_notify::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NotifyPlugin)
//!         .add_systems(Update, notify_example_system.run_if(input_just_pressed(KeyCode::Space))
//!         .run();
//! }
//!
//! fn notify_example_system(mut toast_event: EventWriter<ToastEvent>) {
//!      toast_event.send(ToastEvent {
//!          caption: "hello".into(),
//!          options: ToastOptions {
//!              level: ToastLevel::Success,
//!              closable: false,
//!              show_progress_bar: true,
//!              ..Default::default()
//!          },
//!      });
//! }
//! ```

use std::{num::NonZeroU8, time::Duration};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
// Reexport symbols from egui_notify
pub use egui_notify::Anchor;
pub use egui_notify::ToastLevel;

/// Bring all symbols into scope that you need to use this crate
pub mod prelude {
    pub use super::{Anchor, NotifyPlugin, ToastEvent, ToastLevel, ToastOptions};
}

/// Adds event `ToastEvent` to be used in systems.
/// Uses a `Update` system to render the toasts on the screen at the specified
/// anchor
pub struct NotifyPlugin {
    /// Anchor for where to render the toasts in the main window
    /// Defaults to `Anchor::TopCenter`
    pub anchor: egui_notify::Anchor,
    /// Maximum number of toasts to show at once
    /// Defaults to 5
    /// When the max is reached, the oldest toast is removed
    pub max:    NonZeroU8,
}

impl Default for NotifyPlugin {
    fn default() -> Self {
        Self {
            anchor: egui_notify::Anchor::TopCenter,
            // anchor: egui_notify::Anchor::BottomCenter,
            max:    NonZeroU8::new(5).expect("5 > 0"),
        }
    }
}

impl Plugin for NotifyPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }

        let toasts = egui_notify::Toasts::new().with_anchor(self.anchor);

        app.insert_resource(Toasts::new(toasts, self.max))
            .add_event::<ToastEvent>()
            .add_systems(Update, update_toasts);
    }
}

#[derive(Resource)]
struct Toasts {
    toasts: egui_notify::Toasts,
    max:    NonZeroU8,
}

impl Toasts {
    fn new(toasts: egui_notify::Toasts, max: NonZeroU8) -> Self {
        Self { toasts, max }
    }

    fn add(&mut self, toast: egui_notify::Toast) {
        if self.toasts.len() == self.max.get() as usize {
            self.toasts.remove_oldest_toast();
            debug!(
                "removed the oldest toast to satisfy the set max constraint: {}",
                self.max.get()
            );
        }
        self.toasts.add(toast);
    }

    #[inline]
    fn show(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }
}

/// Event for creating a toast
#[derive(Event)]
pub struct ToastEvent {
    /// The caption of the toast
    pub caption: String,
    /// Options for the toast, to configure properties like duration, closable,
    /// etc
    pub options: ToastOptions,
}

impl ToastEvent {
    /// Create an info toast
    /// options is set to `ToastOptions::default()`
    pub fn info(caption: String) -> Self {
        Self {
            caption,
            options: ToastOptions {
                level: ToastLevel::Info,
                ..Default::default()
            },
        }
    }

    /// Create a success toast
    /// options is set to `ToastOptions::default()`
    pub fn success(caption: String) -> Self {
        Self {
            caption,
            options: ToastOptions {
                level: ToastLevel::Success,
                ..Default::default()
            },
        }
    }

    /// Create an error toast
    /// options is set to `ToastOptions::default()`
    pub fn error(caption: String) -> Self {
        Self {
            caption,
            options: ToastOptions {
                level: ToastLevel::Error,
                ..Default::default()
            },
        }
    }

    /// Create a warning toast
    /// options is set to `ToastOptions::default()`
    pub fn warning(caption: String) -> Self {
        Self {
            caption,
            options: ToastOptions {
                level: ToastLevel::Warning,
                ..Default::default()
            },
        }
    }

    /// Create a custom toast
    /// options is set to `ToastOptions::default()`
    pub fn custom(caption: String, level: ToastLevel) -> Self {
        Self {
            caption,
            options: ToastOptions {
                level,
                ..Default::default()
            },
        }
    }
}

/// Options for the toast
#[derive(Debug, Clone)]
pub struct ToastOptions {
    /// Duration of the toast
    /// Defaults to 5 seconds
    /// If `None`, the toast will exist until closed by the user or number of
    /// toasts exceeds the [max](struct.NotifyPlugin.html#structfield.max)
    pub duration:          Option<std::time::Duration>,
    /// Level of the toast
    pub level:             ToastLevel,
    /// Whether to show a progress bar
    /// Defaults to true
    pub show_progress_bar: bool,
    /// Whether the toast can be closed by using the mouse
    /// Defaults to true
    pub closable:          bool,
}

impl Default for ToastOptions {
    /// Returns `ToastOptions` with default values
    fn default() -> Self {
        Self {
            duration:          Some(Duration::from_secs(5)),
            level:             ToastLevel::default(),
            show_progress_bar: true,
            closable:          true,
        }
    }
}

fn update_toasts(
    mut egui_ctx: EguiContexts,
    mut toasts: ResMut<Toasts>,
    mut toast_event: EventReader<ToastEvent>,
) {
    for ToastEvent {
        caption,
        ref options,
    } in toast_event.read()
    {
        debug!("received toast event");
        trace!("toast, caption: {}, options: {:?}", caption, options);
        let mut toast = egui_notify::Toast::custom(caption, options.level.clone());
        toast
            .set_closable(options.closable)
            .set_show_progress_bar(options.show_progress_bar)
            .set_duration(options.duration);
        toasts.add(toast);
    }

    toasts.show(egui_ctx.ctx_mut());
}
