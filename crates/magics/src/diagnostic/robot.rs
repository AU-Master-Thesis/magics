use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
    time::common_conditions::on_timer,
};
use units::sample_rate::SampleRate;

use crate::{
    factorgraph::prelude::FactorGraph,
    planner::{collisions::resources::RobotRobotCollisions, RobotConnections},
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

#[derive(Default)]
pub struct RobotDiagnosticsPlugin {
    pub sample_rates: SampleRates,
}

pub struct SampleRates {
    pub robots: Option<SampleRate>,
    pub robot_collisions: Option<SampleRate>,
    pub variables_and_factors: Option<SampleRate>,
    // pub messages_sent: Option<SampleRate>,
}

impl Default for SampleRates {
    fn default() -> Self {
        Self {
            robots: None,
            robot_collisions: Some(SampleRate::from_hz(5.try_into().expect("1 > 0"))),
            variables_and_factors: Some(SampleRate::from_hz(2.try_into().expect("2 > 0"))),
            // messages_sent: Some(SampleRate::from_hz(2.try_into().expect("2 > 0"))),
        }
    }
}

/// Helper macro to reduce boilerplate for registering diagnostics system
macro_rules! add_diagnostic_system {
    ($app:ident, $samplerate_cfg:expr, $method:path) => {
        if let Some(duration) = $samplerate_cfg.map(SampleRate::as_duration) {
            info!(
                "sampling diagnostic {} every {:?}",
                stringify!($method),
                duration
            );
            $app.add_systems(PostUpdate, $method.run_if(on_timer(duration)));
        } else {
            $app.add_systems(PostUpdate, $method);
        }
    };
}

impl Plugin for RobotDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.register_diagnostic(Diagnostic::new(Self::ROBOT_COUNT))
            .register_diagnostic(Diagnostic::new(Self::VARIABLE_COUNT))
            .register_diagnostic(Diagnostic::new(Self::FACTOR_COUNT))
            // .register_diagnostic(Diagnostic::new(Self::EXTERNAL_MESSAGES_SENT_COUNT))
            .register_diagnostic(Diagnostic::new(Self::MESSAGES_RECEIVED_INTERNAL_COUNT))
            .register_diagnostic(Diagnostic::new(Self::MESSAGES_RECEIVED_EXTERNAL_COUNT))
            .register_diagnostic(Diagnostic::new(Self::MESSAGES_SENT_EXTERNAL_COUNT))
            .register_diagnostic(Diagnostic::new(Self::MESSAGES_SENT_INTERNAL_COUNT))
            .register_diagnostic(Diagnostic::new(Self::ROBOT_COLLISION_COUNT));

        add_diagnostic_system!(app, self.sample_rates.robots, Self::robots);
        add_diagnostic_system!(
            app,
            self.sample_rates.variables_and_factors,
            Self::variables_and_factors
        );
        // add_diagnostic_system!(app, self.sample_rates.messages_sent,
        // Self::messages_sent);

        add_diagnostic_system!(
            app,
            self.sample_rates.robot_collisions,
            Self::count_robot_collisions
        );

        app.add_systems(
            Update,
            Self::flush_diagnostics
                .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
        );
    }
}

impl RobotDiagnosticsPlugin {
    pub const ENVIRONMENT_COLLISION_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("environment_collision_count");
    pub const EXTERNAL_MESSAGES_SENT_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("external_messages_sent_count");
    pub const FACTOR_COUNT: DiagnosticPath = DiagnosticPath::const_new("factor_count");
    pub const MESSAGES_RECEIVED_EXTERNAL_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("messages_received_internal_count");
    pub const MESSAGES_RECEIVED_INTERNAL_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("messages_received_external_count");
    // pub const MESSAGES_SENT_COUNT: DiagnosticPath =
    // DiagnosticPath::const_new("messages_sent_count");
    pub const MESSAGES_SENT_EXTERNAL_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("messages_sent_external_count");
    pub const MESSAGES_SENT_INTERNAL_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("messages_sent_internal_count");
    pub const ROBOT_COLLISION_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("robot_collision_count");
    pub const ROBOT_COUNT: DiagnosticPath = DiagnosticPath::const_new("robot_count");
    pub const VARIABLE_COUNT: DiagnosticPath = DiagnosticPath::const_new("variable_count");

    #[allow(clippy::cast_precision_loss)]
    fn robots(mut diagnostics: Diagnostics, robots: Query<(), With<RobotConnections>>) {
        diagnostics.add_measurement(&Self::ROBOT_COUNT, || robots.iter().count() as f64);
    }

