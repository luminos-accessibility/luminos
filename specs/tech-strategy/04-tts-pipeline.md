# 04 -- TTS Pipeline

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-15
**Audience:** Engineers, AI agents implementing the text-to-speech pipeline
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 5, 7.3, 8), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Section 4.4), [System Architecture](./01-system-architecture.md) (Sections 4.5, 5.3, 6.2, 7.1, 9), [Platform Abstraction](./02-platform-abstraction.md) (TtsEngine, AudioOutput)

---

## 1. Overview

### 1.1 Purpose

This document defines the text-to-speech pipeline that transforms on-screen text into spoken audio. It is the engineering specification for the second major subsystem in Luminos -- the bridge between visual magnification and auditory comprehension.

This document answers: **How does text become speech in Luminos, with <200ms latency, across all platforms?**

### 1.2 Scope

This document covers:
- Pipeline stages from text input through audio playback
- Text preprocessing (sentence segmentation, normalization, abbreviation expansion)
- espeak-ng subprocess protocol (spawn, communication, crash recovery, lifecycle)
- sherpa-onnx neural inference integration (Kokoro primary, Piper fallback)
- Audio output via cpal (ring buffer, sample rate, mixing)
- Voice model management (discovery, loading, memory budget, downloading)
- Concurrency model (pipelining, threading, back-pressure)
- End-to-end latency budget
- Word-level highlighting synchronization with the rendering pipeline
- Speech queue management (interrupt, queue, priority)
- Platform-native TTS fallback path
- Testing strategy for TTS code

This document does NOT cover:
- TtsEngine and AudioOutput trait definitions (see [02 -- Platform Abstraction](./02-platform-abstraction.md), Sections 3.4, 3.7)
- Thread model overview (see [01 -- System Architecture](./01-system-architecture.md), Section 6.2)
- Magnification pipeline or rendering (see [03 -- Rendering Pipeline](./03-rendering-pipeline.md))
- Control panel voice selection UI (see [05 -- Control Panel](./05-control-panel.md))
- Text extraction via accessibility APIs or OCR (see [02 -- Platform Abstraction](./02-platform-abstraction.md), FocusTracker)

### 1.3 Phase Attribution

TTS is a **Phase 2** feature per the Product Strategy (Section 7.3, Months 7-9). However, the architecture accommodates TTS from Phase 0:

| Phase | TTS Milestone |
|-------|---------------|
| Phase 0 | `TtsEngine` and `AudioOutput` traits exist as stubs. Mock implementations validate the trait boundary. No TTS functionality. |
| Phase 1 | No TTS features. Focus tracking (AT-SPI2) provides the text source that TTS will later consume. |
| Phase 2 | Full TTS pipeline: Kokoro via sherpa-onnx, espeak-ng subprocess, "read what I see," selective TTS, macOS support. |
| Phase 2 (P1) | Reading speed/voice control, platform-native TTS fallback, read aloud with word highlighting. |
| Phase 3 | OCR-to-TTS pipeline (extract text from images/scanned docs, feed to TTS). |
| Phase 5 | Context-aware TTS (neural voices adjust pacing/emphasis based on content type). |

---

## 2. Pipeline Architecture

### 2.1 High-Level Data Flow

