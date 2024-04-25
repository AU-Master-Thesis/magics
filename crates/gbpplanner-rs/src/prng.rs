use bevy::prelude::*;
use rand::SeedableRng;

use crate::{
    config::Config,
    simulation_loader::{EndSimulation, LoadSimulation, ReloadSimulation},
};

pub struct PrngPlugin;
// pub struct PrngPlugin {
//     pub seed: u64,
// }

impl Plugin for PrngPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(EntropyPlugin::<WyRand>::default());
        // app.insert_resource(PrngSeed(self.seed));
        // app.add_systems(OnEnter(SimulationStates::Loading),
        // setup_prng_entity);

        // app.add_systems(
        //     Update,
        //     setup_prng_entity
        //         .run_if(on_event::<LoadSimulation>().
        // or_else(on_event::<ReloadSimulation>())), );

        // app.add_systems(OnEnter(SimulationStates::Ended),
        // delete_prng_entity); app.add_systems(
        //     Update,
        //     delete_prng_entity.run_if(on_event::<EndSimulation>()),
        // );
    }
}

// /// Simple resource
// #[derive(Resource)]
// struct PrngSeed(u64);
//
// impl FromWorld for PrngSeed {
//     fn from_world(world: &mut World) -> Self {
//         let Some(config) = world.get_resource::<Config>() else {
//             panic!("config resource not found");
//         };
//
//         Self(config.simulation.prng_seed)
//     }
// }

/// Marker component for for entities with a PRNG source
#[derive(Component)]
pub struct Prng(rand_chacha::ChaCha8Rng);

/// Adds a PRNG entity to the world
fn setup_prng_entity(world: &mut World) {
    // let PrngSeed(seed) = world.get_resource::<PrngSeed>().unwrap();
    let seed = world
        .get_resource::<Config>()
        .expect("config exists in the ecs world")
        .simulation
        .prng_seed;
    let rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    world.spawn(Prng(rng));
    info!("spawned PRNG entity with seed: {}", seed);
    // world.remo
}

// fn setup_prng_entity(mut commands: Commands, seed: Res<PrngSeed>) {
//     commands
//         .spawn()
//         .insert(bevy_rand::resource::GlobalEntropy::<bevy_prng::WyRand>::default())
//         .insert(Prng);
// }

/// Deletes all PRNG entities
/// Entended to be called when the current simulation ends, to clean up before
/// the next simulation
fn delete_prng_entity(mut commands: Commands, query: Query<Entity, With<Prng>>) {
    for e in &query {
        commands.entity(e).despawn();
        info!("deleted PRNG entity");
    }
}
