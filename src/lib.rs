use std::mem::MaybeUninit;

const TILE_IDX_REMAP: [usize; 64] = [
    0x00, 0x01, 0x08, 0x09, 0x02, 0x03, 0x0A, 0x0B, 0x10, 0x11, 0x18, 0x19, 0x12, 0x13, 0x1A, 0x1B,
    0x04, 0x05, 0x0C, 0x0D, 0x06, 0x07, 0x0E, 0x0F, 0x14, 0x15, 0x1C, 0x1D, 0x16, 0x17, 0x1E, 0x1F,
    0x20, 0x21, 0x28, 0x29, 0x22, 0x23, 0x2A, 0x2B, 0x30, 0x31, 0x38, 0x39, 0x32, 0x33, 0x3A, 0x3B,
    0x24, 0x25, 0x2C, 0x2D, 0x26, 0x27, 0x2E, 0x2F, 0x34, 0x35, 0x3C, 0x3D, 0x36, 0x37, 0x3E, 0x3F,
];

/// swizzle an 8x8 tile
pub fn swizzle_tile<T>(tile: &mut [T]) {
    assert_eq!(tile.len(), 64);

    unsafe {
        let src = core::ptr::read(tile.as_ptr() as *const [T; 64]);
        for (i, dst) in tile.iter_mut().enumerate() {
            let remap = core::ptr::read(src.as_ptr().add(TILE_IDX_REMAP[i]));
            core::ptr::write(dst, remap);
        }
    }
}

pub fn swizzle_in_place<T>(img: &mut &[T], width: usize) {
    assert!(img.len().is_power_of_two(), "");
    assert!(width.is_power_of_two());
    let height = img.len() / width;
    assert_eq!(width, height);

    assert_eq!(width % 8, 0);
    assert_eq!(height % 8, 0);
    let mk_idx = |x, y| y * width + x;

    let w_tiles = width / 8;
    let h_tiles = height / 8;

    for x in 0..w_tiles {
        for y in 0..h_tiles {
            unsafe {
                let img_p = img.as_ptr();
                let mut tile: [T; 64] = MaybeUninit::uninit().assume_init();
                let offset_x = x * 8;
                let offset_y = y * 8;
                for ypx in 0..8 {
                    core::ptr::copy_nonoverlapping(
                        img[mk_idx(offset_x, offset_y + ypx)],
                        tile.as_mut_ptr().add(mk_idx()),
                        count,
                    )
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swizzle_basic() {
        let mut input = (0..64).collect::<Vec<usize>>();
        swizzle_tile(&mut input);
        assert_eq!(input, TILE_IDX_REMAP);
    }
}
