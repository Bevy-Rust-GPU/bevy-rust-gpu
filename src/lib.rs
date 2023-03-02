//! # bevy-rust-gpu
//!
//! A set of `bevy` plugins supporting the use of `rust-gpu` shaders.
//!
//! Features include hot-reloading, metadata-based entry point validation,
//! and active entry point export.
//!
//! Can be used in conjunction with `rust-gpu-builder` and `permutate-macro`
//! to drive a real-time shader recompilation pipeline.

mod entry_point;
mod load_rust_gpu_shader;
mod plugin;
mod rust_gpu;
mod rust_gpu_material;

pub mod systems;

pub use entry_point::*;
pub use load_rust_gpu_shader::LoadRustGpuShader;
pub use plugin::RustGpuPlugin;
pub use rust_gpu::*;
pub use rust_gpu_material::RustGpuMaterial;

#[cfg(feature = "bevy-pbr-rust")]
pub mod bevy_pbr_rust;

#[cfg(feature = "shader-meta")]
pub mod shader_meta;

#[cfg(feature = "entry-point-export")]
pub mod entry_point_export;

pub mod prelude;
