use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use gbp_config::Config;

use crate::pause_play::PausePlay;

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManualModeState {
    #[default]
    Disabled,
    Enabled {
        iterations_remaining: usize,
    },
}

impl ManualModeState {
    #[inline]
    pub fn enabled(state: Res<State<Self>>) -> bool {
        matches!(state.get(), Self::Enabled { .. })
    }

    #[inline]
    pub fn disabled(state: Res<State<Self>>) -> bool {
        matches!(state.get(), Self::Disabled)
    }
}

pub fn start_manual_step(
    config: Res<Config>,
    manual_mode_state: Res<State<ManualModeState>>,
    mut next_manual_mode_state: ResMut<NextState<ManualModeState>>,
    mut evr_keyboard_input: EventReader<KeyboardInput>,
    mut evw_pause_play: EventWriter<PausePlay>,
) {
    for event in evr_keyboard_input.read() {
        let (KeyCode::KeyM, ButtonState::Pressed) = (event.key_code, event.state) else {
            continue;
        };

        match manual_mode_state.get() {
            ManualModeState::Disabled => {
                next_manual_mode_state.set(ManualModeState::Enabled {
                    iterations_remaining: config.manual.timesteps_per_step.into(),
                });
                evw_pause_play.send(PausePlay::Play);
            }
            ManualModeState::Enabled { .. } => {
                warn!("manual step already in progress");
            }
        }
    }
}

pub fn finish_manual_step(
    state: Res<State<ManualModeState>>,
    mut next_state: ResMut<NextState<ManualModeState>>,
    mut pause_play_event: EventWriter<PausePlay>,
) {
    match state.get() {
        ManualModeState::Enabled {
            iterations_remaining,
        } if (0..=1).contains(iterations_remaining) => {
            next_state.set(ManualModeState::Disabled);
            pause_play_event.send(PausePlay::Pause);
        }
        ManualModeState::Enabled {
            iterations_remaining,
        } => {
            next_state.set(ManualModeState::Enabled {
                iterations_remaining: iterations_remaining - 1,
            });
        }
        ManualModeState::Disabled => {
            error!("manual step not in progress");
        }
    };
}
