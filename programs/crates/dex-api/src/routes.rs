use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use dex_core::{find_leader_pda, find_registry_pda, find_subscription_pda};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/rust/sentiment", get(sentiment))
        .route("/api/v1/rust/tokens/trending", get(trending))
        .route("/api/v1/rust/registry/pdas", get(registry_pdas))
        .route("/api/v1/rust/copy/simulate", post(simulate_copy))
}

#[derive(Serialize)]
struct HealthResponse {
    ok: bool,
    service: &'static str,
    mode: String,
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        ok: true,
        service: "dex-api",
        mode: state.solana.mode().to_string(),
    })
}

async fn sentiment(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut engine = state.sentiment.lock().await;
    match engine.tick().await {
        Ok(snapshot) => Json(serde_json::json!({
            "ok": true,
            "scores": snapshot.scores,
            "generatedAt": snapshot.generated_at,
        })),
        Err(err) => Json(serde_json::json!({ "ok": false, "error": err.to_string() })),
    }
}

#[derive(Deserialize)]
struct TrendingQuery {
    limit: Option<usize>,
}

async fn trending(
    State(state): State<AppState>,
    Query(query): Query<TrendingQuery>,
) -> Json<serde_json::Value> {
    let limit = query.limit.unwrap_or(10);
    match state.pumpfun.list_trending(limit).await {
        Ok(tokens) => Json(serde_json::json!({ "ok": true, "tokens": tokens })),
        Err(err) => Json(serde_json::json!({ "ok": false, "error": err.to_string() })),
    }
}

#[derive(Deserialize)]
struct RegistryPdaQuery {
    leader: Option<String>,
    follower: Option<String>,
    program_id: Option<String>,
}

async fn registry_pdas(Query(query): Query<RegistryPdaQuery>) -> Json<serde_json::Value> {
    let program_id = query
        .program_id
        .as_deref()
        .and_then(|v| Pubkey::from_str(v).ok())
        .unwrap_or_else(|| {
            Pubkey::from_str("TradeRegistry1111111111111111111111111111111")
                .expect("default program id")
        });

    let (registry, registry_bump) = find_registry_pda(&program_id);
    let mut payload = serde_json::json!({
        "ok": true,
        "programId": program_id.to_string(),
        "registry": registry.to_string(),
        "registryBump": registry_bump,
    });

    if let Some(leader) = query.leader.as_deref() {
        if let Ok(leader_key) = Pubkey::from_str(leader) {
            let (leader_pda, leader_bump) = find_leader_pda(&program_id, &leader_key);
            payload["leaderPda"] = serde_json::json!(leader_pda.to_string());
            payload["leaderBump"] = serde_json::json!(leader_bump);
        }
    }

    if let (Some(leader), Some(follower)) = (query.leader.as_deref(), query.follower.as_deref()) {
        if let (Ok(leader_key), Ok(follower_key)) =
            (Pubkey::from_str(leader), Pubkey::from_str(follower))
        {
            let (sub_pda, sub_bump) =
                find_subscription_pda(&program_id, &follower_key, &leader_key);
            payload["subscriptionPda"] = serde_json::json!(sub_pda.to_string());
            payload["subscriptionBump"] = serde_json::json!(sub_bump);
        }
    }

    Json(payload)
}

#[derive(Deserialize)]
struct SimulateCopyBody {
    leader_address: String,
    signature: Option<String>,
    mint: Option<String>,
    symbol: Option<String>,
    side: Option<String>,
    amount_sol: Option<f64>,
    user_id: Option<String>,
    follower_public_key: Option<String>,
    size_pct: Option<f64>,
}

async fn simulate_copy(
    State(state): State<AppState>,
    Json(body): Json<SimulateCopyBody>,
) -> Json<serde_json::Value> {
    use copy_engine::types::{CopySubscriptionRecord, LeaderTradeEvent};

    let user_id = body.user_id.unwrap_or_else(|| "demo-user".into());
    let sub_id = format!("{user_id}:{}", body.leader_address);
    state.copy_trade.subscribe(CopySubscriptionRecord {
        id: sub_id,
        user_id,
        leader_address: body.leader_address.clone(),
        follower_wallet_id: "demo-wallet".into(),
        follower_public_key: body
            .follower_public_key
            .unwrap_or_else(|| "Follower1111111111111111111111111111111".into()),
        size_pct: body.size_pct.unwrap_or(100.0),
        active: true,
    });

    let event = LeaderTradeEvent {
        leader_address: body.leader_address,
        signature: body
            .signature
            .unwrap_or_else(|| format!("sim-{}", chrono::Utc::now().timestamp_millis())),
        mint: body.mint,
        symbol: body.symbol,
        side: body.side.unwrap_or_else(|| "buy".into()),
        amount_sol: body.amount_sol.unwrap_or(0.05),
    };

    match state.copy_trade.handle_leader_activity(&event) {
        Ok(trades) => Json(serde_json::json!({
            "ok": true,
            "mirrored": trades.len(),
            "trades": trades,
        })),
        Err(copy_engine::engine::CopyEngineError::NoSubscriptions) => {
            Json(serde_json::json!({ "ok": false, "error": "no_subscriptions" }))
        }
        Err(_) => Json(serde_json::json!({ "ok": false, "error": "copy_failed" })),
    }
}
