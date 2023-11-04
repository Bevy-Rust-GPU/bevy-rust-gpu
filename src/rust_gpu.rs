//! Wrapper for extending a `Material` with `rust-gpu` shader functionality.

use std::{any::TypeId, marker::PhantomData, path::PathBuf, sync::RwLock};

use bevy::{
    asset::Asset,
    pbr::MaterialPipelineKey,
    prelude::{
        default, info, warn, AssetEvent, Assets,  EventReader, Handle, Image,
         Material, MaterialPlugin, Plugin, ResMut, PreUpdate,
    },
    reflect::{TypeUuid, Reflect},
    render::render_resource::{
        AsBindGroup, PreparedBindGroup, ShaderRef, SpecializedMeshPipelineError,
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin},
    utils::HashMap,
};
use once_cell::sync::Lazy;
use rust_gpu_builder_shared::RustGpuBuilderOutput;

use crate::prelude::{EntryPoint, RustGpuMaterial};

static MATERIAL_SETTINGS: Lazy<RwLock<HashMap<TypeId, RustGpuSettings>>> = Lazy::new(default);

/// Configures backend [`Material`] support for [`RustGpu<M>`].
pub struct RustGpuMaterialPlugin<M>
where
    M: RustGpuMaterial,
{
    _phantom: PhantomData<M>,
}

impl<M> Default for RustGpuMaterialPlugin<M>
where
    M: Material + RustGpuMaterial,
{
    fn default() -> Self {
        RustGpuMaterialPlugin {
            _phantom: default(),
        }
    }
}

impl<M> Plugin for RustGpuMaterialPlugin<M>
where
    M: Material + RustGpuMaterial  + bevy::prelude::FromReflect,
    M::Data: Clone + Eq + std::hash::Hash,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(MaterialPlugin::<RustGpu<M>>::default());
        app.add_systems(PreUpdate, reload_materials::<M>);
    }
}

/// Configures backend [`Material2d`] support for [`RustGpu<M>`].
pub struct RustGpuMaterial2dPlugin<M>
where
    M: RustGpuMaterial,
{
    _phantom: PhantomData<M>,
}

impl<M> Default for RustGpuMaterial2dPlugin<M>
where
    M: Material2d + RustGpuMaterial  + bevy::prelude::FromReflect,
{
    fn default() -> Self {
        RustGpuMaterial2dPlugin {
            _phantom: default(),
        }
    }
}

impl<M> Plugin for RustGpuMaterial2dPlugin<M>
where
    M: Material2d + RustGpuMaterial  + bevy::prelude::FromReflect,
    M::Data: Clone + Eq + std::hash::Hash,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(Material2dPlugin::<RustGpu<M>>::default());
        app.add_systems(PreUpdate, reload_materials::<M>);
    }
}

/// Type-level RustGpu material settings
#[derive(Debug, Default, Copy, Clone)]
pub struct RustGpuSettings {
    /// If true, use M::vertex as a fallback instead of ShaderRef::default
    pub fallback_base_vertex: bool,
    /// If true, use M::fragment as a fallback instead of ShaderRef::default
    pub fallback_base_fragment: bool,
}

/// [`RustGpu`] pipeline key.
pub struct RustGpuKey<M>
where
    M: AsBindGroup,
{
    pub base: M::Data,
    pub vertex_shader: Option<Handle<RustGpuBuilderOutput>>,
    pub fragment_shader: Option<Handle<RustGpuBuilderOutput>>,
    pub iteration: usize,
}

impl<M> Clone for RustGpuKey<M>
where
    M: AsBindGroup,
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        RustGpuKey {
            base: self.base.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            iteration: self.iteration.clone(),
        }
    }
}

impl<M> PartialEq for RustGpuKey<M>
where
    M: AsBindGroup,
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base)
            && self.vertex_shader.eq(&other.vertex_shader)
            && self.fragment_shader.eq(&other.fragment_shader)
            && self.iteration.eq(&other.iteration)
    }
}

impl<M> Eq for RustGpuKey<M>
where
    M: AsBindGroup,
    M::Data: Eq,
{
}

impl<M> std::hash::Hash for RustGpuKey<M>
where
    M: AsBindGroup,
    M::Data: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.vertex_shader.hash(state);
        self.fragment_shader.hash(state);
        self.iteration.hash(state);
    }
}

