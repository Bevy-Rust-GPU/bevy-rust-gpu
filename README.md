# `bevy-rust-gpu`

A set of bevy plugins supporting the use of `rust-gpu` crates.

Feature includes hot-reloading, metadata-based entrypoint validation, and hot-recompiling via runtime export of active entrypoints.

## Usage

`bevy-rust-gpu`'s core function is to codify `rust-gpu` shaders as an easy-to-use `bevy` asset.

This is achieved via the `EntryPoint` trait, which can be implemented on a type to describe an entry point within a SPIR-V shader.

Implementors of this trait can be plugged into `RustGpuMaterial` alongside handles to the corresponding SPIR-V module,
which will then specialize its render machinery appropriately.

## Feature Flags

### `shader-meta`

Adds `ShaderMetaPlugin` and the `ModuleMeta` asset, which represents a `.spv.json` metadata file.
This can be inserted into the `ShaderMetaMap` resource to enable runtime entrypoint validation.

This will prevent bevy from panicking when loading a shader with a missing entrypoint, fall back to the default,
and re-specialize the material if it becomes available after a reload.

### `entry-point-export`

Adds `EntryPointExportPlugin` and the `EntryPointExport` resource, which can be used to retrieve an `EntryPointSender` represending an output JSON file.
When passed into `RustGpuMaterial`, this will cause active entrypoints to be aggregated and exported to the corresponding file.

This can be used in concert with `shader-meta`'s entrypoint validation, `rust-gpu-builder`'s file watching functionality,
and `permutate-macro`'s static permutation generation to drive a hot-recompile workflow:

* The bevy app loads a `RustGpuMaterial`, tries to specialize it, and exports the set of required entry points to `entry_points.json`
* `rust-gpu-builder` picks up the change to `entry_points.json` and triggers a recompile
* `permutate-macro` attributes in the target shader crates read `entry_points.json`, and conditionally generate the required entry points
* `rust-gpu` compiles the generated code, outputting `shader.spv` and `shader.spv.json`
* The bevy app picks up the changes to `shader.spv` and `shader.spv.json`, hot-reloads them, and respecializes the material with the now-available entry points
* Repeat as new materials are loaded by the bevy app
