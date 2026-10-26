//! Writes the packaged raster icons from the mark the app itself draws
//! (`spotifast::util::app_icon_rgba`), so the window, the tray and every
//! package show one picture. `packaging/icons/spotifast.svg` and
//! `docs/assets/images/logo.svg` draw the same geometry as vectors; change
//! them together.
//!
//! `cargo run --example app_icon` regenerates
//! `packaging/windows/spotifast.ico` (16 to 256 pixels) and
//! `packaging/macos/icon-1024.png` (the tile on Apple's 824-pixel grid with
//! a soft shadow beneath it).

use std::path::Path;

use image::ImageEncoder;
use spotifast::util::app_icon_rgba;

const WINDOWS_SIZES: [u32; 7] = [16, 24, 32, 48, 64, 128, 256];

fn png(rgba: &[u8], size: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(rgba, size, size, image::ExtendedColorType::Rgba8)
        .expect("the icon encodes");
    bytes
}

/// An ICO file holding one PNG per size, as Windows Vista and later read.
fn ico() -> Vec<u8> {
    let images: Vec<(u32, Vec<u8>)> = WINDOWS_SIZES
        .iter()
        .map(|&size| (size, png(&app_icon_rgba(size as usize), size)))
        .collect();
    let mut file = Vec::new();
    file.extend_from_slice(&[0, 0, 1, 0]);
    file.extend_from_slice(&(images.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * images.len() as u32;
    for (size, bytes) in &images {
        // A side of 256 is written as 0.
        let side = (*size % 256) as u8;
        file.extend_from_slice(&[side, side, 0, 0]);
        file.extend_from_slice(&1u16.to_le_bytes());
        file.extend_from_slice(&32u16.to_le_bytes());
        file.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        file.extend_from_slice(&offset.to_le_bytes());
        offset += bytes.len() as u32;
    }
    for (_, bytes) in &images {
        file.extend_from_slice(bytes);
    }
    file
}

/// Three box blurs of `radius` across and down, close to a Gaussian.
fn blur(values: &mut [f32], side: usize, radius: usize) {
    let mut scratch = vec![0.0; values.len()];
    for _ in 0..3 {
        for (step, stride) in [(1, side), (side, 1)] {
            for line in 0..side {
                let start = line * stride;
                let mut sum = 0.0;
                for i in 0..=radius.min(side - 1) {
                    sum += values[start + i * step];
                }
                for i in 0..side {
                    scratch[start + i * step] = sum / (2 * radius + 1) as f32;
                    if i + radius + 1 < side {
                        sum += values[start + (i + radius + 1) * step];
                    }
                    if i >= radius {
                        sum -= values[start + (i - radius) * step];
                    }
                }
            }
            values.copy_from_slice(&scratch);
        }
    }
}

/// The macOS icon: the tile 824 pixels wide in the middle of 1024, over a
/// shadow that falls 10 pixels below it.
fn mac_icon() -> Vec<u8> {
    const CANVAS: usize = 1024;
    const TILE: usize = 824;
    // The mark keeps two pixels of margin around its square.
    let raster = TILE + 4;
    let at = (CANVAS - raster) / 2;
    let tile = app_icon_rgba(raster);
    let mut shadow = vec![0.0f32; CANVAS * CANVAS];
    for y in 0..raster {
        for x in 0..raster {
            let target = (at + y + 10).min(CANVAS - 1) * CANVAS + at + x;
            shadow[target] = f32::from(tile[(y * raster + x) * 4 + 3]) / 255.0;
        }
    }
    blur(&mut shadow, CANVAS, 9);
    let mut rgba = vec![0u8; CANVAS * CANVAS * 4];
    for y in 0..CANVAS {
        for x in 0..CANVAS {
            let below = 0.3 * shadow[y * CANVAS + x];
            let (mut colour, mut alpha) = ([0.0f32; 3], 0.0f32);
            if (at..at + raster).contains(&x) && (at..at + raster).contains(&y) {
                let source = ((y - at) * raster + x - at) * 4;
                alpha = f32::from(tile[source + 3]) / 255.0;
                colour = [tile[source], tile[source + 1], tile[source + 2]].map(f32::from);
            }
            // The tile over its black shadow.
            let total = alpha + below * (1.0 - alpha);
            let index = (y * CANVAS + x) * 4;
            if total > 0.0 {
                for (channel, value) in colour.iter().enumerate() {
                    rgba[index + channel] = (value * alpha / total).round() as u8;
                }
            }
            rgba[index + 3] = (total * 255.0).round() as u8;
        }
    }
    rgba
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let windows = root.join("packaging/windows/spotifast.ico");
    std::fs::write(&windows, ico()).expect("the Windows icon writes");
    let mac = root.join("packaging/macos/icon-1024.png");
    std::fs::write(&mac, png(&mac_icon(), 1024)).expect("the macOS icon writes");
    println!("wrote {} and {}", windows.display(), mac.display());
}
