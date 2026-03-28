//! Text-to-speech pipeline for Luminos.
//!
//! Manages espeak-ng subprocess phonemization, Kokoro ONNX inference
//! via sherpa-rs, and cpal audio output with ring buffer streaming.
