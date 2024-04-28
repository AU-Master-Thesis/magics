use env_to_png::env_to_png::{env_to_image, env_to_sdf_image, Percentage, PixelsPerTile};
use gbpplanner_rs::config::Environment;

fn main() {
    let environment = Environment::from_file("./config/environment.yaml").unwrap();
    let resolution = PixelsPerTile::new(100);
    if let Ok(image) = env_to_image(&environment, resolution, Percentage::new(0.0)) {
        image.save("./output/img.png").unwrap();
    }

    if let Ok(sdf) = env_to_sdf_image(
        &environment,
        resolution,
        Percentage::new(0.1),
        Percentage::new(5.0),
    ) {
        sdf.save("./output/sdf.png").unwrap();
    }
}
