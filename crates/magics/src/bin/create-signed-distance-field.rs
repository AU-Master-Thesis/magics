use std::path::PathBuf;

use clap::{arg, value_parser};
use image::{ImageBuffer, Rgb};

fn main() -> anyhow::Result<()> {
    let matches = clap::command!()
        .arg(
            arg!(-i --input <FILE> "input grayscale image")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-o --output <FILE> "output distance field image")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    // println!("{:#?}", matches);

    let input = matches.get_one::<PathBuf>("input").unwrap();
    let output = matches.get_one::<PathBuf>("output").unwrap();
    let input = image::io::Reader::open(input)?.decode()?;

    let sdf = signed_distance_field(input.into());

    sdf.save(output)?;

    Ok(())
}

fn signed_distance_field(image: ImageBuffer<Rgb<u8>, Vec<u8>>) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let sigma = 5.0;
    let blurred = image::imageops::blur(&image, sigma);
    blurred
}