    #[allow(clippy::cast_precision_loss)]
    fn variables_and_factors(
        mut diagnostics: Diagnostics,
        factorgraphs: Query<&FactorGraph, With<RobotConnections>>,
    ) {
        diagnostics.add_measurement(&Self::VARIABLE_COUNT, || {
            factorgraphs
                .iter()
                .map(|factorgraph| factorgraph.node_count().variables)
                .sum::<usize>() as f64
            // query.par_iter().for_each(|factorgraph| )
        });
        diagnostics.add_measurement(&Self::FACTOR_COUNT, || {
            factorgraphs
                .iter()
                .map(|factorgraph| factorgraph.node_count().factors)
                .sum::<usize>() as f64
        });
    }

    // #[allow(clippy::cast_precision_loss)]
    // fn messages_sent(
    //     mut diagnostics: Diagnostics,
    //     mut factorgraphs: Query<&mut FactorGraph>,
    //     mut messages_sent_in_total: Local<usize>,
    // ) {
    //     diagnostics.add_measurement(&Self::MESSAGES_SENT_COUNT, || {
    //         let messages_sent = factorgraphs
    //             .iter_mut()
    //             .map(|mut factorgraph| factorgraph.messages_sent())
    //             .sum::<usize>();
    //
    //         *messages_sent_in_total += messages_sent;
    //         *messages_sent_in_total as f64
    //     });
    // }

    // #[allow(clippy::cast_precision_loss)]
    // fn count_external_messages_sent(
    //     mut diagnostics: Diagnostics,
    //     mut messages_sent_in_total: Local<usize>,
    //     // FIXME: remove mut requirement
    //     mut factorgraphs: Query<&mut FactorGraph, With<RobotState>>,
    // ) {
    //     diagnostics.add_measurement(&Self::EXTERNAL_MESSAGES_SENT_COUNT, || {
    //         *messages_sent_in_total += *messages_sent_in_total;
    //         *messages_sent_in_total as f64
    //     });
    // }

    #[allow(clippy::cast_precision_loss)]
    fn count_robot_collisions(
        mut diagnostics: Diagnostics,
        robot_collisions: Res<RobotRobotCollisions>,
    ) {
        diagnostics.add_measurement(&Self::ROBOT_COLLISION_COUNT, || {
            robot_collisions.num_collisions() as f64
        });
    }

    // #[allow(clippy::cast_precision_loss)]
    // fn robot_collisions(
    //     mut diagnostics: Diagnostics,
    //     robots: Query<(&Transform, &Ball), With<RobotState>>,
    //     mut aabbs: Local<Vec<parry2d::bounding_volume::Aabb>>,
    //     // TODO: move into a component/resource so it can be queried by other
    // systems     mut robot_collisions: Local<RobotCollisions>,
    // ) {
    //     diagnostics.add_measurement(&Self::ROBOT_COLLISION_COUNT, || {
    //         // reuse the same vector for performance
    //         aabbs.clear();
    //
    //         let iter = robots.iter().map(|(tf, ball)| {
    //             let position =
    // parry2d::na::Isometry2::translation(tf.translation.x, tf.translation.z); //
    // bevy uses xzy coordinates             ball.aabb(&position)
    //         });
    //
    //         aabbs.extend(iter);
    //
    //         if aabbs.len() < 2 {
    //             // No collisions if there is less than two robots
    //             return 0.0;
    //         }
    //
    //         for (r, c) in
    // seq::upper_triangular_exclude_diagonal(aabbs.len().try_into().expect("more
    // than one robot"))             .expect("more than one robot")
    //         {
    //             let is_colliding = aabbs[r].intersects(&aabbs[c]);
    //             robot_collisions.update(r, c, is_colliding);
    //         }
    //
    //         let collisions = robot_collisions.collisions();
    //
    //         // let collisions =
    //         //
    // seq::upper_triangular_exclude_diagonal(aabbs.len().try_into().expect("
    //         // more than one robot"))         .expect("more that one robot")
    //         //         .filter(|(r, c)| aabbs[*r].intersects(&aabbs[*c]))
    //         //         .count();
    //
    //         collisions as f64
    //     });
    // }

    /// **Bevy** system to clear the history of every diagnostic source of this
    /// plugin.
    fn flush_diagnostics(mut store: ResMut<bevy::diagnostic::DiagnosticsStore>) {
        for path in &[
            Self::FACTOR_COUNT,
            Self::ROBOT_COUNT,
            Self::VARIABLE_COUNT,
            Self::MESSAGES_SENT_EXTERNAL_COUNT,
            Self::MESSAGES_SENT_INTERNAL_COUNT,
            Self::MESSAGES_RECEIVED_EXTERNAL_COUNT,
            Self::MESSAGES_RECEIVED_INTERNAL_COUNT,
            Self::EXTERNAL_MESSAGES_SENT_COUNT,
            Self::ROBOT_COLLISION_COUNT,
            Self::ENVIRONMENT_COLLISION_COUNT,
        ] {
            if let Some(diagnostic) = store.get_mut(path) {
                diagnostic.clear_history();
                info!("clearing history of diagnostic source: {:?}", path);
            }
        }
    }
}
