//! Half-block ANSI renderer.
//!
//! Each terminal cell stacks two pixels: top is fg (SGR 38;2;r;g;b), bottom is
//! bg (SGR 48;2;r;g;b). Glyph choice depends on which halves are opaque:
//!
//!   - both opaque        -> '▀' with fg=top, bg=bottom
//!   - only top opaque    -> '▀' with fg only
//!   - only bottom opaque -> '▄' with fg only
//!   - both transparent   -> ' '
//!
//! Alpha threshold: pixels with alpha < 16 are transparent, 16+ opaque. No
//! alpha-blending between the two pixels.

use image::{DynamicImage, RgbaImage, imageops};

#[cfg(test)]
use image::{ImageBuffer, Rgba};

/// Pixels with alpha below this are treated as transparent.
pub const ALPHA_THRESHOLD: u8 = 16;

/// Background-detection tolerance per channel for `trim_bg`.
const TRIM_TOL: i32 = 12;

/// Render an RGBA image as half-block ANSI text.
///
/// - `width`: target pixel width. 0 keeps native width. Otherwise the image is
///   Lanczos-resized so its width matches; height scales by the same ratio
///   (truncated toward zero, floored to 1).
/// - `trim`: crop uniform-background margins first. Helps item icons where the
///   subject sits small inside a card; monster cards fill the frame so trim is
///   a no-op for them.
pub fn render_halfblock(img: &DynamicImage, width: u32, trim: bool) -> String {
    let rgba = to_rgba(img);
    let rgba = if trim { trim_bg(&rgba) } else { rgba };

    let target = if width != 0 && rgba.width() != width {
        resize_lanczos(&rgba, width)
    } else {
        rgba
    };

    let (w, h) = (target.width(), target.height());

    let mut out = String::with_capacity((w * h / 2) as usize * 16);

    let at = |x: u32, y: u32| -> [u8; 4] { target.get_pixel(x, y).0 };

    let mut y = 0u32;
    while y < h {
        let bot_y = y + 1;
        let has_bot_row = bot_y < h;
        for x in 0..w {
            let top = at(x, y);
            let bot = if has_bot_row {
                at(x, bot_y)
            } else {
                [0, 0, 0, 0] // sentinel: fully transparent
            };
            let ta = top[3];
            let ba = bot[3];
            let top_opaque = ta >= ALPHA_THRESHOLD;
            let bot_opaque = ba >= ALPHA_THRESHOLD;

            // Reset bg every cell: terminal bg state is sticky, so a transparent
            // half would otherwise inherit the previous cell's bg (visible as a
            // colored fringe where pixels meet transparency).
            if !top_opaque && !bot_opaque {
                out.push_str(RESET_BG);
                out.push(' ');
            } else if !top_opaque {
                out.push_str(RESET_BG);
                out.push_str(&fg(bot[0], bot[1], bot[2]));
                out.push('▄');
            } else if !bot_opaque {
                out.push_str(RESET_BG);
                out.push_str(&fg(top[0], top[1], top[2]));
                out.push('▀');
            } else {
                // Both opaque: bg() overwrites any sticky bg from the prior cell.
                out.push_str(&fg(top[0], top[1], top[2]));
                out.push_str(&bg(bot[0], bot[1], bot[2]));
                out.push('▀');
            }
        }
        out.push_str(RESET);
        out.push('\n');
        y += 2;
    }

    // Drop the trailing newline; callers split on '\n' for icon rows.
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

/// Decode any image view to an owned RGBA buffer.
fn to_rgba(img: &DynamicImage) -> RgbaImage {
    if let Some(rgba) = img.as_rgba8() {
        return rgba.clone();
    }
    // 16-bit / non-8-bit RGBA falls back to DynamicImage's conversion.
    img.to_rgba8()
}

/// Crop uniform-background margins off `img`.
///
/// Matches preview.py `_trim_bg`: bg color is the majority of the 4 corner
/// pixels; a pixel is "non-bg" if any RGB channel differs from bg by more than
/// `TRIM_TOL`. If the non-bg bounding box has at least 1px margin on every
/// edge, expand by 1px and crop; otherwise leave the image alone (subject
/// fills the frame, cropping would eat into it).
fn trim_bg(img: &RgbaImage) -> RgbaImage {
    let (w, h) = (img.width(), img.height());
    if w == 0 || h == 0 {
        return img.clone();
    }

    let bg = majority_corner(img);

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;

    let is_bg = |p: [u8; 4]| -> bool {
        (i32::from(p[0]) - i32::from(bg[0])).abs() <= TRIM_TOL
            && (i32::from(p[1]) - i32::from(bg[1])).abs() <= TRIM_TOL
            && (i32::from(p[2]) - i32::from(bg[2])).abs() <= TRIM_TOL
    };

    for y in 0..h {
        for x in 0..w {
            if !is_bg(img.get_pixel(x, y).0) {
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    if max_x < min_x {
        // No non-bg pixels.
        return img.clone();
    }

    let margin_l = min_x;
    let margin_r = w - 1 - max_x;
    let margin_t = min_y;
    let margin_b = h - 1 - max_y;
    const MIN_MARGIN: u32 = 1;
    if margin_l < MIN_MARGIN
        || margin_r < MIN_MARGIN
        || margin_t < MIN_MARGIN
        || margin_b < MIN_MARGIN
    {
        return img.clone();
    }

    let x0 = min_x.saturating_sub(1);
    let y0 = min_y.saturating_sub(1);
    let x1 = (max_x + 1).min(w - 1);
    let y1 = (max_y + 1).min(h - 1);

    imageops::crop_imm(img, x0, y0, x1 - x0 + 1, y1 - y0 + 1).to_image()
}

/// Most-frequent pixel among the four corners (ties broken by first-seen).
/// Matches preview.py's `Counter(corners).most_common(1)[0][0]`.
fn majority_corner(img: &RgbaImage) -> [u8; 4] {
    let (w, h) = (img.width(), img.height());
    let corners = [
        img.get_pixel(0, 0).0,
        img.get_pixel(w - 1, 0).0,
        img.get_pixel(0, h - 1).0,
        img.get_pixel(w - 1, h - 1).0,
    ];
    let mut best = corners[0];
    let mut best_count = 1usize;
    for i in 1..4 {
        let mut count = 1;
        for j in 0..i {
            if corners[j] == corners[i] {
                count += 1;
            }
        }
        if count > best_count {
            best = corners[i];
            best_count = count;
        }
    }
    best
}

/// Lanczos3 resize that mirrors preview.py's `int(height * ratio)` truncation.
/// `image`'s `resize_exact` rounds; we instead compute height by truncation so
/// the output matches preview.py byte-for-byte for the same input.
fn resize_lanczos(img: &RgbaImage, width: u32) -> RgbaImage {
    let src_w = img.width();
    let src_h = img.height();
    if src_w == 0 || width == 0 {
        return img.clone();
    }
    // (height * ratio) truncated toward zero, floored to 1.
    let new_h = (((src_h as u64) * (width as u64)) / (src_w as u64)) as u32;
    let new_h = new_h.max(1);
    // resize() preserves aspect ratio of the passed (w,h); passing our
    // truncated height makes it match preview.py exactly.
    imageops::resize(img, width, new_h, imageops::FilterType::Lanczos3)
}

// ============================================================ ANSI ======

pub const RESET: &str = "\x1b[0m";

/// Default background, used on the transparent half of a half-block glyph so
/// the terminal's own bg shows through. Needed because terminal bg state is
/// sticky across cells (see `render_halfblock`).
pub const RESET_BG: &str = "\x1b[49m";

/// Set foreground truecolor: `\x1b[38;2;r;g;bm`.
pub fn fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

/// Set background truecolor: `\x1b[48;2;r;g;bm`.
pub fn bg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{r};{g};{b}m")
}

/// Strip SGR escape sequences so tests can assert on visible content.
#[cfg(test)]
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut bytes = s.chars().peekable();
    while let Some(c) = bytes.next() {
        if c == '\x1b' && bytes.peek() == Some(&'[') {
            // consume CSI sequence up to and including the final letter
            bytes.next(); // '['
            while let Some(&n) = bytes.peek() {
                bytes.next();
                if n.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 4×4 RGBA image from per-row pixel arrays.
    fn grid4(rows: [[Rgba<u8>; 4]; 4]) -> RgbaImage {
        let mut img = ImageBuffer::new(4, 4);
        for (y, row) in rows.iter().enumerate() {
            for (x, px) in row.iter().enumerate() {
                img.put_pixel(x as u32, y as u32, *px);
            }
        }
        img
    }

    fn rgba(c: [u8; 4]) -> Rgba<u8> {
        Rgba(c)
    }

    #[test]
    fn fully_transparent_renders_as_spaces() {
        // 4x4 all-transparent yields 2 rows of 4 spaces, each with RESET_BG.
        let clear = rgba([0, 0, 0, 0]);
        let img = DynamicImage::ImageRgba8(grid4([
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
        ]));
        let out = render_halfblock(&img, 0, false);
        let rows: Vec<&str> = out.split('\n').collect();
        assert_eq!(rows.len(), 2);
        for r in &rows {
            assert_eq!(strip_ansi(r), "    ");
            assert!(
                r.contains("\x1b[49m"),
                "transparent cell must reset bg: {r:?}"
            );
        }
    }

    #[test]
    fn opaque_pair_uses_upper_block_with_fg_and_bg() {
        let red = rgba([255, 0, 0, 255]);
        let green = rgba([0, 255, 0, 255]);
        let clear = rgba([0, 0, 0, 0]);
        let img = DynamicImage::ImageRgba8(grid4([
            [red, red, red, red],
            [green, green, green, green],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
        ]));
        let out = render_halfblock(&img, 0, false);
        let row0 = out.split('\n').next().unwrap();
        assert!(row0.contains('▀'));
        assert!(
            row0.contains("38;2;255;0;0"),
            "row0 should set fg=red: {row0:?}"
        );
        assert!(
            row0.contains("48;2;0;255;0"),
            "row0 should set bg=green: {row0:?}"
        );
        assert_eq!(strip_ansi(row0), "▀▀▀▀");
    }

    #[test]
    fn top_only_transparent_uses_lower_block_with_fg_and_reset_bg() {
        // Top transparent, bottom opaque red -> ▄ with fg=red and RESET_BG.
        let red = rgba([255, 0, 0, 255]);
        let clear = rgba([0, 0, 0, 0]);
        let img = DynamicImage::ImageRgba8(grid4([
            [clear, clear, clear, clear],
            [red, red, red, red],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
        ]));
        let out = render_halfblock(&img, 0, false);
        let row0 = out.split('\n').next().unwrap();
        assert!(row0.contains('▄'));
        assert!(
            row0.contains("38;2;255;0;0"),
            "row0 should set fg=red: {row0:?}"
        );
        assert!(
            row0.contains("\x1b[49m"),
            "row0 should reset bg to default: {row0:?}"
        );
        assert!(
            !row0.contains("48;2;"),
            "row0 must NOT paint the transparent top half with a bg color"
        );
    }

    #[test]
    fn alpha_threshold_is_exactly_16() {
        let almost = rgba([255, 0, 0, 15]); // transparent (alpha < 16)
        let barely = rgba([0, 255, 0, 16]); // opaque (alpha >= 16)
        let clear = rgba([0, 0, 0, 0]);
        let img = DynamicImage::ImageRgba8(grid4([
            [almost, almost, almost, almost],
            [barely, barely, barely, barely],
            [clear, clear, clear, clear],
            [clear, clear, clear, clear],
        ]));
        let out = render_halfblock(&img, 0, false);
        let row0 = out.split('\n').next().unwrap();
        // Top transparent (alpha 15) -> ▄ with fg=green from bottom + RESET_BG.
        assert!(row0.contains('▄'));
        assert!(
            row0.contains("38;2;0;255;0"),
            "fg should be green: {row0:?}"
        );
        assert!(
            !row0.contains("38;2;255"),
            "must not set fg=red (top is transparent)"
        );
    }

    #[test]
    fn trim_bg_is_noop_when_subject_fills_frame() {
        // 4x4 with 1px transparent border, 2x2 opaque center. The +1px box
        // expansion makes this a no-op crop.
        let red = rgba([255, 0, 0, 255]);
        let clear = rgba([0, 0, 0, 0]);
        let img = grid4([
            [clear, clear, clear, clear],
            [clear, red, red, clear],
            [clear, red, red, clear],
            [clear, clear, clear, clear],
        ]);
        let trimmed = trim_bg(&img);
        assert_eq!(trimmed.dimensions(), (4, 4));
    }

    #[test]
    fn trim_bg_crops_with_sufficient_margin() {
        // 8x8 with 2px border, 4x4 interior. Box x=2..5,y=2..5 expanded by 1
        // -> 6x6 crop.
        let red = rgba([255, 0, 0, 255]);
        let clear = rgba([0, 0, 0, 0]);
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(8, 8);
        for y in 0..8u32 {
            for x in 0..8u32 {
                let on_border = x < 2 || y < 2 || x > 5 || y > 5;
                img.put_pixel(x, y, if on_border { clear } else { red });
            }
        }
        let trimmed = trim_bg(&img);
        assert_eq!(trimmed.dimensions(), (6, 6));
    }

    #[test]
    fn trim_bg_returns_input_when_no_subject() {
        let clear = rgba([0, 0, 0, 0]);
        let img = ImageBuffer::from_pixel(4, 4, clear);
        let trimmed = trim_bg(&img);
        assert_eq!(trimmed.dimensions(), (4, 4));
    }

    #[test]
    fn lanczos_resize_truncates_height() {
        let img: RgbaImage = ImageBuffer::from_pixel(48, 48, Rgba([255, 0, 0, 255]));
        assert_eq!(resize_lanczos(&img, 40).dimensions(), (40, 40));
        assert_eq!(resize_lanczos(&img, 32).dimensions(), (32, 32));

        // 50x48 → width=40. new_h = (48*40)/50 = 38 (truncated, not 39).
        let img: RgbaImage = ImageBuffer::from_pixel(50, 48, Rgba([255, 0, 0, 255]));
        assert_eq!(resize_lanczos(&img, 40).dimensions(), (40, 38));
    }

    #[test]
    fn majority_corner_picks_most_common() {
        let red = rgba([255, 0, 0, 255]);
        let blue = rgba([0, 0, 255, 255]);
        // 3 red corners, 1 blue → red wins.
        let img: RgbaImage =
            ImageBuffer::from_fn(4, 4, |x, y| if (x, y) == (3, 3) { blue } else { red });
        assert_eq!(majority_corner(&img), [255, 0, 0, 255]);
    }
}
