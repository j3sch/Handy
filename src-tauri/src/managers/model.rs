use crate::settings::{get_settings, write_settings};
use anyhow::Result;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{App, AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub url: Option<String>,
    pub size_mb: u64,
    pub is_downloaded: bool,
    pub is_downloading: bool,
    pub partial_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

pub struct ModelManager {
    app_handle: AppHandle,
    models_dir: PathBuf,
    available_models: Mutex<HashMap<String, ModelInfo>>,
}

impl ModelManager {
    pub fn new(app: &App) -> Result<Self> {
        let app_handle = app.app_handle().clone();

        // Create models directory in app data
        let models_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| anyhow::anyhow!("Failed to get app data dir: {}", e))?
            .join("models");

        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)?;
        }

        let mut available_models = HashMap::new();

        available_models.insert(
            "small".to_string(),
            ModelInfo {
                id: "small".to_string(),
                name: "Whisper Small".to_string(),
                description: "Fast and efficient, great for most use cases".to_string(),
                filename: "ggml-small.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-small.bin".to_string()),
                size_mb: 244,
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
            },
        );

        // Add downloadable models
        available_models.insert(
            "medium".to_string(),
            ModelInfo {
                id: "medium".to_string(),
                name: "Whisper Medium".to_string(),
                description: "Good accuracy, medium speed".to_string(),
                filename: "whisper-medium-q4_1.bin".to_string(),
                url: Some("https://blob.handy.computer/whisper-medium-q4_1.bin".to_string()),
                size_mb: 491, // Approximate size
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
            },
        );

        available_models.insert(
            "turbo".to_string(),
            ModelInfo {
                id: "turbo".to_string(),
                name: "Whisper Turbo".to_string(),
                description: "Balanced accuracy and speed.".to_string(),
                filename: "ggml-large-v3-turbo.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-large-v3-turbo.bin".to_string()),
                size_mb: 1600, // Approximate size
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
            },
        );

        available_models.insert(
            "large".to_string(),
            ModelInfo {
                id: "large".to_string(),
                name: "Whisper Large".to_string(),
                description: "Highest accuracy, but slow.".to_string(),
                filename: "ggml-large-v3-q5_0.bin".to_string(),
                url: Some("https://blob.handy.computer/ggml-large-v3-q5_0.bin".to_string()),
                size_mb: 1080, // Approximate size
                is_downloaded: false,
                is_downloading: false,
                partial_size: 0,
            },
        );

        // Add Mistral Voxtral Mini Transcribe model (API-based)
        available_models.insert(
            "voxtral-mini".to_string(),
            ModelInfo {
                id: "voxtral-mini".to_string(),
                name: "Voxtral Mini Transcribe (API)".to_string(),
                description: "Fast cloud transcription via Mistral API - requires API key".to_string(),
                filename: "".to_string(), // No file needed for API model
                url: None,
                size_mb: 0,
                is_downloaded: true, // Always available when API key is set
                is_downloading: false,
                partial_size: 0,
            },
        );

        // Add Deepgram Nova-3 model (API-based)
        available_models.insert(
            "nova-3".to_string(),
            ModelInfo {
                id: "nova-3".to_string(),
                name: "Nova-3 (Deepgram API)".to_string(),
                description: "High-accuracy cloud transcription via Deepgram API - requires API key".to_string(),
                filename: "".to_string(), // No file needed for API model
                url: None,
                size_mb: 0,
                is_downloaded: true, // Always available when API key is set
                is_downloading: false,
                partial_size: 0,
            },
        );

        // Add AssemblyAI Universal model (API-based)
        available_models.insert(
            "universal".to_string(),
            ModelInfo {
                id: "universal".to_string(),
                name: "Universal (AssemblyAI API)".to_string(),
                description: "Universal speech recognition via AssemblyAI API - requires API key".to_string(),
                filename: "".to_string(), // No file needed for API model
                url: None,
                size_mb: 0,
                is_downloaded: true, // Always available when API key is set
                is_downloading: false,
                partial_size: 0,
            },
        );

        let manager = Self {
            app_handle,
            models_dir,
            available_models: Mutex::new(available_models),
        };

        // Migrate any bundled models to user directory
        manager.migrate_bundled_models()?;

        // Check which models are already downloaded
        manager.update_download_status()?;

        // Auto-select a model if none is currently selected
        manager.auto_select_model_if_needed()?;

        Ok(manager)
    }

    pub fn get_available_models(&self) -> Vec<ModelInfo> {
        let models = self.available_models.lock().unwrap();
        models.values().cloned().collect()
    }

    pub fn get_model_info(&self, model_id: &str) -> Option<ModelInfo> {
        let models = self.available_models.lock().unwrap();
        models.get(model_id).cloned()
    }

    fn migrate_bundled_models(&self) -> Result<()> {
        // Check for bundled models and copy them to user directory
        let bundled_models = ["ggml-small.bin"]; // Add other bundled models here if any

        for filename in &bundled_models {
            let bundled_path = self.app_handle.path().resolve(
                &format!("resources/models/{}", filename),
                tauri::path::BaseDirectory::Resource,
            );

            if let Ok(bundled_path) = bundled_path {
                if bundled_path.exists() {
                    let user_path = self.models_dir.join(filename);

                    // Only copy if user doesn't already have the model
                    if !user_path.exists() {
                        println!("Migrating bundled model {} to user directory", filename);
                        fs::copy(&bundled_path, &user_path)?;
                        println!("Successfully migrated {}", filename);
                    }
                }
            }
        }

        Ok(())
    }

    fn update_download_status(&self) -> Result<()> {
        let mut models = self.available_models.lock().unwrap();

        for model in models.values_mut() {
            // Skip API-based models
            if model.id == "voxtral-mini" || model.id == "nova-3" {
                continue;
            }

            let model_path = self.models_dir.join(&model.filename);
            let partial_path = self.models_dir.join(format!("{}.partial", &model.filename));

            model.is_downloaded = model_path.exists();
            model.is_downloading = partial_path.exists();

            // Get partial file size if it exists
            if partial_path.exists() {
                model.partial_size = partial_path.metadata().map(|m| m.len()).unwrap_or(0);
            } else {
                model.partial_size = 0;
            }
        }

        Ok(())
    }

    fn auto_select_model_if_needed(&self) -> Result<()> {
        // Check if we have a selected model in settings
        let settings = get_settings(&self.app_handle);

        // If no model is selected or selected model is empty
        if settings.selected_model.is_empty() {
            // Find the first available (downloaded) model
            let models = self.available_models.lock().unwrap();
            if let Some(available_model) = models.values().find(|model| model.is_downloaded) {
                println!(
                    "Auto-selecting model: {} ({})",
                    available_model.id, available_model.name
                );

                // Update settings with the selected model
                let mut updated_settings = settings;
                updated_settings.selected_model = available_model.id.clone();
                write_settings(&self.app_handle, updated_settings);

                println!("Successfully auto-selected model: {}", available_model.id);
            }
        }

        Ok(())
    }

    pub async fn download_model(&self, model_id: &str) -> Result<()> {
        let model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info =
            model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        let url = model_info
            .url
            .ok_or_else(|| anyhow::anyhow!("No download URL for model"))?;
        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));

        // Don't download if complete version already exists
        if model_path.exists() {
            // Clean up any partial file that might exist
            if partial_path.exists() {
                let _ = fs::remove_file(&partial_path);
            }
            self.update_download_status()?;
            return Ok(());
        }

        // Check if we have a partial download to resume
        let resume_from = if partial_path.exists() {
            let size = partial_path.metadata()?.len();
            println!("Resuming download of model {} from byte {}", model_id, size);
            size
        } else {
            println!("Starting fresh download of model {} from {}", model_id, url);
            0
        };

        // Mark as downloading
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = true;
            }
        }

        // Create HTTP client with range request for resuming
        let client = reqwest::Client::new();
        let mut request = client.get(&url);

        if resume_from > 0 {
            request = request.header("Range", format!("bytes={}-", resume_from));
        }

        let response = request.send().await?;

        // Check for success or partial content status
        if !response.status().is_success()
            && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
        {
            // Mark as not downloading on error
            {
                let mut models = self.available_models.lock().unwrap();
                if let Some(model) = models.get_mut(model_id) {
                    model.is_downloading = false;
                }
            }
            return Err(anyhow::anyhow!(
                "Failed to download model: HTTP {}",
                response.status()
            ));
        }

        let total_size = if resume_from > 0 {
            // For resumed downloads, add the resume point to content length
            resume_from + response.content_length().unwrap_or(0)
        } else {
            response.content_length().unwrap_or(0)
        };

        let mut downloaded = resume_from;
        let mut stream = response.bytes_stream();

        // Open file for appending if resuming, or create new if starting fresh
        let mut file = if resume_from > 0 {
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&partial_path)?
        } else {
            std::fs::File::create(&partial_path)?
        };

        // Emit initial progress
        let initial_progress = DownloadProgress {
            model_id: model_id.to_string(),
            downloaded,
            total: total_size,
            percentage: if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            },
        };
        let _ = self
            .app_handle
            .emit("model-download-progress", &initial_progress);

        // Download with progress
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                // Mark as not downloading on error
                {
                    let mut models = self.available_models.lock().unwrap();
                    if let Some(model) = models.get_mut(model_id) {
                        model.is_downloading = false;
                    }
                }
                e
            })?;

            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            let percentage = if total_size > 0 {
                (downloaded as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            // Emit progress event
            let progress = DownloadProgress {
                model_id: model_id.to_string(),
                downloaded,
                total: total_size,
                percentage,
            };

            let _ = self.app_handle.emit("model-download-progress", &progress);
        }

        file.flush()?;
        drop(file); // Ensure file is closed before moving

        // Move partial file to final location
        fs::rename(&partial_path, &model_path)?;

        // Update download status
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = false;
                model.is_downloaded = true;
                model.partial_size = 0;
            }
        }

        // Emit completion event
        let _ = self.app_handle.emit("model-download-complete", model_id);

        println!(
            "Successfully downloaded model {} to {:?}",
            model_id, model_path
        );

        Ok(())
    }

    pub fn delete_model(&self, model_id: &str) -> Result<()> {
        println!("ModelManager: delete_model called for: {}", model_id);

        let model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let model_info =
            model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        println!("ModelManager: Found model info: {:?}", model_info);

        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));
        println!("ModelManager: Model path: {:?}", model_path);
        println!("ModelManager: Partial path: {:?}", partial_path);

        let mut deleted_something = false;

        // Delete complete model file if it exists
        if model_path.exists() {
            println!("ModelManager: Deleting model file at: {:?}", model_path);
            fs::remove_file(&model_path)?;
            println!("ModelManager: Model file deleted successfully");
            deleted_something = true;
        }

        // Delete partial file if it exists
        if partial_path.exists() {
            println!("ModelManager: Deleting partial file at: {:?}", partial_path);
            fs::remove_file(&partial_path)?;
            println!("ModelManager: Partial file deleted successfully");
            deleted_something = true;
        }

        if !deleted_something {
            return Err(anyhow::anyhow!("No model files found to delete"));
        }

        // Update download status
        self.update_download_status()?;
        println!("ModelManager: Download status updated");

        Ok(())
    }

    pub fn get_model_path(&self, model_id: &str) -> Result<PathBuf> {
        // API-based models don't have local files
        if model_id == "voxtral-mini" || model_id == "nova-3" {
            return Ok(PathBuf::new()); // Return empty path for API models
        }

        let model_info = self
            .get_model_info(model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        if !model_info.is_downloaded {
            return Err(anyhow::anyhow!("Model not available: {}", model_id));
        }

        // Ensure we don't return partial files
        if model_info.is_downloading {
            return Err(anyhow::anyhow!(
                "Model is currently downloading: {}",
                model_id
            ));
        }

        let model_path = self.models_dir.join(&model_info.filename);
        let partial_path = self
            .models_dir
            .join(format!("{}.partial", &model_info.filename));

        // Ensure we only return complete model files, not partial ones
        if model_path.exists() && !partial_path.exists() {
            Ok(model_path)
        } else {
            Err(anyhow::anyhow!(
                "Complete model file not found: {}",
                model_id
            ))
        }
    }

    pub fn cancel_download(&self, model_id: &str) -> Result<()> {
        println!("ModelManager: cancel_download called for: {}", model_id);

        let _model_info = {
            let models = self.available_models.lock().unwrap();
            models.get(model_id).cloned()
        };

        let _model_info =
            _model_info.ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))?;

        // Mark as not downloading
        {
            let mut models = self.available_models.lock().unwrap();
            if let Some(model) = models.get_mut(model_id) {
                model.is_downloading = false;
            }
        }

        // Note: The actual download cancellation would need to be handled
        // by the download task itself. This just updates the state.
        // The partial file is kept so the download can be resumed later.

        // Update download status to reflect current state
        self.update_download_status()?;

        println!("ModelManager: Download cancelled for: {}", model_id);
        Ok(())
    }
}
