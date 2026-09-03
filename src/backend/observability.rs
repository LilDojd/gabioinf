use crate::backend::auth::AuthSession;
use crate::shared::models::Guest;
use axum::{extract::Request, middleware::Next, response::Response};
use sentry::{ClientInitGuard, ClientOptions, protocol::User};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_TRACES_SAMPLE_RATE: f32 = 0.1;

pub(crate) fn init(level: Level) -> ClientInitGuard {
    let guard = sentry::init(
        ClientOptions::new()
            .maybe_release(sentry::release_name!())
            .send_default_pii(false)
            .traces_sample_rate(traces_sample_rate()),
    );

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy()
        .add_directive(
            "hyper_util=warn"
                .parse()
                .expect("static directive is valid"),
        );

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry::integrations::tracing::layer())
        .try_init()
        .expect("failed to init logger");

    guard
}

pub(super) async fn sentry_user_context(request: Request, next: Next) -> Response {
    if let Some(user) = request
        .extensions()
        .get::<AuthSession>()
        .and_then(|session| session.user.as_ref())
    {
        sentry::configure_scope(|scope| scope.set_user(Some(sentry_user(user))));
    }

    next.run(request).await
}

fn traces_sample_rate() -> f32 {
    std::env::var("SENTRY_TRACES_SAMPLE_RATE")
        .ok()
        .as_deref()
        .and_then(parse_traces_sample_rate)
        .unwrap_or(DEFAULT_TRACES_SAMPLE_RATE)
}

fn parse_traces_sample_rate(value: &str) -> Option<f32> {
    value.parse().ok().filter(|rate| (0.0..=1.0).contains(rate))
}

fn sentry_user(user: &Guest) -> User {
    User {
        id: Some(user.id.to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::models::GuestId;
    use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
    use sentry::protocol::{Context as SentryContext, EnvelopeItem, SpanStatus};
    use tower::ServiceExt;

    #[test]
    fn trace_sample_rate_rejects_invalid_values() {
        assert_eq!(parse_traces_sample_rate("0"), Some(0.0));
        assert_eq!(parse_traces_sample_rate("0.25"), Some(0.25));
        assert_eq!(parse_traces_sample_rate("1"), Some(1.0));
        assert_eq!(parse_traces_sample_rate("-0.1"), None);
        assert_eq!(parse_traces_sample_rate("1.1"), None);
        assert_eq!(parse_traces_sample_rate("invalid"), None);
    }

    #[test]
    fn sentry_user_contains_only_internal_id() {
        let user = sentry_user(&Guest {
            id: GuestId(42),
            username: "public-name".to_string(),
            ..Default::default()
        });

        assert_eq!(
            user,
            User {
                id: Some("42".to_string()),
                ..Default::default()
            }
        );
    }

    #[test]
    fn request_transaction_and_tracing_error_have_http_context() {
        let envelopes = sentry::test::with_captured_envelopes_options(
            || {
                let subscriber =
                    tracing_subscriber::registry().with(sentry::integrations::tracing::layer());
                tracing::subscriber::with_default(subscriber, || {
                    tokio::runtime::Runtime::new()
                        .expect("runtime should start")
                        .block_on(async {
                            let app = Router::new()
                                .route(
                                    "/users/{id}",
                                    get(|| async {
                                        let error = std::io::Error::other("test error");
                                        tracing::error!(
                                            error = &error as &dyn std::error::Error,
                                            "request failed"
                                        );
                                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                                    }),
                                )
                                .layer(
                                    sentry::integrations::tower::SentryHttpLayer::new()
                                        .enable_transaction(),
                                )
                                .layer(sentry::integrations::tower::NewSentryLayer::<
                                    Request,
                                >::new_from_top());

                            app.oneshot(
                                Request::builder()
                                    .uri("/users/42")
                                    .header("host", "example.com")
                                    .body(axum::body::Body::empty())
                                    .expect("request should build"),
                            )
                            .await
                            .expect("request should succeed");
                        });
                });
            },
            ClientOptions::new().traces_sample_rate(1.0),
        );

        let mut event = None;
        let mut transaction = None;
        for envelope in &envelopes {
            for item in envelope.items() {
                match item {
                    EnvelopeItem::Event(item) => event = Some(item.as_ref()),
                    EnvelopeItem::Transaction(item) => transaction = Some(item.as_ref()),
                    _ => {}
                }
            }
        }

        let event = event.expect("tracing error should create an event");
        assert_eq!(
            event.request.as_ref().and_then(|req| req.method.as_deref()),
            Some("GET")
        );

        let transaction = transaction.expect("request should create a transaction");
        assert_eq!(transaction.name.as_deref(), Some("GET /users/{id}"));
        assert_eq!(
            transaction
                .request
                .as_ref()
                .and_then(|request| request.method.as_deref()),
            Some("GET")
        );
        let trace = match transaction.contexts.get("trace") {
            Some(SentryContext::Trace(trace)) => trace,
            other => panic!("expected trace context, got {other:?}"),
        };
        assert_eq!(trace.status, Some(SpanStatus::InternalError));
    }
}
