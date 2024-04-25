#![allow(missing_docs)]

use bevy::{input::common_conditions::input_just_pressed, prelude::*};
use bevy_notify::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NotifyPlugin {
            anchor: Anchor::BottomCenter,
            ..Default::default()
        })
        .add_systems(Update, create_toast.run_if(input_just_pressed(KeyCode::Space)))
        .run();
}

fn create_toast(mut toast_event: EventWriter<ToastEvent>) {
    toast_event.send(ToastEvent {
        caption: "hello".into(),
        options: ToastOptions {
            level: ToastLevel::Success,
            // closable: false,
            // show_progress_bar: false,
            ..Default::default()
        },
    });
}
