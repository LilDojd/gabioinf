use thiserror::Error;

pub(crate) type BResult<T> = std::result::Result<T, BackendError>;

#[derive(Debug)]
pub enum UseCase {
    UserRegister,
    UserLogin,
    UpdateUser,
    Placeholder,
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error")]
    InternalErr,

    #[error("{0} already exists")]
    AlreadyExists(String),

    #[error("{0} not found")]
    NotFound(String),

    #[error("Unhandled error")]
    UnhandledErr,
}

impl From<(sqlx::Error, UseCase)> for BackendError {
    fn from(value: (sqlx::Error, UseCase)) -> Self {
        log::debug!("from((sqlx::Error, UseCase)): value={:?}", value);
        let (err, use_case) = value;
        match use_case {
            UseCase::UserRegister => {
                let db_err = err.into_database_error();
                match db_err {
                    Some(e) => {
                        e.code()
                            .map_or(BackendError::InternalErr, |code| match code.as_ref() {
                                "23505" => BackendError::AlreadyExists("User".to_string()),
                                _ => BackendError::InternalErr,
                            })
                    }
                    None => BackendError::InternalErr,
                }
            }
            UseCase::UserLogin => match &err {
                sqlx::Error::RowNotFound => BackendError::Unauthorized("wrong credentials".into()),
                _ => BackendError::InternalErr,
            },
            UseCase::UpdateUser => todo!(),
            _ => BackendError::UnhandledErr,
        }
    }
}
