use env_to_png::{env_to_image, env_to_sdf_image, Percentage, PixelsPerTile};
use gbp_environment::Environment;

fn main() {
    let environment =
        Environment::from_file("./config/environment.yaml").expect("Config file not found");
    let resolution = PixelsPerTile::new(1000);
    if let Ok(image) = env_to_image(&environment, resolution, Percentage::new(0.0)) {
        image
            .save("./output/img.png")
            .expect("Failed to save normal image");
    }
    if let Ok(image) = env_to_image(&environment, resolution, Percentage::new(0.01)) {
        image
            .save("./output/img_exp.png")
            .expect("Failed to save normal image");
    }

    if let Ok(sdf) = env_to_sdf_image(
        &environment,
        resolution,
        Percentage::new(0.01),
        Percentage::new(0.01),
    ) {
        sdf.save("./output/sdf.png")
            .expect("Failed to save SDF image");
    }
}
