pub fn add(left: usize, right: usize) -> usize {
    left + right
}

    const TILE_IDX_REMAP: [usize; 64] = [
        0x00, 0x01, 0x08, 0x09, 0x02, 0x03, 0x0A, 0x0B,
        0x10, 0x11, 0x18, 0x19, 0x12, 0x13, 0x1A, 0x1B,
        0x04, 0x05, 0x0C, 0x0D, 0x06, 0x07, 0x0E, 0x0F,
        0x14, 0x15, 0x1C, 0x1D, 0x16, 0x17, 0x1E, 0x1F,
        0x20, 0x21, 0x28, 0x29, 0x22, 0x23, 0x2A, 0x2B,
        0x30, 0x31, 0x38, 0x39, 0x32, 0x33, 0x3A, 0x3B,
        0x24, 0x25, 0x2C, 0x2D, 0x26, 0x27, 0x2E, 0x2F,
        0x34, 0x35, 0x3C, 0x3D, 0x36, 0x37, 0x3E, 0x3F,];


/// swizzle an 8x8 tile
fn swizzle_tile<T>(tile: &mut [T]) {
    assert_eq!(tile.len(), 8 * 8);

    struct Remap {
        from: usize,
        to: usize,
    }

    impl Remap {
        fn new(from: usize, to: usize) -> Self {
            Self { from, to }
        }
    }
    // fixed: [0, 0], []
    // 0x0, 0x1, 0x8, 0x9, 0x2, 0x3, 0xa, 0xb, 0x10, 0x11,
    /*let remaps = [
        Remap::new(0x2, 0x4),  // 2 <- 4, 4 <- 2
        Remap::new(0x2, 0x10), // 2 <- 10, 10 <- 4
        Remap::new(0x2, 0x8),  // 2 <- 8, 8 <- 10
        //
        Remap::new(0x3, 0x5),  // 3 <- 5, 5 <- 3
        Remap::new(0x3, 0x11), // 3 <- 11, 11 <- 5
        Remap::new(0x3, 0x9),  // 3 <- 9, 9 <- 11
        //
        //
        Remap::new(0xA, 0x6),  // a <- 6, 6 <- a
        Remap::new(0xA, 0x14), // a <- 14, 14 <- 6
        Remap::new(0xA, 0x18), // a <- 18, 18 <- 14
        //
        Remap::new(0xB, 0x7),  // b <- 7, 7 <- b
        Remap::new(0xB, 0x15), // b <- 7, 7 <- b
        Remap::new(0xB, 0x19), // b <- 7, 7 <- b
                               //
                               //
        Remap::new(0x12, 0xC), // 12 <- C, C <- 12
        Remap::new(0x13, 0xD),
        //
        Remap::new(0x1A, 0x1C), // 1A <- 1C, 1C <- 1A
        Remap::new(0x1A, 0x0E), // 1A <- E, E <- 1C
        Remap::new(0x0E, 0x1C), // E <- 1C, 1C <- E
        Remap::new(0x0E, 0x1A), // E <- 1A, 1A <- 1C
        Remap::new(0x0F, 0x1D), // F <- 1D, 1D <- F
        Remap::new(0x0F, 0x1B), // F <- 1B, 1B <- 1D
    ];*/
    let mut index_remap = TILE_IDX_REMAP;
/*
        Remap::new(0x2, 0x8),
        Remap::new(0x3, 0x9),

        Remap::new(0x4, 0x2),
        Remap::new(0x5, 0x3),

        Remap::new(0x6, 0xa),
        Remap::new(0x7, 0xb),

        Remap::new(0x8, 0x10),
        Remap::new(0x9, 0x11),

        Remap::new(0x10, 0x4),
        Remap::new(0x11, 0x5),
        Remap::new(0x12, 0x18),
        Remap::new(0x13, 0x19),
        Remap::new(0x14, 0x18),
        Remap::new(0x12, 0x18),
        Remap::new(0x12, 0x18),
        Remap::new(0x12, 0x18),
        Remap::new(0x12, 0x18),
        Remap::new(0x12, 0x18),*/
    /*for i in 0..tile.len() {
        let mut wanted_i = index_remap[i];
        let mut c_i = i;
        while wanted_i != c_i {
            wanted_i = index_remap[c_i];
            index_remap[c_i] = c_i;
            tile.swap(wanted_i, c_i);
            c_i = wanted_i;
        }
    }*/
    unsafe {
        let src = std::ptr::read(tile.as_ptr() as *const [T; 64]);
        for (i, remap) in tile.iter_mut().enumerate() {
            std::ptr::write(remap, std::ptr::read(src.as_ptr().add(TILE_IDX_REMAP[i])));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swizzle_basic() {

    const EXPECTED: [usize; 64] = [
        0x00, 0x01, 0x04, 0x05, 0x10, 0x11, 0x14, 0x15,
        0x02, 0x03, 0x06, 0x07, 0x12, 0x13, 0x16, 0x17,
        0x08, 0x09, 0x0C, 0x0D, 0x18, 0x19, 0x1C, 0x1D,
        0x0A, 0x0B, 0x0E, 0x0F, 0x1A, 0x1B, 0x1E, 0x1F,
        0x20, 0x21, 0x24, 0x25, 0x30, 0x31, 0x34, 0x35,
        0x22, 0x23, 0x26, 0x27, 0x32, 0x33, 0x36, 0x37,
        0x28, 0x29, 0x2C, 0x2D, 0x38, 0x39, 0x3C, 0x3D,
        0x2A, 0x2B, 0x2E, 0x2F, 0x3A, 0x3B, 0x3E, 0x3F,];
        let mut input = (0..64).collect::<Vec<usize>>();
        swizzle_tile(&mut input);
        assert_eq!(input, TILE_IDX_REMAP);

    }
}
