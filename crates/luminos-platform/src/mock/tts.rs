//! Mock implementation of [`TtsEngine`].

use std::future::Future;
use std::pin::Pin;

use crate::traits::{TtsEngine, TtsError, Voice};

/// Mock implementation of `TtsEngine` for unit testing.
///
/// The `speak` method returns a boxed future that resolves immediately.
/// Supports error injection and basic state tracking (`is_speaking`).
pub struct MockTtsEngine {
    /// Voices returned by `get_voices()`.
    voices: Vec<Voice>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> TtsError + Send + Sync>>,
}

impl MockTtsEngine {
    /// Creates a mock with a pre-configured voice list.
    #[must_use]
    pub fn generate_test_mock_tts_engine(voices: Vec<Voice>) -> Self {
        Self {
            voices,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    #[must_use]
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> TtsError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

impl TtsEngine for MockTtsEngine {
    fn speak(
        &self,
        _text: &str,
        _interrupt: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), TtsError>> + Send + '_>> {
        if let Some(ref factory) = self.error_factory {
            let err = factory();
            return Box::pin(async move { Err(err) });
        }
        Box::pin(async { Ok(()) })
    }

    fn stop(&self) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_voice(&self, _voice_id: &str) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_rate(&self, _rate: f32) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_pitch(&self, _pitch: f32) -> Result<(), TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn get_voices(&self) -> Result<Vec<Voice>, TtsError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.voices.clone())
    }

    fn is_speaking(&self) -> bool {
        // Mock never speaks -- always returns false
        false
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::traits::tts_engine::TtsBackend;

    fn generate_test_voice() -> Voice {
        Voice {
            id: "kokoro-af_heart".to_string(),
            name: "Heart".to_string(),
            language: "en-US".to_string(),
            requires_download: false,
            engine: TtsBackend::Kokoro,
        }
    }

    #[tokio::test]
    async fn mock_tts_engine_speak_success() {
        let voices = vec![generate_test_voice()];
        let tts = MockTtsEngine::generate_test_mock_tts_engine(voices);

        let result = tts.speak("hello world", false).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn mock_tts_engine_speak_with_error_returns_injected_error() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![])
            .with_error(|| TtsError::VoiceNotFound("missing".into()));

        let result = tts.speak("hello", false).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TtsError::VoiceNotFound(id) if id == "missing"
        ));
    }

    #[test]
    fn mock_tts_engine_stop_success() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![]);

        let result = tts.stop();
        assert!(result.is_ok());
    }

    #[test]
    fn mock_tts_engine_stop_with_error() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![])
            .with_error(|| TtsError::VoiceNotFound("missing".into()));

        let result = tts.stop();
        assert!(result.is_err());
    }

    #[test]
    fn mock_tts_engine_set_voice_success() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![]);

        let result = tts.set_voice("kokoro-af_heart");
        assert!(result.is_ok());
    }

    #[test]
    fn mock_tts_engine_set_rate_success() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![]);

        let result = tts.set_rate(1.5);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_tts_engine_set_pitch_success() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![]);

        let result = tts.set_pitch(1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_tts_engine_get_voices_returns_configured_voices() {
        let voices = vec![generate_test_voice()];
        let tts = MockTtsEngine::generate_test_mock_tts_engine(voices.clone());

        let result = tts.get_voices().unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, voices[0].id);
        assert_eq!(result[0].name, voices[0].name);
    }

    #[test]
    fn mock_tts_engine_get_voices_with_error() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![])
            .with_error(|| TtsError::VoiceNotFound("missing".into()));

        let result = tts.get_voices();
        assert!(result.is_err());
    }

    #[test]
    fn mock_tts_engine_is_speaking_returns_false() {
        let tts = MockTtsEngine::generate_test_mock_tts_engine(vec![]);

        assert!(!tts.is_speaking());
    }
}
