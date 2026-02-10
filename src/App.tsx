import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface DictationState {
  isListening: boolean;
  transcript: string;
  partialResult: string;
  micLevel: number;
  language: string;
  isOnDevice: boolean;
}

function App() {
  const [state, setState] = useState<DictationState>({
    isListening: false,
    transcript: "",
    partialResult: "",
    micLevel: 0,
    language: "en-US",
    isOnDevice: true,
  });

  useEffect(() => {
    // Listen for events from the Rust/Swift backend
    const unlisten = Promise.all([
      listen<{ text: string }>("transcript-partial", (e) => {
        setState((s) => ({ ...s, partialResult: e.payload.text }));
      }),
      listen<{ text: string }>("transcript-final", (e) => {
        setState((s) => ({
          ...s,
          transcript: e.payload.text,
          partialResult: "",
        }));
      }),
      listen<{ level: number }>("mic-level", (e) => {
        setState((s) => ({ ...s, micLevel: e.payload.level }));
      }),
      listen<{ listening: boolean }>("listening-state", (e) => {
        setState((s) => ({
          ...s,
          isListening: e.payload.listening,
          ...(e.payload.listening ? {} : { micLevel: 0 }),
        }));
      }),
    ]);

    return () => {
      unlisten.then((fns) => fns.forEach((fn) => fn()));
    };
  }, []);

  const displayText = state.partialResult || state.transcript;

  return (
    <div className="hud">
      <div className="status-row">
        <div className={`mic-indicator ${state.isListening ? "" : "idle"}`} />
        <div className="level-bar">
          <div
            className="level-fill"
            style={{ width: `${state.micLevel * 100}%` }}
          />
        </div>
        <span
          className={`on-device-badge ${state.isOnDevice ? "" : "cloud"}`}
        >
          {state.isOnDevice ? "On-Device" : "Cloud"}
        </span>
      </div>

      <div className={`transcript ${displayText ? "" : "placeholder"}`}>
        {displayText || "Press ⌥Space to start dictation…"}
      </div>

      <div className="controls">
        <span className="lang-badge">{state.language}</span>
        <span className="shortcut-hint">
          <kbd>⌥</kbd> + <kbd>Space</kbd>
        </span>
      </div>
    </div>
  );
}

export default App;
