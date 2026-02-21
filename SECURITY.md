# Security Policy

## Reporting a Vulnerability

1. **Do not** open a public issue
2. Use [GitHub's private vulnerability reporting](https://github.com/izyuumi/koe/security/advisories/new)
3. Include steps to reproduce and potential impact

## Security Model

Koe runs as a local macOS app with these security considerations:

- **Microphone access**: Required for speech recognition. Audio is processed by Apple's Speech framework — either on-device or via Apple's servers (user's choice).
- **Accessibility permission**: Required for text insertion via simulated keystrokes. Only used to paste transcribed text.
- **Clipboard**: Temporarily used during text insertion. The previous clipboard content is restored after 300ms, but only if no other app modified it in the meantime.
- **No network**: Koe itself makes no network calls. On-device recognition is fully offline. Cloud recognition goes through Apple's Speech API.
- **No telemetry**: No data is collected or transmitted.
- **Speech helper**: A sandboxed Swift subprocess. Communicates with the main app only via stdout (one-way text protocol).
