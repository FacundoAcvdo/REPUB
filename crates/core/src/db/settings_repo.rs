use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::settings::AppSettings;
use crate::error::CoreResult;

const KEY_APP_SETTINGS: &str = "app_settings";

pub struct SettingsRepository;

impl SettingsRepository {
    /// Carga la configuración global de la aplicación o devuelve los valores por defecto.
    pub fn load_settings(conn: &Connection) -> CoreResult<AppSettings> {
        match Self::get_raw(conn, KEY_APP_SETTINGS)? {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => Ok(AppSettings::default()),
        }
    }

    /// Guarda de forma atómica la configuración global de la aplicación.
    pub fn save_settings(conn: &Connection, settings: &AppSettings) -> CoreResult<()> {
        let json = serde_json::to_string(settings)?;
        Self::set_raw(conn, KEY_APP_SETTINGS, &json)
    }

    /// Obtiene un valor crudo por clave desde la tabla `settings`.
    pub fn get_raw(conn: &Connection, key: &str) -> CoreResult<Option<String>> {
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(Into::into)
    }

    /// Inserta o actualiza un par clave-valor en la tabla `settings`.
    pub fn set_raw(conn: &Connection, key: &str, value: &str) -> CoreResult<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}
