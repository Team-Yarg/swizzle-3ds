use std::{mem::MaybeUninit, ops::DerefMut};

use image::GenericImage;

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

pub fn swizzle_in_place(img: &mut impl GenericImage) {
    assert_eq!(img.width(), img.height());
    assert!(img.width().is_power_of_two());
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
    use image::ImageBuffer;

    use super::*;

    #[test]
    fn swizzle_tile_basic() {
        let mut input = (0..64).collect::<Vec<usize>>();
        let mut img = ImageBuffer::from_fn(8, 8, |x, y| image::Rgba([x + y * 8, 0, 0, 0]));
        swizzle_tile(&mut img);
        assert_eq!(
            img.pixels().map(|px| px.0[0] as usize).collect::<Vec<_>>(),
            TILE_IDX_REMAP
        );
    }

    #[test]
    fn swizzle_basic() {
        let mut img = ImageBuffer::from_fn(8, 8, |x, y| image::Rgba([x + y * 8, 0, 0, 0]));
        swizzle_in_place(&mut img);
        assert_eq!(
            img.pixels().map(|px| px.0[0] as usize).collect::<Vec<_>>(),
            TILE_IDX_REMAP
        );
    }
}
