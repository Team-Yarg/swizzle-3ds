use std::{fs::File, io::Write};

use image::{io::Reader as ImageReader, DynamicImage, EncodableLayout};
use swizzle_3ds::swizzle_in_place;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let input = &args[1];
    let output = &args[2];

    let mut img = ImageReader::open(input).unwrap().decode().unwrap();
    swizzle_in_place(&mut img);

    let mut outfile = File::create(output).unwrap();
    let bytes = img
        .to_rgba8()
        .pixels()
        .flat_map(|px| px.0.iter().copied().rev())
        .collect::<Vec<_>>();
    outfile.write_all(&bytes).unwrap();
}
