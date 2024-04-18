use bevy::{
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
    time::common_conditions::on_timer,
};
use units::sample_rate::SampleRate;

use crate::{
    factorgraph::prelude::FactorGraph,
    planner::RobotState,
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

#[derive(Default)]
pub struct RobotDiagnosticsPlugin {
    pub config: RobotDiagnosticsConfig,
}

pub struct SampleRates {
    pub robots: Option<SampleRate>,
    pub variables_and_factors: Option<SampleRate>,
    pub messages_sent: Option<SampleRate>,
}

pub struct RobotDiagnosticsConfig {
    pub sample_rates: SampleRates,
}

impl Default for RobotDiagnosticsConfig {
    fn default() -> Self {
        Self {
            sample_rates: SampleRates {
                robots: None,
                variables_and_factors: Some(SampleRate::from_hz(2.try_into().expect("2 > 0"))),
                messages_sent: Some(SampleRate::from_hz(2.try_into().expect("2 > 0"))),
            },
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
            .register_diagnostic(Diagnostic::new(Self::EXTERNAL_MESSAGES_SENT_COUNT))
            .register_diagnostic(Diagnostic::new(Self::MESSAGES_SENT_COUNT));

        add_diagnostic_system!(app, self.config.sample_rates.robots, Self::count_robots);
        add_diagnostic_system!(
            app,
            self.config.sample_rates.variables_and_factors,
            Self::count_variables_and_factors
        );
        add_diagnostic_system!(
            app,
            self.config.sample_rates.messages_sent,
            Self::count_messages_sent
        );

        app.add_systems(
            Update,
            Self::flush_diagnostics
                .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
        );
    }
}

impl RobotDiagnosticsPlugin {
    pub const EXTERNAL_MESSAGES_SENT_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("external_messages_sent_count");
    pub const FACTOR_COUNT: DiagnosticPath = DiagnosticPath::const_new("factor_count");
    pub const MESSAGES_SENT_COUNT: DiagnosticPath =
        DiagnosticPath::const_new("messages_sent_count");
    pub const ROBOT_COUNT: DiagnosticPath = DiagnosticPath::const_new("robot_count");
    pub const VARIABLE_COUNT: DiagnosticPath = DiagnosticPath::const_new("variable_count");

    #[allow(clippy::cast_precision_loss)]
    fn count_robots(mut diagnostics: Diagnostics, robots: Query<(), With<RobotState>>) {
        diagnostics.add_measurement(&Self::ROBOT_COUNT, || robots.iter().count() as f64);
    }

    #[allow(clippy::cast_precision_loss)]
    fn count_variables_and_factors(
        mut diagnostics: Diagnostics,
        factorgraphs: Query<&FactorGraph, With<RobotState>>,
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

    #[allow(clippy::cast_precision_loss)]
    fn count_messages_sent(
        mut diagnostics: Diagnostics,
        mut factorgraphs: Query<&mut FactorGraph, With<RobotState>>,
        mut messages_sent_in_total: Local<usize>,
    ) {
        diagnostics.add_measurement(&Self::MESSAGES_SENT_COUNT, || {
            let messages_sent = factorgraphs
                .iter_mut()
                .map(|mut factorgraph| factorgraph.messages_sent())
                .sum::<usize>();

            *messages_sent_in_total += messages_sent;
            *messages_sent_in_total as f64
        });
    }

    fn count_external_messages_sent(
        mut diagnostics: Diagnostics,
        mut messages_sent_in_total: Local<usize>,
        mut factorgraphs: Query<&mut FactorGraph, With<RobotState>>,
    ) {
        diagnostics.add_measurement(&Self::EXTERNAL_MESSAGES_SENT_COUNT, || {
            *messages_sent_in_total += *messages_sent_in_total;
            *messages_sent_in_total as f64
        })
    }

    /// **Bevy** system to clear the history of every diagnostic source of this
    /// plugin.
    fn flush_diagnostics(mut store: ResMut<bevy::diagnostic::DiagnosticsStore>) {
        for path in &[
            Self::FACTOR_COUNT,
            Self::ROBOT_COUNT,
            Self::VARIABLE_COUNT,
            Self::MESSAGES_SENT_COUNT,
            Self::EXTERNAL_MESSAGES_SENT_COUNT,
        ] {
            if let Some(diagnostic) = store.get_mut(path) {
                diagnostic.clear_history();
                info!("clearing history of diagnostic source: {:?}", path);
            }
        }
    }
}
