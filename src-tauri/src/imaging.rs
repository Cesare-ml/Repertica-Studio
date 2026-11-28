//! Image processing for the media center: crop, perspective correction, rotation, mirroring and
//! light/colour adjustments, plus previews, thumbnails, export and a perceptual hash for finding
//! similar pictures. Everything runs locally; originals are never modified here.
//!
//! Edits are described as a list of [`Operation`]s using coordinates relative to the image
//! (0..1), so the same edit can be previewed on a small copy and applied to the full picture.
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
    imageops::{self, FilterType},
    DynamicImage, ExtendedColorType, ImageDecoder, ImageEncoder, ImageFormat, ImageReader, Limits, Rgb, RgbImage,
};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Largest picture that will be decoded (width × height).
pub const MAX_PIXELS: u64 = 120_000_000;
const MAX_SIDE: u32 = 20_000;
const MAX_OUTPUT_SIDE: u32 = 8_000;
const JPEG_QUALITY: u8 = 92;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Point { pub x: f64, pub y: f64 }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Operation {
    Crop { x: f64, y: f64, width: f64, height: f64 },
    /// Corners in the order top-left, top-right, bottom-right, bottom-left.
    Perspective { points: Vec<Point>, #[serde(rename = "outputAspectRatio", default)] output_aspect_ratio: Option<f64> },
    Rotate { degrees: i32 },
    Flip { #[serde(default)] horizontal: bool, #[serde(default)] vertical: bool },
    Adjust { #[serde(default)] exposure: f64, #[serde(default)] contrast: f64, #[serde(default)] saturation: f64 },
}

#[derive(Debug)]
pub struct Encoded {
    pub bytes: Vec<u8>,
    pub extension: &'static str,
    pub mime: &'static str,
    pub width: u32,
    pub height: u32,
}

fn unit(v: f64) -> bool { v.is_finite() && (0.0..=1.0).contains(&v) }

// Reject self-intersecting, concave and almost-collinear selections before
// computing a homography. Corner coordinates are normalized to [0, 1], so
// the same threshold works for previews and full-resolution exports.
fn convex_quad(points: &[Point]) -> bool {
    if points.len() != 4 { return false; }
    let mut winding = 0.0_f64;
    for i in 0..4 {
        let (a, b, c) = (&points[i], &points[(i + 1) % 4], &points[(i + 2) % 4]);
        let cross = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if !cross.is_finite() || cross.abs() <= 1e-8 { return false; }
        if i == 0 { winding = cross.signum(); }
        else if cross.signum() != winding { return false; }
    }
    true
}

pub fn validate(operations: &[Operation]) -> Result<(), String> {
    if operations.is_empty() { return Err("Nessuna modifica da applicare".into()); }
    if operations.len() > 12 { return Err("Troppe operazioni di modifica".into()); }
    for operation in operations {
        let valid = match operation {
            Operation::Crop { x, y, width, height } => unit(*x) && unit(*y) && *width > 0.0 && *height > 0.0
                && unit(*width) && unit(*height) && x + width <= 1.0 + 1e-9 && y + height <= 1.0 + 1e-9,
            Operation::Perspective { points, output_aspect_ratio } => points.len() == 4
                && points.iter().all(|p| unit(p.x) && unit(p.y))
                && convex_quad(points)
                && output_aspect_ratio.is_none_or(|r| r.is_finite() && (0.1..=10.0).contains(&r)),
            Operation::Rotate { degrees } => [0, 90, 180, 270, -90, -180, -270].contains(degrees),
            Operation::Flip { .. } => true,
            Operation::Adjust { exposure, contrast, saturation } =>
                [exposure, contrast, saturation].iter().all(|v| v.is_finite() && (-1.0..=1.0).contains(*v)),
        };
        if !valid { return Err("Parametri di modifica non validi".into()); }
    }
    Ok(())
}

fn reader(bytes: &[u8]) -> Result<ImageReader<Cursor<&[u8]>>, String> {
    let mut reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|e| e.to_string())?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(1 << 30);
    reader.limits(limits);
    Ok(reader)
}

/// Pixel size after applying the EXIF orientation, without decoding the picture.
pub fn dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    let mut decoder = reader(bytes)?.into_decoder().map_err(|e| format!("Immagine non leggibile: {e}"))?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS { return Err("Immagine troppo grande (oltre 120 megapixel)".into()); }
    let swapped = decoder.orientation().map(|o| matches!(o,
        image::metadata::Orientation::Rotate90 | image::metadata::Orientation::Rotate270 |
        image::metadata::Orientation::Rotate90FlipH | image::metadata::Orientation::Rotate270FlipH)).unwrap_or(false);
    Ok(if swapped { (height, width) } else { (width, height) })
}

fn decode(bytes: &[u8]) -> Result<(RgbImage, ImageFormat), String> {
    let reader = reader(bytes)?;
    let format = reader.format().ok_or("Formato immagine non riconosciuto")?;
    if !matches!(format, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
        return Err("Sono consentite solo immagini JPEG, PNG e WebP".into());
    }
    let mut decoder = reader.into_decoder().map_err(|e| format!("Immagine non leggibile: {e}"))?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_PIXELS { return Err("Immagine troppo grande (oltre 120 megapixel)".into()); }
    let orientation = decoder.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| format!("Immagine non leggibile: {e}"))?;
    image.apply_orientation(orientation);
    Ok((image.to_rgb8(), format))
}

