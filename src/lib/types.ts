import { z } from "zod";

export const ShortcutBindingSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  default_binding: z.string(),
  current_binding: z.string(),
});

export const ShortcutBindingsMapSchema = z.record(
  z.string(),
  ShortcutBindingSchema,
);

export const AudioDeviceSchema = z.object({
  index: z.string(),
  name: z.string(),
  is_default: z.boolean(),
});

export const OverlayPositionSchema = z.enum(["none", "top", "bottom"]);
export type OverlayPosition = z.infer<typeof OverlayPositionSchema>;

export const ModelUnloadTimeoutSchema = z.enum([
  "never",
  "immediately",
  "min2",
  "min5",
  "min10",
  "min15",
  "hour1",
  "sec5",
]);
export type ModelUnloadTimeout = z.infer<typeof ModelUnloadTimeoutSchema>;

export const PasteMethodSchema = z.enum(["ctrl_v", "direct"]);
export type PasteMethod = z.infer<typeof PasteMethodSchema>;

export const ClipboardHandlingSchema = z.enum(["dont_modify", "copy_to_clipboard"]);
export type ClipboardHandling = z.infer<typeof ClipboardHandlingSchema>;

export const SettingsSchema = z.object({
  bindings: ShortcutBindingsMapSchema,
  push_to_talk: z.boolean(),
  audio_feedback: z.boolean(),
  audio_feedback_volume: z.number().optional().default(1.0),
  sound_theme: z
    .enum(["marimba", "pop", "custom"])
    .optional()
    .default("marimba"),
  start_hidden: z.boolean().optional().default(false),
  autostart_enabled: z.boolean().optional().default(false),
  selected_model: z.string(),
  always_on_microphone: z.boolean(),
  selected_microphone: z.string().nullable().optional(),
  selected_output_device: z.string().nullable().optional(),
  translate_to_english: z.boolean(),
  selected_language: z.string(),
  overlay_position: OverlayPositionSchema,
  debug_mode: z.boolean(),
  custom_words: z.array(z.string()).optional().default([]),
  model_unload_timeout: ModelUnloadTimeoutSchema.optional().default("never"),
  word_correction_threshold: z.number().optional().default(0.18),
  mistral_api_key: z.string().nullable().optional(),
  deepgram_api_key: z.string().nullable().optional(),
  assemblyai_api_key: z.string().nullable().optional(),
  gladia_api_key: z.string().nullable().optional(),
  transcription_provider: z.string().optional(),
  history_limit: z.number().optional().default(5),
  paste_method: PasteMethodSchema.optional().default("ctrl_v"),
  clipboard_handling: ClipboardHandlingSchema.optional().default("dont_modify"),
  mute_while_recording: z.boolean().optional().default(false),
});

export const BindingResponseSchema = z.object({
  success: z.boolean(),
  binding: ShortcutBindingSchema.nullable(),
  error: z.string().nullable(),
});

export type AudioDevice = z.infer<typeof AudioDeviceSchema>;
export type BindingResponse = z.infer<typeof BindingResponseSchema>;
export type ShortcutBinding = z.infer<typeof ShortcutBindingSchema>;
export type ShortcutBindingsMap = z.infer<typeof ShortcutBindingsMapSchema>;
export type Settings = z.infer<typeof SettingsSchema>;

export const ModelInfoSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  filename: z.string(),
  url: z.string().optional(),
  size_mb: z.number(),
  is_downloaded: z.boolean(),
  is_downloading: z.boolean(),
  partial_size: z.number(),
  is_directory: z.boolean(),
  accuracy_score: z.number(),
  speed_score: z.number(),
});

export type ModelInfo = z.infer<typeof ModelInfoSchema>;