The TTS pipeline transforms text into audio through five stages, each running on dedicated threads fully decoupled from the magnification render loop. (The [System Architecture](./01-system-architecture.md) Section 4.5 uses a coarser three-stage decomposition -- phonemization, synthesis, playback -- grouping text input and preprocessing into the caller's responsibility.)

```
User Trigger ("Read this")
         |
         v
+--------------------+
| 1. TEXT INPUT      |  Text source: accessibility API, clipboard, OCR (Phase 3)
| Text Extraction    |
+--------------------+
         |
         v  UTF-8 text
+--------------------+
| 2. PREPROCESSING   |  Sentence segmentation, abbreviation expansion, number
| Text Preprocessor  |  normalization. Pure Rust, no external dependencies.
+--------------------+
         |
         v  Sentence chunks (Vec<Sentence>)
+--------------------+          +--------------------+
| 3. PHONEMIZATION   | stdin -> | espeak-ng process  |
| Subprocess Manager | stdout<- | (long-lived)       |
+--------------------+          +--------------------+
         |
         v  IPA phonemes (per sentence)
+--------------------+
| 4. SYNTHESIS       |
| sherpa-onnx        |  Kokoro-82M (primary) or Piper VITS (language fallback)
| Neural Inference   |  via sherpa-rs Rust bindings
+--------------------+
         |
         v  Audio samples (f32, 24kHz mono for Kokoro)
+--------------------+
| 5. PLAYBACK        |
| cpal AudioOutput   |  Ring buffer -> audio device callback
+--------------------+
         |
         v
    Speaker / Headphones
```

### 2.2 Pipeline Properties

| Property | Value | Rationale |
|----------|-------|-----------|
| **Pipelining** | Yes -- sentence-level | While Kokoro synthesizes sentence N, espeak-ng phonemizes sentence N+1. Hides phonemization latency for all sentences after the first. |
| **Streaming** | Sentence-granularity | Each sentence is phonemized and synthesized as a unit. Sub-sentence streaming (word-level) is a Phase 5 optimization. |
| **Interrupt** | Immediate | A new speech request with `interrupt: true` stops current playback and begins the new request. No drain delay. |
| **Queue depth** | 1 (replace) | The `speech_request` channel has capacity 1 with replace-newest semantics. Only the most recent request matters. |
| **Thread isolation** | Full | TTS threads never touch render state. Communication with the render pipeline is limited to the `highlight_events` channel. |
| **Crash isolation** | espeak-ng in subprocess | espeak-ng runs as a separate OS process. A segfault or memory leak does not affect the main application. |

---

## 3. Text Input and Extraction

### 3.1 Text Sources

The TTS pipeline receives text from three sources, all funneled through a single `SpeechRequest` type:

| Source | Trigger | Phase | Implementation |
|--------|---------|-------|----------------|
| **Accessibility API** | "Read what I see" hotkey | Phase 2 | `FocusTracker` provides the focused element's text content via AT-SPI2 (Linux), AXUIElement (macOS), or UIA (Windows). |
| **Clipboard** | "Read selection" hotkey | Phase 2 | User selects text, presses hotkey. Text read from clipboard via `arboard` crate. |
| **OCR** | "Read from screen" hotkey | Phase 3 | Screen region captured, OCR extracts text, fed to TTS. |

```rust
/// A request to speak text, sent from the main thread or IPC thread
/// to the TTS Coordinator.
pub struct SpeechRequest {
    /// The text to speak.
    pub text: String,
    /// If true, interrupt any current speech and begin this request immediately.
    /// If false, queue after current speech completes.
    pub interrupt: bool,
    /// The source of this request (for logging and analytics).
    pub source: TextSource,
}

/// Identifies where the text came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextSource {
    /// Text extracted from the focused UI element via accessibility APIs.
    AccessibilityApi,
    /// Text from the system clipboard (user selected text and triggered TTS).
    Clipboard,
    /// Text extracted from screen content via OCR (Phase 3).
    Ocr,
    /// Text provided directly by the control panel UI (e.g., test speech).
    ControlPanel,
}
```

### 3.2 Channel Design

The speech request channel follows the design from [01 -- System Architecture](./01-system-architecture.md) Section 6.4:

| Channel | Sender | Receiver | Type | Capacity | Back-pressure |
|---------|--------|----------|------|----------|---------------|
| `speech_request` | Main thread / IPC thread | TTS Coordinator | `SpeechRequest` | 1 | Replace (newest wins) |

**Replace semantics:** If the user presses "read this" twice rapidly, only the second request matters. The channel replaces any pending (unprocessed) request with the newest one. This is implemented via `tokio::sync::watch` or a custom single-slot channel with `std::sync::Mutex`.

**Known limitation:** The capacity-1 replace semantics mean that sequential "read next paragraph" workflows may silently drop queued text if the user triggers multiple non-interrupt requests while speech is in progress. This is an acceptable trade-off for Phase 2 simplicity. If user feedback indicates sequential reading workflows are common, the queue depth can be expanded in a later phase.

---

## 4. Text Preprocessing

### 4.1 Purpose

Raw text from accessibility APIs and clipboard often contains artifacts that produce poor TTS output: abbreviations, raw numbers, URLs, email addresses, and inconsistent whitespace. The preprocessor normalizes text into clean sentence chunks optimized for phonemization and synthesis.

### 4.2 Processing Stages

```
Raw text
    |
    v
+----------------------------+
| 1. Unicode normalization   |  NFC form, strip zero-width chars, normalize whitespace
+----------------------------+
    |
    v
+----------------------------+
| 2. Abbreviation expansion  |  "Dr." -> "Doctor", "St." -> "Street" (context-sensitive)
+----------------------------+
    |
    v
+----------------------------+
| 3. Number normalization    |  "1,234.56" -> "one thousand two hundred thirty-four
|                            |   point five six" (locale-aware)
+----------------------------+
    |
    v
+----------------------------+
| 4. URL / email handling    |  URLs -> "link" or domain name; emails -> read as words
+----------------------------+
    |
    v
+----------------------------+
| 5. Sentence segmentation   |  Split into sentences. Respects abbreviations ("Dr. Smith"
|                            |   is one sentence, not two).
+----------------------------+
    |
    v
Vec<Sentence>
```

### 4.3 Sentence Type

```rust
/// A preprocessed sentence ready for phonemization.
#[derive(Debug, Clone)]
pub struct Sentence {
    /// The normalized text of this sentence.
    pub text: String,
    /// Byte offset of this sentence in the original input text.
    /// Used for word highlighting synchronization.
    pub source_offset: usize,
    /// Byte length in the original input text.
    pub source_len: usize,
}
```

### 4.4 Implementation Notes

- The preprocessor is **pure Rust, synchronous, and allocation-minimal**. It does not call any external processes or perform I/O.
- Sentence segmentation uses a rule-based approach (Unicode sentence break rules from UAX #29), with a curated abbreviation dictionary to avoid false breaks at "Dr.", "Mr.", "U.S.A.", etc.
- Number normalization is locale-aware (BCP 47 language tag from the active voice determines decimal separator and grouping conventions).
- The preprocessor lives in `crates/luminos-tts/src/preprocessor.rs`.
- Phase 2 scope: English, with a framework for adding locale-specific rules. Full locale support grows with Kokoro's language coverage.

---

## 5. Phonemization (espeak-ng Subprocess)

### 5.1 Architecture Decision

espeak-ng converts text (graphemes) into IPA phonemes that Kokoro uses as input. It is run as a **long-lived subprocess** (not spawned per request) for three engineering reasons:

1. **Crash isolation.** espeak-ng is a C program. A segfault or buffer overflow in espeak-ng does not crash the main Luminos process.
2. **Resource isolation.** espeak-ng memory leaks do not grow the main process's heap. The subprocess can be killed and restarted if its memory exceeds a threshold.
3. **Testability.** The subprocess boundary provides a clean mock point for unit testing the phonemizer interface.

**Licensing note:** The project's GPLv3 license makes espeak-ng (GPL-3.0) fully compatible. Subprocess isolation is an engineering decision, not a legal requirement. espeak-ng could be linked directly. See [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) Section 4.4.

### 5.2 Subprocess Protocol

The espeak-ng subprocess is managed by the `EspeakSubprocess` struct in `crates/luminos-tts/src/espeak.rs`.

#### Spawn

```
espeak-ng -q --ipa --stdin -v {lang}
```

| Flag | Purpose |
|------|---------|
| `-q` | Quiet mode -- suppress audio output, produce text-only phoneme output |
| `--ipa` | Use International Phonetic Alphabet notation for phoneme output |
| `--stdin` | Read input text from stdin (interactive mode, one line at a time) |
| `-v {lang}` | Set the voice/language (e.g., `-v en`, `-v fr`, `-v ja`) |

The subprocess is spawned once on the first `speak()` call and kept alive for the application's lifetime. If the active voice language changes, the subprocess is restarted with the new `-v` flag.

#### Communication Protocol

```
Main Process                          espeak-ng Subprocess
     |                                        |
     |--- "Hello world.\n" ---- stdin ------->|
     |                                        |
     |<-- "həˈloʊ wɜːɹld.\n" -- stdout ------|
     |                                        |
     |--- "Next sentence.\n" -- stdin ------->|
     |                                        |
     |<-- "nɛkst sɛntəns.\n" -- stdout ------|
```

- **Input:** One sentence per line, UTF-8 encoded, terminated by `\n`.
- **Output:** One phoneme sequence per line, IPA notation, terminated by `\n`.
- **Synchronous request-response:** Each input line produces exactly one output line. The subprocess reader blocks on stdout until a complete line is received.
- **Timeout:** If no response is received within 5 seconds, the subprocess is considered hung. Kill it and respawn.

#### Rust Interface

```rust
/// Manages the espeak-ng subprocess lifecycle and communication.
pub(crate) struct EspeakSubprocess {
    /// The child process handle.
    child: Option<std::process::Child>,
    /// Writer to the subprocess stdin.
    stdin: Option<std::io::BufWriter<std::process::ChildStdin>>,
    /// Reader from the subprocess stdout.
    stdout: Option<std::io::BufReader<std::process::ChildStdout>>,
    /// The current language the subprocess is configured for.
    language: String,
    /// Maximum memory usage before forced restart (bytes).
    memory_limit: usize,
    /// Number of consecutive failures before giving up.
    max_consecutive_failures: u32,
    /// Current consecutive failure count.
    consecutive_failures: u32,
}

impl EspeakSubprocess {
    /// Creates a new subprocess manager. Does NOT spawn the process yet.
    /// The process is spawned lazily on the first `phonemize` call.
    pub fn new(language: &str) -> Self { /* ... */ }

    /// Phonemizes a single sentence. Spawns the subprocess if not running.
    /// Returns IPA phoneme string.
    pub fn phonemize(&mut self, text: &str) -> Result<String, TtsError> { /* ... */ }

    /// Changes the phonemization language. Restarts the subprocess.
    pub fn set_language(&mut self, language: &str) -> Result<(), TtsError> { /* ... */ }

    /// Shuts down the subprocess gracefully.
    pub fn shutdown(&mut self) { /* ... */ }
}
```

### 5.3 Crash Recovery

If the espeak-ng subprocess exits unexpectedly (crash, killed, pipe broken):

1. Detect the failure (broken pipe on stdin write, or EOF on stdout read).
2. Log the crash at `warn` level: `log::warn!("espeak-ng subprocess exited unexpectedly, restarting")`.
3. Increment `consecutive_failures`.
4. If `consecutive_failures < max_consecutive_failures` (default: 3): respawn the subprocess and retry the current sentence.
5. If `consecutive_failures >= max_consecutive_failures`: return `TtsError::PhonemizerFailed` and stop retrying. The caller (TTS Coordinator) falls back to the platform-native TTS engine.
6. Reset `consecutive_failures` to 0 on any successful phonemization.

### 5.4 Memory Watchdog

The subprocess memory is monitored periodically (every 60 seconds) using platform-specific APIs:

| Platform | Memory Query Method |
|----------|---------------------|
| Linux | `/proc/{pid}/status` (VmRSS field) |
| macOS | `mach_task_info` (MACH_TASK_BASIC_INFO, resident_size) |
| OpenBSD | `sysctl` with `KERN_PROC` / `kvm_getprocs` (OpenBSD does not mount `/proc` by default) |
| Windows | `GetProcessMemoryInfo` (WorkingSetSize) |

If resident memory exceeds `memory_limit` (default: 100MB), the subprocess is gracefully terminated (SIGTERM, then SIGKILL after 2 seconds) and respawned. This prevents espeak-ng memory leaks from degrading system performance over long-running sessions.

### 5.5 espeak-ng Availability

espeak-ng must be installed on the system for phonemization to work. The pipeline detects espeak-ng availability at startup:

| Platform | Expected Location | Package |
|----------|-------------------|---------|
| Linux (Debian/Ubuntu) | `/usr/bin/espeak-ng` | `espeak-ng` via apt |
| Linux (Fedora) | `/usr/bin/espeak-ng` | `espeak-ng` via dnf |
| macOS | `/opt/homebrew/bin/espeak-ng` or `/usr/local/bin/espeak-ng` | `espeak-ng` via Homebrew |
| OpenBSD | `/usr/local/bin/espeak-ng` | `espeak-ng` via pkg_add |
| Windows | Bundled in application package | Included in MSI installer |

If espeak-ng is not found:
1. Log at `warn` level: `log::warn!("espeak-ng not found at expected path, TTS unavailable")`.
2. `TtsEngine::speak()` returns `TtsError::PhonemizerFailed` with a message indicating espeak-ng is missing.
3. The control panel displays a user-facing message: "Text-to-speech requires espeak-ng. Install it with: [platform-specific command]."
4. The platform-native TTS fallback (Phase 2 P1) does not require espeak-ng and remains available.

### 5.6 Future: misaki G2P

[misaki](https://github.com/hexgrad/misaki) is Kokoro's transformer-based grapheme-to-phoneme library. It is available on PyPI and supports English, Japanese, Korean, and Chinese. If misaki is ported to Rust (or made available as a C library), it could replace espeak-ng for Kokoro's supported languages, eliminating the external dependency for those languages. espeak-ng would remain as a fallback for languages misaki does not support.

This is a **nice-to-have** dependency reduction measure, not a licensing mitigation. See [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) Section 4.4.

---

## 6. Neural Synthesis (sherpa-onnx)

### 6.1 Runtime Architecture

Luminos uses [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) (Apache 2.0, 10.8K GitHub stars) as the neural TTS inference runtime. sherpa-onnx provides a unified C API that supports multiple TTS model architectures (Kokoro, Piper VITS, KittenTTS, Matcha). The [sherpa-rs](https://crates.io/crates/sherpa-rs) crate (v0.6.8, MIT) wraps this C API for Rust.

```
+----------------------------------+
| luminos-tts crate                |
|   inference.rs                   |
|     |                            |
|     v                            |
| sherpa-rs (v0.6.8, Rust FFI)     |
|     |                            |
|     v                            |
| sherpa-onnx (C API)              |
|     |                            |
|     v                            |
| ONNX Runtime (inference engine)  |
|     |                            |
|     v                            |
| CPU (default) or GPU (optional)  |
+----------------------------------+
```

### 6.2 Model Selection

| Model | Role | Size (ONNX) | RTF (RPi 4) | Languages | License |
|-------|------|-------------|-------------|-----------|---------|
| **Kokoro-82M (q8)** | Primary | ~92MB | 0.25-0.4 | en-US, en-GB, es, fr, hi, it, ja, pt-BR, zh (9 codes, ~8 unique languages) | Apache 2.0 (model) |
| **Kokoro-82M (fp16)** | Higher quality alternative | ~163MB | ~0.2-0.35 | Same as q8 | Apache 2.0 (model) |
| **Kokoro-82M (fp32)** | Development reference | ~327MB | ~0.2-0.3 | Same as q8 | Apache 2.0 (model) |
| **Piper VITS (medium)** | Language fallback | ~60-75MB per model | 0.1-0.2 | 30+ languages | MIT (model weights) |

*Note on model sizes: Sizes above are from the onnx-community/Kokoro-82M-v1.0-ONNX HuggingFace repository. sherpa-onnx may package models differently (e.g., its own multi-lang ONNX export at ~310MB fp32). Verify actual file sizes against the chosen distribution source before implementation.*

*Note on RTF: RTF values differ between quantization levels. fp32 is the baseline; q8 quantization may be slightly slower due to dequantization overhead on some hardware, or slightly faster due to reduced memory bandwidth. The numbers above are approximate ranges from sherpa-onnx benchmarks on Raspberry Pi 4. Desktop CPUs are substantially faster.*

**Default deployment:** Kokoro-82M q8 quantized variant. This balances quality and memory usage (~92MB). Users who prioritize quality over memory can switch to fp16 (~163MB) via the control panel. On modern desktop hardware, Kokoro's first-chunk latency is under 200ms. See [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) Section 4.4.

### 6.3 Inference Interface

```rust
/// Manages sherpa-onnx model loading and inference.
pub(crate) struct SherpaInference {
    /// The sherpa-onnx TTS instance (wraps the C API via sherpa-rs).
    tts: Option<sherpa_rs::tts::OfflineTts>,
    /// Currently loaded model metadata.
    loaded_model: Option<LoadedModel>,
    /// Audio sample rate produced by the current model (e.g., 24000 for Kokoro).
    sample_rate: u32,
}

/// Metadata about a loaded model.
#[derive(Debug, Clone)]
pub(crate) struct LoadedModel {
    pub model_path: std::path::PathBuf,
    pub backend: TtsBackend,
    pub language: String,
}

impl SherpaInference {
    /// Creates a new inference engine. Does NOT load a model yet.
    pub fn new() -> Self { /* ... */ }

    /// Loads a TTS model from the given path.
    /// This is a blocking operation (~500-1000ms for Kokoro-82M q8).
    /// Call from a background thread, not from the render loop.
    pub fn load_model(&mut self, config: &ModelConfig) -> Result<(), TtsError> { /* ... */ }

    /// Synthesizes audio from IPA phonemes.
    /// Returns raw audio samples (f32 PCM, mono, at the model's native sample rate).
    pub fn synthesize(&self, phonemes: &str, speed: f32) -> Result<AudioSample, TtsError> { /* ... */ }

    /// Returns the sample rate of the currently loaded model.
    pub fn sample_rate(&self) -> u32 { /* ... */ }

    /// Unloads the current model, freeing memory.
    pub fn unload(&mut self) { /* ... */ }
}
```

### 6.4 Inference Threading

Neural synthesis is CPU-intensive (0.25-0.4 RTF on RPi 4, faster on desktop). It runs on a dedicated **Inference Thread** managed by the TTS Coordinator. Key constraints:

- **One inference at a time.** sherpa-onnx's TTS API is not designed for concurrent inference on the same model instance. A `Mutex` around the `sherpa_rs::tts::OfflineTts` instance ensures single-threaded access.
- **Non-blocking to render loop.** The inference thread never interacts with render state or GPU resources.
- **Interruptible.** When a new speech request with `interrupt: true` arrives, the TTS Coordinator signals the inference thread to abort. The inference thread checks an `AtomicBool` flag between sentences. Mid-sentence interruption is not supported (sentence-level granularity is sufficient for <200ms responsiveness given typical sentence lengths).

### 6.5 Audio Output Format

| Parameter | Kokoro-82M | Piper VITS (medium) |
|-----------|-----------|---------------------|
| Sample rate | 24,000 Hz | 22,050 Hz |
| Bit depth | f32 | f32 |
| Channels | Mono (1) | Mono (1) |
| Format | Raw PCM | Raw PCM |

The AudioOutput trait (see [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 3.7) accepts `AudioSample` structs with these parameters.

**Important: cpal does NOT perform automatic sample rate conversion.** If the audio device's native rate differs from the model's output rate (e.g., Kokoro outputs at 24kHz but the device runs at 48kHz), the application must resample explicitly. The `CpalAudioOutput` implementation includes a resampling step using the `rubato` crate (MIT/Apache 2.0) in the audio callback path. See Section 7.2 for details.

---

## 7. Audio Playback

### 7.1 cpal Integration

Audio playback uses the `cpal` crate (Apache 2.0), which provides cross-platform audio device access. The `CpalAudioOutput` struct implements the `AudioOutput` trait.

| Platform | cpal Backend |
|----------|-------------|
| Linux X11/Wayland | ALSA or PulseAudio (cpal auto-selects) |
| macOS | CoreAudio |
| OpenBSD | sndio (pending upstream PR #493; see Section 7.4) |
| Windows | WASAPI |

### 7.2 Ring Buffer Architecture

Audio data flows from the inference thread to the audio device callback through a lock-free ring buffer with an intermediate resampling step:

```
Inference Thread              Ring Buffer              Resampler              cpal Audio Callback
(produces audio samples)     (lock-free SPSC)          (rubato crate)         (consumes samples)
       |                          |                         |                        |
       |--- write f32 samples --->|                         |                        |
       |   (model sample rate,    |--- read f32 samples --->|                        |
       |    e.g. 24kHz Kokoro)    |                         |--- resampled f32 ----->|
       |                          |                         |   (device rate,        |--- output
       |                          |                         |    e.g. 48kHz)         |   to device
```

- **Ring buffer type:** Single-producer, single-consumer (SPSC). The inference thread is the sole producer; the cpal callback is the sole consumer. No locking required.
- **Buffer capacity:** 1 second of audio at the model's sample rate (e.g., 24,000 samples for Kokoro). This provides enough buffer to absorb jitter between inference and playback without introducing noticeable latency.
- **Resampling:** The `rubato` crate (MIT/Apache 2.0) performs sample rate conversion from the model's native rate to the audio device's rate. Resampling is performed in the audio callback path (or a pre-callback stage) to avoid allocating resampled buffers in the inference thread. If the model rate matches the device rate, the resampling step is a no-op passthrough.
- **Underrun handling:** If the cpal callback reads an empty buffer, it outputs silence (zeros). This produces a brief silence gap rather than a click or pop.
- **Overflow handling:** If the ring buffer is full when the inference thread tries to write, the inference thread blocks (via a spin-wait or condvar) until space is available. This applies back-pressure to the inference thread, preventing unbounded memory growth.

### 7.3 Volume and Playback Control

Volume control is applied in the cpal callback before writing to the audio device. This avoids modifying the audio samples in the ring buffer, allowing volume changes to take effect immediately without re-synthesizing.

```rust
// Simplified cpal callback (illustrative)
// Note: Rust's std library does not provide AtomicF32. Use AtomicU32 with
// f32::to_bits()/f32::from_bits(), or the `atomic_float` crate.
fn audio_callback(data: &mut [f32], ring_buffer: &RingBuffer, volume: &AtomicU32) {
    let vol = f32::from_bits(volume.load(Ordering::Relaxed));
    let samples_read = ring_buffer.read(data);
    // Resample if needed (rubato), then apply volume
    for sample in &mut data[..samples_read] {
        *sample *= vol;
    }
    // Fill remainder with silence if ring buffer had fewer samples than requested
    for sample in &mut data[samples_read..] {
        *sample = 0.0;
    }
}
```

The `stop()` method on `AudioOutput` clears the ring buffer and resets the audio device stream, producing immediate silence.

### 7.4 OpenBSD sndio Concern

cpal's sndio backend has a pending upstream PR (#493, submitted 2020, not yet merged). Phase 3 (OpenBSD support) must account for this gap. Options:

1. **Contribute to merging the upstream PR.** Preferred.
2. **Maintain a patched cpal fork** with sndio support.
3. **Use `sndio-sys` directly** -- bypassing cpal with a platform-specific `AudioOutput` impl for OpenBSD.
4. **Use PulseAudio on OpenBSD** (available as a package), which cpal already supports.

See [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 8.4 for the full OpenBSD analysis.

---

## 8. Voice Model Management

### 8.1 Model Storage

Voice model files are stored in a platform-appropriate data directory:

| Platform | Path |
|----------|------|
| Linux | `$XDG_DATA_HOME/luminos/voices/` (typically `~/.local/share/luminos/voices/`) |
| macOS | `~/Library/Application Support/com.luminos.app/voices/` |
| OpenBSD | `~/.local/share/luminos/voices/` |
| Windows | `%APPDATA%\Luminos\voices\` |

Directory structure:

```
voices/
  kokoro/
    kokoro-v1.0-q8.onnx          # Default Kokoro model (q8 quantized)
    kokoro-v1.0-q4.onnx          # Lightweight alternative
    voices.json                   # Voice metadata (speaker IDs, language codes)
    tokens.txt                    # Token vocabulary
    data-dir/                     # espeak-ng data directory (bundled)
  piper/
    en-us-libritts-high.onnx     # Example Piper voice
    en-us-libritts-high.json     # Piper model config
    de-thorsten-medium.onnx      # German voice
    de-thorsten-medium.json
  manifest.json                   # Installed model registry
```

### 8.2 Model Discovery and Loading

```rust
/// Manages voice model files: discovery, loading, downloading.
pub(crate) struct ModelManager {
    /// Root directory for voice model storage.
    voices_dir: std::path::PathBuf,
    /// Registry of installed models (loaded from manifest.json).
    manifest: ModelManifest,
}

/// Registry of installed voice models.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ModelManifest {
    /// Installed Kokoro models.
    pub kokoro: Vec<ModelEntry>,
    /// Installed Piper models.
    pub piper: Vec<ModelEntry>,
}

/// Metadata for a single installed model.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ModelEntry {
    /// Unique model identifier (e.g., "kokoro-v1.0-q8").
    pub id: String,
    /// Path to the ONNX model file, relative to voices_dir.
    pub model_path: String,
    /// Supported language codes (BCP 47).
    pub languages: Vec<String>,
    /// Available speaker/voice IDs within this model.
    pub speakers: Vec<SpeakerEntry>,
    /// Model file size in bytes.
    pub size_bytes: u64,
    /// TTS backend type.
    pub backend: TtsBackend,
}

/// A speaker within a model (Kokoro supports multiple speakers per model).
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct SpeakerEntry {
    /// Speaker ID as used by sherpa-onnx.
    pub id: u32,
    /// Human-readable name (e.g., "Heart", "Bella").
    pub name: String,
    /// BCP 47 language code for this speaker.
    pub language: String,
}
```

### 8.3 Model Loading Lifecycle

```
Application Start
       |
       v
  T=0-400ms: Magnification pipeline initializes (priority)
       |
       v
  T=2000ms: TTS model loading begins (background thread)
       |
       +---> Read manifest.json
       +---> Find default model (Kokoro q8)
       +---> Load ONNX model into sherpa-onnx (500-1000ms)
       +---> Mark TTS as ready
       |
       v
  TTS available for speech requests
```

Model loading is **lazy and non-blocking**: it happens on a background thread after magnification is usable. The user sees a working magnifier within 2 seconds; TTS becomes available shortly after. See [01 -- System Architecture](./01-system-architecture.md) Section 9.4.

### 8.4 Memory Budget

| Component | Budget | Notes |
|-----------|--------|-------|
| Kokoro-82M (q8, default) | ~92MB | Loaded once at startup |
| Kokoro-82M (fp16, quality option) | ~163MB | User-selectable for higher quality |
| Piper model (if loaded) | ~60-75MB per model | Loaded on demand per language |
| espeak-ng subprocess | ~10-30MB | OS overhead for child process |
| TTS working memory | ~10-20MB | Phoneme buffers, audio samples, ring buffer, resampler state |
| **Total TTS subsystem (q8 default)** | **~112-142MB** | Within the application's 4GB total budget |

*Note: These memory figures reflect the ONNX model file sizes from the onnx-community distribution. Runtime memory usage may differ slightly from file size due to ONNX Runtime's memory allocation patterns. The [System Architecture](./01-system-architecture.md) Section 9.3 budgets are approximations from the Tech Stack Evaluation and may need reconciliation with verified model file sizes during implementation.*

Only one neural model is loaded at a time. Switching from Kokoro to a Piper model for a different language unloads Kokoro first, then loads Piper. The memory budget never exceeds one model + espeak-ng + working memory.

### 8.5 Voice Selection

The `TtsEngine::get_voices()` method returns all available voices by scanning the model manifest and querying the platform-native TTS engine:

```
get_voices() -> Vec<Voice>
  |
  +---> Read ModelManifest -> Kokoro speakers -> Voice { engine: TtsBackend::Kokoro, ... }
  +---> Read ModelManifest -> Piper speakers  -> Voice { engine: TtsBackend::Piper, ... }
  +---> Query platform native TTS API         -> Voice { engine: TtsBackend::Native, ... }
  |
  v
  Deduplicated, sorted by language then name
```

Voice switching may require model loading (if switching between Kokoro and Piper, or between different Piper language models). The TTS Coordinator handles this transition asynchronously, queueing speech requests until the new model is ready.

---

## 9. Concurrency Model

### 9.1 Thread Architecture

The TTS pipeline uses the thread model defined in [01 -- System Architecture](./01-system-architecture.md) Section 6.2:

```
TTS Coordinator Thread     [manages speech lifecycle, sentence scheduling]
    |
    +--- espeak-ng Subprocess Reader  [reads phonemes from stdout pipe]
    |
    +--- Inference Thread             [sherpa-onnx Kokoro/Piper synthesis]

Audio Thread (cpal callback)  [reads ring buffer, writes to audio device]
```

### 9.2 TTS Coordinator

The TTS Coordinator is the central orchestrator of the TTS pipeline. It runs on a dedicated thread, receives `SpeechRequest` messages from the speech request channel, and schedules work through the pipeline stages.

```rust
/// The TTS Coordinator manages the speech lifecycle.
/// It runs on a dedicated thread and orchestrates preprocessing,
/// phonemization, inference, and audio playback.
pub(crate) struct TtsCoordinator {
    /// Receives speech requests from the main thread / IPC.
    request_rx: /* channel receiver */,
    /// Text preprocessor (pure Rust, sync).
    preprocessor: TextPreprocessor,
    /// espeak-ng subprocess manager.
    espeak: EspeakSubprocess,
    /// sherpa-onnx inference engine.
    inference: SherpaInference,
    /// Audio output (ring buffer writer end).
    audio_writer: RingBufferWriter,
    /// Channel to send word highlight events to the render pipeline.
    highlight_tx: /* channel sender */,
    /// Flag to signal the coordinator to stop.
    shutdown: Arc<AtomicBool>,
    /// Flag to signal speech interruption.
    interrupt: Arc<AtomicBool>,
}
```

### 9.3 Sentence Pipelining

The key performance optimization: while Kokoro synthesizes sentence N, espeak-ng phonemizes sentence N+1. This hides the phonemization latency (~10-50ms per sentence) for all sentences after the first.

```
Time -->

Sentence 1: [preprocess][phonemize ][synthesize        ][play        ]
Sentence 2:                         [phonemize ][synthesize        ][play        ]
Sentence 3:                                     [phonemize ][synthesize        ][play    ]
                                    ^
                                    overlap: phonemize N+1 during synthesize N
```

Implementation: The TTS Coordinator achieves pipeline overlap by delegating synthesis to the dedicated Inference Thread while performing phonemization on the Coordinator thread itself:

1. **Coordinator thread:** Preprocesses text into sentences. For each sentence, calls `EspeakSubprocess::phonemize()` (blocking, ~10-50ms). Once phonemes are ready, sends them to the Inference Thread via a bounded channel.
2. **Inference Thread:** Receives phoneme strings, calls `SherpaInference::synthesize()` (blocking, 50-150ms), and writes audio samples to the ring buffer.

Because phonemization (on the Coordinator thread) and synthesis (on the Inference Thread) run on separate threads, they overlap naturally: while the Inference Thread synthesizes sentence N, the Coordinator thread is free to phonemize sentence N+1 via blocking I/O on the espeak-ng subprocess. No async I/O is needed -- the two blocking operations run on separate threads.

### 9.4 Interrupt Handling

When a new `SpeechRequest` arrives with `interrupt: true`:

1. The TTS Coordinator sets the `interrupt` AtomicBool flag.
2. The inference thread checks the flag between sentences and aborts if set.
3. The audio ring buffer is cleared (flushed to silence).
4. The current espeak-ng phonemization result (if pending) is discarded.
5. The new request's text is preprocessed and the pipeline begins anew.

This produces near-instant speech interruption: the user hears silence within one cpal callback period (~5-20ms depending on buffer size), then the new speech begins.

---

## 10. Latency Budget

### 10.1 End-to-End Target

**<200ms from speech request to first audible audio.** This is the Product Strategy's target (Section 5.3) and the System Architecture's performance target (Section 9.1).

### 10.2 Per-Stage Budget (First Sentence)

| Stage | Typical Latency | Notes |
|-------|----------------|-------|
| Text preprocessing | <1ms | Pure Rust string processing |
| espeak-ng phonemization | 10-50ms | Subprocess IPC overhead + phonemization. Long-lived process avoids spawn cost. |
| Kokoro inference (first chunk) | 50-150ms | Depends on sentence length and CPU. Desktop CPUs (Intel i5+, Apple M-series, AMD Ryzen 5+) achieve <100ms for typical sentences. |
| Ring buffer write + cpal callback | 5-20ms | Depends on audio device buffer size |
| **Total (first sentence)** | **65-221ms** | Meets <200ms target on modern desktop CPUs for typical sentence lengths. Worst case (long sentences + large audio buffer) may exceed 200ms; see optimization levers below. |

### 10.3 Subsequent Sentences

For sentences after the first, phonemization latency is hidden by pipelining (Section 9.3). Effective latency per subsequent sentence is dominated by inference time only, producing gapless audio playback.

### 10.4 Latency Optimization Levers

| Optimization | Impact | Phase |
|-------------|--------|-------|
| Keep espeak-ng subprocess warm | Eliminates ~100-200ms spawn time | Phase 2 (default) |
| Sentence pipelining | Hides phonemization latency for sentences 2+ | Phase 2 (default) |
| Kokoro q4 quantized model | Faster inference, lower quality | Phase 2 (user option) |
| misaki G2P (replace espeak-ng) | Eliminates subprocess IPC latency | Future (when Rust port available) |
| GPU-accelerated inference | 2-5x inference speedup on discrete GPUs | Future (requires sherpa-onnx CUDA/Metal support) |

---

## 11. Word Highlighting

### 11.1 Purpose

Word highlighting synchronizes visual feedback in the magnification overlay with the current word being spoken. As TTS reads text, the currently-spoken word is highlighted in the magnified view. This is a **Phase 2 P1 feature** per the Product Strategy (Section 7.3).

### 11.2 Architecture

```
TTS Coordinator                Render Thread
      |                              |
      |--- HighlightEvent ---------> |  (via highlight_events channel)
      |    { word_offset,            |
      |      word_len,               |
      |      sentence_offset }       |
      |                              |--- Draw highlight overlay
```

The TTS pipeline emits `HighlightEvent` messages to the render thread via a bounded channel (capacity 8, drop-oldest back-pressure). The render thread consumes these events and draws a highlight rectangle over the corresponding word in the magnified view.

### 11.3 Timing Derivation

Word timing is derived from the audio output: Kokoro produces audio for an entire sentence at once. Word boundaries are estimated by dividing the audio duration proportionally by character count (a rough heuristic) or by using espeak-ng's phoneme timing data (more accurate).

```rust
/// A word highlight event sent from the TTS pipeline to the render thread.
#[derive(Debug, Clone)]
pub struct HighlightEvent {
    /// Byte offset of the highlighted word in the original input text.
    pub word_offset: usize,
    /// Byte length of the highlighted word.
    pub word_len: usize,
    /// Estimated time (in audio samples from sentence start) when this word begins.
    pub audio_start_sample: u64,
    /// Estimated time (in audio samples from sentence start) when this word ends.
    pub audio_end_sample: u64,
}
```

**Accuracy:** Word highlighting timing is approximate. Proportional character-count estimation produces noticeable drift for sentences with mixed word lengths. Phase 5 (context-aware TTS) may introduce forced alignment for precise word timing. For Phase 2, the heuristic is acceptable -- users perceive highlighting as "following along" even with ~100ms drift.

### 11.4 Channel Design

| Channel | Sender | Receiver | Type | Capacity | Back-pressure |
|---------|--------|----------|------|----------|---------------|
| `highlight_events` | TTS Coordinator | Render thread | `HighlightEvent` | 8 | Drop oldest |

Drop-oldest semantics ensure the render thread always has the most recent highlight position. If the render thread falls behind (unlikely -- it runs at 60fps), stale highlights are discarded.

---

## 12. Platform-Native TTS Fallback

### 12.1 Purpose

For languages not covered by Kokoro or Piper, and as a fallback if sherpa-onnx fails to initialize, the pipeline falls back to platform-native TTS engines. This is a **Phase 2 P1 feature**.

### 12.2 Platform APIs

| Platform | API | Crate/Binding | Notes |
|----------|-----|---------------|-------|
| Linux | speech-dispatcher | `speech-dispatcher` crate or D-Bus | Bridges to espeak-ng, Festival, or other system voices |
| macOS | AVSpeechSynthesizer | `objc2` bindings | High-quality system voices (Siri voices) |
| OpenBSD | speech-dispatcher (if installed) | Same as Linux | May not be available in base system |
| Windows | SAPI (Speech API) | `windows` crate | Microsoft voices, third-party SAPI voices |

### 12.3 Fallback Selection Logic

```
speak(text, voice_id) called
    |
    +---> Look up voice_id in model manifest
    |
    +---> if voice.engine == TtsBackend::Kokoro || voice.engine == TtsBackend::Piper:
    |         Use sherpa-onnx pipeline (espeak-ng -> inference -> cpal)
    |
    +---> if voice.engine == TtsBackend::Native:
    |         Use NativeTtsEngine (platform API -> platform audio)
    |
    +---> if sherpa-onnx pipeline fails (model load error, inference error):
    |         log::warn!("Sherpa-onnx pipeline failed, falling back to native TTS")
    |         Use NativeTtsEngine with system default voice
    |
    +---> if NativeTtsEngine also fails:
              Return TtsError::AudioUnavailable
```

### 12.4 NativeTtsEngine

The `NativeTtsEngine` struct implements the `TtsEngine` trait using platform-native APIs. Unlike `SherpaEngine`, it does not use espeak-ng or the ring buffer audio path -- the platform API handles phonemization, synthesis, and audio output internally.

```rust
/// Platform-native TTS engine (AVSpeech, SAPI, speech-dispatcher).
/// Used as a fallback for languages not covered by Kokoro/Piper.
pub(crate) struct NativeTtsEngine {
    // Platform-specific implementation behind cfg gates
}
```

The core engine holds `Box<dyn TtsEngine>` and may switch between `SherpaEngine` and `NativeTtsEngine` at runtime based on the selected voice. This is why the `TtsEngine` trait is object-safe (boxed future for `speak`, no RPITIT). See [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 3.4.

---

## 13. Speech Queue Management

### 13.1 Behavior

| Scenario | `interrupt` | Behavior |
|----------|-------------|----------|
| No speech in progress | `true` or `false` | Begin speaking immediately. |
| Speech in progress, user says "read this" | `true` | Stop current speech, begin new text. |
| Speech in progress, programmatic append | `false` | Queue after current speech completes. Only one item can be queued (capacity 1, newest wins). |

### 13.2 State Machine

```
            speak(interrupt=false)
                    |
                    v
+--------+    +----------+    +---------+
|  Idle  |--->| Speaking  |--->| Draining|---> Idle
+--------+    +----------+    +---------+
    ^              |                |
    |              | stop()         | (audio buffer empty)
    |              v                |
    |         +----------+         |
    +---------|  Idle    |<--------+
              +----------+

            speak(interrupt=true)
                    |
                    v
            +----------+
            | Speaking  |  (immediately replaces current)
            +----------+
```

**States:**
- **Idle:** No speech in progress, no audio playing.
- **Speaking:** Text is being processed through the pipeline (preprocessing, phonemization, synthesis). Audio is playing.
- **Draining:** All sentences have been synthesized. The ring buffer is draining (playing remaining audio). Transitions to Idle when the buffer is empty.

The `TtsEngine::is_speaking()` method returns `true` in both Speaking and Draining states.

---

## 14. Testing Strategy

### 14.1 Approach

TTS code is tested at three levels:

| Level | What | How | Dependencies |
|-------|------|-----|--------------|
| **Unit** | Preprocessor, sentence segmentation, model manifest parsing, state machine | Standard `#[test]`, pure Rust | None |
| **Integration** | espeak-ng subprocess protocol, full pipeline (text -> phonemes -> audio samples) | `#[test]` with real espeak-ng binary | espeak-ng installed on test machine |
| **Mock** | TTS Coordinator with mocked espeak + inference | Mock `EspeakSubprocess` and `SherpaInference` | None |

### 14.2 Mock Strategy

The espeak-ng subprocess and sherpa-onnx inference are mocked for unit testing:

```rust
#[cfg(test)]
mod tests {
    /// Generates a mock espeak subprocess that returns predetermined phonemes.
    pub fn generate_test_espeak_mock(responses: Vec<&str>) -> MockEspeakSubprocess {
        MockEspeakSubprocess { responses: responses.into_iter().map(String::from).collect(), index: 0 }
    }

    /// Generates a mock inference engine that returns silence audio.
    pub fn generate_test_inference_mock(sample_rate: u32) -> MockSherpaInference {
        MockSherpaInference { sample_rate }
    }
}
```

### 14.3 Test Naming Convention

Following the project's testing standards (CLAUDE.md), TTS tests use hierarchical prefixes:

| Test | Name |
|------|------|
| Preprocessor splits sentences correctly | `tts_preprocessor_sentence_split_basic` |
| Preprocessor handles abbreviations | `tts_preprocessor_abbreviation_expansion` |
| Preprocessor normalizes numbers | `tts_preprocessor_number_normalization` |
| espeak subprocess spawns and phonemizes | `tts_espeak_subprocess_phonemize_hello_world` |
| espeak subprocess recovers from crash | `tts_espeak_subprocess_crash_recovery` |
| espeak subprocess respects timeout | `tts_espeak_subprocess_timeout_kills_process` |
| Model manifest parses correctly | `tts_model_manifest_parse_valid_json` |
| Model manifest handles missing file | `tts_model_manifest_missing_file_returns_error` |
| Voice selection returns all voices | `tts_voice_selection_lists_all_engines` |
| Speech state machine transitions | `tts_state_machine_idle_to_speaking` |
| Interrupt stops current speech | `tts_interrupt_stops_current_and_starts_new` |
| Ring buffer handles underrun | `tts_ring_buffer_underrun_produces_silence` |
| Full pipeline text to audio | `tts_pipeline_integration_text_to_audio` |

### 14.4 CI Considerations

- **espeak-ng must be installed** in CI environments for integration tests. CI configuration installs `espeak-ng` via package manager.
- **No audio device required.** Integration tests use a mock `AudioOutput` that captures samples to a `Vec<f32>` instead of playing through cpal. This verifies the pipeline produces audio samples without requiring speakers.
- **sherpa-onnx model files** must be available for integration tests. CI downloads the Kokoro q4 model (smallest, ~80MB) as a test fixture. Tests that require a model are gated behind `#[cfg(feature = "integration_tests")]`.
- **Performance benchmarks** (latency measurement) run as separate `cargo bench` targets, not in the standard test suite.

---

## 15. Module Organization

The TTS pipeline lives in the `luminos-tts` crate within the Cargo workspace defined by [01 -- System Architecture](./01-system-architecture.md) Section 7.1. The layout below expands on the system architecture's sketch (which lists only lib.rs, espeak.rs, inference.rs, models.rs, preprocessor.rs) with additional modules identified during detailed TTS pipeline design. This document is authoritative for the luminos-tts crate structure:

```
crates/luminos-tts/
  Cargo.toml
  src/
    lib.rs                  # Public API: SherpaEngine, NativeTtsEngine
    coordinator.rs          # TtsCoordinator: pipeline orchestration, state machine
    espeak.rs               # EspeakSubprocess: spawn, communicate, crash recovery
    inference.rs            # SherpaInference: sherpa-onnx model loading and synthesis
    models.rs               # ModelManager: voice model discovery, manifest, loading
    preprocessor.rs         # TextPreprocessor: sentence split, normalization
    ring_buffer.rs          # SPSC ring buffer for audio sample transfer
    highlight.rs            # HighlightEvent generation and word timing
    native/                 # Platform-native TTS fallback implementations
      mod.rs                # NativeTtsEngine dispatcher
      linux.rs              # speech-dispatcher integration
      macos.rs              # AVSpeechSynthesizer integration
      windows.rs            # SAPI integration
    tests/
      mod.rs                # Test utilities and mock generators
      preprocessor_tests.rs
      espeak_tests.rs
      coordinator_tests.rs
      ring_buffer_tests.rs
```

### 15.1 Crate Dependencies

```
luminos-tts
  |
  +---> luminos-platform    (AudioOutput trait, TtsEngine trait, TtsError, Voice types)
  +---> sherpa-rs            (sherpa-onnx Rust bindings, TTS inference)
  +---> cpal                 (audio device access for CpalAudioOutput)
  +---> rubato               (sample rate conversion -- cpal does not resample automatically)
  +---> serde + serde_json   (model manifest parsing)
  +---> log                  (structured logging)
  +---> thiserror            (error types, used transitively via luminos-platform)
```

`luminos-tts` does **not** depend on `luminos-core`, `luminos-gpu`, or `luminos-app`. It depends only on `luminos-platform` (for trait definitions and error types). The core engine crate (`luminos-core`) depends on `luminos-tts` -- not the other way around.

---

## 16. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| TtsEngine trait definition | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 3.4 |
| AudioOutput trait definition | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 3.7 |
| Voice, TtsBackend, TtsError types | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 3.4 |
| Thread model | [01 -- System Architecture](./01-system-architecture.md) | 6.2 |
| Inter-thread channels | [01 -- System Architecture](./01-system-architecture.md) | 6.4 |
| TTS data flow (high level) | [01 -- System Architecture](./01-system-architecture.md) | 5.3 |
| Memory budget | [01 -- System Architecture](./01-system-architecture.md) | 9.3 |
| Startup sequence | [01 -- System Architecture](./01-system-architecture.md) | 9.4 |
| Kokoro vs Piper evaluation | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | 4.4 |
| espeak-ng subprocess rationale | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | 4.4 |
| TTS latency target | [Product Strategy](../PRODUCT_STRATEGY.md) | 5.3 (Product Principles, item 3) |
| Phase 2 TTS features | [Product Strategy](../PRODUCT_STRATEGY.md) | 7.3 |
| Performance targets | [01 -- System Architecture](./01-system-architecture.md) | 9.1 |
| Process isolation (security) | [01 -- System Architecture](./01-system-architecture.md) | 10.2 |
| Cargo workspace layout | [01 -- System Architecture](./01-system-architecture.md) | 7.1 |
| Control panel voice selection | [05 -- Control Panel](./05-control-panel.md) | Section 9 |
| Consolidated performance targets (TTS latency) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.1 |
| Memory budget (TTS model sizes) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 2.2 |
| Error handling policy and TtsError recovery | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 7 |
| Licensing (espeak-ng GPL-3.0, Kokoro Apache-2.0) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 4.2 |
| Privacy policy (no screen text exfiltration) | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Section 3.2 |

---

## 17. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-15 | Initial TTS pipeline strategy |
| 1.1 | 2026-03-15 | Post-audit review: fixed espeak-ng CLI flags (-q --ipa -v, not --phoneme-only --language); corrected Kokoro q8 model size (~92MB, not ~165MB); removed Korean from Kokoro language list (not supported in v1.0); fixed "four stages" to "five stages"; corrected cpal sample rate conversion claim (cpal does NOT resample -- added rubato dependency); added resampling step to ring buffer architecture; corrected AtomicF32 to AtomicU32 with bit conversion; fixed OpenBSD memory monitoring (sysctl, not /proc); added pipeline overlap threading clarification; documented queue depth limitation; reconciled memory budget with corrected model sizes; improved cross-reference precision |