fn limit_side(image: RgbImage, max_side: u32) -> RgbImage {
    let (w, h) = image.dimensions();
    let longest = w.max(h);
    if longest <= max_side { return image; }
    let scale = f64::from(max_side) / f64::from(longest);
    imageops::resize(&image, ((f64::from(w) * scale).round() as u32).max(1), ((f64::from(h) * scale).round() as u32).max(1), FilterType::Triangle)
}

fn encode(image: &RgbImage, png: bool) -> Result<Encoded, String> {
    let (width, height) = image.dimensions();
    let mut bytes = Vec::new();
    if png {
        PngEncoder::new(&mut bytes).write_image(image.as_raw(), width, height, ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
    } else {
        JpegEncoder::new_with_quality(&mut bytes, JPEG_QUALITY).write_image(image.as_raw(), width, height, ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
    }
    Ok(Encoded { bytes, extension: if png { "png" } else { "jpg" }, mime: if png { "image/png" } else { "image/jpeg" }, width, height })
}

// ------------------------------------------------------------------ operations

fn crop(image: &RgbImage, x: f64, y: f64, width: f64, height: f64) -> RgbImage {
    let (w, h) = image.dimensions();
    let left = ((x * f64::from(w)).round() as u32).min(w - 1);
    let top = ((y * f64::from(h)).round() as u32).min(h - 1);
    let cw = ((width * f64::from(w)).round() as u32).clamp(1, w - left);
    let ch = ((height * f64::from(h)).round() as u32).clamp(1, h - top);
    imageops::crop_imm(image, left, top, cw, ch).to_image()
}

/// Solve the 8×8 system of a homography; `None` when the corners are degenerate.
fn solve(mut a: [[f64; 9]; 8]) -> Option<[f64; 8]> {
    for column in 0..8 {
        let pivot = (column..8).max_by(|&i, &j| a[i][column].abs().total_cmp(&a[j][column].abs()))?;
        if a[pivot][column].abs() < 1e-12 { return None; }
        a.swap(column, pivot);
        for row in 0..8 {
            if row == column { continue; }
            let factor = a[row][column] / a[column][column];
            for k in column..9 { a[row][k] -= factor * a[column][k]; }
        }
    }
    let mut result = [0.0; 8];
    for i in 0..8 { result[i] = a[i][8] / a[i][i]; }
    Some(result)
}

fn bilinear(image: &RgbImage, x: f64, y: f64) -> Rgb<u8> {
    let (w, h) = image.dimensions();
    let x = (x - 0.5).clamp(0.0, f64::from(w - 1));
    let y = (y - 0.5).clamp(0.0, f64::from(h - 1));
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
    let (fx, fy) = (x - f64::from(x0), y - f64::from(y0));
    let mut out = [0u8; 3];
    for c in 0..3 {
        let top = f64::from(image.get_pixel(x0, y0)[c]) * (1.0 - fx) + f64::from(image.get_pixel(x1, y0)[c]) * fx;
        let bottom = f64::from(image.get_pixel(x0, y1)[c]) * (1.0 - fx) + f64::from(image.get_pixel(x1, y1)[c]) * fx;
        out[c] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
    }
    Rgb(out)
}

fn perspective(image: &RgbImage, points: &[Point], ratio: Option<f64>) -> Result<RgbImage, String> {
    let (w, h) = image.dimensions();
    let q: Vec<(f64, f64)> = points.iter().map(|p| (p.x * f64::from(w), p.y * f64::from(h))).collect();
    let dist = |a: (f64, f64), b: (f64, f64)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let (top, bottom, left, right) = (dist(q[0], q[1]), dist(q[3], q[2]), dist(q[0], q[3]), dist(q[1], q[2]));
    let mut out_w = (top + bottom) / 2.0;
    let mut out_h = (left + right) / 2.0;
    if let Some(r) = ratio { out_h = out_w / r; }
    let longest = out_w.max(out_h);
    if longest > f64::from(MAX_OUTPUT_SIDE) { let s = f64::from(MAX_OUTPUT_SIDE) / longest; out_w *= s; out_h *= s; }
    let (ow, oh) = ((out_w.round() as u32).max(16), (out_h.round() as u32).max(16));
    // Homography from the output rectangle to the quadrilateral in the source.
    let dst = [(0.0, 0.0), (f64::from(ow), 0.0), (f64::from(ow), f64::from(oh)), (0.0, f64::from(oh))];
    let mut system = [[0.0_f64; 9]; 8];
    for i in 0..4 {
        let ((x, y), (u, v)) = (dst[i], q[i]);
        system[2 * i] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        system[2 * i + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }
    let hm = solve(system).ok_or("I quattro punti della prospettiva non formano un quadrilatero valido")?;
    let mut out = RgbImage::new(ow, oh);
    for py in 0..oh {
        for px in 0..ow {
            let (x, y) = (f64::from(px) + 0.5, f64::from(py) + 0.5);
            let d = hm[6] * x + hm[7] * y + 1.0;
            if d.abs() < 1e-12 { continue; }
            out.put_pixel(px, py, bilinear(image, (hm[0] * x + hm[1] * y + hm[2]) / d, (hm[3] * x + hm[4] * y + hm[5]) / d));
        }
    }
    Ok(out)
}

fn adjust(image: &mut RgbImage, exposure: f64, contrast: f64, saturation: f64) {
    let gain = 2f32.powf(exposure as f32);
    let (contrast, saturation) = (1.0 + contrast as f32, 1.0 + saturation as f32);
    for pixel in image.pixels_mut() {
        let mut rgb = [0f32; 3];
        for c in 0..3 {
            let v = f32::from(pixel[c]) / 255.0 * gain;
            rgb[c] = (v - 0.5) * contrast + 0.5;
        }
        let luma = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
        for c in 0..3 {
            pixel[c] = ((luma + (rgb[c] - luma) * saturation).clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }
}

fn apply(mut image: RgbImage, operations: &[Operation]) -> Result<RgbImage, String> {
    for operation in operations {
        image = match operation {
            Operation::Crop { x, y, width, height } => crop(&image, *x, *y, *width, *height),
            Operation::Perspective { points, output_aspect_ratio } => perspective(&image, points, *output_aspect_ratio)?,
            Operation::Rotate { degrees } => match degrees.rem_euclid(360) {
                90 => imageops::rotate90(&image), 180 => imageops::rotate180(&image), 270 => imageops::rotate270(&image), _ => image,
            },
            Operation::Flip { horizontal, vertical } => {
                if *horizontal { imageops::flip_horizontal_in_place(&mut image); }
                if *vertical { imageops::flip_vertical_in_place(&mut image); }
                image
            }
            Operation::Adjust { exposure, contrast, saturation } => { adjust(&mut image, *exposure, *contrast, *saturation); image }
        };
    }
    Ok(image)
}

// ------------------------------------------------------------------ public API

/// Apply `operations` to the full-resolution picture. PNG stays PNG; JPEG and WebP become JPEG.
pub fn process(bytes: &[u8], operations: &[Operation]) -> Result<Encoded, String> {
    validate(operations)?;
    let (image, format) = decode(bytes)?;
    encode(&apply(image, operations)?, format == ImageFormat::Png)
}

/// A JPEG preview no larger than `max_side`; edits are applied to the reduced copy.
pub fn preview(bytes: &[u8], max_side: u32, operations: &[Operation]) -> Result<Encoded, String> {
    if !operations.is_empty() { validate(operations)?; }
    let (image, _) = decode(bytes)?;
    let mut image = limit_side(image, max_side);
    if !operations.is_empty() {
        image = limit_side(apply(image, operations)?, max_side);
    }
    encode(&image, false)
}

pub fn thumbnail(bytes: &[u8], max_side: u32) -> Result<Encoded, String> {
    let (image, _) = decode(bytes)?;
    let image = limit_side(image, max_side);
    let (width, height) = image.dimensions();
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, 80).write_image(image.as_raw(), width, height, ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
    Ok(Encoded { bytes: out, extension: "jpg", mime: "image/jpeg", width, height })
}

/// Re-encode as `jpeg` or `png`, optionally shrinking to fit `max_width` × `max_height`.
pub fn export(bytes: &[u8], png: bool, quality: u8, max_width: Option<u32>, max_height: Option<u32>) -> Result<Encoded, String> {
    let (mut image, _) = decode(bytes)?;
    let (w, h) = image.dimensions();
    let scale = [max_width.map(|m| f64::from(m) / f64::from(w)), max_height.map(|m| f64::from(m) / f64::from(h)), Some(1.0)]
        .into_iter().flatten().fold(f64::INFINITY, f64::min);
    if scale < 1.0 {
        image = imageops::resize(&image, ((f64::from(w) * scale).round() as u32).max(1), ((f64::from(h) * scale).round() as u32).max(1), FilterType::Lanczos3);
    }
    if png { return encode(&image, true); }
    let (width, height) = image.dimensions();
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, quality.clamp(40, 100)).write_image(image.as_raw(), width, height, ExtendedColorType::Rgb8).map_err(|e| e.to_string())?;
    Ok(Encoded { bytes: out, extension: "jpg", mime: "image/jpeg", width, height })
}

/// 64-bit difference hash: similar pictures have hashes that differ in only a few bits.
pub fn dhash(bytes: &[u8]) -> Result<u64, String> {
    let (image, _) = decode(bytes)?;
    let gray = imageops::grayscale(&imageops::resize(&image, 9, 8, FilterType::Triangle));
    let mut hash = 0_u64;
    for y in 0..8 {
        for x in 0..8 {
            hash = (hash << 1) | u64::from(gray.get_pixel(x, y)[0] > gray.get_pixel(x + 1, y)[0]);
        }
    }
    Ok(hash)
}

pub fn hamming(a: u64, b: u64) -> u32 { (a ^ b).count_ones() }

#[cfg(test)]
mod tests {
    use super::*;

    /// A picture whose colour tells where a pixel is: red grows to the right, green downwards.
    fn gradient(w: u32, h: u32) -> RgbImage {
        RgbImage::from_fn(w, h, |x, y| Rgb([(x * 255 / (w - 1)) as u8, (y * 255 / (h - 1)) as u8, 90]))
    }
    fn png(image: &RgbImage) -> Vec<u8> { encode(image, true).unwrap().bytes }
    fn decoded(encoded: &Encoded) -> RgbImage { decode(&encoded.bytes).unwrap().0 }
    fn close(a: Rgb<u8>, b: [u8; 3], tolerance: i32) -> bool { (0..3).all(|c| (i32::from(a[c]) - i32::from(b[c])).abs() <= tolerance) }

    #[test]
    fn crop_keeps_the_selected_region() {
        let result = process(&png(&gradient(100, 50)), &[Operation::Crop { x: 0.5, y: 0.0, width: 0.5, height: 1.0 }]).unwrap();
        assert_eq!((result.width, result.height, result.extension), (50, 50, "png"));
        let image = decoded(&result);
        assert!(image.get_pixel(0, 0)[0] >= 125, "the left edge of the crop is the middle of the original");
    }

    #[test]
    fn rotation_and_mirroring_move_pixels_exactly() {
        let source = gradient(40, 20);
        let rotated = decoded(&process(&png(&source), &[Operation::Rotate { degrees: 90 }]).unwrap());
        assert_eq!(rotated.dimensions(), (20, 40));
        assert_eq!(rotated.get_pixel(19, 0), source.get_pixel(0, 0), "clockwise: top-left goes to top-right");
        let turned = decoded(&process(&png(&source), &[Operation::Rotate { degrees: 180 }]).unwrap());
        assert_eq!(turned.get_pixel(39, 19), source.get_pixel(0, 0));
        let flipped = decoded(&process(&png(&source), &[Operation::Flip { horizontal: true, vertical: false }]).unwrap());
        assert_eq!(flipped.get_pixel(0, 0), source.get_pixel(39, 0));
        let both = decoded(&process(&png(&source), &[Operation::Flip { horizontal: true, vertical: true }]).unwrap());
        assert_eq!(both.get_pixel(0, 0), source.get_pixel(39, 19));
    }

    #[test]
    fn light_and_colour_adjustments_do_what_the_sliders_say() {
        let grey = RgbImage::from_pixel(8, 8, Rgb([100, 100, 100]));
        let brighter = decoded(&process(&png(&grey), &[Operation::Adjust { exposure: 0.5, contrast: 0.0, saturation: 0.0 }]).unwrap());
        let darker = decoded(&process(&png(&grey), &[Operation::Adjust { exposure: -0.5, contrast: 0.0, saturation: 0.0 }]).unwrap());
        assert!(brighter.get_pixel(0, 0)[0] > 100 && darker.get_pixel(0, 0)[0] < 100);
        let colourful = RgbImage::from_pixel(8, 8, Rgb([200, 80, 40]));
        let mono = decoded(&process(&png(&colourful), &[Operation::Adjust { exposure: 0.0, contrast: 0.0, saturation: -1.0 }]).unwrap());
        let p = mono.get_pixel(0, 0);
        assert!(p[0] == p[1] && p[1] == p[2], "full desaturation gives grey");
        let vivid = decoded(&process(&png(&colourful), &[Operation::Adjust { exposure: 0.0, contrast: 0.0, saturation: 0.6 }]).unwrap());
        assert!(i32::from(vivid.get_pixel(0, 0)[0]) - i32::from(vivid.get_pixel(0, 0)[2]) > 160);
        let flat = decoded(&process(&png(&gradient(32, 32)), &[Operation::Adjust { exposure: 0.0, contrast: 0.0, saturation: 0.0 }]).unwrap());
        assert!(close(*flat.get_pixel(5, 5), gradient(32, 32).get_pixel(5, 5).0, 1), "neutral settings change nothing");
    }

    #[test]
    fn perspective_straightens_a_tilted_rectangle() {
        // A 200x200 picture where the artwork is a slanted quadrilateral filled with distinct corner colours.
        let mut source = RgbImage::from_pixel(200, 200, Rgb([255, 255, 255]));
        let quad = [(40.0, 30.0), (170.0, 50.0), (160.0, 170.0), (30.0, 150.0)];
        for y in 0..200u32 {
            for x in 0..200u32 {
                let (fx, fy) = (f64::from(x), f64::from(y));
                let inside = (0..4).all(|i| {
                    let (a, b) = (quad[i], quad[(i + 1) % 4]);
                    (b.0 - a.0) * (fy - a.1) - (b.1 - a.1) * (fx - a.0) >= 0.0
                });
                if inside { source.put_pixel(x, y, Rgb([220, 30, 30])); }
            }
        }
        let points: Vec<Point> = quad.iter().map(|(x, y)| Point { x: x / 200.0, y: y / 200.0 }).collect();
        let result = process(&png(&source), &[Operation::Perspective { points, output_aspect_ratio: None }]).unwrap();
        let image = decoded(&result);
        let (w, h) = image.dimensions();
        assert!(w > 100 && h > 100);
        for (x, y) in [(2, 2), (w - 3, 2), (w - 3, h - 3), (2, h - 3), (w / 2, h / 2)] {
            assert!(close(*image.get_pixel(x, y), [220, 30, 30], 4), "pixel ({x},{y}) must come from inside the artwork");
        }
        let squared = process(&png(&source), &[Operation::Perspective {
            points: quad.iter().map(|(x, y)| Point { x: x / 200.0, y: y / 200.0 }).collect(), output_aspect_ratio: Some(1.0) }]).unwrap();
        assert_eq!(squared.width, squared.height, "a fixed aspect ratio shapes the output");
    }

    #[test]
    fn perspective_with_the_full_frame_is_an_identity() {
        let source = gradient(60, 40);
        let full = vec![Point { x: 0.0, y: 0.0 }, Point { x: 1.0, y: 0.0 }, Point { x: 1.0, y: 1.0 }, Point { x: 0.0, y: 1.0 }];
        let result = decoded(&process(&png(&source), &[Operation::Perspective { points: full, output_aspect_ratio: None }]).unwrap());
        assert_eq!(result.dimensions(), (60, 40));
        assert!(close(*result.get_pixel(30, 20), source.get_pixel(30, 20).0, 3));
        let degenerate = vec![Point { x: 0.2, y: 0.2 }; 4];
        assert!(process(&png(&source), &[Operation::Perspective { points: degenerate, output_aspect_ratio: None }]).is_err());
    }

    #[test]
    fn perspective_refuses_crossed_or_concave_corners_before_rendering() {
        let square = vec![
            Point { x: 0.1, y: 0.1 }, Point { x: 0.9, y: 0.1 },
            Point { x: 0.9, y: 0.9 }, Point { x: 0.1, y: 0.9 },
        ];
        assert!(validate(&[Operation::Perspective {
            points: square.clone(), output_aspect_ratio: None,
        }]).is_ok());

        // Both winding directions are valid, but the four corners must
        // traverse a convex perimeter rather than cross or fold inward.
        let mut reversed = square.clone();
        reversed.reverse();
        assert!(validate(&[Operation::Perspective {
            points: reversed, output_aspect_ratio: None,
        }]).is_ok());

        let crossed = vec![square[0], square[2], square[1], square[3]];
        let concave = vec![square[0], square[1], Point { x: 0.4, y: 0.4 }, square[3]];
        let collinear = vec![
            square[0], Point { x: 0.5, y: 0.1 },
            square[1], square[3],
        ];
        let source = png(&gradient(40, 40));
        let nearly_collinear = vec![square[0], Point { x: 0.5, y: 0.1 - 1e-10 }, square[1], square[3]];
        let repeated = vec![square[0], square[1], square[1], square[3]];
        for corners in [crossed, concave, collinear, nearly_collinear, repeated] {
            let edits = [Operation::Perspective { points: corners, output_aspect_ratio: None }];
            assert!(validate(&edits).is_err(), "invalid quadrilateral must fail validation");
            assert!(process(&source, &edits).is_err(), "full-size export must refuse it");
            assert!(preview(&source, 20, &edits).is_err(), "preview must refuse it too");
            assert_eq!(preview(b"not an image", 20, &edits).unwrap_err(), "Parametri di modifica non validi", "preview validates before decoding");
        }
    }

    #[test]
    fn operations_are_validated_before_any_work() {
        let bytes = png(&gradient(10, 10));
        for bad in [
            vec![],
            vec![Operation::Crop { x: 0.6, y: 0.0, width: 0.6, height: 1.0 }],
            vec![Operation::Crop { x: 0.0, y: 0.0, width: 0.0, height: 1.0 }],
            vec![Operation::Rotate { degrees: 45 }],
            vec![Operation::Adjust { exposure: 5.0, contrast: 0.0, saturation: 0.0 }],
            vec![Operation::Perspective { points: vec![Point { x: 0.0, y: 0.0 }], output_aspect_ratio: None }],
            vec![Operation::Perspective { points: vec![Point { x: 0.0, y: 0.0 }; 4], output_aspect_ratio: Some(0.0) }],
            vec![Operation::Rotate { degrees: 90 }; 13],
        ] {
            assert!(process(&bytes, &bad).is_err(), "{bad:?}");
        }
        assert!(process(b"not an image", &[Operation::Rotate { degrees: 90 }]).is_err());
    }

    #[test]
    fn operations_parse_from_the_json_the_interface_sends() {
        let json = r#"[{"type":"perspective","points":[{"x":0.1,"y":0.1},{"x":0.9,"y":0.1},{"x":0.9,"y":0.9},{"x":0.1,"y":0.9}],"outputAspectRatio":1.5},
            {"type":"crop","x":0.1,"y":0.2,"width":0.5,"height":0.5},{"type":"rotate","degrees":90},
            {"type":"flip","horizontal":true,"vertical":false},{"type":"adjust","exposure":0.1,"contrast":-0.2,"saturation":0.3}]"#;
        let parsed: Vec<Operation> = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.len(), 5);
        assert!(matches!(&parsed[0], Operation::Perspective { output_aspect_ratio: Some(r), .. } if (*r - 1.5).abs() < 1e-9));
        assert!(validate(&parsed).is_ok());
        assert!(serde_json::from_str::<Vec<Operation>>(r#"[{"type":"drop_tables"}]"#).is_err());
    }

    #[test]
    fn previews_thumbnails_and_exports_respect_size_and_format() {
        let bytes = png(&gradient(400, 200));
        let thumb = thumbnail(&bytes, 100).unwrap();
        assert_eq!((thumb.width, thumb.height, thumb.mime), (100, 50, "image/jpeg"));
        assert!(thumb.bytes.starts_with(&[0xff, 0xd8, 0xff]));
        let small = preview(&bytes, 1000, &[]).unwrap();
        assert_eq!((small.width, small.height), (400, 200), "previews never enlarge");
        let edited = preview(&bytes, 200, &[Operation::Rotate { degrees: 90 }]).unwrap();
        assert_eq!((edited.width, edited.height), (100, 200));
        let jpeg = export(&bytes, false, 85, Some(200), None).unwrap();
        assert_eq!((jpeg.width, jpeg.height), (200, 100));
        let boxed = export(&bytes, true, 90, Some(300), Some(40)).unwrap();
        assert_eq!((boxed.height, boxed.extension), (40, "png"));
        assert!(boxed.bytes.starts_with(&[137, 80, 78, 71]));
        assert_eq!(dimensions(&bytes).unwrap(), (400, 200));
        assert!(dimensions(b"nope").is_err());
    }

    #[test]
    fn similar_pictures_have_close_hashes_and_different_ones_do_not() {
        let original = gradient(160, 120);
        let brighter = decoded(&process(&png(&original), &[Operation::Adjust { exposure: 0.15, contrast: 0.0, saturation: 0.0 }]).unwrap());
        let (a, b) = (dhash(&png(&original)).unwrap(), dhash(&png(&brighter)).unwrap());
        assert!(hamming(a, b) <= 6, "a small exposure change keeps the picture recognisable ({})", hamming(a, b));
        assert_eq!(hamming(a, a), 0);
        let different = RgbImage::from_fn(160, 120, |x, y| if (x / 20 + y / 20) % 2 == 0 { Rgb([250, 250, 250]) } else { Rgb([5, 5, 5]) });
        assert!(hamming(a, dhash(&png(&different)).unwrap()) > 12);
    }

    #[test]
    fn unsupported_formats_and_oversized_pictures_are_refused() {
        let mut gif = b"GIF89a".to_vec();
        gif.extend([0; 32]);
        assert!(decode(&gif).is_err());
        assert!(decode(b"%PDF-1.7 not a picture").is_err());
        assert!(decode(&[]).is_err());
    }
}
