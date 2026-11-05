//! Render the pairing QR payload to pixels (replaces the WebView <canvas>).

use qrcode::{Color, QrCode};
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// One module = `scale` pixels, with a 4-module quiet zone.
pub fn render(data: &str, scale: u32) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let code = QrCode::new(data.as_bytes()).ok()?;
    let width = code.width() as u32;
    let quiet = 4;
    let side = (width + quiet * 2) * scale;
    let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(side, side);
    let colors = code.to_colors();
    let pixels = buf.make_mut_slice();
    for py in 0..side {
        for px in 0..side {
            let mx = (px / scale) as i64 - quiet as i64;
            let my = (py / scale) as i64 - quiet as i64;
            let dark = mx >= 0
                && my >= 0
                && (mx as u32) < width
                && (my as u32) < width
                && colors[(my as u32 * width + mx as u32) as usize] == Color::Dark;
            let v = if dark { 0x11 } else { 0xff };
            pixels[(py * side + px) as usize] = Rgba8Pixel {
                r: v,
                g: v,
                b: v,
                a: 0xff,
            };
        }
    }
    Some(buf)
}
