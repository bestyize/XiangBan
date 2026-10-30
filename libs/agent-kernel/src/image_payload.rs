//! Bounded, request-only image preparation. Never overwrites the source file.
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};
use std::io::Cursor;

const MAX_INPUT: usize = 10 * 1024 * 1024;
const MAX_SIDE: u32 = 16_384;
const MAX_ALLOC: u64 = 256 * 1024 * 1024;

/// Prepare an image for a new conversation entry, keeping the source elsewhere untouched.
pub fn prepare_data_url(url: &str) -> Result<Option<String>, String> {
    let mut current = url.to_owned();
    for edge in [2000, 1500, 1024, 768, 512] {
        if let Some(smaller) = optimize_data_url(&current, edge, 1024 * 1024)? { current = smaller; }
        let encoded = current.split_once(',').map_or(0, |(_, data)| data.len());
        if encoded <= (1024 * 1024usize).div_ceil(3) * 4 { break; }
    }
    Ok((current != url).then_some(current))
}

/// Return a smaller model-facing data URL when needed. Invalid, unsupported or very large
/// images return an error so callers can decide whether to keep the original or report it.
pub fn optimize_data_url(url: &str, max_edge: u32, target_bytes: usize) -> Result<Option<String>, String> {
    let (header, data) = url.split_once(',').ok_or("invalid image URL")?;
    if !matches!(header, "data:image/png;base64" | "data:image/jpeg;base64" | "data:image/gif;base64" | "data:image/webp;base64") { return Ok(None); }
    if data.len() > MAX_INPUT.div_ceil(3) * 4 { return Err("image exceeds decoding limit".into()); }
    let bytes = STANDARD.decode(data).map_err(|_| "invalid image base64")?;
    let reader = || ImageReader::new(Cursor::new(&bytes)).with_guessed_format().map_err(|e| e.to_string());
    let (width, height) = reader()?.into_dimensions().map_err(|e| e.to_string())?;
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE { return Err("image dimensions exceed decoding limit".into()); }
    if bytes.len() <= target_bytes && width.max(height) <= max_edge { return Ok(None); }
    let mut reader = reader()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_ALLOC);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
    if decoder.total_bytes() > MAX_ALLOC { return Err("image exceeds decoded size limit".into()); }
    let orientation = decoder.orientation().map_err(|e| e.to_string())?;
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
    image.apply_orientation(orientation);
    let edge = max_edge.max(1).min(image.width().max(image.height()));
    let resized = image.resize(edge, edge, FilterType::Lanczos3);
    let mut encoded = Vec::new();
    let mime = if resized.color().has_alpha() {
        resized.write_to(&mut Cursor::new(&mut encoded), ImageFormat::Png).map_err(|e| e.to_string())?;
        "image/png"
    } else {
        JpegEncoder::new_with_quality(&mut encoded, 82).encode_image(&resized).map_err(|e| e.to_string())?;
        "image/jpeg"
    };
    if encoded.len() >= bytes.len() && width.max(height) <= max_edge { return Ok(None); }
    Ok(Some(format!("data:{mime};base64,{}", STANDARD.encode(encoded))))
}
