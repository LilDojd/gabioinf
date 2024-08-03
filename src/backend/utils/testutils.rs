use crate::backend::{
    domain::models::{GithubId, Guest},
    repos::{PgRepository, Repository},
};
use sqlx::PgPool;
#[allow(dead_code)]
pub(crate) async fn setup_guest(pool: &PgPool) -> Guest {
    let guest_repo = PgRepository::<Guest>::new(pool.clone());
    let guest = Guest {
        github_id: GithubId(0),
        name: "Test User".to_string(),
        username: "testuser".to_string(),
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
            ..Default::default()
        };
        guest_repo.create(&guest).await.unwrap();
    }
}
