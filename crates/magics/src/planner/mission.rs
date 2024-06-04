use bevy::prelude::*;

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {}
}

pub mod components {
    use std::collections::VecDeque;

    use super::*;

    #[derive(Component)]
    pub struct Mission {
        pub tasks: VecDeque<MissionTask>,
    }
}

pub struct MissionTask;
