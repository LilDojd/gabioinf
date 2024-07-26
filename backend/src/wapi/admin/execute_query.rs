use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Column;
use sqlx::Row;
use sqlx::ValueRef;

use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct SqlQueryRequest {
    query: String,
}

#[derive(Serialize, Debug)]
pub struct SqlQueryResponse {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct SqlQueryErrorResponse {
    error: String,
}

pub async fn execute_sql_query(
    State(state): State<AppState>,
    Json(payload): Json<SqlQueryRequest>,
) -> impl IntoResponse {
    // Execute the query
    match sqlx::query(&payload.query)
        .fetch_all(state.db.as_ref())
        .await
    {
        Ok(result) => {
            // Extract column names
            let columns: Vec<String> = if let Some(first_row) = result.first() {
                first_row
                    .columns()
                    .iter()
                    .map(|col| col.name().to_string())
                    .collect()
            } else {
                vec![]
            };

            // Extract row data
            let rows: Vec<Vec<String>> = result
                .iter()
                .map(|row: &PgRow| {
                    columns
                        .iter()
                        .enumerate()
                        .map(|(i, _)| {
                            let value = row.try_get_raw(i).unwrap();
                            if value.is_null() {
                                "NULL".to_string()
                            } else {
                                match value.type_info().to_string().as_ref() {
                                    "BOOL" => row
                                        .try_get::<bool, _>(i)
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "INT2" | "INT4" | "INT8" => row
                                        .try_get::<i64, _>(i)
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "FLOAT4" | "FLOAT8" => row
                                        .try_get::<f64, _>(i)
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "VARCHAR" | "TEXT" => row
                                        .try_get::<String, _>(i)
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "TIMESTAMP" | "TIMESTAMPTZ" => row
                                        .try_get::<chrono::DateTime<chrono::Utc>, _>(i)
                                        .map(|v| v.to_rfc3339())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "DATE" => row
                                        .try_get::<chrono::NaiveDate, _>(i)
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    "TIME" => row
                                        .try_get::<chrono::NaiveTime, _>(i)
                                        .map(|v| v.to_string())
                                        .unwrap_or_else(|_| "ERROR".to_string()),
                                    _ => "UNSUPPORTED_TYPE".to_string(),
                                }
                            }
                        })
                        .collect()
                })
                .collect();

            let response = SqlQueryResponse { columns, rows };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error_message = format!("SQL query execution error: {}", e);
            tracing::error!("{}", error_message);
            let error_response = SqlQueryErrorResponse {
                error: error_message,
            };
            (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
        }
    }
}
