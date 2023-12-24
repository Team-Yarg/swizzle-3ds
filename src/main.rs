use std::{fs::File, io::BufWriter};

use image::io::Reader as ImageReader;
use swizzle_3ds::{encode, swizzle_image};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let input = &args[1];
    let output = &args[2];

    let img = ImageReader::open(input).unwrap().decode().unwrap();

    let outfile = File::create(output).unwrap();
    let tex = swizzle_image(&img);
    let enc = encode::Tex3dsEncoder::new(BufWriter::new(outfile));
    tex.write_with_encoder(enc).unwrap();
}