/// Extends a `Material` with `rust-gpu` shader support.
#[derive(Debug, Default, Clone, TypeUuid, Reflect)]
#[uuid = "6d355e05-c567-4a29-a84a-362df79111de"]
pub struct RustGpu<M> {
    /// Base material.
    pub base: M,

    /// If `Some`, overrides [`Material::vertex_shader`] during specialization.
    pub vertex_shader: Option<Handle<RustGpuBuilderOutput>>,

    /// If `Some`, overrides [`Material::fragment_shader`] during specialization.
    pub fragment_shader: Option<Handle<RustGpuBuilderOutput>>,

    /// Current reload iteration, used to drive hot-reloading.
    pub iteration: usize,
}

impl<M> PartialEq for RustGpu<M>
where
    M: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.base.eq(&other.base)
            && self.vertex_shader.eq(&other.vertex_shader)
            && self.fragment_shader.eq(&other.fragment_shader)
            && self.iteration.eq(&other.iteration)
    }
}

impl<M> Eq for RustGpu<M> where M: Eq {}

impl<M> PartialOrd for RustGpu<M>
where
    M: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        [
            (self.base.partial_cmp(&other.base)),
            (self.vertex_shader.partial_cmp(&other.vertex_shader)),
            (self.fragment_shader.partial_cmp(&other.fragment_shader)),
            (self.iteration.partial_cmp(&other.iteration)),
        ]
        .into_iter()
        .fold(None, |acc, next| match (acc, next) {
            (None, None) => None,
            (None, Some(next)) => Some(next),
            (Some(acc), None) => Some(acc),
            (Some(acc), Some(next)) => Some(acc.then(next)),
        })
    }
}

impl<M> Ord for RustGpu<M>
where
    M: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        [
            (self.base.cmp(&other.base)),
            (self.vertex_shader.cmp(&other.vertex_shader)),
            (self.fragment_shader.cmp(&other.fragment_shader)),
            (self.iteration.cmp(&other.iteration)),
        ]
        .into_iter()
        .fold(std::cmp::Ordering::Equal, std::cmp::Ordering::then)
    }
}

