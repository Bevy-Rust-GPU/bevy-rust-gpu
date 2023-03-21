//! Main Rust-GPU plugin.

use bevy::prelude::Plugin;

use crate::prelude::BuilderOutputPlugin;

/// Main Rust-GPU plugin.
///
/// Adds support for `RustGpuBuilderOutput` assets,
/// and configures entry point export if the `hot-reload` feature is enabled.
pub struct RustGpuPlugin;

impl Plugin for RustGpuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BuilderOutputPlugin);

        #[cfg(feature = "hot-rebuild")]
        app.add_plugin(crate::prelude::EntryPointExportPlugin);
    }
}
