//! Base Rust-GPU plugin

use bevy::{
    prelude::{default, info, Plugin},
    render::{settings::WgpuSettings, RenderPlugin},
};

/// Handles enforcing WGPU limitations required by `rust-gpu`
pub struct BevyRustGpuPlugin;

impl Plugin for BevyRustGpuPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Panic if added too late for `WgpuSettings` to take effect
        if app.is_plugin_added::<RenderPlugin>() {
            panic!("BevyRustGpuPlugin must be added before bevy_render::RenderPlugin");
        }

        // Forcibly disable storage buffers to account for rust-gpu limitations
        let mut wgpu_settings = app
            .world
            .get_resource_or_insert_with::<WgpuSettings>(default);

        let constrained_limits = match &mut wgpu_settings.constrained_limits {
            Some(constrained_limits) => {
                info!("Constrained limits exists");
                constrained_limits
            }
            None => {
                info!("Constrained limits does not exist");
                wgpu_settings.constrained_limits = Some(wgpu_settings.limits.clone());
                wgpu_settings.constrained_limits.as_mut().unwrap()
            }
        };

        info!("Setting max storage buffers per shader stage");
        constrained_limits.max_storage_buffers_per_shader_stage = 0;
    }
}

