# Contributing to Koe

Thanks for your interest in contributing! Here's how to get set up.

## Prerequisites

- **macOS 13+** (Ventura or later)
- **Rust** (latest stable) — `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Node.js 18+** — `brew install node`
- **Xcode Command Line Tools** — `xcode-select --install`

## Quick Start

```bash
# Clone the repo
git clone https://github.com/izyuumi/koe.git
cd koe

# Install npm dependencies
npm install

# Build the Swift speech helper + run in dev mode
make dev
```

## Project Structure

```
koe/
├── src/                    # React frontend (HUD UI)
│   ├── App.tsx             # Main HUD component
│   ├── main.tsx            # Entry point
│   └── styles.css          # HUD styles
├── src-tauri/              # Tauri + Rust backend
│   ├── src/
│   │   ├── lib.rs          # App setup, commands, tray menu
│   │   ├── speech.rs       # Speech recognition (spawns Swift helper)
│   │   └── insertion.rs    # Text insertion via clipboard
│   ├── icons/              # App + tray icons
│   └── tauri.conf.json     # Tauri config
├── speech-helper/
│   └── main.swift          # macOS Speech framework bridge
└── Makefile                # Build orchestration
```

## Architecture

Koe is a **Tauri** app with three layers:

1. **React HUD** — A transparent, always-on-top overlay that shows dictation status and transcript
2. **Rust backend** — Manages global shortcuts, tray menu, and coordinates speech ↔ insertion
3. **Swift helper** — A sidecar binary that uses Apple's Speech framework for recognition (Rust can't call Speech.framework directly)

Communication: Rust spawns the Swift helper as a child process. The helper writes `PARTIAL:`, `FINAL:`, `LEVEL:`, and `ERROR:` lines to stdout, which Rust reads and forwards to the frontend via Tauri events.

## Key Permissions

Koe needs three macOS permissions to function:
- **Microphone** — for audio capture
- **Speech Recognition** — for on-device/cloud speech-to-text
- **Accessibility** — for typing into other apps via simulated keystrokes

## Building

```bash
make build    # Production build (outputs .app bundle)
make helper   # Rebuild just the Swift speech helper
make clean    # Clean all build artifacts
```

## Code Style

- Rust: `cargo fmt` and `cargo clippy`
- TypeScript/React: standard Vite + React conventions
- Commits: [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, etc.)

## Pull Requests

1. Branch off `main`
2. Keep changes focused — one feature/fix per PR
3. Test manually (automated tests coming soon)
4. Describe what changed and why
