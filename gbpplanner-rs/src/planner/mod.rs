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
pub use visualiser::{factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser};

use self::{
    debug::PlannerDebugPlugin, robot::RobotPlugin, spawner::SpawnerPlugin,
    visualiser::VisualiserPlugin,
};

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PausePlay>()
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

/// **Bevy** [`Resource`] for pausing or playing the simulation
#[derive(Default, Resource)]
pub struct PausePlay(bool);

impl PausePlay {
    pub fn pause(&mut self) {
        self.0 = false;
    }

    pub fn play(&mut self) {
        self.0 = true;
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    pub fn is_paused(&self) -> bool {
        !self.0
    }
}

#[allow(dead_code)]
#[derive(Event, Clone, Debug, Default)]
pub enum PausePlayEvent {
    #[default]
    Toggle,
    Pause,
    Play,
}

fn pause_play_simulation(
    mut pause_play: ResMut<PausePlay>,
    mut pause_play_event_reader: EventReader<PausePlayEvent>,
    mut time: ResMut<Time<Virtual>>,
) {
    for pause_play_event in pause_play_event_reader.read() {
        error!("received event: {:?}", pause_play_event);
        match pause_play_event {
            PausePlayEvent::Toggle => pause_play.toggle(),
            PausePlayEvent::Pause if !pause_play.is_paused() => pause_play.pause(),
            PausePlayEvent::Pause => pause_play.play(),
            PausePlayEvent::Play if pause_play.is_paused() => pause_play.play(),
            PausePlayEvent::Play => pause_play.pause(),
        }

        if pause_play.is_paused() {
            time.unpause();
            // time.set_relative_speed(1.0);
        } else {
            // time.set_relative_speed(0.0);
            time.pause();
        }
    }
}
