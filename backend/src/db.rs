pub type DbConnPool = sqlx::Pool<sqlx::Postgres>;

pub async fn ping_db(conn: &DbConnPool) -> bool {
    //
    let z = sqlx::query("SELECT 1").execute(conn).await;
    match z {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to ping the database: {e}");
            false
        }
    }
}
