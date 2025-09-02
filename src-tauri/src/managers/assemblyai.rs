use crate::settings::get_settings;
use anyhow::Result;
use serde::Deserialize;
use tauri::AppHandle;
use log::{debug, info, error};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct AssemblyAIUploadResponse {
    upload_url: String,
}

#[derive(Debug, Deserialize)]
struct AssemblyAITranscriptResponse {
    id: String,
}

#[derive(Debug, Deserialize)]
struct AssemblyAITranscriptStatus {
    status: String,
    text: Option<String>,
    error: Option<String>,
}

pub struct AssemblyAIApiManager {
    app_handle: AppHandle,
    client: reqwest::Client,
}

impl AssemblyAIApiManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            client: reqwest::Client::new(),
        }
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        info!("[AssemblyAI] Starting transcription with {} audio samples", audio_data.len());
        
        let settings = get_settings(&self.app_handle);
        let api_key = settings.assemblyai_api_key.ok_or_else(|| {
            error!("[AssemblyAI] API key not set in settings");
            anyhow::anyhow!("AssemblyAI API key not set")
        })?;
        
        debug!("[AssemblyAI] API key found, length: {} chars", api_key.len());

        // Convert f32 audio to wav in memory
        info!("[AssemblyAI] Converting audio data to WAV format");
        let wav_data = float_to_wav(&audio_data)?;
        info!("[AssemblyAI] WAV data created: {} bytes", wav_data.len());

        // Step 1: Upload audio file
        info!("[AssemblyAI] Uploading audio to AssemblyAI");
        let upload_response = self
            .client
            .post("https://api.assemblyai.com/v2/upload")
            .header("authorization", &api_key)
            .body(wav_data)
            .send()
            .await
            .map_err(|e| {
                error!("[AssemblyAI] Failed to upload audio: {}", e);
                e
            })?;

        let status = upload_response.status();
        if !status.is_success() {
            let error_text = upload_response.text().await?;
            error!("[AssemblyAI] Upload failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "AssemblyAI upload failed with status {}: {}",
                status,
                error_text
            ));
        }

        let upload_result: AssemblyAIUploadResponse = upload_response.json().await?;
        let audio_url = upload_result.upload_url;
        info!("[AssemblyAI] Audio uploaded successfully: {}", audio_url);

        // Step 2: Submit transcription request
        // Convert app language setting to AssemblyAI language code
        let language_code = convert_to_assemblyai_language(&settings.selected_language);
        debug!("[AssemblyAI] Using language code: {}", language_code);
        
        let mut transcript_request = serde_json::json!({
            "audio_url": audio_url,
            "speech_model": "universal"
        });
        
        // Only add language_code if it's not "auto"
        if language_code != "auto" {
            transcript_request["language_code"] = serde_json::Value::String(language_code);
        }

        info!("[AssemblyAI] Submitting transcription request");
        debug!("[AssemblyAI] URL: https://api.assemblyai.com/v2/transcript");
        debug!("[AssemblyAI] Model: universal");

        let transcript_response = self
            .client
            .post("https://api.assemblyai.com/v2/transcript")
            .header("authorization", &api_key)
            .header("Content-Type", "application/json")
            .json(&transcript_request)
            .send()
            .await
            .map_err(|e| {
                error!("[AssemblyAI] Failed to submit transcription request: {}", e);
                e
            })?;

        let status = transcript_response.status();
        if !status.is_success() {
            let error_text = transcript_response.text().await?;
            error!("[AssemblyAI] Transcription request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "AssemblyAI transcription request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let transcript_result: AssemblyAITranscriptResponse = transcript_response.json().await?;
        let transcript_id = transcript_result.id;
        info!("[AssemblyAI] Transcription job submitted with ID: {}", transcript_id);

        // Step 3: Poll for completion
        let polling_url = format!("https://api.assemblyai.com/v2/transcript/{}", transcript_id);
        
        loop {
            debug!("[AssemblyAI] Polling transcription status");
            let polling_response = self
                .client
                .get(&polling_url)
                .header("authorization", &api_key)
                .send()
                .await
                .map_err(|e| {
                    error!("[AssemblyAI] Failed to poll transcription status: {}", e);
                    e
                })?;

            let status = polling_response.status();
            if !status.is_success() {
                let error_text = polling_response.text().await?;
                error!("[AssemblyAI] Polling failed with status {}: {}", status, error_text);
                return Err(anyhow::anyhow!(
                    "AssemblyAI polling failed with status {}: {}",
                    status,
                    error_text
                ));
            }

            let status_result: AssemblyAITranscriptStatus = polling_response.json().await?;
            
            match status_result.status.as_str() {
                "completed" => {
                    let transcript = status_result.text.unwrap_or_default();
                    info!("[AssemblyAI] Transcription successful: {}", transcript);
                    return Ok(transcript);
                },
                "error" => {
                    let error_msg = status_result.error.unwrap_or("Unknown error".to_string());
                    error!("[AssemblyAI] Transcription failed: {}", error_msg);
                    return Err(anyhow::anyhow!("AssemblyAI transcription failed: {}", error_msg));
                },
                _ => {
                    debug!("[AssemblyAI] Transcription status: {}, waiting...", status_result.status);
                    sleep(Duration::from_secs(3)).await;
                }
            }
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

fn convert_to_assemblyai_language(app_language: &str) -> String {
    match app_language {
        "auto" => "auto".to_string(),
        "en" => "en_us".to_string(),
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
        _ => "en_us".to_string(),
    }
}