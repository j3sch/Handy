use crate::settings::get_settings;
use anyhow::Result;
use reqwest::multipart;
use serde::Deserialize;
use tauri::AppHandle;
use log::{debug, info, error};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct GladiaUploadResponse {
    audio_url: String,
}

#[derive(Debug, Deserialize)]
struct GladiaTranscriptionResponse {
    id: String,
    result_url: String,
}

#[derive(Debug, Deserialize)]
struct GladiaTranscriptionResult {
    result: GladiaResult,
}

#[derive(Debug, Deserialize)]
struct GladiaResult {
    transcription: GladiaTranscription,
}

#[derive(Debug, Deserialize)]
struct GladiaTranscription {
    full_transcript: Option<String>,
}

pub struct GladiaApiManager {
    app_handle: AppHandle,
    client: reqwest::Client,
}

impl GladiaApiManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            client: reqwest::Client::new(),
        }
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        info!("[Gladia] Starting transcription with {} audio samples", audio_data.len());
        
        let settings = get_settings(&self.app_handle);
        let api_key = settings.gladia_api_key.ok_or_else(|| {
            error!("[Gladia] API key not set in settings");
            anyhow::anyhow!("Gladia API key not set")
        })?;
        
        debug!("[Gladia] API key found, length: {} chars", api_key.len());

        // Convert f32 audio to wav in memory
        info!("[Gladia] Converting audio data to WAV format");
        let wav_data = float_to_wav(&audio_data)?;
        info!("[Gladia] WAV data created: {} bytes", wav_data.len());

        // Step 1: Upload audio file
        info!("[Gladia] Uploading audio to Gladia");
        let part = multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;
        let form = multipart::Form::new().part("audio", part);

        let upload_response = self
            .client
            .post("https://api.gladia.io/v2/upload")
            .header("x-gladia-key", &api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("[Gladia] Failed to upload audio: {}", e);
                e
            })?;

        let status = upload_response.status();
        if !status.is_success() {
            let error_text = upload_response.text().await?;
            error!("[Gladia] Upload failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "Gladia upload failed with status {}: {}",
                status,
                error_text
            ));
        }

        let upload_result: GladiaUploadResponse = upload_response.json().await?;
        let audio_url = upload_result.audio_url;
        info!("[Gladia] Audio uploaded successfully: {}", audio_url);

        // Step 2: Submit transcription request
        // Convert app language setting to Gladia language code
        let language_code = convert_to_gladia_language(&settings.selected_language);
        debug!("[Gladia] Using language code: {}", language_code);
        
        let mut transcript_request = serde_json::json!({
            "audio_url": audio_url,
            "detect_language": language_code == "auto"
        });
        
        // Only add language if not auto-detecting
        if language_code != "auto" {
            transcript_request["language"] = serde_json::Value::String(language_code);
        }

        info!("[Gladia] Submitting transcription request");
        debug!("[Gladia] URL: https://api.gladia.io/v2/pre-recorded");
        debug!("[Gladia] Model: Whisper-Zero");

        let transcript_response = self
            .client
            .post("https://api.gladia.io/v2/pre-recorded")
            .header("x-gladia-key", &api_key)
            .header("Content-Type", "application/json")
            .json(&transcript_request)
            .send()
            .await
            .map_err(|e| {
                error!("[Gladia] Failed to submit transcription request: {}", e);
                e
            })?;

        let status = transcript_response.status();
        if !status.is_success() {
            let error_text = transcript_response.text().await?;
            error!("[Gladia] Transcription request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "Gladia transcription request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let transcript_result: GladiaTranscriptionResponse = transcript_response.json().await?;
        let transcript_id = transcript_result.id;
        let result_url = transcript_result.result_url;
        info!("[Gladia] Transcription job submitted with ID: {}", transcript_id);

        // Step 3: Poll for completion
        loop {
            debug!("[Gladia] Polling transcription status");
            let polling_response = self
                .client
                .get(&result_url)
                .header("x-gladia-key", &api_key)
                .send()
                .await
                .map_err(|e| {
                    error!("[Gladia] Failed to poll transcription status: {}", e);
                    e
                })?;

            let status = polling_response.status();
            if !status.is_success() {
                let error_text = polling_response.text().await?;
                error!("[Gladia] Polling failed with status {}: {}", status, error_text);
                return Err(anyhow::anyhow!(
                    "Gladia polling failed with status {}: {}",
                    status,
                    error_text
                ));
            }

            // Try to parse as completed result
            let response_text = polling_response.text().await?;
            debug!("[Gladia] Raw response: {}", response_text);
            
            // Check if the response contains a completed transcription
            if let Ok(status_result) = serde_json::from_str::<GladiaTranscriptionResult>(&response_text) {
                if let Some(transcript) = status_result.result.transcription.full_transcript {
                    info!("[Gladia] Transcription successful: {}", transcript);
                    return Ok(transcript);
                }
            }
            
            // If we get here, the transcription is still processing
            debug!("[Gladia] Transcription still processing, waiting...");
            sleep(Duration::from_secs(2)).await;
        }
    }
}

fn float_to_wav(audio_data: &[f32]) -> Result<Vec<u8>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut cursor = std::io::Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, spec)?;
    for &sample in audio_data {
        let amplitude = (sample * i16::MAX as f32) as i16;
        writer.write_sample(amplitude)?;
    }
    writer.finalize()?;
    Ok(cursor.into_inner())
}

fn convert_to_gladia_language(app_language: &str) -> String {
    match app_language {
        "auto" => "auto".to_string(),
        "en" => "en".to_string(),
        "es" => "es".to_string(),
        "fr" => "fr".to_string(),
        "de" => "de".to_string(),
        "it" => "it".to_string(),
        "pt" => "pt".to_string(),
        "nl" => "nl".to_string(),
        "hi" => "hi".to_string(),
        "ja" => "ja".to_string(),
        "ko" => "ko".to_string(),
        "pl" => "pl".to_string(),
        "ru" => "ru".to_string(),
        "tr" => "tr".to_string(),
        "vi" => "vi".to_string(),
        "uk" => "uk".to_string(),
        "zh" => "zh".to_string(),
        "ar" => "ar".to_string(),
        "ca" => "ca".to_string(),
        "cs" => "cs".to_string(),
        "da" => "da".to_string(),
        "fi" => "fi".to_string(),
        "el" => "el".to_string(),
        "he" => "he".to_string(),
        "hu" => "hu".to_string(),
        "id" => "id".to_string(),
        "ms" => "ms".to_string(),
        "no" => "no".to_string(),
        "ro" => "ro".to_string(),
        "sk" => "sk".to_string(),
        "sv" => "sv".to_string(),
        "th" => "th".to_string(),
        "ur" => "ur".to_string(),
        "fa" => "fa".to_string(),
        "bg" => "bg".to_string(),
        "hr" => "hr".to_string(),
        "et" => "et".to_string(),
        "lv" => "lv".to_string(),
        "lt" => "lt".to_string(),
        "mk" => "mk".to_string(),
        "sl" => "sl".to_string(),
        "sr" => "sr".to_string(),
        "az" => "az".to_string(),
        "bn" => "bn".to_string(),
        "kn" => "kn".to_string(),
        "ml" => "ml".to_string(),
        "ta" => "ta".to_string(),
        "te" => "te".to_string(),
        "cy" => "cy".to_string(),
        // Fallback to English for unsupported languages
        _ => "en".to_string(),
    }
}