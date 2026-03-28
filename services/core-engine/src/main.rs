use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Default, Serialize)]
struct Stats {
    plans_computed: u64,
    rrt_runs: u64,
    meshes_built: u64,
    obstacle_updates: u64,
    total_requests: u64,
}

type AppState = Arc<Mutex<Stats>>;

// --- request / response types ---

#[derive(Debug, Deserialize)]
struct PlanRequest {
    start: Vec<f64>,
    goal: Vec<f64>,
    algorithm: Option<String>,
    grid_resolution: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RrtRequest {
    start: Vec<f64>,
    goal: Vec<f64>,
    max_iterations: Option<u32>,
    step_size: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct MeshRequest {
    bounds_min: Vec<f64>,
    bounds_max: Vec<f64>,
    agent_radius: Option<f64>,
    cell_size: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct ObstaclesRequest {
    obstacles: Vec<serde_json::Value>,
    operation: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    request_id: String,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        ok: true,
        request_id: Uuid::new_v4().to_string(),
        data,
    })
}

// --- handlers ---

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "alice-navigation-saas-core" }))
}

async fn nav_plan(
    State(state): State<AppState>,
    Json(req): Json<PlanRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.plans_computed += 1;
    s.total_requests += 1;
    let algo = req.algorithm.unwrap_or_else(|| "astar".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "algorithm": algo,
            "start": req.start,
            "goal": req.goal,
            "path_id": Uuid::new_v4().to_string(),
            "waypoints": 14,
            "path_length": 18.6,
            "compute_ms": 3,
        })),
    )
}

async fn nav_rrt(
    State(state): State<AppState>,
    Json(req): Json<RrtRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.rrt_runs += 1;
    s.total_requests += 1;
    let iters = req.max_iterations.unwrap_or(5000);
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "start": req.start,
            "goal": req.goal,
            "iterations_used": iters / 3,
            "tree_nodes": iters / 10,
            "path_id": Uuid::new_v4().to_string(),
            "path_cost": 7.42,
            "found": true,
        })),
    )
}

async fn nav_mesh(
    State(state): State<AppState>,
    Json(req): Json<MeshRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.meshes_built += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "mesh_id": Uuid::new_v4().to_string(),
            "bounds_min": req.bounds_min,
            "bounds_max": req.bounds_max,
            "agent_radius": req.agent_radius.unwrap_or(0.3),
            "cell_size": req.cell_size.unwrap_or(0.1),
            "polygon_count": 3_840,
            "build_ms": 28,
        })),
    )
}

async fn nav_obstacles(
    State(state): State<AppState>,
    Json(req): Json<ObstaclesRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.obstacle_updates += 1;
    s.total_requests += 1;
    let op = req.operation.unwrap_or_else(|| "add".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "operation": op,
            "count": req.obstacles.len(),
            "replanned": true,
        })),
    )
}

async fn nav_stats(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let s = state.lock().unwrap();
    ok(serde_json::json!({
        "plans_computed": s.plans_computed,
        "rrt_runs": s.rrt_runs,
        "meshes_built": s.meshes_built,
        "obstacle_updates": s.obstacle_updates,
        "total_requests": s.total_requests,
    }))
}

// --- main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/nav/plan", post(nav_plan))
        .route("/api/v1/nav/rrt", post(nav_rrt))
        .route("/api/v1/nav/mesh", post(nav_mesh))
        .route("/api/v1/nav/obstacles", post(nav_obstacles))
        .route("/api/v1/nav/stats", get(nav_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8121".to_string())
        .parse()
        .unwrap_or(8121);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("alice-navigation-saas-core listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
