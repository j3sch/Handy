import React from "react";
import { ModelInfo } from "../../lib/types";
import { ProgressBar, ProgressData } from "../shared";

interface DownloadProgress {
  model_id: string;
  downloaded: number;
  total: number;
  percentage: number;
}

interface ModelDropdownProps {
  models: ModelInfo[];
  currentModelId: string;
  downloadProgress: Map<string, DownloadProgress>;
  onModelSelect: (modelId: string) => void;
  onModelDownload: (modelId: string) => void;
  onModelDelete: (modelId: string) => Promise<void>;
  onError?: (error: string) => void;
}

const ModelDropdown: React.FC<ModelDropdownProps> = ({
  models,
  currentModelId,
  downloadProgress,
  onModelSelect,
  onModelDownload,
  onModelDelete,
  onError,
}) => {
  const availableModels = models.filter((m) => m.is_downloaded);
  const downloadableModels = models.filter((m) => !m.is_downloaded);
  const isFirstRun = availableModels.length === 0 && models.length > 0;

  const handleDeleteClick = async (e: React.MouseEvent, modelId: string) => {
    e.preventDefault();
    e.stopPropagation();

    try {
      await onModelDelete(modelId);
    } catch (err) {
      const errorMsg = `Failed to delete model: ${err}`;
      onError?.(errorMsg);
    }
  };

  const handleModelClick = (modelId: string) => {
    if (downloadProgress.has(modelId)) {
      return; // Don't allow interaction while downloading
    }
    onModelSelect(modelId);
  };

  const handleDownloadClick = (modelId: string) => {
    if (downloadProgress.has(modelId)) {
      return; // Don't allow interaction while downloading
    }
    onModelDownload(modelId);
  };

  return (
    <div className="absolute bottom-full left-0 mb-2 w-64 bg-background border border-mid-gray/20 rounded-lg shadow-lg py-2 z-50">
      {/* First Run Welcome */}
      {isFirstRun && (
        <div className="px-3 py-2 bg-logo-primary/10 border-b border-logo-primary/20">
          <div className="text-xs font-medium text-logo-primary mb-1">
            Welcome to Handy!
          </div>
          <div className="text-xs text-text/70">
            Download a model below to get started with transcription.
          </div>
        </div>
      )}

      {/* Available Models */}
      {availableModels.length > 0 && (
        <div>
          <div className="px-3 py-1 text-xs font-medium text-text/80 border-b border-mid-gray/10">
            Available Models
          </div>
          {availableModels.map((model) => (
            <div
              key={model.id}
              onClick={() => handleModelClick(model.id)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  handleModelClick(model.id);
                }
              }}
              tabIndex={0}
              role="button"
              className={`w-full px-3 py-2 text-left hover:bg-mid-gray/10 transition-colors cursor-pointer focus:outline-none ${
                currentModelId === model.id
                  ? "bg-logo-primary/10 text-logo-primary"
                  : ""
              }`}
            >
              <div className="flex items-center justify-between">
                <div>
                  <div className="text-sm">{model.name}</div>
                  <div className="text-xs text-text/40 italic pr-4">
                    {model.description}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {currentModelId === model.id && (
                    <div className="text-xs text-logo-primary">Active</div>
                  )}
                  {currentModelId !== model.id && !["voxtral-mini", "nova-3"].includes(model.id) && (
                    <button
                      onClick={(e) => handleDeleteClick(e, model.id)}
                      className="text-red-400 hover:text-red-300 p-1 hover:bg-red-500/10 rounded transition-colors"
                      title={`Delete ${model.name}`}
                    >
                      <svg
                        className="w-3 h-3"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                      >
                        <path
                          fillRule="evenodd"
                          d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                          clipRule="evenodd"
                        />
                      </svg>
                    </button>
                  )}
                  {currentModelId !== model.id && ["voxtral-mini", "nova-3"].includes(model.id) && (
                    <div className="text-xs text-text/40">API</div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Downloadable Models */}
      {downloadableModels.length > 0 && (
        <div>
          {(availableModels.length > 0 || isFirstRun) && (
            <div className="border-t border-mid-gray/10 my-1" />
          )}
          <div className="px-3 py-1 text-xs font-medium text-text/80">
            {isFirstRun ? "Choose a Model" : "Download Models"}
          </div>
          {downloadableModels.map((model) => {
            const isDownloading = downloadProgress.has(model.id);
            const progress = downloadProgress.get(model.id);

            return (
              <div
                key={model.id}
                onClick={() => handleDownloadClick(model.id)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    handleDownloadClick(model.id);
                  }
                }}
                tabIndex={0}
                role="button"
                aria-disabled={isDownloading}
                className={`w-full px-3 py-2 text-left hover:bg-mid-gray/10 transition-colors cursor-pointer focus:outline-none ${
                  isDownloading
                    ? "opacity-50 cursor-not-allowed hover:bg-transparent"
                    : ""
                }`}
              >
                <div className="flex items-center justify-between">
                  <div>
                    <div className="text-sm">
                      {model.name}
                      {model.id === "small" && isFirstRun && (
                        <span className="ml-2 text-xs bg-logo-primary/20 text-logo-primary px-1.5 py-0.5 rounded">
                          Recommended
                        </span>
                      )}
                    </div>
                    <div className="text-xs text-text/40 italic pr-4">
                      {model.description}
                    </div>
                  </div>
                  <div className="text-xs text-logo-primary tabular-nums">
                    {isDownloading && progress ? (
                      `${Math.max(0, Math.min(100, Math.round(progress.percentage)))}%`
                    ) : (
                      "Download"
                    )}
                  </div>
                </div>

                {isDownloading && progress && (
                  <div className="mt-2">
                    <ProgressBar
                      progress={[{
                        id: model.id,
                        percentage: progress.percentage,
                        label: model.name
                      }]}
                      size="small"
                    />
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* No Models Available */}
      {availableModels.length === 0 && downloadableModels.length === 0 && (
        <div className="px-3 py-2 text-sm text-text/60">
          No models available
        </div>
      )}
    </div>
  );
};

export default ModelDropdown;
