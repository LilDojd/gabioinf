//! Database connection and utility functions.
//!
//! This module provides types and functions for managing database connections
//! and performing basic database operations.
/// A type alias for the PostgreSQL connection pool.
///
/// This alias simplifies the usage of SQLx's connection pool throughout the application.
pub type DbConnPool = sqlx::Pool<sqlx::Postgres>;
/// Checks if the database connection is alive.
///
/// This function executes a simple query to verify if the database connection is working.
///
/// # Arguments
///
/// * `conn` - A reference to the database connection pool.
///
/// # Returns
///
/// * `true` if the database connection is successful.
/// * `false` if the connection fails.
pub async fn ping_db(conn: &DbConnPool) -> bool {
    let z = sqlx::query("SELECT 1").execute(conn).await;
    match z {
        Ok(_) => true,
        Err(e) => {
            dioxus_logger::tracing::error!("Failed to ping the database: {e}");
            false
        }
    }
}
