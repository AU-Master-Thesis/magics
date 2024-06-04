//! Module for pausing and resuming the simulation.

use bevy::prelude::*;

/// Plugin for pausing and resuming the simulation.
#[derive(Default)]
pub struct PausePlayPlugin;

impl Plugin for PausePlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PausePlay>()
            .add_systems(PreUpdate, pause_play_virtual_time);
    }
}

/// Event for pausing and resuming the simulation.
#[derive(Debug, Clone, Copy, Default, Event, PartialEq, Eq)]
pub enum PausePlay {
    #[default]
    Toggle,
    Pause,
    Play,
}

/// System that reacts to events for pausing and resuming the simulation.
fn pause_play_virtual_time(
    mut evr_pause_play: EventReader<PausePlay>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    for pause_play in evr_pause_play.read() {
        match pause_play {
            PausePlay::Pause => {
                virtual_time.pause();
            }
            PausePlay::Play => {
                virtual_time.unpause();
            }
            PausePlay::Toggle => {
                if virtual_time.is_paused() {
                    virtual_time.unpause();
                } else {
                    virtual_time.pause();
                }
            }
        }
    }
}
