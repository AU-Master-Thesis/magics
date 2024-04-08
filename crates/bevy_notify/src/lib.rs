//! bevy_notify

use std::{borrow::Borrow, time::Duration};

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};

// Reexport symbols from egui_notify
pub use egui_notify::Anchor;
pub use egui_notify::ToastLevel;

pub mod prelude {
    pub use super::{Anchor, NotifyPlugin, ToastEvent, ToastLevel, ToastOptions};
}

pub struct NotifyPlugin {
    pub anchor: egui_notify::Anchor,
}

impl Default for NotifyPlugin {
    fn default() -> Self {
        Self {
            anchor: egui_notify::Anchor::TopCenter,
        }
    }
}

impl Plugin for NotifyPlugin {
    fn build(&self, app: &mut App) {
        if app.get_added_plugins::<EguiPlugin>().is_empty() {
            app.add_plugins(EguiPlugin);
        }

        let toasts = egui_notify::Toasts::new().with_anchor(self.anchor);

        app.insert_resource(Toasts(toasts))
            .add_event::<ToastEvent>()
            .add_systems(Update, update_toasts);
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct Toasts(egui_notify::Toasts);

// pub use egui_notify::ToastOptions;

#[derive(Event)]
pub struct ToastEvent {
    pub caption: String,
    pub options: ToastOptions,
}

#[derive(Clone)]
pub struct ToastOptions {
    pub duration: Option<std::time::Duration>,
    pub level: ToastLevel,
    pub show_progress_bar: bool,
    pub closable: bool,
}

impl Default for ToastOptions {
    fn default() -> Self {
        Self {
            duration: Some(Duration::from_secs(5)),
            level: ToastLevel::default(),
            show_progress_bar: true,
            closable: true,
        }
    }
}

pub fn update_toasts(
    mut egui_ctx: EguiContexts,
    mut toasts: ResMut<Toasts>,
    mut toast_event: EventReader<ToastEvent>,
) {
    for ToastEvent {
        caption,
        ref options,
    } in toast_event.read()
    {
        // let level = unsafe { std::mem::transmute_copy(&options.level) };
        let mut toast = egui_notify::Toast::custom(caption, options.level.clone());
        toast
            // .set_level(options.level)
            .set_closable(options.closable)
            .set_show_progress_bar(options.show_progress_bar)
            .set_duration(options.duration);
        toasts.add(toast);
    }

    toasts.show(egui_ctx.ctx_mut());
}
