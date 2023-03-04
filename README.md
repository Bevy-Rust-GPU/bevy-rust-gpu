<div align="center">

# `🐉bevy-rust-gpu`

[![Documentation](https://img.shields.io/badge/docs-API-blue)](https://bevy-rust-gpu.github.io/bevy-rust-gpu/bevy_rust_gpu/)

A set of bevy plugins supporting the use of [`rust-gpu`](https://github.com/EmbarkStudios/rust-gpu) shader crates.

Features include hot-reloading with metadata-based entrypoint validation, and hot-recompiling via runtime export of active entrypoints.

![Hot-rebuild workflow](https://github.com/Bevy-Rust-GPU/bevy-rust-gpu/blob/static-assets/hot-rebuild-workflow.gif?raw=true)

</div>

## Usage

First, implement a `Material` type to describe your material's bind group layout and pipeline specialization.
The vertex and fragment shaders specified here will be used as the fallback if their `rust-gpu` equivalents are not available,
so can be left to the default `Material` implementation if desired.

Then, implement `RustGpuMaterial` for your `Material` type.
This will require creating marker structs to represent its vertex and fragment shaders,
and implementing the `EntryPoint` trait on them to describe entry point names and compile-time parameters.
(See [`bevy_pbr_rust.rs`](https://github.com/Bevy-Rust-GPU/bevy-rust-gpu/blob/master/src/bevy_pbr_rust.rs) for the [`bevy-pbr-rust`](https://github.com/Bevy-Rust-GPU/bevy-pbr-rust)-backed `StandardMaterial` reference implementation.)

Next, add `RustGpuPlugin` to your bevy app to configure the backend.
Currently this must occur before `RenderPlugin` is added (most often via `DefaultPlugins`), as it requires early access to `WgpuSettings` to disable storage buffer support.

For each `RustGpuMaterial` implementor, add a `RustGpuMaterialPlugin::<M>` to your app to setup rendering machinery and hot-reload / hot-rebuild support if the respective features are enabled (see below.)

When instantiating `RustGpu` materials, `RustGpuShader` handles will be required.
These are equivalent to `Handle<Shader>` with some extra hot-reloading machinery,
and can be acquired via the `AssetServer::load_rust_gpu_shader` extension method provided by the `LoadRustGpuShader` trait.

## Feature Flags

### `hot-reload`

Enables hot-reloading support.

Automatically loads the `.spv.json` metadata generated by `rust-gpu` alongside its associated `.spv` file,
and uses it to validate entry points at material specialization time.

This prevents bevy from panicking when an invalid entrypoint is requested, falls back to the default shader,
and re-specializes the material if it becomes available after a reload.

Note: AssetServer gives up on trying to load an asset if it does not exist,
so the `.spv` file must be compiled at least once prior to app startup in order to hot-reload successfully.

### `hot-rebuild`

Adds the `EntryPointExport` resource, which can be used to retrieve an `ExportHandle` corresponding to a JSON output file.
When passed to a `RustGpu` material, active entrypoints to be aggregated and exported to the corresponding file on change.

This can be used in concert with the `hot-reload` feature, [`rust-gpu-builder`](https://github.com/Bevy-Rust-GPU/rust-gpu-builder)'s file watching functionality,
and [`permutate-macro`](https://github.com/Bevy-Rust-GPU/permutate-macro)'s static permutation generation to drive a hot-rebuild workflow:

* The bevy app loads a `RustGpu` material, tries to specialize it, and exports the set of required entry points to `entry_points.json`
* [`rust-gpu-builder`](https://github.com/Bevy-Rust-GPU/rust-gpu-builder) picks up the change to `entry_points.json` and triggers a recompile
* [`permutate-macro`](https://github.com/Bevy-Rust-GPU/permutate-macro) attributes in the target shader crates read `entry_points.json`, and conditionally generate the required entry points
* `rust-gpu` compiles the generated code, outputting `shader.spv` and `shader.spv.json`
* The bevy app picks up the changes to `shader.spv` and `shader.spv.json`, hot-reloads them, and respecializes the material with the now-available entry points
* Repeat as new `RustGpu` materials are loaded by the bevy app

### `bevy-pbr-rust`

Implements `RustGpu` for `StandardMaterial` via the `MeshVertex` and `PbrFragment` markers,
which corresponding to entry points defined in [`bevy-pbr-rust`](https://github.com/Bevy-Rust-GPU/bevy-pbr-rust).
