use charming::{
    component::Legend,
    element::ItemStyle,
    series::{Pie, PieRoseType},
    Chart, ImageRenderer,
};
/// Reimplementation of the 1D line fitting example from https://gaussianbp.github.io/
/// jupyter notebook: https://colab.research.google.com/drive/1-nrE95X4UC9FBLR0-cTnsIP_XhA_PZKW?usp=sharing#scrollTo=kiAOHWV4uMGY
use gbp_rs::prelude::*;
fn main() {
    // let mut fg = FactorGraph::new(None);

    let chart = Chart::new().legend(Legend::new().top("bottom")).series(
        Pie::new()
            .name("Nightingale Chart")
            .rose_type(PieRoseType::Radius)
            .radius(vec!["50", "250"])
            .center(vec!["50%", "50%"])
            .item_style(ItemStyle::new().border_radius(8))
            .data(vec![
                (40.0, "rose 1"),
                (38.0, "rose 2"),
                (32.0, "rose 3"),
                (30.0, "rose 4"),
                (28.0, "rose 5"),
                (26.0, "rose 6"),
                (22.0, "rose 7"),
                (18.0, "rose 8"),
            ]),
    );

    let mut renderer = ImageRenderer::new(1000, 800);
    renderer.save(&chart, "/tmp/nightingale.svg");
}
