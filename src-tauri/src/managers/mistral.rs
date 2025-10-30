use crate::settings::get_settings;
use anyhow::Result;
use reqwest::multipart;
use serde::Deserialize;
use tauri::AppHandle;
use log::{debug, info, error};

#[derive(Debug, Deserialize)]
struct MistralTranscriptionResponse {
    text: String,
}

#[derive(Clone)]
pub struct MistralApiManager {
    app_handle: AppHandle,
    client: reqwest::Client,
}

impl MistralApiManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            client: reqwest::Client::new(),
        }
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        info!("[Mistral] Starting transcription with {} audio samples", audio_data.len());
        
        let settings = get_settings(&self.app_handle);
        let api_key = settings.mistral_api_key.ok_or_else(|| {
            error!("[Mistral] API key not set in settings");
            anyhow::anyhow!("Mistral API key not set")
        })?;
        
        debug!("[Mistral] API key found, length: {} chars", api_key.len());

        // Convert f32 audio to wav in memory
        info!("[Mistral] Converting audio data to WAV format");
        let wav_data = float_to_wav(&audio_data)?;
        info!("[Mistral] WAV data created: {} bytes", wav_data.len());

        let part = multipart::Part::bytes(wav_data)
            .file_name("audio.wav")
            .mime_str("audio/wav")?;
        let form = multipart::Form::new()
            .part("file", part)
            .text("model", "voxtral-mini-latest");

        info!("[Mistral] Sending request to Mistral API endpoint");
        debug!("[Mistral] URL: https://api.mistral.ai/v1/audio/transcriptions");
        debug!("[Mistral] Model: voxtral-mini-latest");
        
        let response = self
            .client
            .post("https://api.mistral.ai/v1/audio/transcriptions")
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                error!("[Mistral] Failed to send request: {}", e);
                e
            })?;
        
        info!("[Mistral] Received response with status: {}", response.status());

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("[Mistral] API request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "Mistral API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        debug!("[Mistral] Parsing JSON response");
        let response_text = response.text().await?;
        debug!("[Mistral] Raw response: {}", response_text);
        
        let transcription: MistralTranscriptionResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("[Mistral] Failed to parse response JSON: {}", e);
                error!("[Mistral] Response was: {}", response_text);
                anyhow::anyhow!("Failed to parse Mistral response: {}", e)
            })?;
        
        info!("[Mistral] Transcription successful: {}", transcription.text);
        Ok(transcription.text)
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
