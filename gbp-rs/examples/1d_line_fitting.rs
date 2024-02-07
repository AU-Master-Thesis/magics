use std::default;

use charming::{
    component::{Axis, Legend},
    element::ItemStyle,
    series::{Pie, PieRoseType, Scatter},
    Chart, ImageRenderer,
};
use color_eyre::eyre::Ok;
use gbp_rs::{
    factorgraph::{factorgraph::GbpSettings, UnitInterval},
    prelude::*,
};
use nalgebra::DVector;

#[derive(Debug)]
struct Factor<L: Loss> {
    pub measurement: f64,
    pub measurement_model: MeasurementModel<L>,
}

impl<L: Loss> Factor<L> {
    pub fn new(measurement: f64, measurement_model: MeasurementModel<L>) -> Self {
        Self {
            measurement,
            measurement_model,
        }
    }
}

impl<L: Loss> gbp_rs::factorgraph::factor::Factor<L> for Factor<L> {
    fn compute_messages(&mut self, damping: f64) -> Vec<Message> {
        todo!()
    }

    fn energy(&self, evaluation_point: Option<DVector<f64>>) -> f64 {
        let residual = self.residual(evaluation_point);
        0.5 * residual.transpose()
            * self.measurement_model.loss.effective_covariance(Some(&residual))
            * residual
    }

    fn residual(&self, evaluation_point: Option<DVector<f64>>) -> DVector<f64> {
        let evaluation_point = evaluation_point.ok_or_else(self.adj_means);
        self.measurement_model.measurement(evaluation_point) - self.measurement
    }

    fn adj_means(&self) -> DVector<f64> {
        
    }

    fn compute(&self) -> f64 {
        todo!()
    }

    fn robustify_loss(&self) {
        todo!()
    }

    fn measurement_model(&self) -> MeasurementModel<L> {
        todo!()
    }

    fn linerisation_point(&self) -> DVector<f64> {
        todo!()
    }

    fn get_gaussian(&self) -> &gaussian::MultivariateNormal {
        todo!()
    }
}

/// Reimplementation of the 1D line fitting example from https://gaussianbp.github.io/
/// jupyter notebook: https://colab.research.google.com/drive/1-nrE95X4UC9FBLR0-cTnsIP_XhA_PZKW?usp=sharing#scrollTo=kiAOHWV4uMGY
fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    // let mut fg = FactorGraph::new(None);

    // x: [2.3184, 6.0120, 4.0925, 6.1231, 5.3754, 1.3485, 1.7960, 8.4883, 1.2685, 9.3003, 1.9147, 6.1845, 8.0064, 1.0738, 7.4615]
    // y: [0.3872,  0.1248, -0.6335,  0.1158, -1.0122,  0.8969,  0.8861,  0.9530, 0.9156,  0.2199,  0.7894, -0.3240,  1.2007,  0.9846,  0.6048]

    let x = vec![
        2.3184, 6.0120, 4.0925, 6.1231, 5.3754, 1.3485, 1.7960, 8.4883, 1.2685, 9.3003, 1.9147,
        6.1845, 8.0064, 1.0738, 7.4615,
    ];
    let y = vec![
        0.3872, 0.1248, -0.6335, 0.1158, -1.0122, 0.8969, 0.8861, 0.9530, 0.9156, 0.2199, 0.7894,
        -0.3240, 1.2007, 0.9846, 0.6048,
    ];

    let measurements: Vec<Vec<f64>> = x
        .into_iter()
        .zip(y.into_iter())
        .map(|(x, y)| vec![x, y])
        .collect();

    let chart = Chart::new()
        .x_axis(Axis::new())
        .y_axis(Axis::new())
        .series(Scatter::new().symbol_size(20).data(measurements));

    let number_of_variables = 20;
    let gbp_settings = GbpSettings::builder()
        .damping(0.1)
        .beta(0.01)
        .number_of_undamped_iterations(1)
        .minimum_linear_iteration(1)
        .dropout(UnitInterval::new(0.0).unwrap())
        .build();

    // let prior_covariance =
    // let data_covariance =
    // let smoothness_covariance =
    // let data_std_dev = // sqrt of data_covariance

    let x_range = 10;
    let xs = (0..number_of_variables)
        .map(|i| (i as f64) * x_range / (number_of_variables - 1) as f64)
        .collect::<Vec<f64>>();

    let mut renderer = ImageRenderer::new(1000, 800);
    renderer.save(&chart, "/tmp/nightingale.svg").unwrap();
    Ok(())
}
