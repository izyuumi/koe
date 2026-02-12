import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface DictationState {
  isListening: boolean;
  transcript: string;
  partialResult: string;
  micLevel: number;
  language: string;
  isOnDevice: boolean;
  error: string | null;
}

function App() {
  const [showOnboarding, setShowOnboarding] = useState(() => {
    return localStorage.getItem("koe.onboarding.permissions.v1") !== "done";
  });
  const [onboardingError, setOnboardingError] = useState<string | null>(null);

  // Load persisted settings
  const [language, setLanguage] = useState<string>(() => {
    return localStorage.getItem("koe.language") || "en-US";
  });
  const [isOnDevice, setIsOnDevice] = useState<boolean>(() => {
    return localStorage.getItem("koe.onDevice") !== "false";
  });

  const [state, setState] = useState<DictationState>({
    isListening: false,
    transcript: "",
    partialResult: "",
    micLevel: 0,
    language,
    isOnDevice,
    error: null,
  });

  // Persist settings to localStorage
  useEffect(() => {
    localStorage.setItem("koe.language", language);
    setState((s) => ({ ...s, language }));
  }, [language]);

  useEffect(() => {
    localStorage.setItem("koe.onDevice", String(isOnDevice));
    setState((s) => ({ ...s, isOnDevice }));
  }, [isOnDevice]);

  // Push language setting to backend
  const updateBackendSettings = useCallback(async () => {
    try {
      await invoke("set_dictation_settings", { language, onDevice: isOnDevice });
    } catch {
      // Backend may not be ready yet
    }
  }, [language, isOnDevice]);

  useEffect(() => {
    updateBackendSettings();
  }, [updateBackendSettings]);

  useEffect(() => {
    const unlisten = Promise.all([
      listen<{ text: string }>("transcript-partial", (e) => {
        setState((s) => ({ ...s, partialResult: e.payload.text, error: null }));
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
          ...(e.payload.listening ? { error: null } : { micLevel: 0 }),
        }));
      }),
      listen<{ message: string }>("speech-error", (e) => {
        setState((s) => ({
          ...s,
          error: e.payload.message,
          isListening: false,
          micLevel: 0,
        }));
      }),
    ]);

    return () => {
      unlisten.then((fns) => fns.forEach((fn) => fn()));
    };
  }, []);

  const toggleLanguage = () => {
    setLanguage((l) => (l === "en-US" ? "ja-JP" : "en-US"));
  };

  const toggleOnDevice = () => {
    setIsOnDevice((v) => !v);
  };

  const displayText = state.partialResult || state.transcript;
  const isPartial = !!state.partialResult;
  const micLevelWidth = `${Math.max(0, Math.min(state.micLevel, 1)) * 100}%`;
  const statusText = state.error
    ? "Error"
    : state.isListening
      ? "Listening"
      : "Idle";
  const transcriptHint = state.isListening
    ? "Start speaking. Your words appear here."
    : "Press Option + Space to start dictation.";

  const markOnboardingDone = () => {
    localStorage.setItem("koe.onboarding.permissions.v1", "done");
    setShowOnboarding(false);
    setOnboardingError(null);
  };

  const openMicSettings = async () => {
    try {
      await invoke("open_microphone_settings");
      setOnboardingError(null);
    } catch {
      setOnboardingError("Could not open Microphone settings.");
    }
  };

  const openSpeechSettings = async () => {
    try {
      await invoke("open_speech_settings");
      setOnboardingError(null);
    } catch {
      setOnboardingError("Could not open Speech Recognition settings.");
    }
  };

  return (
    <div className="hud" role="application" aria-label="Koe dictation HUD">
      {showOnboarding ? (
        <section className="onboarding" aria-label="Permission onboarding">
          <p className="onboarding-title">Enable Permissions</p>
          <p className="onboarding-copy">
            Koe needs access to Microphone and Speech Recognition.
          </p>
          <div className="onboarding-actions">
            <button type="button" className="onboarding-btn" onClick={openMicSettings}>
              Microphone
            </button>
            <button type="button" className="onboarding-btn" onClick={openSpeechSettings}>
              Speech
            </button>
            <button
              type="button"
              className="onboarding-btn onboarding-btn-primary"
              onClick={markOnboardingDone}
            >
              I&apos;ve Granted Access
            </button>
          </div>
          {onboardingError ? <p className="onboarding-error">{onboardingError}</p> : null}
        </section>
      ) : null}

      {state.error ? (
        <div className="error-banner" role="alert">
          <span className="error-icon">⚠</span>
          <span className="error-text">{state.error}</span>
        </div>
      ) : null}

      <div className="status-row">
        <div className="status-group">
          <div className={`mic-indicator ${state.error ? "error" : state.isListening ? "" : "idle"}`} />
          <span className="status-text" aria-live="polite">
            {statusText}
          </span>
        </div>
        <div
          className="level-bar"
          role="progressbar"
          aria-label="Microphone input level"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={Math.round(Math.max(0, Math.min(state.micLevel, 1)) * 100)}
        >
          <div
            className="level-fill"
            style={{ width: micLevelWidth }}
          />
        </div>
        <button
          type="button"
          className={`on-device-badge ${state.isOnDevice ? "" : "cloud"}`}
          title={state.isOnDevice ? "On-device processing (click to toggle)" : "Cloud processing (click to toggle)"}
          onClick={toggleOnDevice}
        >
          {state.isOnDevice ? "On-Device" : "Cloud"}
        </button>
      </div>

      <div className="transcript-wrap">
        <div className={`transcript ${displayText ? (isPartial ? "partial" : "") : "placeholder"}`}>
          {displayText || transcriptHint}
        </div>
      </div>

      <div className="controls">
        <button type="button" className="lang-badge" onClick={toggleLanguage} title="Toggle language">
          {language}
        </button>
        <span className="shortcut-hint">
          <kbd>⌥</kbd> + <kbd>Space</kbd>
        </span>
      </div>
    </div>
  );
}

export default App;
