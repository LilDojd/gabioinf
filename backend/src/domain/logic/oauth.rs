//! Handles OAuth flow, user auth and session management

use crate::{
    domain::models::{Guest, NewGuest, Session},
    errors::{ApiError, BResult},
    repos::Repository,
    AppState,
};
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};
use chrono::{Duration, Local};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

pub async fn protected(guest: Guest) -> impl IntoResponse {
    let guest_json = serde_json::to_string_pretty(&guest)
        .unwrap_or_else(|_| "Error serializing guest data".to_string());

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Protected Page</title>
            <style>
                body {{ font-family: Arial, sans-serif; line-height: 1.6; padding: 20px; }}
                h1 {{ color: #333; }}
                pre {{ background-color: #f4f4f4; padding: 10px; border-radius: 5px; }}
                form {{ margin-top: 20px; }}
                button {{ padding: 10px 15px; background-color: #007bff; color: white; border: none; border-radius: 5px; cursor: pointer; }}
                button:hover {{ background-color: #0056b3; }}
            </style>
        </head>
        <body>
            <h1>Welcome to the Protected Page</h1>
            <p>Hello, {name}! You are successfully authenticated.</p>
            <h2>Your Guest Information:</h2>
            <pre>{guest_json}</pre>
            <form action="/v1/auth/logout" method="post">
                <button type="submit">Logout</button>
            </form>
        </body>
        </html>
        "#,
        name = guest.name,
        guest_json = guest_json
    ))
}
