use crate::db::DbError;
use serde::Deserialize;
use std::collections::HashSet;
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

const USER_AGENT: &str = "Zenith/0.2 (personal bucket list; local)";
const MAX_JSON_BYTES: u64 = 64 * 1024;
const MAX_THUMB_BYTES: u64 = 2 * 1024 * 1024;

static YOUTUBE_MISS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

#[derive(Debug)]
pub struct YoutubePreview {
    pub title: String,
    pub thumb: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
struct OEmbed {
    title: Option<String>,
    thumbnail_url: Option<String>,
}

pub fn require_http_url(raw: &str) -> Result<url::Url, DbError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(DbError::msg("Paste a URL first."));
    }

    let parsed = url::Url::parse(trimmed)
        .or_else(|_| {
            if trimmed.contains(' ') || trimmed.contains(':') {
                Err(url::ParseError::EmptyHost)
            } else {
                url::Url::parse(&format!("https://{trimmed}"))
            }
        })
        .map_err(|_| DbError::msg("Not a valid URL"))?;

    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        _ => Err(DbError::msg("Only http and https URLs are allowed")),
    }
}

pub fn is_youtube(url: &str) -> bool {
    let Ok(parsed) = require_http_url(url) else {
        return false;
    };
    matches!(
        parsed.host_str(),
        Some("youtube.com" | "www.youtube.com" | "m.youtube.com" | "youtu.be" | "www.youtu.be")
    )
}

pub fn youtube_preview(video_url: &str) -> Result<YoutubePreview, DbError> {
    let key = video_url.trim().to_string();
    if youtube_missed(&key) {
        return Err(DbError::msg("YouTube did not respond"));
    }
    match fetch_youtube_preview(&key) {
        Ok(preview) => Ok(preview),
        Err(err) => {
            remember_youtube_miss(&key);
            Err(err)
        }
    }
}

fn fetch_youtube_preview(video_url: &str) -> Result<YoutubePreview, DbError> {
    if !is_youtube(video_url) {
        return Err(DbError::msg("Not a YouTube link"));
    }
    let endpoint = youtube_oembed_url(video_url)?;
    let body = fetch_limited(endpoint.as_str(), youtube_oembed_host, "application/json", MAX_JSON_BYTES)?;
    let text = String::from_utf8(body).map_err(|_| DbError::msg("YouTube returned something unexpected"))?;
    let (title, thumb_url) = preview_parts(&text)?;
    let thumb = thumb_url.and_then(|url| fetch_limited(&url, youtube_thumb_host, "image/*", MAX_THUMB_BYTES).ok());
    Ok(YoutubePreview { title, thumb })
}

fn youtube_oembed_url(video_url: &str) -> Result<url::Url, DbError> {
    let mut endpoint = url::Url::parse("https://www.youtube.com/oembed")
        .map_err(|_| DbError::msg("YouTube is unavailable"))?;
    endpoint
        .query_pairs_mut()
        .append_pair("url", video_url)
        .append_pair("format", "json");
    Ok(endpoint)
}

fn preview_parts(body: &str) -> Result<(String, Option<String>), DbError> {
    let parsed: OEmbed = serde_json::from_str(body)
        .map_err(|_| DbError::msg("YouTube returned something unexpected"))?;
    let title = clip_title(parsed.title.as_deref().unwrap_or(""));
    if title.is_empty() {
        return Err(DbError::msg("YouTube did not name that video"));
    }
    let thumb = parsed.thumbnail_url.filter(|raw| {
        url::Url::parse(raw).is_ok_and(|url| youtube_thumb_host(&url))
    });
    Ok((title, thumb))
}

fn clip_title(raw: &str) -> String {
    raw.trim().chars().take(180).collect()
}

fn youtube_oembed_host(url: &url::Url) -> bool {
    url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(url.host_str(), Some("www.youtube.com" | "youtube.com"))
        && url.path() == "/oembed"
}

fn youtube_thumb_host(url: &url::Url) -> bool {
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    host == "i.ytimg.com" || host == "img.youtube.com" || host.ends_with(".ytimg.com")
}

fn youtube_misses() -> std::sync::MutexGuard<'static, HashSet<String>> {
    YOUTUBE_MISS
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap_or_else(|err| err.into_inner())
}

fn youtube_missed(url: &str) -> bool {
    youtube_misses().contains(url)
}

fn remember_youtube_miss(url: &str) {
    youtube_misses().insert(url.to_string());
}

