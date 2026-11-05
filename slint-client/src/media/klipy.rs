//! Klipy GIF API (port of lib/klipy.ts).
//!
//! `GET https://api.klipy.com/api/v1/{KEY}/gifs/{trending|search}?per_page=24&page=N[&q=…]`
//! The key comes from `KLIPY_API_KEY` at build time (build.rs also reads
//! `slint-client/.env`), or from `KLIPY_API_KEY` / `VITE_KLIPY_API_KEY` at runtime.

use serde_json::Value;

const BASE: &str = "https://api.klipy.com/api/v1";
pub const PER_PAGE: u32 = 24;

pub fn api_key() -> Option<String> {
    std::env::var("KLIPY_API_KEY")
        .or_else(|_| std::env::var("VITE_KLIPY_API_KEY"))
        .ok()
        .or_else(|| option_env!("KLIPY_API_KEY").map(str::to_string))
        .filter(|k| !k.trim().is_empty())
}

#[derive(Debug, Clone)]
pub struct Gif {
    pub id: String,
    pub preview: String,
    pub gif: String,
    pub mp4: Option<String>,
}

pub struct Page {
    pub items: Vec<Gif>,
    pub has_next: bool,
}

fn url_at(file: &Value, size: &str, fmt: &str) -> Option<String> {
    file.get(size)?
        .get(fmt)?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

fn normalize(item: &Value) -> Option<Gif> {
    let file = item.get("file")?;
    let preview = url_at(file, "sm", "webp")
        .or_else(|| url_at(file, "sm", "gif"))
        .or_else(|| url_at(file, "md", "webp"))
        .or_else(|| url_at(file, "md", "gif"))
        .or_else(|| url_at(file, "xs", "gif"))?;
    let gif = url_at(file, "md", "gif")
        .or_else(|| url_at(file, "hd", "gif"))
        .or_else(|| url_at(file, "sm", "gif"))
        .unwrap_or_else(|| preview.clone());
    let mp4 = url_at(file, "md", "mp4")
        .or_else(|| url_at(file, "hd", "mp4"))
        .or_else(|| url_at(file, "sm", "mp4"));
    let id = match item.get("id") {
        Some(Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
        None => preview.clone(),
    };
    Some(Gif {
        id,
        preview,
        gif,
        mp4,
    })
}

fn encode_query(q: &str) -> String {
    q.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Trending when `query` is blank, else search. Blocking (call off the UI thread).
pub fn fetch(key: &str, query: &str, page: u32) -> anyhow::Result<Page> {
    let q = query.trim();
    let url = if q.is_empty() {
        format!("{BASE}/{key}/gifs/trending?per_page={PER_PAGE}&page={page}")
    } else {
        format!(
            "{BASE}/{key}/gifs/search?per_page={PER_PAGE}&q={}&page={page}",
            encode_query(q)
        )
    };
    let mut resp = ureq::get(&url)
        .call()
        .map_err(|e| anyhow::anyhow!("KLIPY request failed ({e})"))?;
    let body: Value = serde_json::from_reader(resp.body_mut().as_reader())?;
    let data = body.get("data").cloned().unwrap_or(Value::Null);
    let items = data
        .get("data")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(normalize).collect())
        .unwrap_or_default();
    let has_next = data
        .get("has_next")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    Ok(Page { items, has_next })
}

/// Download a URL fully (GIF/MP4/preview/avatar bytes).
pub fn download(url: &str) -> anyhow::Result<Vec<u8>> {
    let mut resp = ureq::get(url).call()?;
    Ok(resp
        .body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_to_vec()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_items() {
        let v: Value = serde_json::json!({
            "id": 7,
            "file": {
                "sm": { "webp": { "url": "p.webp" }, "gif": { "url": "s.gif" } },
                "md": { "gif": { "url": "m.gif" }, "mp4": { "url": "m.mp4" } }
            }
        });
        let g = normalize(&v).unwrap();
        assert_eq!(
            (
                g.id.as_str(),
                g.preview.as_str(),
                g.gif.as_str(),
                g.mp4.as_deref()
            ),
            ("7", "p.webp", "m.gif", Some("m.mp4"))
        );
        assert_eq!(encode_query("cat dance"), "cat%20dance");
    }
}
