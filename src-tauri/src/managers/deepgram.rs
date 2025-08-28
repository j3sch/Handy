use crate::settings::get_settings;
use anyhow::Result;
use serde::Deserialize;
use tauri::AppHandle;
use log::{debug, info, error};

#[derive(Debug, Deserialize)]
struct DeepgramTranscriptionResponse {
    results: DeepgramResults,
}

#[derive(Debug, Deserialize)]
struct DeepgramResults {
    channels: Vec<DeepgramChannel>,
}

#[derive(Debug, Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Deserialize)]
struct DeepgramAlternative {
    transcript: String,
}

pub struct DeepgramApiManager {
    app_handle: AppHandle,
    client: reqwest::Client,
}

impl DeepgramApiManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            client: reqwest::Client::new(),
        }
    }

    pub async fn transcribe(&self, audio_data: Vec<f32>) -> Result<String> {
        info!("[Deepgram] Starting transcription with {} audio samples", audio_data.len());
        
        let settings = get_settings(&self.app_handle);
        let api_key = settings.deepgram_api_key.ok_or_else(|| {
            error!("[Deepgram] API key not set in settings");
            anyhow::anyhow!("Deepgram API key not set")
        })?;
        
        debug!("[Deepgram] API key found, length: {} chars", api_key.len());

        // Convert f32 audio to wav in memory
        info!("[Deepgram] Converting audio data to WAV format");
        let wav_data = float_to_wav(&audio_data)?;
        info!("[Deepgram] WAV data created: {} bytes", wav_data.len());

        info!("[Deepgram] Sending request to Deepgram API endpoint");
        debug!("[Deepgram] URL: https://api.deepgram.com/v1/listen");
        debug!("[Deepgram] Model: nova-3");
        
        let response = self
            .client
            .post("https://api.deepgram.com/v1/listen")
            .query(&[
                ("model", "nova-3"),
                ("smart_format", "true"),
                ("language", "multi")
            ])
            .header("Authorization", format!("Token {}", api_key))
            .header("Content-Type", "audio/wav")
            .body(wav_data)
            .send()
            .await
            .map_err(|e| {
                error!("[Deepgram] Failed to send request: {}", e);
                e
            })?;
        
        info!("[Deepgram] Received response with status: {}", response.status());

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("[Deepgram] API request failed with status {}: {}", status, error_text);
            return Err(anyhow::anyhow!(
                "Deepgram API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        debug!("[Deepgram] Parsing JSON response");
        let response_text = response.text().await?;
        debug!("[Deepgram] Raw response: {}", response_text);
        
        let transcription: DeepgramTranscriptionResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("[Deepgram] Failed to parse response JSON: {}", e);
                error!("[Deepgram] Response was: {}", response_text);
                anyhow::anyhow!("Failed to parse Deepgram response: {}", e)
            })?;
        
        // Extract transcript from Deepgram response structure
        let transcript = transcription
            .results
            .channels
            .first()
            .and_then(|channel| channel.alternatives.first())
            .map(|alternative| alternative.transcript.clone())
            .unwrap_or_default();
        
        info!("[Deepgram] Transcription successful: {}", transcript);
        Ok(transcript)
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