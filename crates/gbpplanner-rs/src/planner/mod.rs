mod debug;
mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod message;
pub mod robot;
mod spawner;
mod variable;
mod visualiser;

use bevy::prelude::*;
pub use factorgraph::{graphviz::NodeKind, FactorGraph, NodeIndex};
pub use robot::{RobotId, RobotState};
pub use visualiser::{
    factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser, RobotTracker,
};

use self::{
    debug::PlannerDebugPlugin, robot::RobotPlugin, spawner::SpawnerPlugin,
    visualiser::VisualiserPlugin,
};

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PausedState::Running)
            .add_event::<PausePlayEvent>()
            .add_systems(PreUpdate, pause_play_simulation)
            .add_plugins((
                RobotPlugin,
                SpawnerPlugin,
                VisualiserPlugin,
                PlannerDebugPlugin,
            ));
    }
}

// TODO(kpbaks): move out of planner

// /// **Bevy** [`Resource`] for pausing or playing the simulation
// #[derive(Default, Resource)]
// pub struct PausePlay(bool);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PausedState {
    #[default]
    Running,
    Paused,
}

impl PausedState {
    /// Returns `true` if the paused state is [`Running`].
    ///
    /// [`Running`]: PausedState::Running
    #[must_use]
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns `true` if the paused state is [`Paused`].
    ///
    /// [`Paused`]: PausedState::Paused
    #[must_use]
    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused)
    }
}

// impl PausePlay {
//     pub fn pause(&mut self) {
//         self.0 = false;
//     }

//     pub fn play(&mut self) {
//         self.0 = true;
//     }

//     pub fn toggle(&mut self) {
//         self.0 = !self.0;
//     }

//     pub fn is_paused(&self) -> bool {
//         !self.0
//     }
// }

#[allow(dead_code)]
#[derive(Event, Clone, Debug, Default)]
pub enum PausePlayEvent {
    #[default]
    Toggle,
    Pause,
    Play,
}

fn pause_play_simulation(
    state: ResMut<State<PausedState>>,
    mut next_state: ResMut<NextState<PausedState>>,
    mut pause_play_event_reader: EventReader<PausePlayEvent>,
    mut time: ResMut<Time<Virtual>>,
) {
    for pause_play_event in pause_play_event_reader.read() {
        info!("received event: {:?}", pause_play_event);
        match (pause_play_event, state.get()) {
            (PausePlayEvent::Pause, PausedState::Paused)
            | (PausePlayEvent::Play, PausedState::Running) => {
                warn!("ignoring duplicate event: {:?}", pause_play_event);
                continue;
            }
            (PausePlayEvent::Pause | PausePlayEvent::Toggle, PausedState::Running) => {
                next_state.set(PausedState::Paused);
                time.pause();
            }
            (PausePlayEvent::Play | PausePlayEvent::Toggle, PausedState::Paused) => {
                next_state.set(PausedState::Running);
                time.unpause();
            }
        };
    }
}
