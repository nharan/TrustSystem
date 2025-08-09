use anyhow::Result;
use once_cell::sync::Lazy;
use dashmap::DashMap;
use serde_json::json;
use trustsystem_core as core;
use crate::services::{atproto, gemini};

static JOBS: Lazy<DashMap<String, String>> = Lazy::new(|| DashMap::new());
static QUEUE: Lazy<DashMap<String, String>> = Lazy::new(|| DashMap::new()); // jobId -> did

pub async fn enqueue_score_job(_did: &str, _job_id: &str, _force: bool) -> Result<()> {
    // TODO: produce to Kafka topic score.jobs
    JOBS.insert(_job_id.to_string(), "queued".into());
    QUEUE.insert(_job_id.to_string(), _did.to_string());
    Ok(())
}

pub async fn get_job_status(job_id: &str) -> Option<String> {
    JOBS.get(job_id).map(|v| v.clone())
}

pub async fn pop_job() -> Option<(String, String)> {
    if let Some(entry) = QUEUE.iter().next() {
        let job_id = entry.key().clone();
        let did = entry.value().clone();
        QUEUE.remove(&job_id);
        JOBS.insert(job_id.clone(), "processing".into());
        return Some((job_id, did));
    }
    None
}

pub async fn mark_done(job_id: &str) {
    JOBS.insert(job_id.to_string(), "done".into());
}

pub async fn process_job_inline(did: String, handle: String, job_id: String) {
    // MVP real-ish flow: fetch posts and derive basic alpha/beta counts
    let mut posts = atproto::fetch_recent_posts(&did, 25).await.unwrap_or_default();
    if posts.is_empty() {
        // Fallback to using handle when DID resolution failed
        posts = atproto::fetch_recent_posts(&handle, 25).await.unwrap_or_default();
    }
    let mut alpha_acc = 0.0f64;
    let mut beta_acc = 0.0f64;
    let mut alpha_civ = 0.0f64;
    let mut beta_civ = 0.0f64;
    let mut evidence: Vec<serde_json::Value> = Vec::new();

    // Lightweight claim heuristic to widen coverage
    fn looks_like_claim(text: &str) -> bool {
        let t = text.to_lowercase();
        if t.len() < 40 { return false; }
        let has_digit = t.chars().any(|c| c.is_ascii_digit());
        let has_link = t.contains("http://") || t.contains("https://");
        let cues = [" is ", " are ", " was ", " were ", " will ", " has ", " have ", "%", " million", " billion", " according to ", " reports ", " says "];
        let cue_hit = cues.iter().any(|p| t.contains(p));
        let words = t.split_whitespace().count();
        words >= 8 && (has_digit || has_link || cue_hit)
    }

    let mut claim_calls = 0usize;
    for (idx, p) in posts.iter().take(100).enumerate() {
        let text = p.record.as_ref().and_then(|r| r.text.clone()).unwrap_or_default();
        if text.is_empty() { continue; }
        let force_call = idx < 10; // always inspect first 10 posts
        if (force_call || looks_like_claim(&text)) && claim_calls < 25 {
            claim_calls += 1;
            let r = gemini::analyze_claim(&text, "politics").await.unwrap_or(gemini::GeminiResponse{classification:"neutral".into(), evidenceRefs:vec![]});
            match r.classification.as_str() {
                "accurate" => alpha_acc += 1.0,
                "inaccurate" => beta_acc += 1.0,
                "contested" => evidence.push(serde_json::json!({"cid": p.cid, "domain":"politics", "classification":"contested", "evidenceRefs": r.evidenceRefs})),
                _ => {}
            }
        }
        // simplistic civility heuristic (only increment on posts > 5 chars)
        if text.len() > 5 {
            if text.to_lowercase().contains("idiot") { beta_civ += 1.0; } else { alpha_civ += 1.0; }
        }
    }

    let o_acc = core::evidence_to_opinion(alpha_acc, beta_acc, 2.0);
    let o_civ = core::evidence_to_opinion(alpha_civ, beta_civ, 2.0);
    let scores = json!({
        "did": did,
        "handle": handle,
        "updatedAt": (chrono::Utc::now().timestamp_millis()),
        "facets": {
            "accuracy": {"alpha": alpha_acc as i64, "beta": beta_acc as i64, "b": o_acc.b, "d": o_acc.d, "u": o_acc.u},
            "civility": {"alpha": alpha_civ as i64, "beta": beta_civ as i64, "b": o_civ.b, "d": o_civ.d, "u": o_civ.u}
        },
        "botProb": 0.12,
        "expertise": [],
        "evidence": evidence
    });
    let _ = crate::services::graph::upsert_user_scores(
        scores.get("did").and_then(|v| v.as_str()).unwrap_or("") ,
        scores.clone()
    ).await;
    mark_done(&job_id).await;
}



