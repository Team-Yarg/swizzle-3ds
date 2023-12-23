use std::mem::MaybeUninit;

const TILE_IDX_REMAP: [usize; 64] = [
    0x00, 0x01, 0x08, 0x09, 0x02, 0x03, 0x0A, 0x0B, 0x10, 0x11, 0x18, 0x19, 0x12, 0x13, 0x1A, 0x1B,
    0x04, 0x05, 0x0C, 0x0D, 0x06, 0x07, 0x0E, 0x0F, 0x14, 0x15, 0x1C, 0x1D, 0x16, 0x17, 0x1E, 0x1F,
    0x20, 0x21, 0x28, 0x29, 0x22, 0x23, 0x2A, 0x2B, 0x30, 0x31, 0x38, 0x39, 0x32, 0x33, 0x3A, 0x3B,
    0x24, 0x25, 0x2C, 0x2D, 0x26, 0x27, 0x2E, 0x2F, 0x34, 0x35, 0x3C, 0x3D, 0x36, 0x37, 0x3E, 0x3F,
];

struct Image<'d, Px> {
    data: &'d mut [Px],
    width: usize,
}

impl<'d, Px> Image<'d, Px> {
    fn get_tile(&self, x: usize, y: usize) -> Tile<Px> {
        let start = self.mk_idx(x, y);
        let finish = self.mk_idx(x + 7, y + 7);
        assert!(start < self.len());
        assert!(finish < self.len());

        let mut buf: [MaybeUninit<Px>; 64] = unsafe { MaybeUninit::uninit().assume_init() };

        for row in 0..8 {
            unsafe {
                core::ptr::copy_nonoverlapping(self.get(x, y + row), buf[row * 8].as_mut_ptr(), 8);
            }
        }
        Tile {
            data: unsafe { core::mem::transmute_copy::<_, [Px; 64]>(&buf) },
            x_offset: x,
            y_offset: y,
        }
    }
    fn apply_tile(&mut self, tile: Tile<Px>) {
        for row in 0..8 {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &tile.data[row * 8],
                    self.get_mut(tile.x_offset, tile.y_offset + row),
                    8,
                )
            }
        }
    }
    fn get(&self, x: usize, y: usize) -> &Px {
        &self.data[self.mk_idx(x, y)]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut Px {
        &mut self.data[self.mk_idx(x, y)]
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn mk_idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }
    pub fn new_from_raw(data: &'d mut [Px], width: usize) -> Self {
        assert!(data.len().is_power_of_two(), "");
        assert!(width.is_power_of_two());
        let height = data.len() / width;
        assert_eq!(width, height);

        assert_eq!(width % 8, 0);
        assert_eq!(height % 8, 0);
        Self { data, width }
    }
    pub fn swizzle_in_place(&mut self) {
        for x in (0..self.width).step_by(8) {
            for y in (0..self.height()).step_by(8) {
                let mut tile = self.get_tile(x, y);
                swizzle_tile(&mut tile.data);
                self.apply_tile(tile);
            }
        }
    }
    pub fn height(&self) -> usize {
        self.len() / self.width
    }
}

struct Tile<T> {
    data: [T; 64],
    x_offset: usize,
    y_offset: usize,
}

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

pub fn swizzle_in_place<T>(img: &mut [T], width: usize) {
    assert!(img.len().is_power_of_two(), "");
    assert!(width.is_power_of_two());
    let height = img.len() / width;
    assert_eq!(width, height);

    assert_eq!(width % 8, 0);
    assert_eq!(height % 8, 0);

    let mut info = Image { data: img, width };

    for x in (0..width).step_by(8) {
        for y in (0..height).step_by(8) {
            let mut tile = info.get_tile(x, y);
            swizzle_tile(&mut tile.data);
            info.apply_tile(tile);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swizzle_tile_basic() {
        let mut input = (0..64).collect::<Vec<usize>>();
        swizzle_tile(&mut input);
        assert_eq!(input, TILE_IDX_REMAP);
    }

    #[test]
    fn swizzle_basic() {
        let mut input = (0..64).collect::<Vec<usize>>();
        swizzle_in_place(&mut input, 8);
        assert_eq!(input, TILE_IDX_REMAP);
    }
}