impl<M> AsBindGroup for RustGpu<M>
where
    M: AsBindGroup,
{
    type Data = RustGpuKey<M>;

    fn as_bind_group(
        &self,
        layout: &bevy::render::render_resource::BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        images: &bevy::render::render_asset::RenderAssets<Image>,
        fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<
        bevy::render::render_resource::PreparedBindGroup<Self::Data>,
        bevy::render::render_resource::AsBindGroupError,
    > {
        self.base
            .as_bind_group(layout, render_device, images, fallback_image)
            .map(|base| PreparedBindGroup {
                bindings: base.bindings,
                bind_group: base.bind_group,
                data: RustGpuKey {
                    base: base.data,
                    vertex_shader: self.vertex_shader.clone(),
                    fragment_shader: self.fragment_shader.clone(),
                    iteration: self.iteration,
                },
            })
    }

    fn bind_group_layout(
        render_device: &bevy::render::renderer::RenderDevice,
    ) -> bevy::render::render_resource::BindGroupLayout {
        M::bind_group_layout(render_device)
    }
}

impl<M> RustGpu<M>
where
    M: AsBindGroup + RustGpuMaterial + Send + Sync + 'static,
{
    fn specialize_generic(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        key: RustGpuKey<M>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        info!("Specializing RustGpu material");
        let v = 'vertex: {
            let Some(vertex_shader) = key.vertex_shader else {
                break 'vertex None;
            };

            info!("Vertex shader is present, aggregating defs");

            let entry_point = M::Vertex::build(&descriptor.vertex.shader_defs);
            info!("Built vertex entrypoint {entry_point:}");

            #[cfg(feature = "hot-rebuild")]
            'hot_rebuild: {
                let exports = crate::prelude::MATERIAL_EXPORTS.read().unwrap();
                let Some(export) = exports.get(&std::any::TypeId::of::<Self>()) else {
                    break 'hot_rebuild;
                };

                let handles = crate::prelude::EXPORT_HANDLES.read().unwrap();
                let Some(handle) = handles.get(export) else {
                    break 'hot_rebuild;
                };

                info!("Entrypoint sender is valid");
                handle
                    .send(crate::prelude::Export {
                        shader: M::Vertex::NAME,
                        permutation: M::Vertex::permutation(&descriptor.vertex.shader_defs),
                        constants: M::Vertex::filter_constants(&descriptor.vertex.shader_defs),
                        types: M::Vertex::types()
                            .into_iter()
                            .map(|(key, value)| (key.to_string(), value.to_string()))
                            .collect(),
                    })
                    .unwrap();
            };

            info!("Vertex meta is present");
            let artifacts = crate::prelude::RUST_GPU_ARTIFACTS.read().unwrap();
            let Some(artifact) = artifacts.get(&vertex_shader) else {
                warn!("Missing vertex artifact.");
                break 'vertex None;
            };

            info!("Checking entry point {entry_point:}");
            if !artifact.entry_points.contains(&entry_point) {
                warn!("Missing vertex entry point {entry_point:}.");
                break 'vertex None;
            }

            let vertex_shader = match &artifact.modules {
                crate::prelude::RustGpuModules::Single(single) => single.clone(),
                crate::prelude::RustGpuModules::Multi(multi) => {
                    let Some(shader) = multi.get(&entry_point) else {
                        break 'vertex None;
                    };

                    shader.clone()
                }
            };

            Some((vertex_shader, entry_point))
        };

        let f = 'fragment: {
            let (Some(fragment_descriptor), Some(fragment_shader)) = (descriptor.fragment.as_mut(), key.fragment_shader) else { break 'fragment None };

            info!("Fragment shader is present, aggregating defs");

            let entry_point = M::Fragment::build(&fragment_descriptor.shader_defs);
            info!("Built fragment entrypoint {entry_point:}");

            #[cfg(feature = "hot-rebuild")]
            'hot_rebuild: {
                let exports = crate::prelude::MATERIAL_EXPORTS.read().unwrap();
                let Some(export) = exports.get(&std::any::TypeId::of::<Self>()) else {
                        break 'hot_rebuild;
                    };

                let handles = crate::prelude::EXPORT_HANDLES.read().unwrap();
                let Some(handle) = handles.get(export) else {
                        break 'hot_rebuild;
                    };

                info!("Entrypoint sender is valid");
                handle
                    .send(crate::prelude::Export {
                        shader: M::Fragment::NAME,
                        permutation: M::Fragment::permutation(&fragment_descriptor.shader_defs),
                        constants: M::Fragment::filter_constants(&fragment_descriptor.shader_defs),
                        types: M::Fragment::types()
                            .into_iter()
                            .map(|(key, value)| (key.to_string(), value.to_string()))
                            .collect(),
                    })
                    .unwrap();
            }

            info!("Fragment meta is present");
            let artifacts = crate::prelude::RUST_GPU_ARTIFACTS.read().unwrap();
            let Some(artifact) = artifacts.get(&fragment_shader) else {
                        warn!("Missing fragment artifact.");
                        break 'fragment None;
                    };

            info!("Checking entry point {entry_point:}");
            if !artifact.entry_points.contains(&entry_point) {
                warn!("Missing fragment entry point {entry_point:}.");
                break 'fragment None;
            }

            let fragment_shader = match &artifact.modules {
                crate::prelude::RustGpuModules::Single(single) => single.clone(),
                crate::prelude::RustGpuModules::Multi(multi) => {
                    let Some(shader) = multi.get(&entry_point) else {
                            warn!("Missing handle for entry point {entry_point:}.");
                        break 'fragment None;
                    };

                    shader.clone()
                }
            };

            Some((fragment_shader, entry_point))
        };

        match (v, descriptor.fragment.as_mut(), f) {
            (Some((vertex_shader, vertex_entry_point)), None, _) => {
                info!("Applying vertex shader and entry point");
                descriptor.vertex.shader = vertex_shader;
                descriptor.vertex.entry_point = vertex_entry_point.into();

                // Clear shader defs to satify ShaderProcessor
                descriptor.vertex.shader_defs.clear();
            }
            (
                Some((vertex_shader, vertex_entry_point)),
                Some(fragment_descriptor),
                Some((fragment_shader, fragment_entry_point)),
            ) => {
                info!("Applying vertex shader and entry point");
                descriptor.vertex.shader = vertex_shader;
                descriptor.vertex.entry_point = vertex_entry_point.into();

                // Clear shader defs to satify ShaderProcessor
                descriptor.vertex.shader_defs.clear();

                info!("Applying fragment shader and entry point");
                fragment_descriptor.shader = fragment_shader;
                fragment_descriptor.entry_point = fragment_entry_point.into();

                // Clear shader defs to satify ShaderProcessor
                fragment_descriptor.shader_defs.clear();
            }
            _ => warn!("Falling back to default shaders."),
        }

        if let Some(label) = &mut descriptor.label {
            *label = format!("rust_gpu_{}", *label).into();
        }

        Ok(())
    }
}

