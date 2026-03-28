//! TTS engine trait and associated types.
//!
//! Defines the [`TtsEngine`] trait for text-to-speech synthesis, along with
//! [`Voice`], [`TtsBackend`], and [`TtsError`].

use std::future::Future;
use std::pin::Pin;

/// Metadata about an available TTS voice.
#[derive(Debug, Clone)]
pub struct Voice {
    /// Unique identifier for this voice (e.g., "kokoro-af_heart", "system-en-us-jenny").
    pub id: String,
    /// Human-readable name (e.g., "Heart (American English)").
    pub name: String,
    /// BCP 47 language tag (e.g., "en-US", "ja-JP").
    pub language: String,
    /// Whether this voice requires a model download before use.
    pub requires_download: bool,
    /// The engine providing this voice.
    pub engine: TtsBackend,
}

/// Identifies which TTS engine provides a voice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TtsBackend {
    /// Kokoro model via sherpa-onnx runtime.
    Kokoro,
    /// Piper VITS model via sherpa-onnx runtime (language breadth fallback).
    Piper,
    /// Platform-native TTS (`AVSpeech`, SAPI, speech-dispatcher).
    Native,
}

/// Errors that can occur during TTS operations.
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    /// The requested voice is not installed or available.
    #[error("voice not found: '{0}'")]
    VoiceNotFound(String),

    /// The TTS model failed to load (corrupted, missing, incompatible).
    #[error("model load failed: {message}")]
    ModelLoadFailed {
        /// Description of the load failure.
        message: String,
    },

    /// The phonemizer (espeak-ng subprocess) failed.
    #[error("phonemizer error: {message}")]
    PhonemizerFailed {
        /// Description of the phonemizer failure.
        message: String,
    },

    /// The inference engine returned an error.
    #[error("inference error: {message}")]
    InferenceFailed {
        /// Description of the inference failure.
        message: String,
    },

    /// The audio output device is unavailable.
    #[error("audio output unavailable: {message}")]
    AudioUnavailable {
        /// Description of the audio failure.
        message: String,
    },

    /// A platform-specific error occurred.
    #[error("platform TTS error: {message}")]
    Platform {
        /// Description of the platform error.
        message: String,
    },
}

/// Text-to-speech engine abstraction.
///
/// The `speak` method is async because TTS involves I/O-bound work:
/// subprocess communication with espeak-ng for phonemization, ONNX model
/// inference, and audio buffer playback. The method returns when audio
/// playback begins (not when it completes).
///
/// # Object Safety
///
/// This trait is **object-safe** (`dyn TtsEngine` is supported). The
/// `speak` method returns a boxed future rather than using RPITIT
/// (`-> impl Future`) to preserve object safety.
pub trait TtsEngine: Send + Sync {
    /// Speaks the given text asynchronously.
    ///
    /// Returns when audio playback begins. Target: <200ms from call to first audio.
    ///
    /// If `interrupt` is `true`, stops current speech and begins new speech.
    /// If `false`, queues after current speech completes.
    fn speak(
        &self,
        text: &str,
        interrupt: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), TtsError>> + Send + '_>>;

    /// Stops any speech currently in progress or queued.
    ///
    /// # Errors
    ///
    /// Returns [`TtsError`] if stopping speech fails.
    fn stop(&self) -> Result<(), TtsError>;

    /// Sets the active voice by its ID.
    ///
    /// # Errors
    ///
    /// Returns [`TtsError`] if the voice is not found or cannot be activated.
    fn set_voice(&self, voice_id: &str) -> Result<(), TtsError>;

    /// Sets the speech rate. 1.0 = normal, clamped to \[0.25, 4.0\].
    ///
    /// # Errors
    ///
    /// Returns [`TtsError`] if the rate cannot be applied.
    fn set_rate(&self, rate: f32) -> Result<(), TtsError>;

    /// Sets the speech pitch. 1.0 = normal, clamped to \[0.5, 2.0\].
    ///
    /// # Errors
    ///
    /// Returns [`TtsError`] if the pitch cannot be applied.
    fn set_pitch(&self, pitch: f32) -> Result<(), TtsError>;

    /// Returns all available voices across all engines.
    ///
    /// # Errors
    ///
    /// Returns [`TtsError`] if the voice list cannot be retrieved.
    fn get_voices(&self) -> Result<Vec<Voice>, TtsError>;

    /// Returns `true` if speech is currently being played.
    fn is_speaking(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_voice_fields_and_tts_backend_variants() {
        let voice = Voice {
            id: "kokoro-af_heart".into(),
            name: "Heart".into(),
            language: "en-US".into(),
            requires_download: true,
            engine: TtsBackend::Kokoro,
        };
        assert_eq!(voice.id, "kokoro-af_heart");
        assert_eq!(voice.engine, TtsBackend::Kokoro);

        // Verify all 3 variants exist and have correct derives
        let backends = [TtsBackend::Kokoro, TtsBackend::Piper, TtsBackend::Native];
        for b in &backends {
            let _ = format!("{b:?}");
            let cloned = b.clone();
            assert_eq!(b, &cloned);
        }
    }

    #[test]
    fn error_tts_error_display_voice_not_found() {
        let err = TtsError::VoiceNotFound("missing-voice".into());
        assert_eq!(err.to_string(), "voice not found: 'missing-voice'");
    }

    #[test]
    fn error_tts_error_display_model_load_failed() {
        let err = TtsError::ModelLoadFailed {
            message: "corrupted ONNX".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("model load failed"));
        assert!(msg.contains("corrupted ONNX"));
    }

    #[test]
    fn error_tts_error_display_phonemizer_failed() {
        let err = TtsError::PhonemizerFailed {
            message: "espeak-ng crashed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("phonemizer error"));
        assert!(msg.contains("espeak-ng crashed"));
    }

    #[test]
    fn error_tts_error_display_inference_failed() {
        let err = TtsError::InferenceFailed {
            message: "OOM".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("inference error"));
        assert!(msg.contains("OOM"));
    }

    #[test]
    fn error_tts_error_display_audio_unavailable() {
        let err = TtsError::AudioUnavailable {
            message: "no device".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("audio output unavailable"));
        assert!(msg.contains("no device"));
    }

    #[test]
    fn error_tts_error_display_platform() {
        let err = TtsError::Platform {
            message: "SAPI error".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("platform TTS error"));
        assert!(msg.contains("SAPI error"));
    }
}
