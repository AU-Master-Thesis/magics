//! Visualizes the tracking information.
//! Includes the trackings paths, and the tracking factors

use bevy::prelude::*;
use gbp_config::Config;
use itertools::Itertools;

use crate::{
    factorgraph::prelude::FactorGraph,
    planner::robot::{Mission, Route, StateVector},
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt},
};

pub struct TrackingVisualizerPlugin;

impl Plugin for TrackingVisualizerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                visualise_tracking_factors.run_if(enabled),
                visualise_tracking_paths.run_if(enabled),
            ),
        );
    }
}

fn gradient(a: &Color, b: &Color) -> colorgrad::Gradient {
    // let [r, g, b] = a.
    // let a colorgrad::Color::from_linear_rgba(r, g, b, a)
    let a = colorgrad::Color::from_linear_rgba(a.r() as f64, a.g() as f64, a.b() as f64, 255.0);
    let b = colorgrad::Color::from_linear_rgba(b.r() as f64, b.g() as f64, b.b() as f64, 255.0);
    colorgrad::CustomGradient::new()
        .colors(&[a, b])
        .domain(&[0.0, 1.0])
        .mode(colorgrad::BlendMode::Hsv)
        .build()
        .unwrap()
}

fn visualise_tracking_factors(
    mut gizmos: Gizmos,
    factorgraphs: Query<(&FactorGraph, &Mission)>,
    // theme: Res<CatppuccinTheme>,
    config: Res<Config>,
) {
    let red = Color::RED;
    let green = Color::GREEN;
    let gradient = gradient(&green, &red);
    // let gradient = theme.gradient(theme.green(), theme.red());

    for (factorgraph, mission) in &factorgraphs {
        if mission.state.idle() {
            continue;
        }

        for (variable, tracking_factor) in factorgraph.variable_and_their_tracking_factors() {
            let last_measurement = tracking_factor.last_measurement();
            let estimated_position = variable.estimated_position_vec2();

            let color = gradient.at(last_measurement.value);
            let color = Color::rgba(
                color.r as f32,
                color.g as f32,
                color.b as f32,
                color.a as f32,
            );

            // line from estimated position to last measurement
            let start = estimated_position
                .extend(-config.visualisation.height.objects)
                .xzy();
            let end = last_measurement
                .pos
                .extend(-config.visualisation.height.objects)
                .xzy();

            gizmos.line(start, end, color);
        }
    }
}

fn visualise_tracking_paths(
    mut gizmos: Gizmos,
    factorgraphs: Query<(&FactorGraph, &ColorAssociation, &Route, &StateVector)>,
    theme: Res<CatppuccinTheme>,
) {
    for (factorgraph, color_association, route, initial_state) in &factorgraphs {
        let color = Color::from_catppuccin_colour_with_alpha(
            theme.get_display_colour(&color_association.name),
            0.5,
        );

        let points = route.waypoints().iter().map(|waypoint| waypoint.position());

        points.tuple_windows().for_each(|(start, end)| {
            let start = start.extend(0.0).xzy();
            let end = end.extend(0.0).xzy();

            gizmos.line(start, end, color);
        });
    }
}

/// **Bevy** run condition for drawing obstacle factors
#[inline]
fn enabled(config: Res<Config>) -> bool {
    config.visualisation.draw.tracking && config.gbp.factors_enabled.tracking
}
