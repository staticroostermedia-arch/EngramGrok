use crate::store::SharedStore;
use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use tracing::{info, warn};

// ── Models ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RememberReq {
    concept: String,
    text: String,
}

#[derive(Deserialize)]
struct RecallReq {
    query: String,
    #[serde(default = "default_k")]
    k: usize,
}
fn default_k() -> usize { 5 }

#[derive(Deserialize)]
struct ForgetReq {
    concept: String,
}

#[derive(Deserialize)]
struct TraceReq {
    term_a: String,
    op: String,
    term_b: String,
    #[serde(default = "default_k")]
    k: usize,
}

#[derive(Serialize)]
struct MemoryRes {
    concept: String,
    score: f32,
    crs: f32,
    text: String,
}

#[derive(Serialize)]
struct GenericRes {
    status: &'static str,
    message: String,
}

#[derive(Deserialize)]
struct RelateReq {
    concept_a: String,
    concept_b: String,
    label: String,
}

#[derive(Deserialize)]
struct SolutionReq {
    error_pattern: String,
    solution: String,
}

#[derive(Deserialize)]
struct ExportReq {
    summary: String,
}

// ── Middleware ─────────────────────────────────────────────────────────

async fn auth_middleware(req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    if let Ok(key) = env::var("ENGRAM_API_KEY") {
        if key.trim().is_empty() { return Ok(next.run(req).await); }

        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        match auth_header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = header.trim_start_matches("Bearer ").trim();
                if token != key {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
            _ => return Err(StatusCode::UNAUTHORIZED),
        }
    }
    Ok(next.run(req).await)
}

// ── Handlers ───────────────────────────────────────────────────────────

async fn remember(
    State(store): State<SharedStore>,
    Json(payload): Json<RememberReq>,
) -> impl IntoResponse {
    let concept = payload.concept.trim();
    let text = payload.text.trim();
    if concept.is_empty() || text.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(GenericRes {
            status: "error", message: "concept and text are required".into(),
        }));
    }

    match store.lock().unwrap().remember(concept, text) {
        Ok(_) => {
            info!("rest: remembered {concept}");
            (StatusCode::OK, Json(GenericRes {
                status: "success", message: format!("Stored '{concept}'"),
            }))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(GenericRes {
            status: "error", message: e.to_string(),
        })),
    }
}

async fn recall(
    State(store): State<SharedStore>,
    Json(payload): Json<RecallReq>,
) -> impl IntoResponse {
    let query = payload.query.trim();
    if query.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(vec![]));
    }

    let k = payload.k.clamp(1, 20);
    let results = store.lock().unwrap().recall(query, k);
    
    let res: Vec<MemoryRes> = results.into_iter().map(|m| MemoryRes {
        concept: m.concept,
        score: m.score,
        crs: m.crs,
        text: m.provlog,
    }).collect();

    (StatusCode::OK, Json(res))
}

async fn forget(
    State(store): State<SharedStore>,
    Json(payload): Json<ForgetReq>,
) -> impl IntoResponse {
    let concept = payload.concept.trim();
    if concept.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(GenericRes {
            status: "error", message: "concept required".into(),
        }));
    }

    match store.lock().unwrap().forget(concept) {
        Ok(_) => {
            info!("rest: forgot {concept}");
            (StatusCode::OK, Json(GenericRes {
                status: "success", message: format!("Deleted '{concept}'"),
            }))
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(GenericRes {
            status: "error", message: e.to_string(),
        })),
    }
}

