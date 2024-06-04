use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*, time::common_conditions::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .configure_sets(Update, MySystemSet.run_if(not(virtual_time_is_paused)))
        .add_systems(
            Update,
            (
                system_a.run_if(on_timer(Duration::from_secs(1))),
                system_b.run_if(on_timer(Duration::from_secs(2))),
                system_c.run_if(on_timer(Duration::from_secs(3))),
            )
                .in_set(MySystemSet),
        )
        .add_systems(
            Update,
            pause_play_virtual_time.run_if(input_just_pressed(KeyCode::Space)),
        )
        .run();
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct MySystemSet;

fn system_a() {
    println!("system a");
}

fn system_b() {
    println!("system b");
}

fn system_c() {
    println!("system c");
}

/// run criteria if time is not paused
#[inline]
fn virtual_time_is_paused(time: Res<Time<Virtual>>) -> bool {
    time.is_paused()
}

fn pause_play_virtual_time(mut time: ResMut<Time<Virtual>>) {
    if time.is_paused() {
        time.unpause();
        println!("unpause");
    } else {
        time.pause();
        println!("pause");
    }
}
