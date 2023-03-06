//! Main Rust-GPU plugin.

use bevy::{
    prelude::{default, Plugin},
    render::settings::{WgpuLimits, WgpuSettings},
};

use crate::prelude::ChangedShaders;

/// Enforces WGPU limitations required by `rust-gpu`,
/// and runs initial backend setup.
pub struct RustGpuPlugin;

impl Plugin for RustGpuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Initialize `ChangedShaders` resource
        app.init_resource::<ChangedShaders>();

        // Add entry point export plugin
        #[cfg(feature = "hot-rebuild")]
        app.add_plugin(crate::prelude::EntryPointExportPlugin);
    }
}

impl RustGpuPlugin {
    pub fn wgpu_settings() -> WgpuSettings {
        WgpuSettings {
            constrained_limits: Some(WgpuLimits {
                max_storage_buffers_per_shader_stage: 0,
                ..default()
            }),
            ..default()
        }
    }
}
