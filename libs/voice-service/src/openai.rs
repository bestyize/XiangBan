use crate::{
    audio, security,
    server::{permit, ApiError, App, OPERATION_TIMEOUT},
    AudioBytes, Capability, ClientInfo, VoiceError,
};
use axum::{
    extract::{rejection::JsonRejection, Multipart, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

pub(crate) async fn transcriptions(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    multipart: Result<Multipart, axum::extract::multipart::MultipartRejection>,
) -> Result<Response, ApiError> {
    let mut multipart = multipart
        .map_err(|_| ApiError::bad("invalid_multipart", "expected multipart/form-data"))?;
    let mut file = None;
    let mut model = None;
    let mut language = None;
    let mut format = None;
    let mut seen = std::collections::HashSet::new();
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ApiError::bad("invalid_multipart", "invalid multipart field"))?
    {
        let name = field.name().unwrap_or("").to_owned();
        if !seen.insert(name.clone()) {
            return Err(ApiError::bad(
                "invalid_multipart",
                "duplicate multipart field",
            ));
        }
        match name.as_str() {
            "file" => {
                file = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| ApiError::bad("invalid_multipart", "invalid file"))?,
                )
            }
            "model" | "language" | "response_format" => {
                let value = field
                    .text()
                    .await
                    .map_err(|_| ApiError::bad("invalid_multipart", "expected UTF-8 text field"))?;
                match name.as_str() {
                    "model" => model = Some(value),
                    "language" => language = Some(value),
                    _ => format = Some(value),
                }
            }
            _ => (), // prompt and unknown extension fields are intentionally ignored.
        }
    }
    if model.is_none() {
        return Err(ApiError::bad(
            "missing_model",
            "model is required (any string accepted)",
        ));
    }
    let file = file.ok_or_else(|| ApiError::bad("missing_file", "file is required"))?;
    if file.is_empty() {
        return Err(ApiError::bad(
            "audio_too_short",
            "audio must be at least 0.1 seconds",
        ));
    }
    let format = format.as_deref().unwrap_or("json");
    if !matches!(format, "json" | "text") {
        return Err(ApiError::bad(
            "unsupported_response_format",
            "supported response formats: json, text",
        ));
    }
    let permit = permit(&app.transcription)?;
    app.approve(&security::client(&headers), &[Capability::Transcribe])
        .await?;
    // The permit remains in the blocking task if the HTTP caller disconnects.
    let (pcm, _permit) = tokio::task::spawn_blocking(move || (audio::decode(&file), permit))
        .await
        .map_err(|_| VoiceError::InvalidInput("decoder failed".into()))?;
    let pcm = pcm?;
    if pcm.samples.len() < audio::SAMPLE_RATE / 10 {
        return Err(ApiError::bad(
            "audio_too_short",
            "audio must be at least 0.1 seconds",
        ));
    }
    let text = app.transcribe(pcm, language.as_deref()).await?;
    Ok(if format == "text" {
        ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], text).into_response()
    } else {
        Json(json!({"text": text})).into_response()
    })
}

#[derive(Deserialize)]
pub(crate) struct SpeechRequest {
    // OpenAI clients supply arbitrary model values; the backend is injected.
    #[serde(rename = "model")]
    _model: Value,
    input: String,
    voice: Option<String>,
    response_format: Option<String>,
    speed: Option<f32>,
}

pub(crate) fn validate_text(text: &str) -> Result<(), ApiError> {
    if text.trim().is_empty() {
        return Err(ApiError::bad("empty_input", "input must not be empty"));
    }
    if text.chars().count() > 4000 {
        return Err(ApiError::bad(
            "input_too_long",
            "input exceeds 4000 characters",
        ));
    }
    Ok(())
}
pub(crate) async fn synthesize(
    app: &App,
    text: &str,
    voice: Option<&str>,
    speed: Option<f32>,
) -> Result<AudioBytes, ApiError> {
    validate_text(text)?;
    if speed.is_some_and(|v| !v.is_finite() || !(0.25..=4.0).contains(&v)) {
        return Err(ApiError::bad(
            "invalid_speed",
            "speed must be between 0.25 and 4.0",
        ));
    }
    let voices = tokio::time::timeout(OPERATION_TIMEOUT, app.backends.synthesizer.voices())
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::GATEWAY_TIMEOUT,
                "timeout",
                "voice listing timed out",
            )
        })??;
    let default = voices
        .first()
        .ok_or_else(|| VoiceError::Unavailable("no voices".into()))?;
    let selected = match voice {
        None
        | Some(
            "alloy" | "ash" | "ballad" | "coral" | "echo" | "fable" | "nova" | "onyx" | "sage"
            | "shimmer" | "verse" | "marin" | "cedar",
        ) => &default.id,
        Some(id) if voices.iter().any(|v| v.id == id) => id,
        Some(_) => {
            return Err(ApiError::bad(
                "unknown_voice",
                format!(
                    "unknown voice; available: {}",
                    voices
                        .iter()
                        .take(10)
                        .map(|v| v.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ))
        }
    };
    let audio = tokio::time::timeout(
        OPERATION_TIMEOUT,
        app.backends
            .synthesizer
            .synthesize(text, Some(selected), speed),
    )
    .await
    .map_err(|_| {
        ApiError::new(
            StatusCode::GATEWAY_TIMEOUT,
            "timeout",
            "synthesis timed out",
        )
    })??;
    if audio.mime != "audio/mpeg" || audio.bytes.is_empty() {
        return Err(VoiceError::Internal("synthesizer must return MP3".into()).into());
    }
    Ok(audio)
}

pub(crate) async fn speech(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    payload: Result<Json<SpeechRequest>, JsonRejection>,
) -> Result<Response, ApiError> {
    let Json(input) = payload.map_err(|_| {
        ApiError::bad(
            "invalid_json",
            "expected model, input and valid speech options",
        )
    })?;
    validate_text(&input.input)?;
    let format = input.response_format.as_deref().unwrap_or("mp3");
    if !matches!(format, "mp3" | "wav") {
        return Err(ApiError::bad(
            "unsupported_response_format",
            "supported response formats: mp3, wav",
        ));
    }
    app.approve(&security::client(&headers), &[Capability::Synthesize])
        .await?;
    let audio = synthesize(&app, &input.input, input.voice.as_deref(), input.speed).await?;
    if format == "mp3" {
        Ok(([(header::CONTENT_TYPE, "audio/mpeg")], audio.bytes).into_response())
    } else {
        let bytes = tokio::task::spawn_blocking(move || audio::wav(&audio::decode(&audio.bytes)?))
            .await
            .map_err(|_| VoiceError::Internal("MP3 decoder failed".into()))??;
        Ok(([(header::CONTENT_TYPE, "audio/wav")], bytes).into_response())
    }
}
pub(crate) async fn voices(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let client: ClientInfo = security::client(&headers);
    app.approve(&client, &[Capability::Synthesize]).await?;
    let voices = tokio::time::timeout(OPERATION_TIMEOUT, app.backends.synthesizer.voices())
        .await
        .map_err(|_| {
            ApiError::new(
                StatusCode::GATEWAY_TIMEOUT,
                "timeout",
                "voice listing timed out",
            )
        })??;
    Ok(Json(json!({"voices": voices})))
}
