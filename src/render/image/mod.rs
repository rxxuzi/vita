#![allow(dead_code)]
//! Image rendering module for vita
//!
//! Supports: PNG, JPEG, WebP, GIF, BMP, TIFF, TGA, QOI, ICO, EXR, PPM
//!
//! Architecture:
//!   mod.rs      - Public API, format support
//!   decoder.rs  - Loading, resizing, preprocessing
//!   renderer.rs - Half-block terminal rendering

mod decoder;
mod renderer;

use std::path::Path;

use crate::output::Output;
use crate::theme::Theme;

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "bmp", "tiff", "tif", "tga", "ico", "qoi", "exr",
    "ppm", "pgm", "pbm", "pam", "ff", "hdr",
];

pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn is_image_magic(buf: &[u8]) -> bool {
    if buf.len() < 12 {
        return false;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if buf.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return true;
    }

    // JPEG: FF D8 FF
    if buf.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return true;
    }

    // GIF: GIF87a / GIF89a
    if buf.starts_with(b"GIF87a") || buf.starts_with(b"GIF89a") {
        return true;
    }

    // WebP: RIFF....WEBP
    if buf.len() >= 12 && &buf[0..4] == b"RIFF" && &buf[8..12] == b"WEBP" {
        return true;
    }

    // BMP: BM
    if buf.starts_with(b"BM") {
        return true;
    }

    // TIFF: II (little-endian) or MM (big-endian)
    if (buf[0] == 0x49 && buf[1] == 0x49 && buf[2] == 0x2A && buf[3] == 0x00)
        || (buf[0] == 0x4D && buf[1] == 0x4D && buf[2] == 0x00 && buf[3] == 0x2A)
    {
        return true;
    }

    // QOI: qoif
    if buf.starts_with(b"qoif") {
        return true;
    }

    // EXR: 76 2F 31 01
    if buf.starts_with(&[0x76, 0x2F, 0x31, 0x01]) {
        return true;
    }

    // ICO: 00 00 01 00
    if buf.starts_with(&[0x00, 0x00, 0x01, 0x00]) {
        return true;
    }

    false
}

pub fn render(path: &Path, max_width: u32, theme: &Theme, out: &Output) {
    let decoded = match decoder::load_and_prepare(path, max_width, out.term_width) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("vita: {}: {}", path.display(), e);
            return;
        }
    };

    println!();
    renderer::render_halfblock(&decoded, out);

    let fmt_name = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("?")
        .to_uppercase();

    out.dim(
        &format!(
            "  {} {}×{} → {}×{}\n",
            fmt_name,
            decoded.original_width,
            decoded.original_height,
            decoded.display_width,
            decoded.display_height / 2
        ),
        theme.hr,
    );
}

pub fn render_bytes(data: &[u8], max_width: u32, theme: &Theme, out: &Output) {
    let decoded = match decoder::load_from_memory(data, max_width, out.term_width) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("vita: image: {}", e);
            return;
        }
    };

    println!();
    renderer::render_halfblock(&decoded, out);

    out.dim(
        &format!(
            "  {}×{} → {}×{}\n",
            decoded.original_width,
            decoded.original_height,
            decoded.display_width,
            decoded.display_height / 2
        ),
        theme.hr,
    );
}
