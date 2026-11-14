use std::fmt;

/// Tipo de error unificado para la capa de dominio y persistencia.
#[derive(Debug)]
pub enum CoreError {
    Database(rusqlite::Error),
    Serialization(serde_json::Error),
    Io(std::io::Error),
    NotFound(String),
    InvalidInput(String),
}

pub type CoreResult<T> = Result<T, CoreError>;

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Database(err) => write!(f, "Error de base de datos: {err}"),
            CoreError::Serialization(err) => write!(f, "Error de serialización: {err}"),
            CoreError::Io(err) => write!(f, "Error de entrada/salida: {err}"),
            CoreError::NotFound(msg) => write!(f, "Recurso no encontrado: {msg}"),
            CoreError::InvalidInput(msg) => write!(f, "Entrada inválida: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CoreError::Database(err) => Some(err),
            CoreError::Serialization(err) => Some(err),
            CoreError::Io(err) => Some(err),
            CoreError::NotFound(_) | CoreError::InvalidInput(_) => None,
        }
    }
}

impl From<rusqlite::Error> for CoreError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Database(err)
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
