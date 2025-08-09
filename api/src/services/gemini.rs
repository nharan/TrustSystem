use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

#[derive(Serialize)]
pub struct GeminiRequest { pub text: String, pub domain: String }

#[derive(Deserialize, Debug, Clone)]
pub struct GeminiResponse {
    pub classification: String, // "accurate" | "inaccurate" | "contested" | "neutral"
    #[serde(default)]
    pub evidenceRefs: Vec<String>,
}

pub async fn analyze_claim(_text: &str, _domain: &str) -> Result<GeminiResponse> {
    // Call Gemini if API key exists; otherwise return neutral
    let api_key = match std::env::var("GEMINI_API_KEY") { Ok(k) if !k.is_empty() => k, _ => {
        return Ok(GeminiResponse { classification: "neutral".into(), evidenceRefs: vec![] });
    }};

    let endpoint = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", api_key);
    let safe_text = _text.replace('"', "'");
    let prompt = format!(r#"You are a factuality and content classifier for social posts. Classify the following post into exactly one of: accurate, inaccurate, contested, neutral.

Rules:
- If the post makes a factual claim and you can infer it is likely correct, output "accurate".
- If it makes a factual claim that is likely false, output "inaccurate".
- If it is a factual claim but contested/uncertain, output "contested".
- If it is not a factual claim, output "neutral".
Return a single-line JSON: {{"classification":"...", "evidenceRefs":["url", ...]}}. Provide 0-2 reputable URLs when available; otherwise an empty list.

Post (domain={domain}):
"{text}""#,
        domain = _domain,
        text = safe_text,
    );

    let body = serde_json::json!({
        "contents": [{ "parts": [{ "text": prompt }] }],
        "generationConfig": { "temperature": 0.2, "maxOutputTokens": 200 }
    });

    let client = reqwest::Client::builder().timeout(Duration::from_secs(12)).build()?;
    let resp = client.post(&endpoint).json(&body).send().await?;
    if !resp.status().is_success() {
        // fall back quietly
        return Ok(GeminiResponse { classification: "neutral".into(), evidenceRefs: vec![] });
    }
    let v: Value = resp.json().await?;
    // Try to parse the first candidate text as JSON
    let txt = v["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("");
    if !txt.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<GeminiResponse>(txt) {
            return Ok(parsed);
        }
    }
    Ok(GeminiResponse { classification: "neutral".into(), evidenceRefs: vec![] })
}


