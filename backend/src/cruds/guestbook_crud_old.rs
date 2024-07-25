use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::AppState;

pub async fn guestbook(
    State(state): State<AppState>,
) -> Result<Json<Vec<GuestbookEntry>>, impl IntoResponse> {
    match sqlx::query_as::<_, GuestbookEntry>("SELECT * FROM guestbook_entries ORDER BY ID DESC")
        .fetch_all(&state.db)
        .await
    {
        Ok(entries) => Ok(Json(entries)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

pub async fn sign_guestbook(
    State(state): State<AppState>,
    Json(new_entry): Json<NewGuestbookEntry>,
) -> Result<StatusCode, impl IntoResponse> {
    let uuid = Uuid::now_v7(); // ms prescision, so we don't really care
    let ts = Utc::now();

    match sqlx::query(
        "INSERT INTO guestbook_entries (id, name, message, signature, created_at) 
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(uuid)
    .bind(new_entry.name)
    .bind(new_entry.message)
    .bind(new_entry.signature)
    .bind(ts)
    .execute(&state.db)
    .await
    {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

pub async fn hide_entry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(reason): Json<String>,
) -> Result<StatusCode, impl IntoResponse> {
    match sqlx::query(
        "UPDATE guestbook_entries SET is_naughty = true, naughty_reason = $1 WHERE id = $2",
    )
    .bind(reason)
    .bind(id)
    .execute(&state.db)
    .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, impl IntoResponse> {
    match sqlx::query("DELETE FROM guestbook_entries WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
    {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::app_state_from_dbppol;
    use axum::http::StatusCode;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_guestbook(pool: PgPool) {
        let state = app_state_from_dbppol(pool);

        // Insert a test entry
        let test_entry = NewGuestbookEntry {
            name: "Test User".to_string(),
            message: "Test Message".to_string(),
            signature: None,
        };
        let _ = sign_guestbook(State(state.clone()), Json(test_entry)).await;

        // Test retrieving entries
        let result = guestbook(State(state)).await;
        assert!(result.is_ok());
        // let entries = result.unwrap();
        // assert!(!entries.is_empty());
        // assert_eq!(entries[0].name, "Test User");
        // assert_eq!(entries[0].message, "Test Message");
    }

    // #[sqlx::test]
    // async fn test_sign_guestbook() {
    //     let state = app_state_from_dbppol(pool);
    //
    //     let new_entry = NewGuestbookEntry {
    //         name: "New User".to_string(),
    //         message: "New Message".to_string(),
    //         signature: Some("base64_encoded_signature".to_string()),
    //     };
    //
    //     let result = sign_guestbook(State(state), Json(new_entry)).await;
    //     assert_eq!(result.unwrap(), StatusCode::CREATED);
    // }
    //
    // #[sqlx::test]
    // async fn test_hide_entry() {
    //     let state = app_state_from_dbppol(pool);
    //
    //     // Insert a test entry
    //     let test_entry = NewGuestbookEntry {
    //         name: "Hide Test User".to_string(),
    //         message: "Hide Test Message".to_string(),
    //         signature: None,
    //     };
    //     let _ = sign_guestbook(State(state.clone()), Json(test_entry)).await;
    //
    //     // Get the ID of the inserted entry
    //     let entries = sqlx::query_as::<_, GuestbookEntry>(
    //         "SELECT * FROM guestbook_entries ORDER BY created_at DESC LIMIT 1",
    //     )
    //     .fetch_all(&state.db)
    //     .await
    //     .unwrap();
    //     let entry_id = entries[0].id;
    //
    //     // Test hiding the entry
    //     let result = hide_entry(
    //         State(state.clone()),
    //         Path(entry_id),
    //         Json("Inappropriate content".to_string()),
    //     )
    //     .await;
    //     assert_eq!(result.unwrap(), StatusCode::OK);
    //
    //     // Verify the entry is hidden
    //     let hidden_entry =
    //         sqlx::query_as::<_, GuestbookEntry>("SELECT * FROM guestbook_entries WHERE id = $1")
    //             .bind(entry_id)
    //             .fetch_one(&state.db)
    //             .await
    //             .unwrap();
    //     assert!(hidden_entry.is_naughty);
    //     assert_eq!(
    //         hidden_entry.naughty_reason,
    //         Some("Inappropriate content".to_string())
    //     );
    // }

    // #[sqlx::test]
    // async fn test_delete_entry() {
    //     let state = app_state_from_dbppol(pool);
    //
    //     // Insert a test entry
    //     let test_entry = NewGuestbookEntry {
    //         name: "Delete Test User".to_string(),
    //         message: "Delete Test Message".to_string(),
    //         signature: None,
    //     };
    //     let _ = sign_guestbook(State(state.clone()), Json(test_entry)).await;
    //
    //     // Get the ID of the inserted entry
    //     let entries = sqlx::query_as::<_, GuestbookEntry>(
    //         "SELECT * FROM guestbook_entries ORDER BY created_at DESC LIMIT 1",
    //     )
    //     .fetch_all(&state.db)
    //     .await
    //     .unwrap();
    //     let entry_id = entries[0].id;
    //
    //     // Test deleting the entry
    //     let result = delete_entry(State(state.clone()), Path(entry_id)).await;
    //     assert_eq!(result.unwrap(), StatusCode::OK);
    //
    //     // Verify the entry is deleted
    //     let deleted_entry =
    //         sqlx::query_as::<_, GuestbookEntry>("SELECT * FROM guestbook_entries WHERE id = $1")
    //             .bind(entry_id)
    //             .fetch_optional(&state.db)
    //             .await
    //             .unwrap();
    //     assert!(deleted_entry.is_none());
    // }
}
