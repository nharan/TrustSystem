use anyhow::{anyhow, Result};
use reqwest::Url;
use serde::Deserialize;

fn base_url() -> String {
    std::env::var("ATPROTO_APPVIEW_URL").unwrap_or_else(|_| "https://public.api.bsky.app".to_string())
}

#[derive(Deserialize)]
struct ProfileResp { did: String }

#[derive(Debug, Clone, Deserialize)]
pub struct PostRecord { pub text: Option<String> }

#[derive(Debug, Clone, Deserialize)]
pub struct FeedPost {
    pub cid: String,
    pub uri: String,
    pub author: Author,
    pub record: Option<PostRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author { pub did: String }

#[derive(Deserialize)]
struct FeedItem { post: FeedPost }

#[derive(Deserialize)]
struct AuthorFeed { feed: Vec<FeedItem> }

pub async fn resolve_handle_to_did(handle: &str) -> Result<String> {
    let base = base_url();
    let url = Url::parse_with_params(
        &format!("{}/xrpc/app.bsky.actor.getProfile", base),
        &[("actor", handle)],
    )?;
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        // Fallback to old stub for offline dev
        return Ok(format!("did:plc:{}", handle.replace('.', "")));
    }
    let profile: ProfileResp = resp.json().await?;
    Ok(profile.did)
}

pub async fn fetch_recent_posts(actor: &str, limit: usize) -> Result<Vec<FeedPost>> {
    let base = base_url();
    let url = Url::parse_with_params(
        &format!("{}/xrpc/app.bsky.feed.getAuthorFeed", base),
        &[("actor", actor), ("limit", &limit.to_string())],
    )?;
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        return Err(anyhow!("getAuthorFeed status={}", resp.status()));
    }
    let feed: AuthorFeed = resp.json().await?;
    Ok(feed.feed.into_iter().map(|i| i.post).collect())
}



