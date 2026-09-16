use crate::db::{self, DbError};
use serde::Deserialize;
use std::fs;
use std::io::Read;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct OEmbed {
    title: Option<String>,
    thumbnail_url: Option<String>,
}

pub fn is_youtube(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("youtube.com/watch")
        || lower.contains("youtube.com/shorts")
        || lower.contains("youtu.be/")
}

pub fn fetch_youtube_meta(url: &str) -> Result<(Option<String>, Option<String>), DbError> {
    let endpoint = format!(
        "https://www.youtube.com/oembed?format=json&url={}",
        urlencoding(url)
    );
    let parsed: OEmbed = ureq::get(&endpoint)
        .set("User-Agent", "Bucket/0.1")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| DbError::msg(format!("YouTube lookup failed: {e}")))?
        .into_json()
        .map_err(|e| DbError::msg(format!("YouTube lookup failed: {e}")))?;
    Ok((parsed.title, parsed.thumbnail_url))
}

pub fn download_thumb(library_root: &std::path::Path, image_url: &str) -> Result<String, DbError> {
    let response = ureq::get(image_url)
        .set("User-Agent", "Bucket/0.1")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .map_err(|e| DbError::msg(format!("Thumbnail download failed: {e}")))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| DbError::msg(format!("Thumbnail download failed: {e}")))?;

    let dir = library_root.join("library/inbox");
    fs::create_dir_all(&dir)?;
    let filename = format!("{}_thumb.jpg", Uuid::new_v4());
    let abs = dir.join(&filename);
    fs::write(&abs, bytes)?;
    db::relpath_from_abs(library_root, &abs)
}

fn urlencoding(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
