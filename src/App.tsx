import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

/** How long to keep the HUD visible after dictation stops (ms) */
const HUD_HIDE_DELAY_MS = 1500;

interface DictationState {
  isListening: boolean;
  transcript: string;
  partialResult: string;
  micLevel: number;
  error: string | null;
}

function App() {
  const [showOnboarding, setShowOnboarding] = useState(() => {
    return localStorage.getItem("koe.onboarding.permissions.v1") !== "done";
  });
  const [onboardingError, setOnboardingError] = useState<string | null>(null);
  const hideTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Load persisted settings
  const [language, setLanguage] = useState<string>(() => {
    return localStorage.getItem("koe.language") || "en-US";
  });
  const [isOnDevice, setIsOnDevice] = useState<boolean>(() => {
    return localStorage.getItem("koe.onDevice") !== "false";
  });

  const [supportsFnGlobeShortcut, setSupportsFnGlobeShortcut] = useState<boolean>(false);

  const [state, setState] = useState<DictationState>({
    isListening: false,
    transcript: "",
    partialResult: "",
    micLevel: 0,
    error: null,
  });

  // If onboarding is done on launch, hide the window
  useEffect(() => {
    if (!showOnboarding) {
      getCurrentWebviewWindow().hide().catch(() => {});
    }
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => {
    invoke<boolean>("supports_fn_globe_shortcut")
      .then((supported) => setSupportsFnGlobeShortcut(Boolean(supported)))
      .catch(() => setSupportsFnGlobeShortcut(false));
  }, []);

  // Dismiss HUD with Escape key (#18)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        getCurrentWebviewWindow().hide().catch(() => {});
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Persist settings to localStorage
  useEffect(() => {
    localStorage.setItem("koe.language", language);
  }, [language]);

  useEffect(() => {
    localStorage.setItem("koe.onDevice", String(isOnDevice));
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
        if (e.payload.listening) {
          // Starting: cancel any pending hide, clear old transcript
          if (hideTimerRef.current) {
            clearTimeout(hideTimerRef.current);
            hideTimerRef.current = null;
          }
          resetCopied();
          setState((s) => ({
            ...s,
            isListening: true,
            transcript: "",
            partialResult: "",
            error: null,
            micLevel: 0,
          }));
        } else {
          // Stopping: keep HUD visible briefly so user sees final text
          setState((s) => ({
            ...s,
            isListening: false,
            micLevel: 0,
          }));
          hideTimerRef.current = setTimeout(() => {
            getCurrentWebviewWindow().hide().catch(() => {});
            hideTimerRef.current = null;
          }, HUD_HIDE_DELAY_MS);
        }
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
      if (hideTimerRef.current) clearTimeout(hideTimerRef.current);
      if (copiedResetTimerRef.current) {
        window.clearTimeout(copiedResetTimerRef.current);
        copiedResetTimerRef.current = null;
      }
    };
  }, []);

  const toggleLanguage = () => {
    setLanguage((l) => (l === "en-US" ? "ja-JP" : "en-US"));
  };

  const toggleOnDevice = () => {
    setIsOnDevice((v) => !v);
  };

  const [copied, setCopied] = useState(false);
  const copiedResetTimerRef = useRef<number | null>(null);

  const clearCopiedResetTimer = () => {
    if (copiedResetTimerRef.current) {
      window.clearTimeout(copiedResetTimerRef.current);
      copiedResetTimerRef.current = null;
    }
  };

  const resetCopied = () => {
    setCopied(false);
    clearCopiedResetTimer();
  };

  const displayText = state.partialResult || state.transcript;
  const isPartial = !!state.partialResult;
  const micLevelWidth = `${Math.max(0, Math.min(state.micLevel, 1)) * 100}%`;
  const statusText = state.error
    ? "Error"
    : state.isListening
      ? "Listening"
      : state.transcript
        ? "Done"
        : "Idle";
  const transcriptHint = "Start speaking. Your words appear here.";

  const copyTranscriptToClipboard = () => {
    if (!displayText) return;

    navigator.clipboard.writeText(displayText).then(() => {
      setCopied(true);
      clearCopiedResetTimer();
      copiedResetTimerRef.current = window.setTimeout(() => {
        setCopied(false);
        copiedResetTimerRef.current = null;
      }, 1500);
    });
  };

  useEffect(() => {
    resetCopied();
  }, [displayText]);

  const markOnboardingDone = () => {
    localStorage.setItem("koe.onboarding.permissions.v1", "done");
    setShowOnboarding(false);
    setOnboardingError(null);
    // Hide HUD after onboarding — it'll show again on ⌥Space
    getCurrentWebviewWindow().hide().catch(() => {});
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

  const openAccessibilitySettings = async () => {
    try {
      await invoke("open_accessibility_settings");
      setOnboardingError(null);
    } catch {
      setOnboardingError("Could not open Accessibility settings.");
    }
  };

  // Onboarding screen (first launch)
  if (showOnboarding) {
    return (
      <div className="hud" role="application" aria-label="Koe setup">
        <section className="onboarding" aria-label="Permission onboarding">
          <p className="onboarding-title">Welcome to Koe 声</p>
          <p className="onboarding-copy">
            Grant these permissions so Koe can listen and type for you.
          </p>
          <div className="onboarding-steps">
            <button type="button" className="onboarding-btn" onClick={openMicSettings}>
              🎤 Microphone
            </button>
            <button type="button" className="onboarding-btn" onClick={openSpeechSettings}>
              🗣 Speech Recognition
            </button>
            <button type="button" className="onboarding-btn" onClick={openAccessibilitySettings}>
              ♿ Accessibility
            </button>
          </div>
          <p className="onboarding-hint">
            After granting all three, click below.
          </p>
          <button
            type="button"
            className="onboarding-btn onboarding-btn-primary onboarding-btn-done"
            onClick={markOnboardingDone}
          >
            All Done — Start Using Koe
          </button>
          {onboardingError ? <p className="onboarding-error">{onboardingError}</p> : null}
        </section>
      </div>
    );
  }

  // Normal HUD (visible during dictation)
  return (
    <div className="hud" role="application" aria-label="Koe dictation HUD">
      <div className="drag-handle" data-tauri-drag-region />
      {state.error ? (
        <div className="error-banner" role="alert">
          <span className="error-icon">⚠</span>
          <span className="error-text">{state.error}</span>
        </div>
      ) : null}

      <div className="status-row">
        <div className="status-group">
          <div
            className={`mic-indicator ${state.error ? "error" : state.isListening ? "active" : "idle"}`}
            style={state.isListening && !state.error ? {
              transform: `scale(${1 + state.micLevel * 0.5})`,
              opacity: 0.6 + state.micLevel * 0.4,
            } : undefined}
          />
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
          className={`on-device-badge ${isOnDevice ? "" : "cloud"}`}
          title={isOnDevice ? "On-device processing (click to toggle)" : "Cloud processing (click to toggle)"}
          onClick={toggleOnDevice}
        >
          {isOnDevice ? "On-Device" : "Cloud"}
        </button>
      </div>

      <div
        className="transcript-wrap"
        role="button"
        tabIndex={0}
        aria-label="Copy transcript to clipboard"
        aria-disabled={!displayText}
        onDoubleClick={copyTranscriptToClipboard}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            copyTranscriptToClipboard();
          }
        }}
        title="Double-click to copy"
      >
        <div className={`transcript ${displayText ? (isPartial ? "partial" : "final") : "placeholder"}`}>
          {copied ? "✓ Copied!" : displayText || transcriptHint}
        </div>
      </div>

      <div className="controls">
        <button type="button" className="lang-badge" onClick={toggleLanguage} title="Toggle language">
          {language}
        </button>
        <span className="shortcut-hint">
          {supportsFnGlobeShortcut ? (
            <>
              <kbd>fn</kbd> or <kbd>⌥</kbd>+<kbd>Space</kbd>
            </>
          ) : (
            <>
              <kbd>⌥</kbd>+<kbd>Space</kbd>
            </>
          )}
        </span>
      </div>
    </div>
  );
}

export default App;
