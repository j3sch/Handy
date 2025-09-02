import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { SettingContainer } from "../ui/SettingContainer";

interface GladiaApiSettingsProps {
  onApiKeySet?: () => void;
}

const GladiaApiSettings: React.FC<GladiaApiSettingsProps> = ({ onApiKeySet }) => {
  const [apiKey, setApiKey] = useState("");
  const [hasApiKey, setHasApiKey] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadApiKeyStatus();
  }, []);

  const loadApiKeyStatus = async () => {
    try {
      const hasKey = await invoke<boolean>("has_gladia_api_key");
      setHasApiKey(hasKey);
      if (hasKey && showApiKey) {
        const key = await invoke<string | null>("get_gladia_api_key");
        setApiKey(key || "");
      }
    } catch (error) {
      console.error("Failed to load API key status:", error);
    }
  };

  const handleSaveApiKey = async () => {
    setLoading(true);
    try {
      await invoke("set_gladia_api_key", { apiKey });
      setHasApiKey(apiKey.length > 0);
      if (!apiKey) {
        setShowApiKey(false);
      }
      onApiKeySet?.();
    } catch (error) {
      console.error("Failed to save API key:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveApiKey = async () => {
    setLoading(true);
    try {
      await invoke("set_gladia_api_key", { apiKey: "" });
      setApiKey("");
      setHasApiKey(false);
      setShowApiKey(false);
    } catch (error) {
      console.error("Failed to remove API key:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <SettingContainer
      title="Gladia API Key"
      description="Required for Whisper-Zero transcription model"
    >
      <div className="space-y-2">
        {hasApiKey && !showApiKey ? (
          <div className="flex items-center gap-2">
            <span className="text-sm text-green-500">✓ API Key configured</span>
            <button
              onClick={() => setShowApiKey(true)}
              className="px-3 py-1 text-sm bg-gray-700 hover:bg-gray-600 rounded transition-colors"
            >
              Edit
            </button>
            <button
              onClick={handleRemoveApiKey}
              disabled={loading}
              className="px-3 py-1 text-sm bg-red-600 hover:bg-red-700 rounded transition-colors"
            >
              Remove
            </button>
          </div>
        ) : (
          <>
            <input
              type={showApiKey ? "text" : "password"}
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your Gladia API key..."
              className="w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded focus:outline-none focus:border-blue-500"
            />
            <div className="flex gap-2">
              <button
                onClick={handleSaveApiKey}
                disabled={loading || !apiKey}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded transition-colors"
              >
                {loading ? "Saving..." : "Save API Key"}
              </button>
              {hasApiKey && (
                <button
                  onClick={() => {
                    setShowApiKey(false);
                    setApiKey("");
                    loadApiKeyStatus();
                  }}
                  className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded transition-colors"
                >
                  Cancel
                </button>
              )}
            </div>
            <p className="text-xs text-gray-500">
              Get your API key from{" "}
              <a
                href="https://app.gladia.io/api-keys"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-400 hover:underline"
              >
                Gladia Console
              </a>
            </p>
          </>
        )}
      </div>
    </SettingContainer>
  );
};

export default GladiaApiSettings;