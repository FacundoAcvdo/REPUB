use std::time::Duration;

use serde_json::{json, Value};

use crate::domain::settings::AiSettings;
use crate::error::{CoreError, CoreResult};

pub struct AiClient;

impl AiClient {
    /// Construye la URL y el cuerpo JSON para el proveedor configurado (útil para pruebas y envío).
    pub fn build_chat_request(
        settings: &AiSettings,
        system_prompt: &str,
        user_prompt: &str,
    ) -> CoreResult<(String, Value)> {
        let provider = settings.provider.trim().to_lowercase();
        let model = if settings.model.trim().is_empty() {
            match provider.as_str() {
                "anthropic" => "claude-3-5-sonnet-latest",
                "gemini" => "gemini-2.0-flash",
                "groq" => "llama-3.3-70b-versatile",
                _ => "gpt-4o-mini",
            }
        } else {
            settings.model.trim()
        };

        if provider.contains("anthropic") {
            let url = "https://api.anthropic.com/v1/messages".to_string();
            let body = json!({
                "model": model,
                "max_tokens": 1024,
                "system": system_prompt,
                "messages": [
                    { "role": "user", "content": user_prompt }
                ]
            });
            Ok((url, body))
        } else if provider.contains("gemini") {
            let key = settings.api_key.trim();
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}"
            );
            let body = json!({
                "systemInstruction": {
                    "parts": [{ "text": system_prompt }]
                },
                "contents": [
                    {
                        "role": "user",
                        "parts": [{ "text": user_prompt }]
                    }
                ],
                "generationConfig": {
                    "temperature": 0.3,
                    "maxOutputTokens": 4096
                }
            });
            Ok((url, body))
        } else if provider.contains("groq") {
            let url = "https://api.groq.com/openai/v1/chat/completions".to_string();
            let body = json!({
                "model": model,
                "temperature": 0.3,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            });
            Ok((url, body))
        } else {
            // OpenAI (por defecto)
            let url = "https://api.openai.com/v1/chat/completions".to_string();
            let body = json!({
                "model": model,
                "temperature": 0.3,
                "messages": [
                    { "role": "system", "content": system_prompt },
                    { "role": "user", "content": user_prompt }
                ]
            });
            Ok((url, body))
        }
    }

    /// Envía una consulta de chat al proveedor cloud configurado y devuelve la respuesta textual.
    pub fn chat_completion(
        settings: &AiSettings,
        system_prompt: &str,
        user_prompt: &str,
    ) -> CoreResult<String> {
        let key = settings.api_key.trim();
        if key.is_empty() || key.contains("••••") {
            return Err(CoreError::InvalidInput(
                "Configura una API Key válida en Ajustes → Proveedores (IA).".into(),
            ));
        }

        let (url, body) = Self::build_chat_request(settings, system_prompt, user_prompt)?;
        let provider = settings.provider.trim().to_lowercase();

        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(45))
            .build();

        let req = agent
            .post(&url)
            .set("Content-Type", "application/json");

        let req = if provider.contains("anthropic") {
            req.set("x-api-key", key)
                .set("anthropic-version", "2023-06-01")
        } else if provider.contains("gemini") {
            req
        } else {
            req.set("Authorization", &format!("Bearer {key}"))
        };

        let resp_json: Value = match req.send_json(body) {
            Ok(resp) => resp
                .into_json()
                .map_err(|e| CoreError::Network(format!("Respuesta JSON inválida: {e}")))?,
            Err(ureq::Error::Status(code, resp)) => {
                let err_body = resp.into_string().unwrap_or_default();
                return Err(CoreError::Network(format!(
                    "HTTP {code} del proveedor: {}",
                    truncate_msg(&err_body, 220)
                )));
            }
            Err(ureq::Error::Transport(t)) => {
                return Err(CoreError::Network(format!("Error de conexión: {t}")));
            }
        };

        Self::extract_chat_text(&provider, &resp_json)
    }

    fn extract_chat_text(provider: &str, json: &Value) -> CoreResult<String> {
        if provider.contains("anthropic") {
            if let Some(text) = json["content"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|item| item["text"].as_str())
                .map(str::trim)
                .filter(|t| !t.is_empty())
            {
                return Ok(text.to_string());
            }
        } else if provider.contains("gemini") {
            if let Some(cand) = json["candidates"].as_array().and_then(|arr| arr.first()) {
                if let Some(parts) = cand["content"]["parts"].as_array() {
                    let answer_parts: Vec<&str> = parts
                        .iter()
                        .filter(|part| part["thought"].as_bool() != Some(true))
                        .filter_map(|part| part["text"].as_str())
                        .map(str::trim)
                        .filter(|t| !t.is_empty())
                        .collect();
                    if !answer_parts.is_empty() {
                        return Ok(answer_parts.join("\n\n"));
                    }
                }
                if let Some(reason) = cand["finishReason"].as_str() {
                    return Err(CoreError::Network(format!(
                        "Gemini no devolvió texto visible (finishReason: {reason})."
                    )));
                }
            }
        } else if let Some(text) = json["choices"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|choice| choice["message"]["content"].as_str())
            .map(str::trim)
            .filter(|t| !t.is_empty())
        {
            return Ok(text.to_string());
        }

        Err(CoreError::Network(
            "El proveedor no devolvió contenido textual en la respuesta.".into(),
        ))
    }

    /// Genera embeddings usando la API cloud del proveedor (OpenAI o Gemini).
    pub fn cloud_embeddings(settings: &AiSettings, texts: &[String]) -> CoreResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }
        let key = settings.api_key.trim();
        if key.is_empty() || key.contains("••••") {
            return Err(CoreError::InvalidInput(
                "API Key requerida para embeddings cloud.".into(),
            ));
        }

        let provider = settings.provider.trim().to_lowercase();
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(60))
            .build();

        if provider.contains("gemini") {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:batchEmbedContents?key={key}"
            );
            let requests: Vec<Value> = texts
                .iter()
                .map(|t| {
                    json!({
                        "model": "models/text-embedding-004",
                        "content": { "parts": [{ "text": t }] }
                    })
                })
                .collect();
            let resp = agent
                .post(&url)
                .set("Content-Type", "application/json")
                .send_json(json!({ "requests": requests }))
                .map_err(|e| CoreError::Network(format!("Error embeddings Gemini: {e}")))?;
            let val: Value = resp
                .into_json()
                .map_err(|e| CoreError::Network(format!("JSON inválido: {e}")))?;

            let mut out = Vec::with_capacity(texts.len());
            if let Some(arr) = val["embeddings"].as_array() {
                for item in arr {
                    let vec = item["values"]
                        .as_array()
                        .map(|vals| {
                            vals.iter()
                                .filter_map(|v| v.as_f64().map(|f| f as f32))
                                .collect::<Vec<f32>>()
                        })
                        .unwrap_or_default();
                    out.push(vec);
                }
            }
            return Ok(out);
        }

        if provider.contains("anthropic") || provider.contains("groq") {
            return Err(CoreError::InvalidInput(
                "Este proveedor no ofrece API de embeddings; usa un modelo local o búsqueda léxica."
                    .into(),
            ));
        }

        // OpenAI embeddings
        let resp = agent
            .post("https://api.openai.com/v1/embeddings")
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {key}"))
            .send_json(json!({
                "model": "text-embedding-3-small",
                "input": texts
            }))
            .map_err(|e| CoreError::Network(format!("Error embeddings OpenAI: {e}")))?;

        let val: Value = resp
            .into_json()
            .map_err(|e| CoreError::Network(format!("JSON inválido: {e}")))?;

        let mut out = Vec::with_capacity(texts.len());
        if let Some(arr) = val["data"].as_array() {
            for item in arr {
                let vec = item["embedding"]
                    .as_array()
                    .map(|vals| {
                        vals.iter()
                            .filter_map(|v| v.as_f64().map(|f| f as f32))
                            .collect::<Vec<f32>>()
                    })
                    .unwrap_or_default();
                out.push(vec);
            }
        }
        Ok(out)
    }

    /// Valida la conectividad y credenciales contra el proveedor configurado.
    pub fn test_connection(settings: &AiSettings) -> CoreResult<String> {
        let reply = Self::chat_completion(
            settings,
            "Responde únicamente con 'OK'.",
            "Prueba de conexión.",
        )?;
        Ok(format!(
            "Conexión exitosa con {} ({})",
            settings.provider,
            truncate_msg(&reply, 40)
        ))
    }
}

fn truncate_msg(s: &str, max: usize) -> String {
    let trimmed = s.trim();
    if trimmed.chars().count() <= max {
        trimmed.to_string()
    } else {
        format!("{}...", trimmed.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_client_payload_serialization() {
        for provider in ["OpenAI", "Anthropic", "Gemini", "Groq"] {
            let mut settings = AiSettings::default();
            settings.provider = provider.into();
            settings.api_key = "test-key".into();
            settings.model = String::new();

            let (url, body) = AiClient::build_chat_request(&settings, "Sys", "User").unwrap();
            assert!(!url.is_empty());
            assert!(body.is_object());
        }
    }
}
