use axum::{routing::{post, get}, Router, extract::Path, Json};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod subjective;
mod services;

#[derive(Deserialize)]
struct LookupReq { handle: String, force: Option<bool> }

#[derive(Serialize)]
struct LookupResp { did: String, jobId: String, status: String }

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    let app = Router::new()
        .route("/v1/lookup", post(lookup))
        .route("/v1/user/:id/scores", get(get_scores))
        .route("/v1/trust", post(post_trust))
        .route("/internal/jobs/score", post(internal_enqueue))
        .route("/internal/jobs/score/:id", get(internal_status))
        .route("/internal/jobs/next", get(internal_next_job))
        .route("/internal/jobs/score/:id/done", post(internal_mark_done))
        .route("/internal/upsert/scores", post(internal_upsert_scores))
        .layer(cors);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn lookup(Json(req): Json<LookupReq>) -> Json<LookupResp> {
    let did = services::atproto::resolve_handle_to_did(&req.handle).await.unwrap_or("did:unknown".into());
    let job_id = Uuid::new_v4().to_string();
    let _ = services::jobs::enqueue_score_job(&did, &job_id, req.force.unwrap_or(false)).await;
    // Spawn inline processing task so results appear without separate workers
    let did_clone = did.clone();
    let handle_clone = req.handle.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        services::jobs::process_job_inline(did_clone, handle_clone, job_id_clone).await;
    });
    Json(LookupResp { did, jobId: job_id, status: "queued".into() })
}

async fn get_scores(Path(id): Path<String>) -> Json<serde_json::Value> {
    let val = services::graph::get_user_scores(&id).await.unwrap_or(serde_json::json!({"did": id}));
    Json(val)
}

#[derive(Deserialize)]
struct ScoreJobReq { did: String, force: Option<bool> }

#[derive(Serialize)]
struct ScoreJobResp { jobId: String, status: String }

async fn internal_enqueue(Json(req): Json<ScoreJobReq>) -> Json<ScoreJobResp> {
    let job_id = Uuid::new_v4().to_string();
    let _ = services::jobs::enqueue_score_job(&req.did, &job_id, req.force.unwrap_or(false)).await;
    Json(ScoreJobResp { jobId: job_id, status: "queued".into() })
}

async fn internal_status(Path(id): Path<String>) -> Json<serde_json::Value> {
    let status = services::jobs::get_job_status(&id).await.unwrap_or("unknown".into());
    Json(serde_json::json!({"jobId": id, "status": status}))
}

#[derive(Deserialize)]
struct TrustReqOpinion { b: f32, d: f32, u: f32 }

#[derive(Deserialize)]
struct TrustReq { fromDid: String, toDid: String, scope: String, opinion: TrustReqOpinion, evidenceRef: Option<String> }

async fn post_trust(Json(req): Json<TrustReq>) -> Json<serde_json::Value> {
    let edge = services::graph::TrustEdge {
        from_did: req.fromDid,
        to_did: req.toDid,
        scope: req.scope,
        b: req.opinion.b,
        d: req.opinion.d,
        u: req.opinion.u,
        evidence_ref: req.evidenceRef,
    };
    let _ = services::graph::upsert_trust_edge(edge).await;
    Json(serde_json::json!({"status": "ok"}))
}

async fn internal_upsert_scores(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let did = body.get("did").and_then(|v| v.as_str()).unwrap_or("did:unknown").to_string();
    let _ = services::graph::upsert_user_scores(&did, body).await;
    Json(serde_json::json!({"status": "ok", "did": did}))
}

async fn internal_next_job() -> impl IntoResponse {
    if let Some((job_id, did)) = services::jobs::pop_job().await {
        return (StatusCode::OK, Json(serde_json::json!({"jobId": job_id, "did": did}))).into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

async fn internal_mark_done(Path(id): Path<String>) -> impl IntoResponse {
    services::jobs::mark_done(&id).await;
    StatusCode::NO_CONTENT
}



