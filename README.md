<div align="center">

# `🐉bevy-rust-gpu`

[![Documentation](https://img.shields.io/badge/docs-API-blue)](https://bevy-rust-gpu.github.io/bevy-rust-gpu/bevy_rust_gpu/)

A bevy plugin supporting the use of [`rust-gpu`](https://github.com/EmbarkStudios/rust-gpu) shader crates.
Part of the [Bevy Rust-GPU](https://github.com/Bevy-Rust-GPU) suite.

Features include hot-reloading with metadata-based entrypoint validation, and hot-recompiling via runtime export of active entrypoints.

![Hot-rebuild workflow](https://github.com/Bevy-Rust-GPU/bevy-rust-gpu/blob/static-assets/hot-rebuild-workflow.gif?raw=true)

</div>

## Current Status 🚧

`bevy-rust-gpu` relies on [`rust-gpu`](https://github.com/EmbarkStudios/rust-gpu), which is in active development.

As such, its use implies all the caveats of the above, plus the use of [a custom `bevy` fork](https://github.com/Bevy-Rust-GPU/bevy-rust-gpu/issues/12).

Beyond that, `bevy-rust-gpu` is also in active development, but has a relatively small user-facing API footprint.
Major changes will be driven by development in the upstream `bevy` and `rust-gpu` crates.

In practical terms, its current state is able to support the hot-rebuild workflow depicted above,
and allows for relatively complex shader implementations, such as [a Rust reimplementation of `bevy_pbr`](https://github.com/Bevy-Rust-GPU/bevy-pbr-rust).

Currently, none of the [Bevy Rust-GPU](https://github.com/Bevy-Rust-GPU) crates are published on crates.io;
this may change as and when the major caveats are solved, but in the meantime will be hosted on github and versioned by tag.

## Usage

First, add `bevy-rust-gpu` to your `Cargo.toml`:

```toml
[dependencies]
bevy-rust-gpu = { git = "https://github.com/Bevy-Rust-GPU/bevy-rust-gpu", tag = "v0.4.0" }
```

Next, implement a `Material` type to describe your material's bind group layout and pipeline specialization:

```rust
#[derive(Debug, Default, Copy, Clone, AsBindGroup, TypeUuid)]
#[uuid = "786779ff-e3ac-4b36-ae96-f4844f8e3064"]
struct MyRustGpuMaterial {
    #[uniform(0)]
    color: Vec4,
}

// The vertex and fragment shaders specified here can be used
// as a fallback when entrypoints are unavailable
// (see the documentation of bevy_rust_gpu::prelude::RustGpuSettings),
// but are otherwise deferred to ShaderRef::Default, so can be left unimplemented.
impl Material for MyRustGpuMaterial {}
```

Then, implement `RustGpuMaterial` for your `Material` type.

```rust
// First, implement some marker structs to represent our shader entry points

pub enum MyVertex {}

impl EntryPoint for MyVertex {
    const NAME: EntryPointName = "vertex";
    const PARAMETERS: EntryPointParameters = &[];
    const CONSTANTS: EntryPointConstants = &[];
}

pub enum MyFragment {}

impl EntryPoint for MyFragment {
    const NAME: EntryPointName = "fragment";
    const PARAMETERS: EntryPointParameters = &[];
    const CONSTANTS: EntryPointConstants = &[];
}

// Then, impl RustGpuMaterial for our material to tie them together

impl RustGpuMaterial for MyRustGpuMaterial {
    type Vertex = MyVertex;
    type Fragment = MyFragment;
}
```

(See [`bevy_pbr_rust.rs`](https://github.com/Bevy-Rust-GPU/bevy-rust-gpu/blob/master/src/bevy_pbr_rust.rs) for the [`bevy-pbr-rust`](https://github.com/Bevy-Rust-GPU/bevy-pbr-rust)-backed `StandardMaterial` reference implementation.)

Next, add `RustGpuPlugin` to your bevy app to configure the backend.

```rust
    let mut app = App::default();

    app.add_plugin(RustGpuPlugin);
```

For each `RustGpuMaterial` implementor, add a `RustGpuMaterialPlugin::<M>` to your app to setup backend rendering machinery.
This will also configure hot-reloading and hot-rebuilding if the corresponding features are enabled.

```rust
    app.add_plugin(RustGpuMaterialPlugin::<MyRustGpuMaterial>::default());

```

If using hot-rebuilding, tell the material where to export its entry points:
```rust
    RustGpu::<ExampleMaterial>::export_to(ENTRY_POINTS_PATH);
```

Rust-GPU shader assets are represented by `.rust-gpu.json` files. This is the combined SPIR-V binary and entry point metadata output by `rust-gpu-builder`,
and can be hot reloaded on change via `AssetServer` in the same way as regular `Shader` assets.

Given one of these files, the last steps are to load it via `AssetServer`, and add it to a `RustGpu` material:

```rust
fn setup(materials: ResMut<Assets<RustGpu<MyRustGpuMaterial>>>) {
    let shader = asset_server.load::<RustGpuBuilderOutput>(SHADER_PATH);

    // Add it to a RustGpu material, which can be used with bevy's MaterialMeshBundle
    let material = materials.add(RustGpu {
        vertex_shader = Some(shader),
        fragment_shader = Some(shader),
        ..default()
    });

    // Create cube mesh
    let mesh = meshes.add(Cube { size: 1.0 }.into());
    
    // Spawn a mesh with our rust-gpu material
    commands.spawn(MaterialMeshBundle {
        mesh,
        material,
        ..default()
    });
}
```

## Feature Flags

### `hot-rebuild`

Enables hot-rebuilding support.

`RustGpu` gains a new `export_to` function, which will register it for entry point aggregation, and export to the provided path alongside any other materials pointing there.

This can be used in concert with the `hot-reload` feature, [`rust-gpu-builder`](https://github.com/Bevy-Rust-GPU/rust-gpu-builder)'s file watching functionality,
and [`permutate-macro`](https://github.com/Bevy-Rust-GPU/permutate-macro)'s static permutation generation to drive a hot-rebuild workflow on par with bevy's WGSL user experience:

* The bevy app loads a `RustGpu` material, tries to specialize it, and exports the set of required entry points to `entry_points.json`
* [`rust-gpu-builder`](https://github.com/Bevy-Rust-GPU/rust-gpu-builder) picks up the change to `entry_points.json` and triggers a recompile
* [`permutate-macro`](https://github.com/Bevy-Rust-GPU/permutate-macro) attributes in the target shader crates read `entry_points.json`, and conditionally generate the required entry points
* `rust-gpu` compiles the generated code, outputting `shader.spv` and `shader.spv.json`
* The bevy app picks up the changes to `shader.spv` and `shader.spv.json`, hot-reloads them, and respecializes the material with the now-available entry points
* Repeat as new `RustGpu` materials are loaded by the bevy app

### `bevy-pbr-rust`

Implements `RustGpu` for `StandardMaterial` via the `MeshVertex` and `PbrFragment` markers,
which corresponding to entry points defined in [`bevy-pbr-rust`](https://github.com/Bevy-Rust-GPU/bevy-pbr-rust).
