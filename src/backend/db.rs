//! Shared PostgreSQL pool and the `/v1/ping` database probe.
use super::{AppState, errors::BResult};
use axum::{extract::State, response::IntoResponse};

pub type DbConnPool = sqlx::PgPool;

/// Returns `200 Pong` when PostgreSQL is reachable; failures use the API error response.
pub async fn ping_db(State(state): State<AppState>) -> BResult<impl IntoResponse> {
    sqlx::query!("SELECT 1 AS one").fetch_one(&state.db).await?;
    Ok((axum::http::StatusCode::OK, "Pong"))
}
