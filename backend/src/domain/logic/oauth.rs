//! Handles OAuth flow, user auth and session management

use std::sync::Arc;

use crate::{
    cruds::GuestCrud,
    domain::models::{GithubUser, Guest},
    errors::{BResult, BackendError},
    AppState,
};
use axum::{
    extract::{FromRequest, Query, State},
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

pub fn build_oauth_client(client_id: String, client_secret: String) -> BasicClient {
    let redirect_url = "http://localhost:8000/v1/auth/authorized".to_string();

    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
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

    let max_age = Local::now().naive_local() + Duration::seconds(expires_in as i64);

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

pub async fn protected(guest: Guest) -> Html<String> {
    let id = guest.username;
    Html(format!(
        r#"
        <p>Ebanutsya!</p>
        <p>Mister {id}, ВЫ В СИСТЕМЕ</p>
    "#
    ))
}
