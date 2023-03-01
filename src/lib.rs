//! # bevy-rust-gpu
//!
//! A set of `bevy` plugins supporting the use of `rust-gpu` shaders.
//!
//! Features include hot-reloading, metadata-based entry point validation,
//! and active entry point export.
//!
//! Can be used in conjunction with `rust-gpu-builder` and `permutate-macro`
//! to drive a real-time shader recompilation pipeline.


pub mod entry_point;
pub mod plugin;
pub mod rust_gpu_material;

#[cfg(feature = "shader-meta")]
pub mod shader_meta;

#[cfg(feature = "entry-point-export")]
pub mod entry_point_export;

pub mod prelude;
