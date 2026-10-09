use crate::capture::{self, is_blocked_host};
use crate::db::{self, AppState, DbError};
use crate::images;
use crate::models::{CoverHit, CoverPage, ImageRecord};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use uuid::Uuid;

const USER_AGENT: &str = "Zenith/0.2 (personal bucket list; local)";
const MAX_IMAGE_BYTES: u64 = 12 * 1024 * 1024;
const PAGE_SIZE: u32 = 20;

static SEARCH_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Deserialize)]
struct OpenversePage {
    results: Vec<OpenverseImage>,
    #[serde(default)]
    page_count: u32,
}

#[derive(Debug, Deserialize)]
struct OpenverseImage {
    id: String,
    title: Option<String>,
    url: Option<String>,
    creator: Option<String>,
    foreign_landing_url: Option<String>,
    thumbnail: Option<String>,
    mature: Option<bool>,
}

pub fn search(library_root: &Path, query: &str, page_num: u32) -> Result<CoverPage, DbError> {
    let _guard = SEARCH_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let query = query.trim();
    if query.is_empty() {
        return Err(DbError::msg("Type something to search"));
    }
    let page_num = page_num.max(1);

    let mut endpoint = url::Url::parse("https://api.openverse.org/v1/images/")
        .map_err(|_| DbError::msg("Cover search is unavailable"))?;
    endpoint
        .query_pairs_mut()
        .append_pair("q", query)
        .append_pair("page", &page_num.to_string())
        .append_pair("page_size", &PAGE_SIZE.to_string())
        .append_pair("mature", "false");

    let dir = library_root.join("library/cover-search");
    if page_num == 1 {
        let _ = fs::remove_dir_all(&dir);
    }
    fs::create_dir_all(&dir)?;

    let keys = load_sources()?;
    let openverse = source_openverse(endpoint.as_str(), page_num);
    let pexels = source_pexels(&keys.pexels, query, page_num);
    let pixabay = source_pixabay(&keys.pixabay, query, page_num);

    let mut jobs = Vec::new();
    let mut page_count = 1u32;
    let mut warnings = Vec::new();
    for outcome in [openverse, pexels, pixabay] {
        match outcome {
            SourceOutcome::Off => {}
            SourceOutcome::Ready { mut ready, pages } => {
                jobs.append(&mut ready);
                page_count = page_count.max(pages);
            }
            SourceOutcome::Failed(message) => warnings.push(message),
        }
    }
    if jobs.is_empty() && !warnings.is_empty() {
        return Err(DbError::msg(warnings.join(" ")));
    }

    let hits = download_thumbs(library_root, &dir, jobs);
    Ok(CoverPage {
        hits,
        page: page_num,
        page_count,
        warning: if warnings.is_empty() {
            None
        } else {
            Some(warnings.join(" "))
        },
    })
}

pub fn apply_stock(
    state: &AppState,
    target: &str,
    owner_id: &str,
    image_id: &str,
    focus_x: f64,
    focus_y: f64,
) -> Result<(), DbError> {
    let stock = resolve_stock(image_id)?;
    let rel = images::save_cover_jpeg(state, &stock.bytes, "cover.jpg")?;
    let old = db::assign_cover(
        &state.conn,
        target,
        owner_id,
        &rel,
        Some(&stock.credit),
        stock.credit_url.as_deref(),
        focus_x,
        focus_y,
    );
    match old {
        Ok(previous) => {
            db::retire_unreferenced_cover(state, previous.as_deref());
            Ok(())
        }
        Err(err) => {
            db::remove_relpath(state, Some(&rel));
            Err(err)
        }
    }
}

pub fn import_stock_photo(
    state: &AppState,
    place_id: &str,
    image_id: &str,
) -> Result<ImageRecord, DbError> {
    let stock = resolve_stock(image_id)?;
    let record = images::import_image_bytes(state, place_id, "photo.jpg", stock.bytes)?;
    if record
        .caption
        .as_deref()
        .map(str::trim)
        .filter(|caption| !caption.is_empty())
        .is_none()
    {
        return db::update_image_caption(&state.conn, &record.id, Some(&stock.credit));
    }
    Ok(record)
}

