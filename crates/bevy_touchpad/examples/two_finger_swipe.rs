use bevy::prelude::*;
use bevy_touchpad::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyTouchpadPlugin::default())
        .add_systems(Update, listen_for_two_finger_swipe)
        .run();
}

fn listen_for_two_finger_swipe(mut evw_two_finger_swipe: EventReader<TwoFingerSwipe>) {
    for event in evw_two_finger_swipe.read() {
        info!("TwoFingerSwipe: {event:?}");
    }
}