impl<M> Material for RustGpu<M>
where
    M: Material + RustGpuMaterial + bevy::prelude::FromReflect,
    M::Data: Clone,
{
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        if let Some(true) = MATERIAL_SETTINGS
            .read()
            .unwrap()
            .get(&TypeId::of::<Self>())
            .map(|settings| settings.fallback_base_vertex)
        {
            M::vertex_shader()
        } else {
            ShaderRef::Default
        }
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        if let Some(true) = MATERIAL_SETTINGS
            .read()
            .unwrap()
            .get(&TypeId::of::<Self>())
            .map(|settings| settings.fallback_base_vertex)
        {
            M::fragment_shader()
        } else {
            ShaderRef::Default
        }
    }

    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        self.base.alpha_mode()
    }

    fn depth_bias(&self) -> f32 {
        self.base.depth_bias()
    }

    fn specialize(
        pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        M::specialize(
            // SAFETY: Transmuted element is PhantomData
            //
            //         Technically relying on a violable implementation detail,
            //         but it seems unlikely to change in such a way that would
            //         introduce UB without a compiler error.
            unsafe { std::mem::transmute(pipeline) },
            descriptor,
            layout,
            MaterialPipelineKey {
                mesh_key: key.mesh_key,
                bind_group_data: key.bind_group_data.base.clone(),
            },
        )?;

        RustGpu::<M>::specialize_generic(descriptor, key.bind_group_data)?;

        Ok(())
    }
}

impl<M> Material2d for RustGpu<M>
where
    M: Material2d + RustGpuMaterial  + bevy::prelude::FromReflect,
    M::Data: Clone,
{
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        if let Some(true) = MATERIAL_SETTINGS
            .read()
            .unwrap()
            .get(&TypeId::of::<Self>())
            .map(|settings| settings.fallback_base_vertex)
        {
            M::vertex_shader()
        } else {
            ShaderRef::Default
        }
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        if let Some(true) = MATERIAL_SETTINGS
            .read()
            .unwrap()
            .get(&TypeId::of::<Self>())
            .map(|settings| settings.fallback_base_vertex)
        {
            M::fragment_shader()
        } else {
            ShaderRef::Default
        }
    }

    fn specialize(
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        M::specialize(
            descriptor,
            layout,
            Material2dKey {
                mesh_key: key.mesh_key,
                bind_group_data: key.bind_group_data.base.clone(),
            },
        )?;

        RustGpu::<M>::specialize_generic(descriptor, key.bind_group_data)?;

        Ok(())
    }
}

impl<M> RustGpu<M>
where
    M: 'static,
{
    pub fn map_settings<F: FnOnce(&mut RustGpuSettings)>(f: F) {
        let mut settings = MATERIAL_SETTINGS.write().unwrap();
        f(&mut settings.entry(TypeId::of::<Self>()).or_default());
    }

    #[cfg(feature = "hot-rebuild")]
    pub fn export_to<P: Into<PathBuf>>(path: P) {
        let mut handles = crate::prelude::MATERIAL_EXPORTS.write().unwrap();
        handles.insert(std::any::TypeId::of::<Self>(), path.into());
    }
}

/// [`RustGpuBuilderOutput`] asset event handler.
///
/// Handles loading shader assets, maintaining static material data, and respecializing materials on reload.
pub fn reload_materials<M>(
    mut builder_output_events: EventReader<AssetEvent<RustGpuBuilderOutput>>,
    mut materials: ResMut<Assets<RustGpu<M>>>,
) where
    M: Asset + RustGpuMaterial  + bevy::prelude::FromReflect,
{
    for event in builder_output_events.iter() {
        if let AssetEvent::Created { handle } | AssetEvent::Modified { handle } = event {
            // Mark any materials referencing this asset for respecialization
            for (_, material) in materials.iter_mut() {
                let mut reload = false;

                if let Some(vertex_shader) = &material.vertex_shader {
                    reload |= vertex_shader == handle;
                }

                if let Some(fragment_shader) = &material.fragment_shader {
                    reload |= fragment_shader == handle;
                }

                if reload {
                    material.iteration += 1;
                }
            }
        }
    }
}
