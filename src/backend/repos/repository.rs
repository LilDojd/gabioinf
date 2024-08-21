#![allow(unused)]
use serde::{de::DeserializeOwned, Serialize};
/// A trait defining the common operations for a repository.
///
/// This trait provides a standardized interface for CRUD operations
/// on entities of type `T`.
#[axum::async_trait]
pub trait Repository<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    /// The type of error that can be returned by repository operations.
    type Error;
    /// The type used to specify criteria for querying entities.
    type Criteria;
    /// Retrieves all entities from the repository.
    async fn read_all(&self) -> Result<Vec<T>, Self::Error>;
    /// Retrieves a single entity based on the provided criteria.
    async fn read(&self, criteria: &Self::Criteria) -> Result<T, Self::Error>;
    /// Creates a new entity in the repository.
    async fn create(&self, entity: &T) -> Result<T, Self::Error>;
    /// Updates an existing entity in the repository.
    async fn update(&self, entity: &T) -> Result<T, Self::Error>;
    /// Deletes an entity from the repository.
    async fn delete(&self, entity: &T) -> Result<(), Self::Error>;
}
/// A PostgreSQL-specific implementation of the Repository trait.
#[derive(Debug, Clone)]
pub struct PgRepository<T> {
    /// The connection pool for PostgreSQL.
    pub pool: sqlx::PgPool,
    /// Phantom data to hold the type parameter T.
    _phantom: std::marker::PhantomData<T>,
}
impl<T> PgRepository<T> {
    /// Creates a new instance of PgRepository.
    ///
    /// # Arguments
    ///
    /// * `pool` - A PostgreSQL connection pool.
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}
