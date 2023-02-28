# `bevy-rust-gpu`

Bevy plugin for hot-recompiling and hot-reloading `rust-gpu` shader crates.

Provides traits for defining interfaces to `rust-gpu` entrypoints and using them via material.

Adds optional support for hot-reloading `rust-gpu` materials based on their `.spv.json` entrypoint metadata.

Adds optional support for exporting used entrypoints to a JSON file for use with `permutate-macro`.