fn fetch_limited(
    url: &str,
    allow: fn(&url::Url) -> bool,
    accept: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, DbError> {
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(4))
        .timeout(Duration::from_secs(8))
        .redirects(0)
        .build();
    let mut current = url.to_string();
    for _ in 0..4 {
        let parsed = url::Url::parse(&current).map_err(|_| DbError::msg("YouTube did not respond"))?;
        if parsed.scheme() != "https" || is_blocked_host(parsed.host_str().unwrap_or("")) || !allow(&parsed) {
            return Err(DbError::msg("YouTube did not respond"));
        }
        match agent
            .get(parsed.as_str())
            .set("User-Agent", USER_AGENT)
            .set("Accept", accept)
            .call()
        {
            Ok(response) => {
                let mut buf = Vec::new();
                response
                    .into_reader()
                    .take(max_bytes)
                    .read_to_end(&mut buf)
                    .map_err(|_| DbError::msg("YouTube did not respond"))?;
                if buf.is_empty() {
                    return Err(DbError::msg("YouTube did not respond"));
                }
                return Ok(buf);
            }
            Err(ureq::Error::Status(code, response)) if matches!(code, 301 | 302 | 303 | 307 | 308) => {
                let location = response
                    .header("location")
                    .ok_or_else(|| DbError::msg("YouTube did not respond"))?
                    .to_string();
                current = parsed
                    .join(&location)
                    .map_err(|_| DbError::msg("YouTube did not respond"))?
                    .to_string();
            }
            Err(_) => return Err(DbError::msg("YouTube did not respond")),
        }
    }
    Err(DbError::msg("YouTube did not respond"))
}

pub(crate) fn is_blocked_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".local") {
        return true;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
            std::net::IpAddr::V6(v) => v.is_loopback() || (v.segments()[0] & 0xfe00) == 0xfc00,
        };
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_dangerous_schemes() {
        assert!(require_http_url("javascript:alert(1)").is_err());
        assert!(require_http_url("file:///C:/Windows/win.ini").is_err());
        assert!(require_http_url("data:text/html,hi").is_err());
        assert!(require_http_url("https://example.com/a").is_ok());
        assert!(require_http_url("http://example.com").is_ok());
        assert!(require_http_url("www.example.com/x").is_ok());
    }

    #[test]
    fn youtube_matches_host_only() {
        assert!(is_youtube("https://www.youtube.com/watch?v=abc"));
        assert!(is_youtube("https://youtu.be/abc"));
        assert!(!is_youtube("https://evil.example/?u=youtube.com/watch"));
        assert!(!is_youtube("javascript:youtube.com/watch"));
    }

    #[test]
    fn oembed_stays_on_youtube_and_keeps_the_title() {
        let endpoint = youtube_oembed_url("https://youtu.be/9L4UdnAfRJc").unwrap();
        assert!(youtube_oembed_host(&endpoint));
        assert_eq!(endpoint.path(), "/oembed");
        assert!(endpoint.query().unwrap().contains("youtu.be"));

        let (title, thumb) = preview_parts(
            r#"{"title":" Guide to Tumpak Sewu ","thumbnail_url":"https://i.ytimg.com/vi/abc/hqdefault.jpg","html":"<iframe src=\"https://evil.example\"></iframe>"}"#,
        )
        .unwrap();
        assert_eq!(title, "Guide to Tumpak Sewu");
        assert_eq!(thumb.as_deref(), Some("https://i.ytimg.com/vi/abc/hqdefault.jpg"));

        let (title, thumb) = preview_parts(
            r#"{"title":"Named","thumbnail_url":"https://evil.example/hqdefault.jpg"}"#,
        )
        .unwrap();
        assert_eq!(title, "Named");
        assert!(thumb.is_none());
        assert!(preview_parts(r#"{"title":"  ","thumbnail_url":"https://i.ytimg.com/x.jpg"}"#).is_err());
    }

    #[test]
    fn thumbnail_host_is_youtube_only() {
        assert!(youtube_thumb_host(&url::Url::parse("https://i.ytimg.com/vi/abc/hqdefault.jpg").unwrap()));
        assert!(youtube_thumb_host(&url::Url::parse("https://img.youtube.com/vi/abc/0.jpg").unwrap()));
        assert!(!youtube_thumb_host(&url::Url::parse("http://i.ytimg.com/vi/abc/hqdefault.jpg").unwrap()));
        assert!(!youtube_thumb_host(&url::Url::parse("https://i.ytimg.com.evil.com/hqdefault.jpg").unwrap()));
        assert!(!youtube_thumb_host(&url::Url::parse("https://evil.example/i.ytimg.com/hqdefault.jpg").unwrap()));
        assert!(!youtube_oembed_host(&url::Url::parse("https://www.youtube.com/watch?v=abc").unwrap()));
        assert!(!youtube_oembed_host(&url::Url::parse("https://evil.example/oembed").unwrap()));
    }

    #[test]
    fn blocks_private_hosts() {
        assert!(is_blocked_host("localhost"));
        assert!(is_blocked_host("127.0.0.1"));
        assert!(is_blocked_host("10.0.0.4"));
        assert!(is_blocked_host("192.168.1.9"));
        assert!(is_blocked_host("fc00::1"));
        assert!(!is_blocked_host("example.com"));
    }
}
