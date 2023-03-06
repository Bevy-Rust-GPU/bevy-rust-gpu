//! Adds support for loading `.spv.json` metadata.

use std::{marker::PhantomData, sync::RwLock};

use once_cell::sync::Lazy;

use bevy::{
    prelude::{
        default, info, AssetEvent, Assets, CoreSet, Deref, DerefMut, EventReader, Handle,
        IntoSystemConfig, Plugin, Res, ResMut, Resource, Shader,
    },
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_common_assets::json::JsonAssetPlugin;

use serde::{Deserialize, Serialize};

use crate::{
    prelude::{reload_materials, RustGpuMaterial},
    systems::shader_events,
    ChangedShaders,
};

pub(crate) static SHADER_META: Lazy<RwLock<ShaderMeta>> = Lazy::new(Default::default);
pub(crate) static SHADER_META_MAP: Lazy<RwLock<ShaderMetaMap>> = Lazy::new(Default::default);

/// Handles the loading of `.spv.json` shader metadata,
/// and using it to conditionally re-specialize `RustGpuMaterial` instances on reload.
pub struct ShaderMetaPlugin<M> {
    _phantom: PhantomData<M>,
}

impl<M> Default for ShaderMetaPlugin<M> {
    fn default() -> Self {
        ShaderMetaPlugin {
            _phantom: default(),
        }
    }
}

impl<M> Plugin for ShaderMetaPlugin<M>
where
    M: RustGpuMaterial,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        if !app.is_plugin_added::<JsonAssetPlugin<ModuleMeta>>() {
            app.add_plugin(JsonAssetPlugin::<ModuleMeta>::new(&["spv.json"]));
        }

        app.add_system(
            module_meta_events::<M>
                .in_base_set(CoreSet::Last)
                .after(shader_events::<M>)
                .before(reload_materials::<M>),
        );
    }
}

/// Mapping between `Handle<Shader>` and `Handle<ModuleMeta>`
#[derive(Debug, Default, Clone, Resource)]
pub struct ShaderMetaMap {
    pub shader_to_meta: HashMap<Handle<Shader>, Handle<ModuleMeta>>,
    pub meta_to_shader: HashMap<Handle<ModuleMeta>, Handle<Shader>>,
}

impl ShaderMetaMap {
    pub fn add(&mut self, shader: Handle<Shader>, meta: Handle<ModuleMeta>) {
        self.shader_to_meta.insert(shader.clone(), meta.clone());
        self.meta_to_shader.insert(meta, shader);
    }

    pub fn meta(&self, shader: &Handle<Shader>) -> Option<&Handle<ModuleMeta>> {
        self.shader_to_meta.get(shader)
    }

    pub fn shader(&self, meta: &Handle<ModuleMeta>) -> Option<&Handle<Shader>> {
        self.meta_to_shader.get(meta)
    }

    pub fn remove_by_shader(&mut self, shader: Handle<Shader>) {
        let meta = self.shader_to_meta.remove(&shader).unwrap();
        self.meta_to_shader.remove(&meta).unwrap();
    }

    pub fn remove_by_meta(&mut self, shader: Handle<ModuleMeta>) {
        let shader = self.meta_to_shader.remove(&shader).unwrap();
        self.shader_to_meta.remove(&shader).unwrap();
    }
}

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub(crate) struct ShaderMeta {
    pub metas: HashMap<Handle<Shader>, ModuleMeta>,
}

/// JSON asset corresponding to a `.json.spv` file.
#[derive(Debug, Default, Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "64a45932-95c4-42c7-a212-0598949d798f"]
pub struct ModuleMeta {
    /// List of entry points compiled into the corresponding `.spv` file.
    pub entry_points: Vec<String>,
    /// Path to corresponding `.spv` file.
    pub module: String,
}

/// Listens for asset events, updates backend data, and triggers material re-specialization
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
