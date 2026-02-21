# Koe (声)

[![CI](https://github.com/izyuumi/koe/actions/workflows/ci.yml/badge.svg)](https://github.com/izyuumi/koe/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

System-wide dictation for macOS. No account, no cloud (when possible), just your voice → text.

## Features

- **Global hotkey** (⌥Space) — toggle dictation from any app
- **Apple Speech framework** — prefers on-device recognition
- **Text insertion** — pastes transcribed text at cursor
- **Menu bar app** — lives in your tray, out of the way
- **Zero accounts** — no signup, no login, no subscription

## Prerequisites

- macOS 13+ (Ventura)
- Rust + Cargo
- Node.js 18+
- Xcode Command Line Tools

## Setup

```bash
npm install
make helper    # Compile Swift speech helper
make icons     # Generate placeholder icons
```

## Development

```bash
make dev       # Run in dev mode (hot reload)
```

## Build

```bash
make build     # Production build (.app bundle)
```

## Architecture

```
Tauri (Rust)          Swift Helper Process
┌─────────────┐      ┌─────────────────┐
│ Global HK   │─────▶│ SFSpeechRecog   │
│ Tray Icon   │      │ AVAudioEngine   │
│ Window Mgmt │◀─────│ Mic Level       │
│ Text Insert  │      └─────────────────┘
└──────┬──────┘
       │
  WebView (React)
┌─────────────┐
│ HUD overlay │
│ Transcript  │
│ Settings    │
└─────────────┘
```

## Permissions Required

- **Microphone** — for audio input
- **Speech Recognition** — for Apple's STT
- **Accessibility** — for text insertion into other apps

## Install

```bash
brew install --cask izyuumi/tap/koe
```

## Roadmap

- [x] Streaming recognition (live transcript)
- [x] Language toggle (en-US ↔ ja-JP)
- [x] On-device / cloud toggle
- [ ] Personal dictionary
- [ ] Filler word removal
- [ ] App-specific profiles
- [ ] Local LLM post-processing (optional)
