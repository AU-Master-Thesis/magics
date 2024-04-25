use std::path::PathBuf;

use clap::{arg, value_parser};
use image::{ImageBuffer, Rgb};

fn main() -> anyhow::Result<()> {
    let matches = clap::command!()
        .arg(
            arg!(-o --output <FILE> "output image")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let output = matches.get_one::<PathBuf>("output").unwrap();

    let size = 1000;

    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(size, size);
    // make every pixel white
    for pixel in image.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    image.save(output)?;

    Ok(())
}
