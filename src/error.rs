use thiserror::Error;

#[derive(Debug, Error)]
pub enum GeneralError {
    #[error("IO error {0}")]
    IO(#[from] std::io::Error),
    #[error("Sql error {0}")]
    Sqlite(#[from] rusqlite::Error),
}
