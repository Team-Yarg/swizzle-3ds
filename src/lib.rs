use std::{mem::MaybeUninit, ops::DerefMut};

use image::{GenericImage, GenericImageView, Pixel};

const TILE_IDX_REMAP: [usize; 64] = [
    0x00, 0x01, 0x08, 0x09, 0x02, 0x03, 0x0A, 0x0B, 0x10, 0x11, 0x18, 0x19, 0x12, 0x13, 0x1A, 0x1B,
    0x04, 0x05, 0x0C, 0x0D, 0x06, 0x07, 0x0E, 0x0F, 0x14, 0x15, 0x1C, 0x1D, 0x16, 0x17, 0x1E, 0x1F,
    0x20, 0x21, 0x28, 0x29, 0x22, 0x23, 0x2A, 0x2B, 0x30, 0x31, 0x38, 0x39, 0x32, 0x33, 0x3A, 0x3B,
    0x24, 0x25, 0x2C, 0x2D, 0x26, 0x27, 0x2E, 0x2F, 0x34, 0x35, 0x3C, 0x3D, 0x36, 0x37, 0x3E, 0x3F,
];

/// swizzle an 8x8 tile
pub fn swizzle_tile(tile: &mut impl GenericImage) {
    assert_eq!(tile.width(), 8);
    assert_eq!(tile.height(), 8);
    let pixels: [_; 64] = {
        let mut iter = tile.pixels();
        core::array::from_fn(|_| iter.next().unwrap().2)
    };

    for x in 0..tile.width() {
        for y in 0..tile.height() {
            let i = y * tile.width() + x;
            tile.put_pixel(x, y, pixels[TILE_IDX_REMAP[i as usize]]);
        }
    }
}

pub fn to_texture<I: GenericImageView>(img: &I) -> Vec<I::Pixel> {
    assert!(img.width().is_power_of_two());
    assert!(img.height().is_power_of_two());
    assert_eq!(img.width() % 8, 0);
    assert_eq!(img.height() % 8, 0);
    let mut out = Vec::with_capacity((img.width() * img.height()) as usize);

    for y in (0..img.height()).step_by(8) {
        for x in (0..img.width()).step_by(8) {
            let tile = img.view(x, y, 8, 8);
            out.extend(tile.pixels().map(|px| px.2));
        }
    }
    out
}

pub fn swizzle_in_place(img: &mut impl GenericImage) {
    assert!(img.width().is_power_of_two());
    assert!(img.height().is_power_of_two());
    assert_eq!(img.width() % 8, 0);
    assert_eq!(img.height() % 8, 0);

    for x in (0..img.width()).step_by(8) {
        for y in (0..img.height()).step_by(8) {
            let mut tile = img.sub_image(x, y, 8, 8);
            swizzle_tile(tile.deref_mut());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use image::{GenericImageView, ImageBuffer, Rgb, Rgba, RgbaImage};

    use super::*;

    fn mk_img(w: u32, h: u32) -> RgbaImage {
        ImageBuffer::from_fn(w, h, |x, y| image::Rgba([(x + y * w) as u8, 0, 0, 0]))
    }
    #[track_caller]
    fn check_match(img: &impl GenericImageView<Pixel = Rgba<u8>>, against: &[usize]) {
        assert_eq!(
            img.pixels()
                .map(|px| px.2 .0[0] as usize)
                .collect::<Vec<_>>(),
            against,
        );
    }

    #[test]
    fn swizzle_tile_basic() {
        let mut img = mk_img(8, 8);
        swizzle_tile(&mut img);
        check_match(&img, &TILE_IDX_REMAP);
    }

    #[test]
    fn swizzle_basic() {
        let mut img = mk_img(8, 8);
        swizzle_in_place(&mut img);
        check_match(&img, &TILE_IDX_REMAP);
    }

    #[test]
    fn swizzle_two_parts() {
        const LEFT_TILE: [usize; 64] = [
            0x00, 0x01, 0x10, 0x11, 0x02, 0x03, 0x12, 0x13, 0x20, 0x21, 0x30, 0x31, 0x22, 0x23,
            0x32, 0x33, 0x04, 0x05, 0x14, 0x15, 0x06, 0x07, 0x16, 0x17, 0x24, 0x25, 0x34, 0x35,
            0x26, 0x27, 0x36, 0x37, 0x40, 0x41, 0x50, 0x51, 0x42, 0x43, 0x52, 0x53, 0x60, 0x61,
            0x70, 0x71, 0x62, 0x63, 0x72, 0x73, 0x44, 0x45, 0x54, 0x55, 0x46, 0x47, 0x56, 0x57,
            0x64, 0x65, 0x74, 0x75, 0x66, 0x67, 0x76, 0x77,
        ];

        const RIGHT_TILE: [usize; 64] = [
            0x08, 0x09, 0x18, 0x19, 0x0A, 0x0B, 0x1A, 0x1B, 0x28, 0x29, 0x38, 0x39, 0x2A, 0x2B,
            0x3A, 0x3B, 0x0C, 0x0D, 0x1C, 0x1D, 0x0E, 0x0F, 0x1E, 0x1F, 0x2C, 0x2D, 0x3C, 0x3D,
            0x2E, 0x2F, 0x3E, 0x3F, 0x48, 0x49, 0x58, 0x59, 0x4A, 0x4B, 0x5A, 0x5B, 0x68, 0x69,
            0x78, 0x79, 0x6A, 0x6B, 0x7A, 0x7B, 0x4C, 0x4D, 0x5C, 0x5D, 0x4E, 0x4F, 0x5E, 0x5F,
            0x6C, 0x6D, 0x7C, 0x7D, 0x6E, 0x6F, 0x7E, 0x7F,
        ];
        let both_tiles = LEFT_TILE
            .iter()
            .chain(RIGHT_TILE.iter())
            .copied()
            .collect::<Vec<_>>();
        let mut img = mk_img(16, 8);
        let mut parts = mk_img(16, 8);
        swizzle_tile(parts.sub_image(0, 0, 8, 8).deref_mut());
        swizzle_tile(parts.sub_image(8, 0, 8, 8).deref_mut());
        let img_bytes = parts
            .view(0, 0, 8, 8)
            .pixels()
            .chain(parts.view(8, 0, 8, 8).pixels())
            .map(|px| px.2 .0[0] as usize)
            .collect::<Vec<_>>();
        assert_eq!(&img_bytes, &both_tiles);

        swizzle_in_place(&mut img);
        assert_eq!(parts, img);
        check_match(img.view(0, 0, 8, 8).deref(), &LEFT_TILE);
        check_match(parts.sub_image(0, 0, 8, 8).deref(), &LEFT_TILE);
        check_match(parts.sub_image(8, 0, 8, 8).deref(), &RIGHT_TILE);
        assert_eq!(
            &to_texture(&img)
                .into_iter()
                .map(|px| px.0[0] as usize)
                .collect::<Vec<_>>(),
            &both_tiles
        );
    }
}
