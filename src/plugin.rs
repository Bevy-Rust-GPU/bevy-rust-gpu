//! Main Rust-GPU plugin.

use std::path::PathBuf;

use bevy::prelude::Plugin;

use crate::prelude::{file_writer, BuilderOutputPlugin, EntryPoints};

/// Main Rust-GPU plugin.
///
/// Adds support for `RustGpuBuilderOutput` assets,
/// and configures entry point export if the `hot-reload` feature is enabled.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RustGpuPlugin<F> {
    #[cfg(feature = "hot-rebuild")]
    pub export_writer: F,

    #[cfg(not(feature = "hot-rebuild"))]
    pub _phantom: PhantomData<F>,
}

impl Default for RustGpuPlugin<fn(PathBuf, EntryPoints)> {
    fn default() -> Self {
        Self {
            #[cfg(target_family = "wasm")]
            export_writer: |_, _| (),

            #[cfg(not(target_family = "wasm"))]
            export_writer: file_writer,
        }
    }
}

impl<F> Plugin for RustGpuPlugin<F>
where
    F: Fn(PathBuf, EntryPoints) + Clone + Send + Sync + 'static,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(BuilderOutputPlugin);

        #[cfg(feature = "hot-rebuild")]
        app.add_plugin(crate::prelude::EntryPointExportPlugin {
            writer: self.export_writer.clone(),
        });
    }
}
