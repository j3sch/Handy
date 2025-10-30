import React from "react";
import { SettingsGroup } from "../ui/SettingsGroup";
import MistralApiSettings from "./MistralApiSettings";
import DeepgramApiSettings from "./DeepgramApiSettings";
import AssemblyAIApiSettings from "./AssemblyAIApiSettings";
import GladiaApiSettings from "./GladiaApiSettings";

export const ApiSettings: React.FC = () => {
  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="Cloud Transcription">
        <MistralApiSettings />
        <DeepgramApiSettings />
        <AssemblyAIApiSettings />
        <GladiaApiSettings />
      </SettingsGroup>
    </div>
  );
};
