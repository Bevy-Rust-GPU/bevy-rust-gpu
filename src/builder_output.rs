use std::{collections::BTreeMap, sync::RwLock};

use bevy::prelude::{
    default, AssetEvent, Assets,  Deref, DerefMut, EventReader, Handle, 
    Plugin, Res, ResMut, Shader, PreUpdate,
};
use once_cell::sync::Lazy;
use rust_gpu_builder_shared::RustGpuBuilderOutput;

/// Static container for `RustGpuArtifacts` to allow access from `Material::specialize`
pub static RUST_GPU_ARTIFACTS: Lazy<RwLock<RustGpuArtifacts>> = Lazy::new(default);

pub struct BuilderOutputPlugin;

impl Plugin for BuilderOutputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "json")]
        app.add_plugin(bevy_common_assets::json::JsonAssetPlugin::<
            RustGpuBuilderOutput,
        >::new(&["rust-gpu.json"]));

        #[cfg(feature = "msgpack")]
        app.add_plugins(bevy_common_assets::msgpack::MsgPackAssetPlugin::<
            RustGpuBuilderOutput,
        >::new(&["rust-gpu.msgpack"]));

        app.add_systems(PreUpdate, builder_output_events);
    }
}

/// Module shader handle container.
#[derive(Debug, Clone)]
pub enum RustGpuModules {
    /// Contains a single unnamed shader.
    Single(Handle<Shader>),
    /// Contains multiple named shaders.
    Multi(BTreeMap<String, Handle<Shader>>),
}

/// Asset containing loaded rust-gpu shaders and entry point metadata.
#[derive(Debug, Clone)]
pub struct RustGpuArtifact {
    pub entry_points: Vec<String>,
    pub modules: RustGpuModules,
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct RustGpuArtifacts {
    pub artifacts: BTreeMap<Handle<RustGpuBuilderOutput>, RustGpuArtifact>,
}

/// [`RustGpuBuilderOutput`] asset event handler.
///
/// Handles loading shader assets, maintaining static material data, and respecializing materials on reload.
pub fn builder_output_events(
    mut builder_output_events: EventReader<AssetEvent<RustGpuBuilderOutput>>,
    builder_outputs: Res<Assets<RustGpuBuilderOutput>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    for event in builder_output_events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            let asset = builder_outputs.get(handle).unwrap().clone();

            // Create a `RustGpuArtifact` from the affected asset
            let artifact = match asset.modules {
                rust_gpu_builder_shared::RustGpuBuilderModules::Single(ref single) => {
                    let shader = shaders.add(Shader::from_spirv(single.clone(), ""));
                    RustGpuArtifact {
                        entry_points: asset.entry_points,
                        modules: RustGpuModules::Single(shader),
                    }
                }
                rust_gpu_builder_shared::RustGpuBuilderModules::Multi(multi) => RustGpuArtifact {
                    entry_points: asset.entry_points,
                    modules: RustGpuModules::Multi(
                        multi
                            .into_iter()
                            .map(|(k, module)| (k.clone(), shaders.add(Shader::from_spirv(module, ""))))
                            .collect(),
                    ),
                },
            };

            // Emplace it in static storage
            RUST_GPU_ARTIFACTS
                .write()
                .unwrap()
                .insert(handle.clone_weak(), artifact);
        }

        // On remove, remove the corresponding artifact from static storage
        if let AssetEvent::Removed { handle } = event {
            RUST_GPU_ARTIFACTS.write().unwrap().remove(&handle);
        }
    }
}