pub fn apply_file(
    state: &AppState,
    target: &str,
    owner_id: &str,
    filename: &str,
    bytes: &[u8],
    focus_x: f64,
    focus_y: f64,
) -> Result<(), DbError> {
    let rel = images::save_cover_jpeg(state, bytes, filename)?;
    match db::assign_cover(&state.conn, target, owner_id, &rel, None, None, focus_x, focus_y) {
        Ok(previous) => {
            db::retire_unreferenced_cover(state, previous.as_deref());
            Ok(())
        }
        Err(err) => {
            db::remove_relpath(state, Some(&rel));
            Err(err)
        }
    }
}

fn credit_line(image: &OpenverseImage) -> String {
    let creator = image
        .creator
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("Unknown");
    let line = match image.title.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(title) => format!("{title} by {creator}"),
        None => format!("Photo by {creator}"),
    };
    let mut chars = line.chars();
    let short: String = chars.by_ref().take(180).collect();
    if chars.next().is_some() {
        format!("{short}…")
    } else {
        short
    }
}

fn valid_image_url(raw: &str) -> bool {
    let Ok(parsed) = capture::require_http_url(raw) else {
        return false;
    };
    parsed.scheme() == "https"
        && parsed
            .host_str()
            .is_some_and(|host| !is_blocked_host(host))
}

fn get_text(url: &str) -> Result<String, DbError> {
    let bytes = get_bytes(url)?;
    String::from_utf8(bytes).map_err(|_| DbError::msg("Cover search returned something unexpected"))
}

