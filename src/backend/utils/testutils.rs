use crate::{
    backend::repos::{PgRepository, Repository},
    shared::models::{GithubId, Guest},
};
use sqlx::PgPool;
use time::OffsetDateTime;
#[allow(dead_code)]
pub(crate) async fn setup_guest(pool: &PgPool) -> Guest {
    let guest_repo = PgRepository::<Guest>::new(pool.clone());
    let guest = Guest {
        github_id: GithubId(0),
        name: "Test User".to_string(),
        username: "testuser".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        ..Default::default()
    };
    guest_repo.create(&guest).await.unwrap();
    guest
}
#[allow(dead_code)]
pub(crate) async fn setup_guests(n: usize, pool: &PgPool) {
    let guest_repo = PgRepository::<Guest>::new(pool.clone());
    for i in 1..n + 1 {
        let guest = Guest {
            github_id: GithubId(i as i64),
            name: "Test User".to_string(),
            username: "testuser".to_string(),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            ..Default::default()
        };
        guest_repo.create(&guest).await.unwrap();
    }
}
