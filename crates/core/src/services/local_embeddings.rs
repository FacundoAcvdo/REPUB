use std::fs;
use std::path::{Path, PathBuf};

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::db::Database;
use crate::error::{CoreError, CoreResult};

pub struct LocalEmbeddings;

impl LocalEmbeddings {
    /// Directorio dedicado donde se almacenan los modelos ONNX locales descargados.
    pub fn models_dir() -> Option<PathBuf> {
        Database::default_data_dir().map(|dir| dir.join("models"))
    }

    pub fn map_model(id: &str) -> CoreResult<EmbeddingModel> {
        match id.trim() {
            "multilingual-e5-small" => Ok(EmbeddingModel::MultilingualE5Small),
            "multilingual-e5-base" => Ok(EmbeddingModel::MultilingualE5Base),
            "multilingual-e5-large" => Ok(EmbeddingModel::MultilingualE5Large),
            "all-MiniLM-L6-v2" => Ok(EmbeddingModel::AllMiniLML6V2),
            other => Err(CoreError::InvalidInput(format!(
                "Modelo local de embeddings desconocido: '{other}'"
            ))),
        }
    }

    /// Descarga (si no está en caché) e inicializa el modelo local seleccionado.
    pub fn ensure_downloaded(model_id: &str) -> CoreResult<PathBuf> {
        let dir = Self::models_dir().ok_or_else(|| {
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No se pudo resolver el directorio de modelos.",
            ))
        })?;
        fs::create_dir_all(&dir)?;

        let model_enum = Self::map_model(model_id)?;
        let _engine = TextEmbedding::try_new(
            InitOptions::new(model_enum)
                .with_cache_dir(dir.clone())
                .with_show_download_progress(false),
        )
        .map_err(|e| CoreError::Network(format!("Error al descargar/cargar modelo local: {e}")))?;

        Ok(dir)
    }

    /// Genera embeddings vectoriales localmente usando ONNX Runtime (`fastembed`).
    pub fn embed_texts(model_id: &str, texts: Vec<String>) -> CoreResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let dir = Self::models_dir().ok_or_else(|| {
            CoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No se pudo resolver el directorio de modelos.",
            ))
        })?;
        fs::create_dir_all(&dir)?;

        let model_enum = Self::map_model(model_id)?;
        let engine = TextEmbedding::try_new(
            InitOptions::new(model_enum)
                .with_cache_dir(dir)
                .with_show_download_progress(false),
        )
        .map_err(|e| CoreError::Network(format!("Error al inicializar modelo local: {e}")))?;

        engine
            .embed(texts, None)
            .map_err(|e| CoreError::InvalidInput(format!("Error generando embeddings locales: {e}")))
    }

    /// Calcula el tamaño total ocupado en disco por los modelos locales descargados.
    pub fn downloaded_bytes() -> u64 {
        let Some(dir) = Self::models_dir() else {
            return 0;
        };
        dir_size_bytes(&dir)
    }

    /// Elimina todos los modelos locales descargados en disco y devuelve los bytes liberados.
    pub fn delete_all_models() -> CoreResult<u64> {
        let Some(dir) = Self::models_dir() else {
            return Ok(0);
        };
        if !dir.exists() {
            return Ok(0);
        }
        let freed = dir_size_bytes(&dir);
        fs::remove_dir_all(&dir)?;
        Ok(freed)
    }
}

fn dir_size_bytes(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            total += dir_size_bytes(&p);
        } else if let Ok(meta) = entry.metadata() {
            total += meta.len();
        }
    }
    total
}
