use trustsystem_core as core;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn};

#[derive(Deserialize)]
struct LookupResp { did: String, jobId: String, status: String }

pub async fn run_loop(api_base: &str) -> Result<()> {
    let client = Client::new();
    loop {
        if let Some((job_id, did)) = try_pop_job(api_base, &client).await? {
            info!(%job_id, %did, "picked job");
            let _ = process_job(&client, api_base, &job_id, &did).await;
        } else {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

pub async fn run_once(api_base: &str) -> Result<bool> {
    let client = Client::new();
    if let Some((job_id, did)) = try_pop_job(api_base, &client).await? {
        info!(%job_id, %did, "picked job (oneshot)");
        let _ = process_job(&client, api_base, &job_id, &did).await;
        return Ok(true);
    }
    Ok(false)
}

async fn try_pop_job(api_base: &str, client: &Client) -> Result<Option<(String, String)>> {
    let resp = client.get(format!("{}/internal/jobs/next", api_base)).send().await?;
    if resp.status().as_u16() == 204 { return Ok(None); }
    if resp.status().is_success() {
        let v: serde_json::Value = resp.json().await?;
        let job_id = v.get("jobId").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        let did = v.get("did").and_then(|v| v.as_str()).unwrap_or_default().to_string();
        if !job_id.is_empty() && !did.is_empty() { return Ok(Some((job_id, did))); }
    } else {
        warn!(status=?resp.status(), "next job request failed");
    }
    Ok(None)
}

pub async fn process_job(client: &Client, api_base: &str, job_id: &str, did: &str) -> Result<()> {
    // Simulate scoring: compute some fake counts and subjective logic
    let alpha_acc = 5.0;
    let beta_acc = 1.0;
    let o_acc = core::evidence_to_opinion(alpha_acc, beta_acc, 2.0);
    let alpha_civ = 8.0;
    let beta_civ = 2.0;
    let o_civ = core::evidence_to_opinion(alpha_civ, beta_civ, 2.0);

    let scores = json!({
        "did": did,
        "handle": did,
        "updatedAt": (chrono::Utc::now().timestamp_millis()),
        "facets": {
            "accuracy": {"alpha": alpha_acc as i64, "beta": beta_acc as i64, "b": o_acc.b, "d": o_acc.d, "u": o_acc.u},
            "civility": {"alpha": alpha_civ as i64, "beta": beta_civ as i64, "b": o_civ.b, "d": o_civ.d, "u": o_civ.u}
        },
        "botProb": 0.12,
        "expertise": [
            {"domain":"politics","score":0.71},
            {"domain":"technology","score":0.63},
            {"domain":"medicine","score":0.12}
        ],
        "evidence": []
    });

    let _ = client.post(format!("{}/internal/upsert/scores", api_base))
        .json(&scores)
        .send().await?;
    info!(%job_id, %did, "upserted scores");
    let _ = client.post(format!("{}/internal/jobs/score/{}/done", api_base, job_id)).send().await?;
    info!(%job_id, "marked done");
    Ok(())
}


