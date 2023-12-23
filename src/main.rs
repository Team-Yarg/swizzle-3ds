use std::{fs::File, io::Write, ops::BitAnd};

use image::{io::Reader as ImageReader, DynamicImage, EncodableLayout};
use swizzle_3ds::swizzle_in_place;

enum CompressionType {
    None,
}

fn compression_header(ty: CompressionType, data_sz: u32) -> Vec<u8> {
    let magic = match ty {
        CompressionType::None => 0x0,
    };
    let mut buf = Vec::new();
    buf.push(magic);
    let sz_bytes = data_sz.to_le_bytes();

    buf.extend_from_slice(&sz_bytes[0..3]);
    if sz_bytes[3] != 0 {
        // special case, it requires 4 bytes, set the marker that we are using it
        buf[0] |= 0x80;
        buf.push(sz_bytes[3]);
        // reserved bytes
        buf.resize(buf.len() + 3, 0);
    }

    buf
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let input = &args[1];
    let output = &args[2];

    let mut img = ImageReader::open(input).unwrap().decode().unwrap();
    swizzle_in_place(&mut img);

    let mut outfile = File::create(output).unwrap();
    let mut bytes = img
        .to_rgba8()
        .pixels()
        .flat_map(|px| px.0.iter().copied().rev())
        .collect::<Vec<_>>();
    let mut output = compression_header(
        CompressionType::None,
        bytes.len().try_into().expect("couldn't fit bytes into u32"),
    );
    output.append(&mut bytes);

    // pad to 4 bytes
    if output.len() % 4 != 0 {
        output.resize(output.len() + (output.len() % 4), 0);
    }

    outfile.write_all(&output).unwrap();
}
