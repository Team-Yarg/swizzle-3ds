use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel};

const TILE_IDX_REMAP: [usize; 64] = [
    0x00, 0x01, 0x08, 0x09, 0x02, 0x03, 0x0A, 0x0B, 0x10, 0x11, 0x18, 0x19, 0x12, 0x13, 0x1A, 0x1B,
    0x04, 0x05, 0x0C, 0x0D, 0x06, 0x07, 0x0E, 0x0F, 0x14, 0x15, 0x1C, 0x1D, 0x16, 0x17, 0x1E, 0x1F,
    0x20, 0x21, 0x28, 0x29, 0x22, 0x23, 0x2A, 0x2B, 0x30, 0x31, 0x38, 0x39, 0x32, 0x33, 0x3A, 0x3B,
    0x24, 0x25, 0x2C, 0x2D, 0x26, 0x27, 0x2E, 0x2F, 0x34, 0x35, 0x3C, 0x3D, 0x36, 0x37, 0x3E, 0x3F,
];

#[track_caller]
fn check_image_requirements(img: &impl GenericImageView) {
    assert!(img.width().is_power_of_two());
    assert!(img.height().is_power_of_two());
    assert_eq!(img.width() % 8, 0);
    assert_eq!(img.height() % 8, 0);
}

struct SwizzleIter<'a, P> {
    tile: &'a [P],
    stride: usize,
    i: usize,
}

impl<'a, P> SwizzleIter<'a, P> {
    fn new(tile: &'a [P], stride: usize) -> Self {
        Self { tile, stride, i: 0 }
    }
}

impl<'a, P: Copy> Iterator for SwizzleIter<'a, P> {
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.i % self.stride;
        let y = self.i / self.stride;
        if self.i >= self.tile.len() {
            None
        } else {
            let px = self.tile[TILE_IDX_REMAP[self.i]];
            if x >= self.stride {
                self.i = self.stride * (y + 1);
            } else {
                self.i += 1;
            }
            Some(px)
        }
    }
}

pub fn swizzle_image<I: GenericImageView>(img: &I) -> DynamicImage
where
    image::DynamicImage: From<
        ImageBuffer<
            <I as GenericImageView>::Pixel,
            Vec<<<I as GenericImageView>::Pixel as Pixel>::Subpixel>,
        >,
    >,
{
    check_image_requirements(img);
    let mut out = Vec::with_capacity(
        (img.width() * img.height() * <I::Pixel as Pixel>::CHANNEL_COUNT as u32) as usize,
    );

    for y in (0..img.height()).step_by(8) {
        for x in (0..img.width()).step_by(8) {
            let tile = img.view(x, y, 8, 8);

            let pixels: [_; 64] = {
                let mut iter = tile.pixels();
                core::array::from_fn(|_| iter.next().unwrap().2)
            };
            out.extend(
                SwizzleIter::new(&pixels, tile.width() as usize)
                    .flat_map(|px| px.channels().to_owned()),
            );
        }
    }
    ImageBuffer::<I::Pixel, _>::from_vec(img.width(), img.height(), out)
        .unwrap()
        .into()
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, DerefMut};

    use image::{GenericImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

    use super::*;

    fn swizzle_in_place(img: &mut impl GenericImage) {
        check_image_requirements(img);

        for x in (0..img.width()).step_by(8) {
            for y in (0..img.height()).step_by(8) {
                let mut tile = img.sub_image(x, y, 8, 8);
                swizzle_tile(tile.deref_mut());
            }
        }
    }

    /// swizzle an 8x8 tile
    fn swizzle_tile(tile: &mut impl GenericImage) {
        assert_eq!(tile.width(), 8);
        assert_eq!(tile.height(), 8);
        let pixels: [_; 64] = {
            let mut iter = tile.pixels();
            core::array::from_fn(|_| iter.next().unwrap().2)
        };
        for (i, px) in SwizzleIter::new(&pixels, tile.width() as usize).enumerate() {
            let x = i as u32 % tile.width();
            let y = i as u32 / tile.width();
            tile.put_pixel(x, y, px);
        }
    }

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
    fn swizzle_iter_correct() {
        let data = (0..64).collect::<Vec<usize>>();
        let swizzed = SwizzleIter::new(&data, 8).collect::<Vec<_>>();
        assert_eq!(&swizzed, &TILE_IDX_REMAP);
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

        assert_eq!(
            &swizzle_image(&img)
                .pixels()
                .map(|px| px.2 .0[0] as usize)
                .collect::<Vec<_>>(),
            &both_tiles
        );
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
    }
}
