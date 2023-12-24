use std::ops::DerefMut;

pub mod encode;
mod swizzle;

pub use swizzle::swizzle_image;

use image::GenericImageView;
