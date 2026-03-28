//! Mock implementation of [`AudioOutput`].

use crate::traits::{AudioError, AudioOutput, AudioSample};

/// Mock implementation of `AudioOutput` for unit testing.
///
/// All methods succeed by default. Does not actually play audio.
pub struct MockAudioOutput {
    /// Device name returned by `get_default_device_name()`.
    device_name: Option<String>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> AudioError + Send + Sync>>,
}

impl MockAudioOutput {
    /// Creates a mock with default (success) behavior.
    pub fn generate_test_mock_audio_output() -> Self {
        Self {
            device_name: Some("Mock Audio Device".to_string()),
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> AudioError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

impl AudioOutput for MockAudioOutput {
    fn play_audio(&self, _sample: AudioSample, _interrupt: bool) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn stop_audio(&self) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_volume(&self, _volume: f32) -> Result<(), AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn get_default_device_name(&self) -> Result<Option<String>, AudioError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.device_name.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_test_audio_sample() -> AudioSample {
        AudioSample {
            data: vec![0.0, 0.5, -0.5, 1.0],
            sample_rate: 24000,
            channels: 1,
        }
    }

    #[test]
    fn mock_audio_output_play_audio_success() {
        let audio = MockAudioOutput::generate_test_mock_audio_output();
        let sample = generate_test_audio_sample();

        let result = audio.play_audio(sample, false);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_audio_output_play_audio_with_error_returns_no_device() {
        let audio =
            MockAudioOutput::generate_test_mock_audio_output().with_error(|| AudioError::NoDevice);
        let sample = generate_test_audio_sample();

        let result = audio.play_audio(sample, false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AudioError::NoDevice));
    }

    #[test]
    fn mock_audio_output_stop_audio_success() {
        let audio = MockAudioOutput::generate_test_mock_audio_output();

        let result = audio.stop_audio();
        assert!(result.is_ok());
    }

    #[test]
    fn mock_audio_output_stop_audio_with_error() {
        let audio =
            MockAudioOutput::generate_test_mock_audio_output().with_error(|| AudioError::NoDevice);

        let result = audio.stop_audio();
        assert!(result.is_err());
    }

    #[test]
    fn mock_audio_output_set_volume_success() {
        let audio = MockAudioOutput::generate_test_mock_audio_output();

        let result = audio.set_volume(0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_audio_output_set_volume_with_error() {
        let audio =
            MockAudioOutput::generate_test_mock_audio_output().with_error(|| AudioError::NoDevice);

        let result = audio.set_volume(0.5);
        assert!(result.is_err());
    }

    #[test]
    fn mock_audio_output_get_default_device_name_returns_configured_name() {
        let audio = MockAudioOutput::generate_test_mock_audio_output();

        let result = audio.get_default_device_name().unwrap();
        assert_eq!(result, Some("Mock Audio Device".to_string()));
    }

    #[test]
    fn mock_audio_output_get_default_device_name_with_error() {
        let audio =
            MockAudioOutput::generate_test_mock_audio_output().with_error(|| AudioError::NoDevice);

        let result = audio.get_default_device_name();
        assert!(result.is_err());
    }
}
