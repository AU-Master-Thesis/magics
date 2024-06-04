use bevy::prelude::*;
use bevy_dev::prelude::*;

#[bevy_main]
fn main() {
    App::new().add_plugins((DefaultPlugins, DevPlugins)).run();
}
