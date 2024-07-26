//! SQL query execution handler.
//!
//! This module contains the handler function for executing arbitrary SQL queries.
//! It's a powerful feature intended for administrative use only.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::Column;
use sqlx::Row;
use sqlx::ValueRef;

use crate::AppState;

/// Represents the request payload for SQL query execution.
#[derive(Deserialize, Debug)]
pub struct SqlQueryRequest {
    query: String,
}

/// Represents the response for a successful SQL query execution.
#[derive(Serialize, Debug)]
pub struct SqlQueryResponse {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

/// Represents the response for a failed SQL query execution.
#[derive(Serialize, Debug)]
pub struct SqlQueryErrorResponse {
    error: String,
}

/// Handler for executing arbitrary SQL queries.
///
/// This function allows execution of any SQL query provided in the request payload.
/// It's designed for administrative use and should be heavily restricted.
///
/// # Arguments
///
/// * `state` - The application state, containing the database connection.
/// * `payload` - The JSON payload containing the SQL query to execute.
///
/// # Returns
///
/// Returns an `impl IntoResponse` which resolves to:
/// - On success: A `200 OK` status with a JSON body containing columns and rows.
/// - On error: A `400 Bad Request` status with a JSON body containing the error message.
///
/// # Security Considerations
///
/// - This endpoint should ONLY be accessible to highly privileged administrators.
/// - It poses significant security risks if misused, including data loss or exposure.
/// - Implement strict access controls, audit logging, and input validation.
/// - Consider whitelisting allowed queries or query types for additional security.
///
/// # Performance Note
///
/// Arbitrary queries can be resource-intensive. Consider implementing query timeouts
/// and resource limits to prevent DoS vulnerabilities.
///
/// # Example
///
/// ```json
/// POST /admin/query
/// Content-Type: application/json
/// Authorization: Bearer <admin-token>
///
/// {
///     "query": "SELECT * FROM users LIMIT 10"
/// }
/// ```
///
/// Successful response:
/// ```json
/// HTTP/1.1 200 OK
/// Content-Type: application/json
///
/// {
///     "columns": ["id", "name", "email"],
///     "rows": [
///         ["1", "John Doe", "john@example.com"],
///         ["2", "Jane Smith", "jane@example.com"]
///     ]
/// }
/// ```
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
