use anyhow::Result;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct UserScores {
    pub did: String,
}

pub async fn upsert_user_basic(_did: &str, _handle: Option<&str>) -> Result<()> {
    // TODO: connect to JanusGraph via Gremlin/HTTP and upsert user vertex
    Ok(())
}

pub async fn get_user_scores(did_or_handle: &str) -> Result<serde_json::Value> {
    if let Some(v) = INMEM_SCORES.get(did_or_handle) { return Ok(v.clone()); }
    Ok(default_scores(did_or_handle))
}

#[derive(Debug)]
pub struct TrustEdge {
    pub from_did: String,
    pub to_did: String,
    pub scope: String,
    pub b: f32,
    pub d: f32,
    pub u: f32,
    pub evidence_ref: Option<String>,
}

pub async fn upsert_trust_edge(_edge: TrustEdge) -> Result<()> {
    // TODO: upsert trusts edge in JanusGraph
    Ok(())
}

static INMEM_SCORES: Lazy<DashMap<String, Value>> = Lazy::new(|| DashMap::new());

pub async fn upsert_user_scores(did: &str, scores: Value) -> Result<()> {
    INMEM_SCORES.insert(did.to_string(), scores);
    Ok(())
}

pub fn default_scores(id: &str) -> Value {
    json!({
        "did": id, "handle": id, "updatedAt": 0,
        "facets": {
            "accuracy": {"alpha": 0, "beta": 0, "b": 0.0, "d": 0.0, "u": 1.0},
            "civility": {"alpha": 0, "beta": 0, "b": 0.0, "d": 0.0, "u": 1.0}
        },
        "botProb": 0.0, "expertise": [], "evidence": []
    })
}


