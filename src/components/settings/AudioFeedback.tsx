import React from "react";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";
import { VolumeSlider } from "./VolumeSlider";
import { SoundPicker } from "./SoundPicker";

interface AudioFeedbackProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AudioFeedback: React.FC<AudioFeedbackProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { getSetting, updateSetting, isUpdating } = useSettings();
    const audioFeedbackEnabled = getSetting("audio_feedback") || false;

    return (
      <div className="flex flex-col">
        <ToggleSwitch
          checked={audioFeedbackEnabled}
          onChange={(enabled) => updateSetting("audio_feedback", enabled)}
          isUpdating={isUpdating("audio_feedback")}
          label="Audio Feedback"
          description="Play sound when recording starts and stops"
          descriptionMode={descriptionMode}
          grouped={grouped}
        />
      </div>
    );
  },
);
