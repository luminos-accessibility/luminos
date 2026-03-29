//! GPU-accelerated rendering pipeline for Luminos.
//!
//! Implements screen magnification via wgpu shaders, frame capture
//! processing, and the winit-based magnification overlay window.

pub mod device;
pub mod error;
pub mod frame_timings;
pub mod renderer;
pub mod shaders;
pub mod surface;
pub mod texture;
pub mod viewport;