fn get_bytes(url: &str) -> Result<Vec<u8>, DbError> {
    let response = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json, image/*")
        .timeout(Duration::from_secs(20))
        .call()
        .map_err(|e| DbError::msg(format!("Cover search failed: {e}")))?;
    let mut buf = Vec::new();
    response
        .into_reader()
        .take(MAX_IMAGE_BYTES)
        .read_to_end(&mut buf)
        .map_err(|e| DbError::msg(format!("Cover search failed: {e}")))?;
    if buf.is_empty() {
        return Err(DbError::msg("Cover search returned an empty file"));
    }
    Ok(buf)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhotoSources {
    #[serde(default)]
    pub pexels: String,
    #[serde(default)]
    pub pixabay: String,
}

struct ThumbJob {
    hit_id: String,
    file_name: String,
    url: String,
    credit: String,
    kind: ThumbKind,
}

#[derive(Clone, Copy)]
enum ThumbKind {
    Openverse,
    Pexels,
    Pixabay,
}

enum SourceOutcome {
    Off,
    Ready { ready: Vec<ThumbJob>, pages: u32 },
    Failed(String),
}

struct ResolvedStock {
    bytes: Vec<u8>,
    credit: String,
    credit_url: Option<String>,
}

#[derive(Deserialize)]
struct PexelsPage {
    #[serde(default)]
    photos: Vec<PexelsPhoto>,
    #[serde(default)]
    total_results: u32,
}

#[derive(Deserialize)]
struct PexelsPhoto {
    id: u64,
    alt: Option<String>,
    photographer: Option<String>,
    url: Option<String>,
    src: PexelsSrc,
}

#[derive(Deserialize)]
struct PexelsSrc {
    #[serde(default)]
    large2x: Option<String>,
    #[serde(default)]
    large: Option<String>,
    #[serde(default)]
    original: Option<String>,
    #[serde(default)]
    small: Option<String>,
    #[serde(default)]
    tiny: Option<String>,
}

#[derive(Deserialize)]
struct PixabayPage {
    #[serde(default, rename = "totalHits")]
    total_hits: u32,
    #[serde(default)]
    hits: Vec<PixabayHit>,
}

#[derive(Deserialize)]
struct PixabayHit {
    id: u64,
    #[serde(default, rename = "pageURL")]
    page_url: Option<String>,
    #[serde(default, rename = "webformatURL")]
    webformat_url: Option<String>,
    #[serde(default, rename = "largeImageURL")]
    large_image_url: Option<String>,
    #[serde(default, rename = "previewURL")]
    preview_url: Option<String>,
    user: Option<String>,
}

pub fn load_sources() -> Result<PhotoSources, DbError> {
    let Ok(path) = sources_path() else {
        return Ok(PhotoSources::default());
    };
    load_sources_at(&path)
}

pub fn save_sources(pexels: &str, pixabay: &str) -> Result<PhotoSources, DbError> {
    let saved = PhotoSources {
        pexels: clean_key(pexels)?,
        pixabay: clean_key(pixabay)?,
    };
    save_sources_at(&sources_path()?, &saved)?;
    Ok(saved)
}

fn sources_path() -> Result<PathBuf, DbError> {
    let base = dirs::data_local_dir()
        .ok_or_else(|| DbError::msg("Could not find a place to store keys"))?;
    let dir = base.join("Bucket");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("photo-sources.json"))
}

fn load_sources_at(path: &Path) -> Result<PhotoSources, DbError> {
    if !path.exists() {
        return Ok(PhotoSources::default());
    }
    let text = fs::read_to_string(path)?;
    serde_json::from_str(&text).map_err(|_| DbError::msg("Photo source keys could not be read"))
}

fn save_sources_at(path: &Path, sources: &PhotoSources) -> Result<(), DbError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = serde_json::to_string(sources)
        .map_err(|_| DbError::msg("Photo source keys could not be saved"))?;
    fs::write(path, text)?;
    Ok(())
}

fn clean_key(raw: &str) -> Result<String, DbError> {
    let key = raw.trim();
    if key.is_empty() {
        return Ok(String::new());
    }
    if key.len() > 128 || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(DbError::msg("That key doesn't look right."));
    }
    Ok(key.to_string())
}

fn source_openverse(url: &str, page_num: u32) -> SourceOutcome {
    let body = match get_text(url) {
        Ok(body) => body,
        Err(err) => return SourceOutcome::Failed(err.to_string()),
    };
    let page: OpenversePage = match serde_json::from_str(&body) {
        Ok(page) => page,
        Err(_) => {
            return SourceOutcome::Failed("Cover search returned something unexpected".into());
        }
    };
    let mut ready = Vec::new();
    for image in page.results {
        if image.mature == Some(true) {
            continue;
        }
        let Some(thumb) = image.thumbnail.as_deref().filter(|value| valid_image_url(value)) else {
            continue;
        };
        let Ok(parsed) = Uuid::parse_str(&image.id) else {
            continue;
        };
        ready.push(ThumbJob {
            hit_id: format!("openverse:{parsed}"),
            file_name: format!("{parsed}.jpg"),
            url: thumb.to_string(),
            credit: credit_line(&image),
            kind: ThumbKind::Openverse,
        });
    }
    let pages = if page.page_count > 0 {
        page.page_count
    } else if ready.len() as u32 == PAGE_SIZE {
        page_num + 1
    } else {
        page_num
    };
    SourceOutcome::Ready { ready, pages }
}

fn source_pexels(key: &str, query: &str, page_num: u32) -> SourceOutcome {
    if key.is_empty() {
        return SourceOutcome::Off;
    }
    let mut endpoint = match url::Url::parse("https://api.pexels.com/v1/search") {
        Ok(endpoint) => endpoint,
        Err(_) => return SourceOutcome::Failed("Pexels search is unavailable".into()),
    };
    endpoint
        .query_pairs_mut()
        .append_pair("query", &clip_query(query))
        .append_pair("page", &page_num.to_string())
        .append_pair("per_page", &PAGE_SIZE.to_string());
    let body = match api_text(endpoint.as_str(), Some(key), key, "Pexels") {
        Ok(body) => body,
        Err(message) => return SourceOutcome::Failed(message),
    };
    let page: PexelsPage = match serde_json::from_str(&body) {
        Ok(page) => page,
        Err(_) => return SourceOutcome::Failed("Pexels returned something unexpected".into()),
    };
    let mut ready = Vec::new();
    for photo in page.photos {
        let Some(thumb) = photo
            .src
            .small
            .as_deref()
            .or(photo.src.tiny.as_deref())
            .filter(|value| pexels_image_host_str(value))
        else {
            continue;
        };
        ready.push(ThumbJob {
            hit_id: format!("pexels:{}", photo.id),
            file_name: format!("pexels-{}.jpg", photo.id),
            url: thumb.to_string(),
            credit: pexels_credit(&photo),
            kind: ThumbKind::Pexels,
        });
    }
    SourceOutcome::Ready {
        ready,
        pages: page_count_for(page.total_results, page_num, PAGE_SIZE),
    }
}

fn source_pixabay(key: &str, query: &str, page_num: u32) -> SourceOutcome {
    if key.is_empty() {
        return SourceOutcome::Off;
    }
    let endpoint = match pixabay_endpoint(key, &[
        ("q", clip_query(query).as_str()),
        ("image_type", "photo"),
        ("safesearch", "true"),
        ("page", &page_num.to_string()),
        ("per_page", &PAGE_SIZE.to_string()),
    ]) {
        Ok(endpoint) => endpoint,
        Err(_) => return SourceOutcome::Failed("Pixabay search is unavailable".into()),
    };
    let body = match api_text(endpoint.as_str(), None, key, "Pixabay") {
        Ok(body) => body,
        Err(message) => return SourceOutcome::Failed(message),
    };
    let page: PixabayPage = match serde_json::from_str(&body) {
        Ok(page) => page,
        Err(_) => return SourceOutcome::Failed("Pixabay returned something unexpected".into()),
    };
    let mut ready = Vec::new();
    for hit in page.hits {
        let Some(thumb) = hit
            .webformat_url
            .as_deref()
            .or(hit.preview_url.as_deref())
            .filter(|value| pixabay_image_host_str(value))
        else {
            continue;
        };
        ready.push(ThumbJob {
            hit_id: format!("pixabay:{}", hit.id),
            file_name: format!("pixabay-{}.jpg", hit.id),
            url: thumb.to_string(),
            credit: pixabay_credit(&hit),
            kind: ThumbKind::Pixabay,
        });
    }
    let pages = page_count_for(page.total_hits.min(500), page_num, PAGE_SIZE);
    SourceOutcome::Ready { ready, pages }
}

fn resolve_stock(image_id: &str) -> Result<ResolvedStock, DbError> {
    let (source, id) = split_stock_id(image_id);
    match source {
        "pexels" => resolve_pexels(id),
        "pixabay" => resolve_pixabay(id),
        _ => resolve_openverse(id),
    }
}

fn resolve_openverse(id: &str) -> Result<ResolvedStock, DbError> {
    let parsed = Uuid::parse_str(id).map_err(|_| DbError::msg("Unknown picture"))?;
    let detail_url = format!("https://api.openverse.org/v1/images/{parsed}/");
    let body = get_text(&detail_url)?;
    let image: OpenverseImage = serde_json::from_str(&body)
        .map_err(|_| DbError::msg("That picture is no longer available"))?;
    let url = image
        .url
        .as_deref()
        .filter(|value| valid_image_url(value))
        .ok_or_else(|| DbError::msg("That picture has no file to save"))?;
    Ok(ResolvedStock {
        bytes: get_bytes(url)?,
        credit: credit_line(&image),
        credit_url: image.foreign_landing_url.filter(|value| valid_image_url(value)),
    })
}

fn resolve_pexels(id: &str) -> Result<ResolvedStock, DbError> {
    let numeric = numeric_id(id)?;
    let key = load_sources()?.pexels;
    if key.is_empty() {
        return Err(DbError::msg("Add a Pexels key from the menu."));
    }
    let url = format!("https://api.pexels.com/v1/photos/{numeric}");
    let body = api_text(&url, Some(&key), &key, "Pexels").map_err(DbError::msg)?;
    let photo: PexelsPhoto = serde_json::from_str(&body)
        .map_err(|_| DbError::msg("That picture is no longer available"))?;
    let file_url = photo
        .src
        .large2x
        .as_deref()
        .or(photo.src.large.as_deref())
        .or(photo.src.original.as_deref())
        .filter(|value| pexels_image_host_str(value))
        .ok_or_else(|| DbError::msg("That picture has no file to save"))?;
    Ok(ResolvedStock {
        bytes: get_checked(file_url, pexels_image_host)?,
        credit: pexels_credit(&photo),
        credit_url: photo.url.filter(|value| {
            value.starts_with("https://www.pexels.com/") || value.starts_with("https://pexels.com/")
        }),
    })
}

fn resolve_pixabay(id: &str) -> Result<ResolvedStock, DbError> {
    let numeric = numeric_id(id)?;
    let key = load_sources()?.pixabay;
    if key.is_empty() {
        return Err(DbError::msg("Add a Pixabay key from the menu."));
    }
    let id_string = numeric.to_string();
    let endpoint = pixabay_endpoint(&key, &[("id", id_string.as_str()), ("image_type", "photo")])?;
    let body = api_text(endpoint.as_str(), None, &key, "Pixabay").map_err(DbError::msg)?;
    let page: PixabayPage = serde_json::from_str(&body)
        .map_err(|_| DbError::msg("That picture is no longer available"))?;
    let hit = page
        .hits
        .into_iter()
        .next()
        .ok_or_else(|| DbError::msg("That picture is no longer available"))?;
    let file_url = hit
        .large_image_url
        .as_deref()
        .filter(|value| pixabay_image_host_str(value))
        .ok_or_else(|| DbError::msg("That picture has no file to save"))?;
    Ok(ResolvedStock {
        bytes: get_checked(file_url, pixabay_image_host)?,
        credit: pixabay_credit(&hit),
        credit_url: hit.page_url.filter(|value| value.starts_with("https://pixabay.com/")),
    })
}

fn download_thumbs(library_root: &Path, dir: &Path, jobs: Vec<ThumbJob>) -> Vec<CoverHit> {
    let mut slots: Vec<Option<CoverHit>> = (0..jobs.len()).map(|_| None).collect();
    let mut start = 0;
    while start < jobs.len() {
        let end = (start + 8).min(jobs.len());
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for (offset, job) in jobs[start..end].iter().enumerate() {
                handles.push(scope.spawn(move || (start + offset, save_thumb(library_root, dir, job))));
            }
            for handle in handles {
                if let Ok((index, Some(hit))) = handle.join() {
                    slots[index] = Some(hit);
                }
            }
        });
        start = end;
    }
    slots.into_iter().flatten().collect()
}

fn save_thumb(library_root: &Path, dir: &Path, job: &ThumbJob) -> Option<CoverHit> {
    let bytes = match job.kind {
        ThumbKind::Openverse => get_bytes(&job.url).ok()?,
        ThumbKind::Pexels => get_checked(&job.url, pexels_image_host).ok()?,
        ThumbKind::Pixabay => get_checked(&job.url, pixabay_image_host).ok()?,
    };
    let abs = dir.join(&job.file_name);
    images::write_cover_thumb(&abs, &bytes).ok()?;
    let rel = db::relpath_from_abs(library_root, &abs).ok()?;
    Some(CoverHit {
        id: job.hit_id.clone(),
        thumb_relpath: rel,
        credit: job.credit.clone(),
    })
}

fn api_text(url: &str, auth: Option<&str>, secret: &str, name: &str) -> Result<String, String> {
    let agent = quiet_agent();
    let mut request = agent.get(url).set("User-Agent", USER_AGENT).set("Accept", "application/json");
    if let Some(auth) = auth {
        request = request.set("Authorization", auth);
    }
    match request.call() {
        Ok(response) => read_text(response, secret, name),
        Err(ureq::Error::Status(code, response)) if code == 401 || code == 403 => {
            let _ = response.into_reader();
            Err(format!("{name} rejected that key."))
        }
        Err(err) => Err(redact(&format!("{name} search failed: {err}"), secret)),
    }
}

fn read_text(response: ureq::Response, secret: &str, name: &str) -> Result<String, String> {
    let mut buf = Vec::new();
    response
        .into_reader()
        .take(MAX_IMAGE_BYTES)
        .read_to_end(&mut buf)
        .map_err(|err| redact(&format!("{name} search failed: {err}"), secret))?;
    String::from_utf8(buf).map_err(|_| format!("{name} returned something unexpected"))
}

fn get_checked(url: &str, allow: fn(&url::Url) -> bool) -> Result<Vec<u8>, DbError> {
    let agent = quiet_agent();
    let mut current = url.to_string();
    for _ in 0..4 {
        let parsed = url::Url::parse(&current).map_err(|_| DbError::msg("That picture has no file to save"))?;
        if !allow(&parsed) {
            return Err(DbError::msg("That picture has no file to save"));
        }
        match agent
            .get(parsed.as_str())
            .set("User-Agent", USER_AGENT)
            .set("Accept", "image/*")
            .call()
        {
            Ok(response) => {
                let mut buf = Vec::new();
                response
                    .into_reader()
                    .take(MAX_IMAGE_BYTES)
                    .read_to_end(&mut buf)
                    .map_err(|err| DbError::msg(format!("Cover search failed: {err}")))?;
                if buf.is_empty() {
                    return Err(DbError::msg("Cover search returned an empty file"));
                }
                return Ok(buf);
            }
            Err(ureq::Error::Status(code, response))
                if matches!(code, 301 | 302 | 303 | 307 | 308) =>
            {
                let location = response
                    .header("location")
                    .ok_or_else(|| DbError::msg("That picture has no file to save"))?
                    .to_string();
                current = parsed
                    .join(&location)
                    .map_err(|_| DbError::msg("That picture has no file to save"))?
                    .to_string();
            }
            Err(err) => return Err(DbError::msg(format!("Cover search failed: {err}"))),
        }
    }
    Err(DbError::msg("That picture has no file to save"))
}

fn quiet_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .redirects(0)
        .build()
}

fn pixabay_endpoint(key: &str, pairs: &[(&str, &str)]) -> Result<url::Url, DbError> {
    let mut endpoint = url::Url::parse("https://pixabay.com/api/")
        .map_err(|_| DbError::msg("Pixabay search is unavailable"))?;
    {
        let mut query = endpoint.query_pairs_mut();
        query.append_pair("key", key);
        for (name, value) in pairs {
            query.append_pair(name, value);
        }
    }
    Ok(endpoint)
}

fn split_stock_id(raw: &str) -> (&str, &str) {
    if let Some((source, id)) = raw.split_once(':') {
        if matches!(source, "openverse" | "pexels" | "pixabay") && !id.is_empty() {
            return (source, id);
        }
    }
    ("openverse", raw)
}

fn numeric_id(id: &str) -> Result<u64, DbError> {
    if id.is_empty() || id.len() > 20 || !id.chars().all(|c| c.is_ascii_digit()) {
        return Err(DbError::msg("Unknown picture"));
    }
    id.parse().map_err(|_| DbError::msg("Unknown picture"))
}

fn clip_query(query: &str) -> String {
    query.chars().take(100).collect()
}

fn page_count_for(total: u32, page_num: u32, page_size: u32) -> u32 {
    if total == 0 {
        return page_num;
    }
    let pages = (total + page_size - 1) / page_size;
    pages.clamp(1, 40).max(page_num)
}

fn pexels_credit(photo: &PexelsPhoto) -> String {
    let name = photo
        .photographer
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Unknown");
    match photo.alt.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        Some(alt) => shorten(format!("{alt} by {name} on Pexels")),
        None => shorten(format!("Photo by {name} on Pexels")),
    }
}

fn pixabay_credit(hit: &PixabayHit) -> String {
    let name = hit
        .user
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Unknown");
    shorten(format!("Photo by {name} on Pixabay"))
}

fn shorten(line: String) -> String {
    let mut chars = line.chars();
    let short: String = chars.by_ref().take(180).collect();
    if chars.next().is_some() {
        format!("{short}…")
    } else {
        short
    }
}

fn pexels_image_host(url: &url::Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("images.pexels.com")
}

fn pixabay_image_host(url: &url::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    match url.host_str() {
        Some("cdn.pixabay.com") => true,
        Some("pixabay.com") => url.path().starts_with("/get/"),
        _ => false,
    }
}

fn pexels_image_host_str(raw: &str) -> bool {
    url::Url::parse(raw).is_ok_and(|url| pexels_image_host(&url))
}

fn pixabay_image_host_str(raw: &str) -> bool {
    url::Url::parse(raw).is_ok_and(|url| pixabay_image_host(&url))
}

fn redact(message: &str, secret: &str) -> String {
    if secret.is_empty() {
        return message.to_string();
    }
    message.replace(secret, "…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_uses_creator() {
        let image = OpenverseImage {
            id: "9e8e564e-a6af-4d6b-aa42-acde86130107".into(),
            title: Some("NYC marathon".into()),
            url: None,
            creator: Some("Ed Yourdon".into()),
            foreign_landing_url: None,
            thumbnail: None,
            mature: Some(false),
        };
        assert_eq!(credit_line(&image), "NYC marathon by Ed Yourdon");
    }

    #[test]
    fn sources_roundtrip_and_reject_bad_key() {
        let dir = std::env::temp_dir().join(format!("bucket-sources-{}", Uuid::new_v4()));
        let path = dir.join("photo-sources.json");
        let saved = PhotoSources {
            pexels: clean_key("pexelskey123").unwrap(),
            pixabay: clean_key("99887766").unwrap(),
        };
        save_sources_at(&path, &saved).unwrap();
        let loaded = load_sources_at(&path).unwrap();
        assert_eq!(loaded.pexels, "pexelskey123");
        assert_eq!(loaded.pixabay, "99887766");
        assert!(clean_key("has space").is_err());
        assert_eq!(clean_key("  ").unwrap(), "");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stock_hosts_are_narrow() {
        assert!(pexels_image_host(
            &url::Url::parse("https://images.pexels.com/photos/1.jpeg").unwrap()
        ));
        assert!(!pexels_image_host(
            &url::Url::parse("https://evil.example/photos/1.jpeg").unwrap()
        ));
        assert!(pixabay_image_host(
            &url::Url::parse("https://cdn.pixabay.com/photo/a.jpg").unwrap()
        ));
        assert!(pixabay_image_host(
            &url::Url::parse("https://pixabay.com/get/abc_1280.jpg").unwrap()
        ));
        assert!(!pixabay_image_host(
            &url::Url::parse("https://pixabay.com/api/?key=1").unwrap()
        ));
        assert_eq!(
            redact("https://pixabay.com/api/?key=SECRET&q=a", "SECRET"),
            "https://pixabay.com/api/?key=…&q=a"
        );
        assert_eq!(split_stock_id("pexels:42"), ("pexels", "42"));
        assert!(numeric_id("42").is_ok());
        assert!(numeric_id("nope").is_err());
    }
}
