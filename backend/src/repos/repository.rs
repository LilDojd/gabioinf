use serde::{de::DeserializeOwned, Serialize};

#[axum::async_trait]
pub trait Repository<T>
where
    T: Serialize + DeserializeOwned + Send + Sync,
{
    type Error;
    type Criteria;

    async fn read_all(&self) -> Result<Vec<T>, Self::Error>;
    async fn read(&self, criteria: &Self::Criteria) -> Result<T, Self::Error>;
    async fn create(&self, entity: &T) -> Result<T, Self::Error>;
    async fn update(&self, entity: &T) -> Result<T, Self::Error>;
    async fn delete(&self, entity: &T) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone)]
pub struct PgRepository<T> {
    pub pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PgRepository<T> {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}
