//! Module for pausing and resuming the simulation.

use bevy::prelude::*;

/// Plugin for pausing and resuming the simulation.
#[derive(Default)]
pub struct PausePlayPlugin;

impl Plugin for PausePlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PausedState::default())
            .add_event::<PausePlay>()
            .add_systems(PreUpdate, pause_play_virtual_time);
    }
}

// TODO: remove this state, a Time<Virtual> already keeps track of it itself
/// State keeping track of whether the simulation is paused or not.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, derive_more::IsVariant)]
pub enum PausedState {
    #[default]
    Running,
    Paused,
}

/// Event for pausing and resuming the simulation.
#[derive(Debug, Clone, Copy, Default, Event)]
pub enum PausePlay {
    #[default]
    Toggle,
    Pause,
    Play,
}

/// System that reacts to events for pausing and resuming the simulation.
fn pause_play_virtual_time(
    paused_state: ResMut<State<PausedState>>,
    mut next_paused_state: ResMut<NextState<PausedState>>,
    mut evr_pause_play: EventReader<PausePlay>,
    mut virtual_time: ResMut<Time<Virtual>>,
) {
    for pause_play_event in evr_pause_play.read() {
        match (pause_play_event, paused_state.get()) {
            (PausePlay::Pause, PausedState::Paused) | (PausePlay::Play, PausedState::Running) => {
                warn!("ignoring duplicate event: {:?}", pause_play_event);
            }
            (PausePlay::Pause | PausePlay::Toggle, PausedState::Running) => {
                next_paused_state.set(PausedState::Paused);
                virtual_time.pause();
            }
            (PausePlay::Play | PausePlay::Toggle, PausedState::Paused) => {
                next_paused_state.set(PausedState::Running);
                virtual_time.unpause();
            }
        };
    }
}
