use std::io::Write;

use image::{error::EncodingError, ImageEncoder, ImageError};

#[derive(Debug, Clone, Copy)]
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

pub struct Tex3dsEncoder<W> {
    to: W,
    compression: CompressionType,
}
impl<W> Tex3dsEncoder<W> {
    pub fn new(to: W) -> Self {
        Self {
            to,
            compression: CompressionType::None,
        }
    }
}

impl<W: Write> Tex3dsEncoder<W> {
    fn write_header(&mut self, data_sz: u32) -> image::ImageResult<()> {
        self.to
            .write_all(&compression_header(self.compression, data_sz))
            .map_err(|e| e.into())
    }
    fn encode_pix(&mut self, buf: &[u8], cty: image::ColorType) -> image::ImageResult<()> {
        let stride = cty.bytes_per_pixel();
        assert_eq!(
            buf.len() % stride as usize,
            0,
            "buffer doesn't have enough space"
        );
        for chunk in buf.chunks_exact(stride as usize) {
            for n in chunk.iter().rev() {
                self.to.write_all(&[*n])?;
            }
        }

        Ok(())
    }
}

impl<W: Write> ImageEncoder for Tex3dsEncoder<W> {
    fn write_image(
        mut self,
        buf: &[u8],
        _width: u32,
        _height: u32,
        color_type: image::ColorType,
    ) -> image::ImageResult<()> {
        self.write_header(buf.len().try_into().map_err(|e| {
            image::ImageError::Encoding(EncodingError::new(
                image::error::ImageFormatHint::Name("3ds texture".to_owned()),
                e,
            ))
        })?)?;

        self.encode_pix(buf, color_type)?;

        if buf.len() % 4 != 0 {
            for _ in 0..buf.len() % 4 {
                self.to.write_all(&[0; 1])?;
            }
        }

        Ok(())
    }
}
