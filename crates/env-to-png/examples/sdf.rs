use env_to_png::env_to_png::{env_to_image, env_to_sdf_image, Percentage, PixelsPerTile};
use gbpplanner_rs::config::Environment;

fn main() {
    let environment = Environment::from_file("./config/environment.yaml").expect("Config file not found");
    let resolution = PixelsPerTile::new(100);
    if let Ok(image) = env_to_image(&environment, resolution, Percentage::new(0.0)) {
        image.save("./output/img.png").expect("Failed to save normal image");

        // let sdf1 = image::imageops::blur(&image, 5.0);
        // sdf1.save("./output/sdf1.png").expect("Failed to save blurred
        // image");
    }

    if let Ok(image) = env_to_image(&environment, resolution, Percentage::new(0.0)) {
        let sdf = image::imageops::blur(&image, 5.0);
        sdf.save("./output/sdf2.png").expect("Failed to save blurred image");
    }

    if let Ok(sdf) = env_to_sdf_image(&environment, resolution, Percentage::new(0.1), Percentage::new(0.1)) {
        sdf.save("./output/sdf.png").expect("Failed to save SDF image");
    }
}
