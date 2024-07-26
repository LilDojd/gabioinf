//! Handles OAuth flow, user auth and session management


use crate::{
    domain::models::{GithubUser, Guest},
    errors::{BResult, BackendError},
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

pub fn build_oauth_client<S: AsRef<str>>(client_id: S, client_secret: S) -> BasicClient {
    let redirect_url = "http://localhost:8000/v1/auth/authorized".to_string();

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id.as_ref().to_owned()),
        Some(ClientSecret::new(client_secret.as_ref().to_owned())),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: String,
}

pub async fn github_auth(Extension(oauth_client): Extension<BasicClient>) -> impl IntoResponse {
    let (authorize_url, _csrf_state) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identity".to_string()))
        .url();

    Redirect::to(authorize_url.as_ref())
}

pub async fn github_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
    Extension(oauth_client): Extension<BasicClient>,
) -> BResult<impl IntoResponse> {
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(async_http_client)
        .await
        .map_err(|err| BackendError::TokenErr(err.to_string()))?;

    let github_user = state
        .ctx
        .get("https://api.github.com/user")
        .header(
            "Authorization",
            format!("Bearer {}", token.access_token().secret()),
        )
        .header("User-Agent", "gabioinf-guestbook")
        .send()
        .await?
        .json::<GithubUser>()
        .await?;

    let guest = state.guest_crud.upsert_guest(&github_user).await?;

    let expires_in = token.expires_in().ok_or(BackendError::OptionErr)?.as_secs();

    let max_age = Local::now().to_utc() + Duration::seconds(expires_in as i64);

    state
        .guest_crud
        .register_session(&guest, token.access_token().secret(), max_age)
        .await?;

    // TODO: !!!
    let cookie = Cookie::build(("sid", token.access_token().secret().to_owned()))
        // .domain(".app.localhost")
        .path("/")
        // .secure(true)
        // .http_only(true)
        .max_age(time::Duration::seconds(expires_in as i64))
        .build();

    tracing::debug!("Stored sid cookie");
    Ok((jar.add(cookie), Redirect::to("/v1/protected")))
}

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
