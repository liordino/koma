//! MangaDex adapter — tier A source (documented, third-party-invited API; decision 4/13).
//!
//! Registry-first: adapters live as data (source table rows). Search/detail/chapters go through
//! the SSRF-guarded OutboundClient on the Api plane (decision 18: separate plane budgets).

use serde::Deserialize;
use serde_json::Value;

use crate::fetch::{OutboundClient, Plane, SharedClient};

pub const MANGADEX_API: &str = "https://api.mangadex.org";
/// Domain allowlist for the MangaDex adapter: API + At-Home image delivery (decision 18).
pub const MANGADEX_ALLOWLIST: &[&str] = &["mangadex.org", "mangadex.network"];

/// A search hit with the badge fields the RPC layer needs (search shows all configured
/// sources, each result carries its source key; owner decision 3).
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    pub source_key: String,
    pub external_id: String,
    pub title: String,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Adapted manga detail.
#[derive(Debug, Clone, Deserialize)]
pub struct MangaDetail {
    pub source_key: String,
    pub external_id: String,
    pub title: String,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Adapted chapter-list entry.
#[derive(Debug, Clone, Deserialize)]
pub struct ChapterEntry {
    pub external_id: String,
    pub chapter: String,
    pub title: Option<String>,
    pub pages: u32,
    pub scanlator: Option<String>,
}

/// MangaDex JSON is shaped `{ "result": "ok", "data": [ {attributes...} ] }`.
fn data_attributes(raw: &Value) -> Vec<Value> {
    raw.get("data")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|m| m.get("type").and_then(Value::as_str) == Some("manga"))
        .collect()
}

fn attrs_of(item: &Value) -> &Value {
    item.get("attributes").unwrap_or(&Value::Null)
}

fn title_of(attrs: &Value) -> String {
    attrs
        .get("title")
        .and_then(|t| t.get("en").or(t.get("ja-ro")).or(t.get("ja")))
        .and_then(Value::as_str)
        .or_else(|| {
            attrs
                .get("title")
                .and_then(Value::as_object)
                .and_then(|m| m.values().next())
                .and_then(Value::as_str)
        })
        .unwrap_or("(untitled)")
        .to_string()
}

fn cover_url_of(item: &Value) -> Option<String> {
    // MangaDex cover: relationships[] type=cover_art -> fileName; URL is the mangadex.network CDN.
    let cov = item
        .get("relationships")
        .and_then(Value::as_array)?
        .iter()
        .find(|r| r.get("type").and_then(Value::as_str) == Some("cover_art"))?;
    let file = cov.get("attributes").and_then(|a| a.get("fileName")).and_then(Value::as_str)?;
    Some(format!("https://mangadex.org/covers/{file}"))
}

fn description_of(attrs: &Value) -> Option<String> {
    attrs
        .get("description")
        .and_then(|d| d.get("en"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Chapter shape: type=chapter, attributes {chapter,title,pages,externalUrl,scanlator}.
fn chapter_list_from(raw: &Value) -> Vec<ChapterEntry> {
    raw.get("data")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter(|c| c.get("type").and_then(Value::as_str) == Some("chapter"))
                .filter_map(|c| {
                    let attrs = attrs_of(c);
                    Some(ChapterEntry {
                        external_id: c.get("id").and_then(Value::as_str)?.to_string(),
                        chapter: attrs
                            .get("chapter")
                            .and_then(Value::as_str)
                            .unwrap_or("?")
                            .to_string(),
                        title: attrs.get("title").and_then(Value::as_str).map(str::to_string),
                        pages: attrs.get("pages").and_then(Value::as_u64).unwrap_or(0) as u32,
                        scanlator: attrs
                            .get("scanlator")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Client bound to the MangaDex adapter (registry row `key='mangadex'`).
#[derive(Clone)]
pub struct MangaDexAdapter {
    http: SharedClient,
    base: String, // overridable for tests (recorded-shape mock)
}

impl MangaDexAdapter {
    pub fn production() -> Result<Self, crate::fetch::FetchError> {
        let http = OutboundClient::new(
            MANGADEX_ALLOWLIST.iter().map(|s| s.to_string()).collect(),
            5, // Api plane budget per window
            10,
            std::time::Duration::from_secs(1),
        )?;
        Ok(Self { http: std::sync::Arc::new(http), base: MANGADEX_API.to_string() })
    }

    pub fn with_client(http: SharedClient, base: &str) -> Self {
        Self { http, base: base.to_string() }
    }

    /// Search titles. `limit` bounded hard — the API allows up to 100.
    pub async fn search(
        &self,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>, crate::fetch::FetchError> {
        let limit = limit.clamp(1, 100);
        let url = format!(
            "{}/manga?title={}&limit={}&contentRating[]=safe&includes[]=cover_art",
            self.base,
            urlencode(query),
            limit
        );
        let raw: Value = self.http.get_json(&url, Plane::Api).await?;
        Ok(data_attributes(&raw)
            .iter()
            .map(|item| {
                let attrs = attrs_of(item);
                SearchResult {
                    source_key: "mangadex".to_string(),
                    external_id: item.get("id").and_then(Value::as_str).unwrap_or("?").to_string(),
                    title: title_of(attrs),
                    cover_url: cover_url_of(item),
                    description: description_of(attrs),
                }
            })
            .collect())
    }

    /// Manga detail by external id.
    pub async fn detail(&self, external_id: &str) -> Result<MangaDetail, crate::fetch::FetchError> {
        let url = format!(
            "{}/manga/{}?includes[]=cover_art",
            self.base,
            urlencode(external_id)
        );
        let raw: Value = self.http.get_json(&url, Plane::Api).await?;
        let first = data_attributes(&raw)
            .into_iter()
            .next()
            .ok_or(crate::fetch::FetchError::Unexpected {
                url,
                status: 404,
            })?;
        let attrs = attrs_of(&first);
        Ok(MangaDetail {
            source_key: "mangadex".to_string(),
            external_id: first.get("id").and_then(Value::as_str).unwrap_or("?").to_string(),
            title: title_of(attrs),
            cover_url: cover_url_of(&first),
            description: description_of(attrs),
        })
    }

    /// Chapter list, newest first (API default order is descending by number).
    pub async fn chapters(
        &self,
        external_id: &str,
        limit: u32,
    ) -> Result<Vec<ChapterEntry>, crate::fetch::FetchError> {
        let limit = limit.clamp(1, 100);
        let url = format!(
            "{}/manga/{}/feed?limit={}&translatedLanguage[]=en&order[chapter]=desc&includes[]=",
            self.base,
            urlencode(external_id),
            limit
        );
        let raw: Value = self.http.get_json(&url, Plane::Api).await?;
        Ok(chapter_list_from(&raw))
    }
}

/// Minimal percent-encoding for query values (space + non-ASCII safe).
fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
