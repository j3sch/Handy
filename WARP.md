# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Development Commands

**Prerequisites:**
- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

**Core Development:**
```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri dev
# If cmake error on macOS:
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# Build for production
bun run tauri build

# Frontend only development
bun run dev        # Start Vite dev server on port 1420
bun run build      # Build frontend (TypeScript + Vite)
bun run preview    # Preview built frontend
```

**Model Setup (Required for Development):**
```bash
# Create models directory and download VAD model
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
```

**Testing Audio Pipeline:**
```bash
# Build and run CLI tool for audio testing
cargo build --bin cli
cargo run --bin cli
```

## Architecture Overview

Handy is a cross-platform desktop speech-to-text application built with Tauri, combining Rust for system-level operations and React/TypeScript for the UI.

### Core Architecture

**Tauri App Structure:**
- **Frontend**: React + TypeScript + Tailwind CSS (port 1420 in dev)
- **Backend**: Rust with managers handling core functionality
- **Single Instance**: App prevents multiple instances, shows settings window instead

### Manager Pattern (Backend - `src-tauri/src/managers/`)

The application uses a manager-based architecture where each manager handles a specific domain:

- **`AudioRecordingManager`** - Audio device enumeration, recording, and real-time processing
- **`ModelManager`** - Whisper model downloading, caching, and lifecycle management  
- **`TranscriptionManager`** - Speech-to-text processing pipeline coordination
- **`MistralManager`** - Optional cloud-based transcription via Mistral API

### Audio Processing Pipeline

```
Microphone Input → VAD Filter → Audio Resampling → Whisper Model → Text Output
```

**Key Components:**
- **Voice Activity Detection (VAD)**: Uses Silero VAD model to filter silence
- **Audio Resampling**: Converts input to 16kHz mono for Whisper compatibility
- **Whisper Processing**: Local inference with GPU acceleration when available
- **Text Post-processing**: Custom word corrections and language translation

### Frontend Architecture (`src/`)

**Component Structure:**
- **`App.tsx`** - Main application with onboarding flow logic
- **`components/onboarding/`** - First-run model download experience
- **`components/settings/`** - Settings panels for shortcuts, audio, models
- **`components/model-selector/`** - Model management UI with download progress
- **`hooks/`** - React hooks for settings persistence and model state

**State Management:**
- Settings stored via Tauri's store plugin with reactive updates
- Model state managed through Tauri commands/events
- UI state handled by React hooks (`useSettings`, `useModels`)

### System Integration

**Global Shortcuts:**
- Configurable keybindings using `rdev` library
- Supports both toggle and push-to-talk modes
- Cross-platform shortcut handling with platform-specific considerations

**Tray Integration:**
- System tray with dynamic icons (idle/recording states)
- Context menu for quick actions and settings access
- Theme-aware icons that adapt to system appearance

**Platform Features:**
- **macOS**: Metal GPU acceleration, accessibility permissions, launch agents
- **Windows**: Vulkan acceleration, code signing integration
- **Linux**: OpenBLAS + Vulkan acceleration, ALSA audio support

### Model System

**Whisper Model Variants:**
- **Small**: Fast inference, good for most use cases (~39MB)
- **Medium**: Better accuracy, balanced performance (~77MB)  
- **Turbo**: Optimized large model with improved speed (~71MB)
- **Large**: Highest accuracy, slower processing (~155MB)

**Model Management:**
- Dynamic downloading from remote sources
- Local caching in application data directory
- Runtime model switching without restart
- Progress tracking for downloads with cancellation support

### Configuration Architecture

**Settings System:**
- JSON-based configuration with Zod schema validation
- Reactive updates across frontend and backend
- Platform-specific defaults and constraints
- Automatic migration for schema changes

**Key Settings Categories:**
- Audio devices (microphone/output selection)
- Keyboard shortcuts (customizable bindings)
- Model preferences and transcription options
- Debug options (word correction, overlay positioning)

### Build System

**Tauri Configuration:**
- Multi-platform builds with platform-specific optimizations
- Code signing on Windows via Azure trusted signing
- macOS notarization with hardened runtime
- Auto-updater integration with GitHub releases

**Development Tools:**
- Vite for fast frontend development with HMR
- TypeScript for type safety across the application
- Tailwind CSS v4 for styling
- Cargo for Rust dependency management and builds
