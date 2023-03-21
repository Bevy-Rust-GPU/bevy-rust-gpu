//! # bevy-rust-gpu
//!
//! A set of `bevy` plugins supporting the use of `rust-gpu` shaders.
//!
//! Features include hot-reloading, metadata-based entry point validation,
//! and active entry point export.
//!
//! Can be used in conjunction with `rust-gpu-builder` and `permutate-macro`
//! to drive a real-time shader recompilation pipeline.

mod builder_output;
mod entry_point;
mod plugin;
mod rust_gpu;
mod rust_gpu_material;

pub use entry_point::*;
pub use plugin::RustGpuPlugin;
pub use rust_gpu::*;
pub use rust_gpu_material::RustGpuMaterial;

pub use rust_gpu_builder_shared::{RustGpuBuilderModules, RustGpuBuilderOutput};

#[cfg(feature = "hot-rebuild")]
pub mod entry_point_export;

#[cfg(feature = "bevy-pbr-rust")]
pub mod bevy_pbr_rust;

pub mod prelude;
