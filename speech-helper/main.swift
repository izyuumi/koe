import Foundation
import Speech
import AVFoundation

// MARK: - Speech Helper Process
// Standalone process that handles speech recognition via Apple's Speech framework.
// Protocol (stdout):
//   PARTIAL:<text>  - interim recognition result
//   FINAL:<text>    - final recognition result
//   LEVEL:<float>   - mic input level (0.0-1.0)
//   ERROR:<msg>     - error occurred
//   READY           - recognition started successfully

class SpeechHelper: NSObject {
    private let audioEngine = AVAudioEngine()
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    private var speechRecognizer: SFSpeechRecognizer?
    private var lastTranscript = ""
    private var isStopping = false

    func start(language: String, preferOnDevice: Bool) {
        SFSpeechRecognizer.requestAuthorization { status in
            guard status == .authorized else {
                self.output("ERROR:Speech recognition not authorized (status: \(status.rawValue))")
                exit(1)
            }

            DispatchQueue.main.async {
                self.beginRecognition(language: language, preferOnDevice: preferOnDevice)
            }
        }
    }

    private func beginRecognition(language: String, preferOnDevice: Bool) {
        let locale = Locale(identifier: language)
        speechRecognizer = SFSpeechRecognizer(locale: locale)

        guard let recognizer = speechRecognizer, recognizer.isAvailable else {
            output("ERROR:Speech recognizer not available for \(language)")
            exit(1)
        }

        let onDevice = preferOnDevice && recognizer.supportsOnDeviceRecognition

        recognitionRequest = SFSpeechAudioBufferRecognitionRequest()
        guard let request = recognitionRequest else {
            output("ERROR:Failed to create recognition request")
            exit(1)
        }

        request.shouldReportPartialResults = true
        request.requiresOnDeviceRecognition = onDevice

        // Configure audio engine with error handling
        let inputNode = audioEngine.inputNode

        // Verify input format is valid
        let recordingFormat = inputNode.outputFormat(forBus: 0)
        guard recordingFormat.sampleRate > 0 && recordingFormat.channelCount > 0 else {
            output("ERROR:Invalid audio input format (sampleRate=\(recordingFormat.sampleRate), channels=\(recordingFormat.channelCount)). Check microphone.")
            exit(1)
        }

        inputNode.installTap(onBus: 0, bufferSize: 1024, format: recordingFormat) { [weak self] buffer, _ in
            self?.recognitionRequest?.append(buffer)

            let channelData = buffer.floatChannelData?[0]
            let frameLength = Int(buffer.frameLength)
            if let data = channelData, frameLength > 0 {
                var sum: Float = 0
                for i in 0..<frameLength {
                    sum += abs(data[i])
                }
                let avg = sum / Float(frameLength)
                let level = min(1.0, avg * 10)
                self?.output("LEVEL:\(String(format: "%.3f", level))")
            }
        }

        recognitionTask = recognizer.recognitionTask(with: request) { [weak self] result, error in
            guard let self = self else { return }

            if let result = result {
                let text = result.bestTranscription.formattedString
                if result.isFinal {
                    self.lastTranscript = text
                    self.output("FINAL:\(text)")
                } else {
                    self.output("PARTIAL:\(text)")
                }
            }

            if let error = error {
                let nsError = error as NSError
                // Ignore cancellation errors (expected on stop)
                if nsError.domain != "kAFAssistantErrorDomain" || nsError.code != 216 {
                    self.output("ERROR:\(error.localizedDescription)")
                }
            }
        }

        audioEngine.prepare()
        do {
            try audioEngine.start()
            output("READY")
        } catch {
            output("ERROR:Audio engine failed: \(error.localizedDescription)")
            exit(1)
        }
    }

    func stop() {
        guard !isStopping else { return }
        isStopping = true

        audioEngine.stop()
        audioEngine.inputNode.removeTap(onBus: 0)
        recognitionRequest?.endAudio()
        recognitionTask?.cancel()

        if !lastTranscript.isEmpty {
            output("FINAL:\(lastTranscript)")
        }
    }

    private func output(_ text: String) {
        print(text)
        fflush(stdout)
    }
}

// MARK: - Main

let helper = SpeechHelper()

// Parse arguments
var language = "en-US"
var onDevice = false

let args = CommandLine.arguments
for i in 0..<args.count {
    if args[i] == "--language" && i + 1 < args.count {
        language = args[i + 1]
    }
    if args[i] == "--on-device" {
        onDevice = true
    }
}

// Safe signal handling using DispatchSource (replaces unsafe signal())
let sigtermSource = DispatchSource.makeSignalSource(signal: SIGTERM, queue: .main)
sigtermSource.setEventHandler {
    helper.stop()
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
        exit(0)
    }
}
sigtermSource.resume()
// Ignore default SIGTERM handling so DispatchSource receives it
signal(SIGTERM, SIG_IGN)

let sigintSource = DispatchSource.makeSignalSource(signal: SIGINT, queue: .main)
sigintSource.setEventHandler {
    helper.stop()
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
        exit(0)
    }
}
sigintSource.resume()
signal(SIGINT, SIG_IGN)

helper.start(language: language, preferOnDevice: onDevice)

// Keep running
RunLoop.main.run()
