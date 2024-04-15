//! Bevy touchpad plugin
use bevy::{input::mouse::MouseWheel, prelude::*};

/// prelude module to bring entire public API into scope
pub mod prelude {
    pub use crate::{BevyTouchpadPlugin, TwoFingerSwipe, TwoFingerSwipeDirection};
}

/// **Bevy** plugin that listens for two finger swipes
#[derive(Debug, Default)]
pub struct BevyTouchpadPlugin;

impl Plugin for BevyTouchpadPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TwoFingerSwipe>()
            .add_systems(Update, detect_two_finger_swipe);
    }
}

/// **Bevy** event that contains the magnitude and direction of a two finger
/// swipe
#[derive(Debug, Clone, Copy, Event)]
pub struct TwoFingerSwipe {
    /// The magnitude of the swipe, guaranteeed to > 0.0
    pub magnitude: f32,
    /// The direction of the swipe
    pub direction: TwoFingerSwipeDirection,
}

/// The direction of a two finger swipe
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoFingerSwipeDirection {
    /// Up
    Up,
    /// Down
    Down,
    /// Left
    Left,
    /// Right
    Right,
    // UpLeft,
    // UpRight,
    // DownLeft,
    // DownRight,
}

/// **Bevy** system that detects two finger swipes
fn detect_two_finger_swipe(
    mut evr_scroll: EventReader<MouseWheel>,
    mut evw_two_finger_swipe: EventWriter<TwoFingerSwipe>,
) {
    for event in evr_scroll.read() {
        // let MouseScrollUnit::Pixel = event.unit else {
        //     return;
        // };

        let (x, y) = (event.x, event.y);
        // println!("x: {x}, y: {y}");

        if x == 0.0 && y == 0.0 {
            continue;
        }

        // let direction = if x > 0.0 {
        //     if y > 0.0 {
        //         TwoFingerSwipeDirection::UpRight
        //     } else if y < 0.0 {
        //         TwoFingerSwipeDirection::DownRight
        //     } else {
        //         TwoFingerSwipeDirection::Right
        //     }
        // } else if x < 0.0 {
        //     if y > 0.0 {
        //         TwoFingerSwipeDirection::UpLeft
        //     } else if y < 0.0 {
        //         TwoFingerSwipeDirection::DownLeft
        //     } else {
        //         TwoFingerSwipeDirection::Left
        //     }
        // } else {
        //     if y > 0.0 {
        //         TwoFingerSwipeDirection::Up
        //     } else if y < 0.0 {
        //         TwoFingerSwipeDirection::Down
        //     } else {
        //         continue;
        //     }
        // };
        //

        let (direction, magnitude) = if x == 0.0 {
            if y < 0.0 {
                (TwoFingerSwipeDirection::Up, y.abs())
            } else {
                (TwoFingerSwipeDirection::Down, y)
            }
        } else if y == 0.0 {
            if x < 0.0 {
                (TwoFingerSwipeDirection::Left, x.abs())
            } else {
                (TwoFingerSwipeDirection::Right, x)
            }
        } else {
            continue;
        };

        evw_two_finger_swipe.send(TwoFingerSwipe {
            // magnitude: (x * x + y * y).sqrt(),
            magnitude,
            direction,
        });
    }
}
