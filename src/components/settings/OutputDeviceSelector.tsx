import React from "react";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";
import { ResetButton } from "../ui/ResetButton";
import { useSettings } from "../../hooks/useSettings";

interface OutputDeviceSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const OutputDeviceSelector: React.FC<OutputDeviceSelectorProps> = React.memo(({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const {
    getSetting,
    updateSetting,
    resetSetting,
    isUpdating,
    isLoading,
    outputDevices,
    refreshOutputDevices,
  } = useSettings();

  const selectedOutputDevice = getSetting("selected_output_device") === "default" ? "Default" : getSetting("selected_output_device") || "Default";

  const handleOutputDeviceSelect = async (deviceName: string) => {
    await updateSetting("selected_output_device", deviceName);
  };

  const handleReset = async () => {
    await resetSetting("selected_output_device");
  };

  const outputDeviceOptions = outputDevices.map(device => ({
    value: device.name,
    label: device.name
  }));

  return (
    <SettingContainer
      title="Output Device"
      description="Select your preferred audio output device for feedback sounds"
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex items-center space-x-1">
        <Dropdown
          options={outputDeviceOptions}
          selectedValue={selectedOutputDevice}
          onSelect={handleOutputDeviceSelect}
          placeholder={isLoading || outputDevices.length === 0 ? "Loading..." : "Select output device..."}
          disabled={isUpdating("selected_output_device") || isLoading || outputDevices.length === 0}
          onRefresh={refreshOutputDevices}
        />
        <ResetButton
          onClick={handleReset}
          disabled={isUpdating("selected_output_device") || isLoading}
        />
      </div>
    </SettingContainer>
  );
});
