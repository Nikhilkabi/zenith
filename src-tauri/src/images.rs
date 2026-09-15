use crate::db::{self, AppState, DbError};
use crate::models::ImageRecord;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::path::Path;
use uuid::Uuid;

const DISPLAY_LONG_EDGE: u32 = 2000;
const THUMB_LONG_EDGE: u32 = 600;

fn is_heic(filename: &str, bytes: &[u8]) -> bool {
    let lower = filename.to_lowercase();
    if lower.ends_with(".heic") || lower.ends_with(".heif") {
        return true;
    }
    (bytes.len() >= 12
        && &bytes[4..8] == b"ftyp"
        && (bytes[8..12] == *b"heic" || bytes[8..12] == *b"heix" || bytes[8..12] == *b"mif1"))
}

fn detect_format(filename: &str, bytes: &[u8]) -> Result<ImageFormat, DbError> {
    if is_heic(filename, bytes) {
        return Err(DbError::msg(
            "HEIC/HEIF images are not supported. Please convert to JPEG, PNG, or WebP first.",
        ));
    }

    image::guess_format(bytes).or_else(|_| {
        let lower = filename.to_lowercase();
        if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            Ok(ImageFormat::Jpeg)
        } else if lower.ends_with(".png") {
            Ok(ImageFormat::Png)
        } else if lower.ends_with(".webp") {
            Ok(ImageFormat::WebP)
        } else {
            Err(DbError::msg(
                "Unsupported image format. Use JPEG, PNG, or WebP.",
            ))
        }
    })
}

fn extension_for(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::WebP => "webp",
        _ => "jpg",
    }
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

    let format = detect_format(filename, &bytes)?;
    let ext = extension_for(format);

    let img = image::load_from_memory(&bytes).map_err(|e| DbError::msg(format!("Invalid image: {e}")))?;

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
    )
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
