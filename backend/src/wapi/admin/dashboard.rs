use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

pub async fn admin_dashboard() -> impl IntoResponse {
    let html = include_str!("./templates/admin_dashboard.html");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(html),
    )
}
