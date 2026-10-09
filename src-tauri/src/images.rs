use crate::db::{self, AppState, DbError};
use crate::models::ImageRecord;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat, ImageReader};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DISPLAY_LONG_EDGE: u32 = 2000;
const THUMB_LONG_EDGE: u32 = 600;

fn is_heic(filename: &str, bytes: &[u8]) -> bool {
    let lower = filename.to_lowercase();
    if lower.ends_with(".heic") || lower.ends_with(".heif") {
        return true;
    }
    bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && (bytes[8..12] == *b"heic" || bytes[8..12] == *b"heix" || bytes[8..12] == *b"mif1")
}

fn is_avif(filename: &str, bytes: &[u8]) -> bool {
    let lower = filename.to_lowercase();
    if lower.ends_with(".avif") {
        return true;
    }
    if bytes.len() < 12 {
        return false;
    }
    if &bytes[4..8] != b"ftyp" {
        return false;
    }
    let brands = &bytes[8..bytes.len().min(64)];
    brands.windows(4).any(|w| w == b"avif" || w == b"avis")
}

fn detect_format(filename: &str, bytes: &[u8]) -> Result<ImageFormat, DbError> {
    // Prefer magic bytes over the file name — Chrome often saves AVIF as .jpg.
    // Check AVIF before HEIC: both can use ISO-BMFF `ftyp` / `mif1`.
    if is_avif(filename, bytes) {
        return Err(DbError::msg(
            "AVIF needs conversion in the app UI. Drop or paste the file again (or use Add photos).",
        ));
    }

    if is_heic(filename, bytes) {
        return Err(DbError::msg(
            "HEIC/HEIF images are not supported. Please convert to JPEG, PNG, or WebP first.",
        ));
    }

    if let Ok(format) = image::guess_format(bytes) {
        if matches!(format, ImageFormat::Avif) {
            return Err(DbError::msg(
                "AVIF needs conversion in the app UI. Drop or paste the file again (or use Add photos).",
            ));
        }
        return Ok(format);
    }

    let lower = filename.to_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Ok(ImageFormat::Jpeg)
    } else if lower.ends_with(".png") {
        Ok(ImageFormat::Png)
    } else if lower.ends_with(".webp") {
        Ok(ImageFormat::WebP)
    } else {
        Err(DbError::msg(
            "Unsupported image format. Use JPEG, PNG, WebP — or paste from the browser (Ctrl+V).",
        ))
    }
}

fn extension_for(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        _ => "jpg",
    }
}

fn load_oriented(bytes: &[u8]) -> Result<DynamicImage, DbError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| DbError::msg(format!("Invalid image: {e}")))?;
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| DbError::msg(format!("Invalid image: {e}")))?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder)
        .map_err(|e| DbError::msg(format!("Invalid image: {e}")))?;
    img.apply_orientation(orientation);
    Ok(img)
}

fn resize_long_edge(img: &DynamicImage, max_long: u32) -> DynamicImage {
    let (w, h) = img.dimensions();
    let long = w.max(h);
    if long <= max_long {
        return img.clone();
    }
    let scale = max_long as f32 / long as f32;
    let new_w = (w as f32 * scale).round().max(1.0) as u32;
    let new_h = (h as f32 * scale).round().max(1.0) as u32;
    img.resize(new_w, new_h, FilterType::Lanczos3)
}

fn save_image(img: &DynamicImage, path: &Path, format: ImageFormat) -> Result<(), DbError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match format {
        ImageFormat::Png => img.save_with_format(path, ImageFormat::Png)?,
        ImageFormat::WebP => img.save_with_format(path, ImageFormat::WebP)?,
        _ => {
            let rgb = img.to_rgb8();
            let mut file = fs::File::create(path)?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut file, 85);
            encoder.encode(
                rgb.as_raw(),
                rgb.width(),
                rgb.height(),
                image::ExtendedColorType::Rgb8,
            )?;
        }
    }
    Ok(())
}

pub fn import_image_bytes(
    state: &AppState,
    place_id: &str,
    filename: &str,
    bytes: Vec<u8>,
) -> Result<ImageRecord, DbError> {
    db::get_place(&state.conn, place_id)?;

    let sha = format!("{:x}", Sha256::digest(&bytes));
    if let Some(existing) = db::find_image_by_hash(&state.conn, place_id, &sha)? {
        return Ok(existing);
    }

    let taken_at = exif_taken_at(&bytes);

    let format = detect_format(filename, &bytes)?;
    let ext = extension_for(format);

    let img = load_oriented(&bytes)?;

    let image_id = Uuid::new_v4().to_string();
    let rel_dir = format!("library/images/{place_id}");
    let abs_dir = state.library_root.join(&rel_dir);
    fs::create_dir_all(&abs_dir)?;

    let original_name = format!("{image_id}_original.{ext}");
    let display_name = format!("{image_id}_display.{ext}");
    let thumb_name = format!("{image_id}_thumb.jpg");

    let original_abs = abs_dir.join(&original_name);
    let display_abs = abs_dir.join(&display_name);
    let thumb_abs = abs_dir.join(&thumb_name);

    fs::write(&original_abs, &bytes)?;

    let display_img = resize_long_edge(&img, DISPLAY_LONG_EDGE);
    let thumb_img = resize_long_edge(&img, THUMB_LONG_EDGE);

    save_image(&display_img, &display_abs, format)?;
    save_image(&thumb_img, &thumb_abs, ImageFormat::Jpeg)?;

    let original_relpath = db::relpath_from_abs(&state.library_root, &original_abs)?;
    let display_relpath = db::relpath_from_abs(&state.library_root, &display_abs)?;
    let thumb_relpath = db::relpath_from_abs(&state.library_root, &thumb_abs)?;

    db::insert_image(
        &state.conn,
        place_id,
        &original_relpath,
        &display_relpath,
        &thumb_relpath,
        Some(&sha),
        None,
        taken_at.as_deref(),
    )
}

fn exif_taken_at(bytes: &[u8]) -> Option<String> {
    let mut reader = Cursor::new(bytes);
    let exif = exif::Reader::new().read_from_container(&mut reader).ok()?;
    let field = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY)?;
    Some(field.display_value().to_string())
}

pub fn import_image_path(
    state: &AppState,
    place_id: &str,
    source_path: &str,
) -> Result<ImageRecord, DbError> {
    let path = PathBuf::from(source_path);
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("photo.jpg")
        .to_string();
    let bytes = fs::read(&path)?;
    import_image_bytes(state, place_id, &filename, bytes)
}

pub fn write_cover_thumb(path: &Path, bytes: &[u8]) -> Result<(), DbError> {
    let img = load_oriented(bytes)?;
    let thumb = resize_long_edge(&img, 480);
    save_image(&thumb, path, ImageFormat::Jpeg)
}

pub fn save_cover_jpeg(state: &AppState, bytes: &[u8], filename: &str) -> Result<String, DbError> {
    detect_format(filename, bytes)?;
    let img = load_oriented(bytes)?;
    let cover = resize_long_edge(&img, 1600);
    let dir = state.library_root.join("library/covers");
    fs::create_dir_all(&dir)?;
    let abs = dir.join(format!("{}.jpg", Uuid::new_v4()));
    save_image(&cover, &abs, ImageFormat::Jpeg)?;
    db::relpath_from_abs(&state.library_root, &abs)
}

