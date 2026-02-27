#![allow(dead_code)]
//! Image decoding and preprocessing
//!
//! Handles:
//! - Loading from file or memory
//! - Format detection and decoding
//! - Smart resizing with aspect ratio correction
//! - Alpha compositing against terminal background
//! - Animated image handling (first frame)

use image::{DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
use std::path::Path;

pub struct DecodedImage {
    /// RGBA pixel data, row-major
    pub pixels: Vec<Pixel>,
    pub display_width: u32,
    pub display_height: u32, // always even (for half-block pairs)
    pub original_width: u32,
    pub original_height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn from_rgba(rgba: &Rgba<u8>) -> Self {
        Self {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        }
    }

    pub fn is_transparent(&self) -> bool {
        self.a < 128
    }

    pub fn composite_over(&self, bg_r: u8, bg_g: u8, bg_b: u8) -> Pixel {
        if self.a == 255 {
            return *self;
        }
        if self.a == 0 {
            return Pixel {
                r: bg_r,
                g: bg_g,
                b: bg_b,
                a: 255,
            };
        }

        let alpha = self.a as f32 / 255.0;
        let inv = 1.0 - alpha;

        Pixel {
            r: (self.r as f32 * alpha + bg_r as f32 * inv) as u8,
            g: (self.g as f32 * alpha + bg_g as f32 * inv) as u8,
            b: (self.b as f32 * alpha + bg_b as f32 * inv) as u8,
            a: 255,
        }
    }
}

impl DecodedImage {
    pub fn get_pixel(&self, x: u32, y: u32) -> Pixel {
        if x < self.display_width && y < self.display_height {
            self.pixels[(y * self.display_width + x) as usize]
        } else {
            Pixel {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }
        }
    }
}

pub fn load_and_prepare(path: &Path, max_width: u32, term_width: u16) -> Result<DecodedImage, String> {
    let img = if let Some(fmt) = format_from_path(path) {
        image::open(path)
            .or_else(|_| {
                let data = std::fs::read(path).map_err(|e| image::ImageError::IoError(e))?;
                image::load_from_memory_with_format(&data, fmt)
            })
            .map_err(|e| format!("{}", e))?
    } else {
        image::open(path).map_err(|e| format!("{}", e))?
    };

    prepare_image(img, max_width, term_width)
}

pub fn load_from_memory(data: &[u8], max_width: u32, term_width: u16) -> Result<DecodedImage, String> {
    let img = if let Ok(fmt) = image::guess_format(data) {
        image::load_from_memory_with_format(data, fmt)
    } else {
        image::load_from_memory(data)
    }
    .map_err(|e| format!("{}", e))?;

    prepare_image(img, max_width, term_width)
}

fn prepare_image(img: DynamicImage, max_width: u32, term_width: u16) -> Result<DecodedImage, String> {
    let (orig_w, orig_h) = img.dimensions();
    let (disp_w, disp_h) = calculate_display_size(orig_w, orig_h, max_width, term_width);

    let rgba: RgbaImage = if disp_w != orig_w || disp_h != orig_h {
        image::imageops::resize(&img.to_rgba8(), disp_w, disp_h, image::imageops::FilterType::Lanczos3)
    } else {
        img.to_rgba8()
    };

    let pixels: Vec<Pixel> = rgba.pixels().map(|p| Pixel::from_rgba(p)).collect();

    Ok(DecodedImage {
        pixels,
        display_width: disp_w,
        display_height: disp_h,
        original_width: orig_w,
        original_height: orig_h,
    })
}

/// Calculate optimal display dimensions
///
/// Constraints:
/// - Fit within max_width and term_width
/// - Maintain aspect ratio
/// - Height must be even (half-block rendering needs pixel pairs)
/// - Minimum size: 4x2
fn calculate_display_size(orig_w: u32, orig_h: u32, max_width: u32, term_width: u16) -> (u32, u32) {
    let max_w = max_width.max(4).min(term_width as u32 - 4); // leave margin

    let mut w = orig_w;
    let mut h = orig_h;

    // Scale down to fit width
    if w > max_w {
        h = (h as f64 * max_w as f64 / w as f64) as u32;
        w = max_w;
    }

    // Ensure minimum dimensions
    w = w.max(4);
    h = h.max(2);

    // Ensure even height for half-block rendering
    if h % 2 != 0 {
        h += 1;
    }

    (w, h)
}

fn format_from_path(path: &Path) -> Option<ImageFormat> {
    let ext = path.extension()?.to_str()?.to_lowercase();

    match ext.as_str() {
        "png" => Some(ImageFormat::Png),
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "gif" => Some(ImageFormat::Gif),
        "webp" => Some(ImageFormat::WebP),
        "bmp" => Some(ImageFormat::Bmp),
        "tiff" | "tif" => Some(ImageFormat::Tiff),
        "tga" => Some(ImageFormat::Tga),
        "ico" => Some(ImageFormat::Ico),
        "qoi" => Some(ImageFormat::Qoi),
        "exr" => Some(ImageFormat::OpenExr),
        "hdr" => Some(ImageFormat::Hdr),
        "ppm" | "pgm" | "pbm" | "pam" => Some(ImageFormat::Pnm),
        "ff" => Some(ImageFormat::Farbfeld),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_size_downscale() {
        let (w, h) = calculate_display_size(1920, 1080, 60, 80);
        assert_eq!(w, 60);
        assert!(h % 2 == 0); // even height
        assert!(h <= 40);    // reasonable
    }

    #[test]
    fn test_display_size_small_image() {
        let (w, h) = calculate_display_size(16, 16, 60, 80);
        assert_eq!(w, 16);
        assert_eq!(h, 16);
    }

    #[test]
    fn test_display_size_minimum() {
        let (w, h) = calculate_display_size(1, 1, 60, 80);
        assert!(w >= 4);
        assert!(h >= 2);
        assert!(h % 2 == 0);
    }

    #[test]
    fn test_pixel_composite() {
        let px = Pixel { r: 255, g: 0, b: 0, a: 128 };
        let result = px.composite_over(0, 0, 0);
        // ~50% red over black = ~128
        assert!(result.r > 100 && result.r < 160);
        assert_eq!(result.a, 255);
    }

    #[test]
    fn test_pixel_transparent() {
        let px = Pixel { r: 255, g: 0, b: 0, a: 0 };
        assert!(px.is_transparent());

        let px2 = Pixel { r: 255, g: 0, b: 0, a: 255 };
        assert!(!px2.is_transparent());
    }
}