async fn trace(
    State(store): State<SharedStore>,
    Json(payload): Json<TraceReq>,
) -> impl IntoResponse {
    use engram_core::ops::{op_add, op_bind};
    
    let term_a = payload.term_a.trim();
    let term_b = payload.term_b.trim();
    let op = payload.op.trim().to_uppercase();
    let k = payload.k.clamp(1, 20);

    if term_a.is_empty() || term_b.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(vec![]));
    }

    let mut lock = store.lock().unwrap();
    let q_a = lock.fetch(term_a).unwrap_or_else(|| Box::new(lock.encode(term_a).q));
    let q_b = lock.fetch(term_b).unwrap_or_else(|| Box::new(lock.encode(term_b).q));

    let q_res = match op.as_str() {
        "ADD" => op_add(&q_a, &q_b),
        "BIND" => op_bind(&q_a, &q_b),
        _ => return (StatusCode::BAD_REQUEST, Json(vec![])),
    };

    let results = lock.query(&q_res, k);
    let res: Vec<MemoryRes> = results.into_iter().map(|m| MemoryRes {
        concept: m.concept,
        score: m.score,
        crs: m.crs,
        text: m.provlog,
    }).collect();

    (StatusCode::OK, Json(res))
}

async fn list_concepts(State(store): State<SharedStore>) -> impl IntoResponse {
    let list = store.lock().unwrap().list();
    (StatusCode::OK, Json(list))
}

// ── Phase 10 Handlers ────────────────────────────────────────────────

async fn status_handler(
    State(store): State<SharedStore>,
    axum::extract::Path(concept): axum::extract::Path<String>,
) -> impl IntoResponse {
    let mut lock = store.lock().unwrap();
    if let Some(status) = lock.status(&concept) {
        (StatusCode::OK, Json(GenericRes { status: "success", message: status }))
    } else {
        (StatusCode::NOT_FOUND, Json(GenericRes { status: "error", message: format!("Concept '{}' not found", concept) }))
    }
}

async fn relate_handler(
    State(store): State<SharedStore>,
    Json(payload): Json<RelateReq>,
) -> impl IntoResponse {
    let mut lock = store.lock().unwrap();
    match lock.relate(&payload.concept_a, &payload.concept_b, &payload.label) {
        Ok(msg) => (StatusCode::OK, Json(GenericRes { status: "success", message: msg })),
        Err(e) => (StatusCode::BAD_REQUEST, Json(GenericRes { status: "error", message: e.to_string() })),
    }
}

async fn remember_solution_handler(
    State(store): State<SharedStore>,
    Json(payload): Json<SolutionReq>,
) -> impl IntoResponse {
    let mut lock = store.lock().unwrap();
    match lock.remember_solution(&payload.error_pattern, &payload.solution) {
        Ok(msg) => (StatusCode::OK, Json(GenericRes { status: "success", message: msg })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(GenericRes { status: "error", message: e.to_string() })),
    }
}

async fn export_context_handler(
    State(store): State<SharedStore>,
    Json(payload): Json<ExportReq>,
) -> impl IntoResponse {
    let mut lock = store.lock().unwrap();
    match lock.export_context(&payload.summary) {
        Ok(msg) => (StatusCode::OK, Json(GenericRes { status: "success", message: msg })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(GenericRes { status: "error", message: e.to_string() })),
    }
}

// ── Server Setup ───────────────────────────────────────────────────────


pub async fn run(store: SharedStore, port: u16) -> anyhow::Result<()> {
    // ── Boot the Background Worker ─────────────────────────────────
    crate::store::StoreHandle::boot_daemon(store.clone());

    if env::var("ENGRAM_API_KEY").is_ok() {
        info!("ENGRAM_API_KEY detected. Bearer token required for all endpoints.");
    } else {
        warn!("Running without ENGRAM_API_KEY. Endpoints are currently unprotected.");
    }

    let app = Router::new()
        .route("/api/remember", post(remember))
        .route("/api/recall", post(recall))
        .route("/api/forget", post(forget))
        .route("/api/trace", post(trace))
        .route("/api/list", get(list_concepts))
        // ── Phase 10 routes ──────────────────────────────────────────────
        .route("/api/status/:concept", get(status_handler))
        .route("/api/relate", post(relate_handler))
        .route("/api/remember_solution", post(remember_solution_handler))
        .route("/api/export_context", post(export_context_handler))
        .layer(middleware::from_fn(auth_middleware))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(store.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Engram REST server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
