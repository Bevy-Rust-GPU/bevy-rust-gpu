//! Plugin systems.

use bevy::prelude::{info, AssetEvent, Assets, EventReader, Res, ResMut, Shader};

use crate::{
    prelude::{ChangedShaders, ModuleMeta, RustGpuMaterial, SHADER_META, SHADER_META_MAP},
    RustGpu,
};

/// Listens for [`Shader`](bevy::prelude::Shader) asset events, clears metadata if the respective flag is enabled,
/// and aggregates change events for application in [`reload_materials`].
pub fn shader_events<M>(
    mut shader_events: EventReader<AssetEvent<Shader>>,
    mut changed_shaders: ResMut<ChangedShaders>,
) where
    M: RustGpuMaterial,
{
    for event in shader_events.iter() {
        match event {
            AssetEvent::Created {
                handle: shader_handle,
            }
            | AssetEvent::Modified {
                handle: shader_handle,
            } => {
                #[cfg(feature = "hot-reload")]
                // Remove meta in case the shader and meta load on different frames
                SHADER_META.write().unwrap().remove(shader_handle);

                // Mark this shader for material reloading
                changed_shaders.insert(shader_handle.clone_weak());
            }
            _ => (),
        }
    }
}

/// Consumes aggregated shader change events and re-specializes affected
/// [`RustGpu`] materials.
pub fn reload_materials<M>(
    mut changed_shaders: ResMut<ChangedShaders>,
    mut materials: ResMut<Assets<RustGpu<M>>>,
) where
    M: RustGpuMaterial,
{
    // Reload all materials with shaders that have changed
    for (_, material) in materials.iter_mut() {
        let mut reload = false;

        if let Some(vertex_shader) = &material.vertex_shader {
            if changed_shaders.contains(&vertex_shader.0) {
                reload = true;
            }
        }

        if let Some(fragment_shader) = &material.fragment_shader {
            if changed_shaders.contains(&fragment_shader.0) {
                reload = true;
            }
        }

        if reload {
            material.iteration += 1;
        }
    }

    changed_shaders.clear();
}

#[cfg(feature = "hot-reload")]
/// Listens for [`ModuleMeta`] asset events, updates backend data,
/// and aggregates change events for application in [`reload_materials`].
pub fn module_meta_events<M>(
    mut module_meta_events: EventReader<AssetEvent<ModuleMeta>>,
    assets: Res<Assets<ModuleMeta>>,
    mut changed_shaders: ResMut<ChangedShaders>,
) where
    M: RustGpuMaterial,
{
    let shader_meta_map = SHADER_META_MAP.read().unwrap();

    for event in module_meta_events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                // If this meta has an associated shader, mark it for material reloading
                if let Some(shader) = shader_meta_map.shader(handle) {
                    changed_shaders.insert(shader.clone_weak());

                    // Update module meta
                    if let Some(asset) = assets.get(handle) {
                        info!("Updating shader meta for {handle:?}");
                        SHADER_META
                            .write()
                            .unwrap()
                            .insert(shader.clone_weak(), asset.clone());
                    }
                }
            }
            _ => (),
        }
    }
}
