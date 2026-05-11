//! Типизированные ошибки приложения и единый тип для IPC (`tauri::command`).

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),

    #[error("state lock poisoned")]
    StateLockPoisoned,

    #[error("sync runtime lock poisoned")]
    SyncRuntimeLockPoisoned,

    #[error("entity id missing")]
    EntityIdMissing,
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        AppError::Message(value)
    }
}

/// Совместимость с вызовами, где ещё указан `Result<_, crate::errors::AppError>` (миграция на `AppError`).
impl From<AppError> for String {
    fn from(value: AppError) -> Self {
        value.to_string()
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<postcard::Error> for AppError {
    fn from(value: postcard::Error) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<redb::Error> for AppError {
    fn from(value: redb::Error) -> Self {
        AppError::Message(value.to_string())
    }
}

/// redb 4+ возвращает отдельные типы ошибок (не всегда `redb::Error`) — маппим через `Display`.
impl From<redb::TransactionError> for AppError {
    fn from(value: redb::TransactionError) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<redb::CommitError> for AppError {
    fn from(value: redb::CommitError) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<redb::TableError> for AppError {
    fn from(value: redb::TableError) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<redb::StorageError> for AppError {
    fn from(value: redb::StorageError) -> Self {
        AppError::Message(value.to_string())
    }
}

impl From<redb::DatabaseError> for AppError {
    fn from(value: redb::DatabaseError) -> Self {
        AppError::Message(value.to_string())
    }
}
