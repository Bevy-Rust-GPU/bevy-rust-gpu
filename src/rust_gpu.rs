//! Wrapper for extending a `Material` with `rust-gpu` shader functionality.

use std::{any::TypeId, marker::PhantomData, path::PathBuf, sync::RwLock};

use bevy::{
    pbr::MaterialPipelineKey,
    prelude::{
        default, info, warn, CoreSet, Deref, DerefMut, Handle, Image, IntoSystemConfig, Material,
        MaterialPlugin, Plugin, Resource, Shader,
    },
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, PreparedBindGroup, ShaderRef},
    utils::{HashMap, HashSet},
};
use once_cell::sync::Lazy;

use crate::{
    load_rust_gpu_shader::RustGpuShader,
    prelude::{EntryPoint, RustGpuMaterial},
    systems::{reload_materials, shader_events},
};

static MATERIAL_SETTINGS: Lazy<RwLock<HashMap<TypeId, RustGpuSettings>>> = Lazy::new(default);

/// Configures backend support for [`RustGpu<M>`].
pub struct RustGpuMaterialPlugin<M>
where
    M: RustGpuMaterial,
{
    _phantom: PhantomData<M>,
}

impl<M> Default for RustGpuMaterialPlugin<M>
where
    M: RustGpuMaterial,
{
    fn default() -> Self {
        RustGpuMaterialPlugin {
            _phantom: default(),
        }
    }
}

impl<M> Plugin for RustGpuMaterialPlugin<M>
where
    M: RustGpuMaterial,
    M::Data: Clone + Eq + std::hash::Hash,
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(reload_materials::<M>.in_base_set(CoreSet::Last));
        app.add_system(shader_events::<M>.before(reload_materials::<M>));

        app.add_plugin(MaterialPlugin::<RustGpu<M>>::default());

        #[cfg(feature = "hot-reload")]
        app.add_plugin(crate::prelude::ShaderMetaPlugin::<M>::default());
    }
}

/// A resource to track `rust-gpu` shaders that have been reloaded on a given frame
#[derive(Debug, Default, Clone, Deref, DerefMut, Resource)]
pub struct ChangedShaders(pub HashSet<Handle<Shader>>);

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
    pub vertex_shader: Option<RustGpuShader>,
    pub fragment_shader: Option<RustGpuShader>,
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
#[derive(Debug, Default, Clone, TypeUuid)]
#[uuid = "6d355e05-c567-4a29-a84a-362df79111de"]
pub struct RustGpu<M> {
    /// Base material.
    pub base: M,

    /// If `Some`, overrides [`Material::vertex_shader`] during specialization.
    pub vertex_shader: Option<RustGpuShader>,

    /// If `Some`, overrides [`Material::fragment_shader`] during specialization.
    pub fragment_shader: Option<RustGpuShader>,

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

impl<M> Material for RustGpu<M>
where
    M: RustGpuMaterial,
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
                bind_group_data: key.bind_group_data.base,
            },
        )?;

        info!("Specializing RustGpu material");
        if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
            info!("Vertex shader is present, aggregating defs");

            let entry_point = M::Vertex::build(&descriptor.vertex.shader_defs);
            info!("Built vertex entrypoint {entry_point:}");

            #[allow(unused_mut)]
            let mut apply = true;

            #[cfg(feature = "hot-reload")]
            {
                let metas = crate::prelude::SHADER_META.read().unwrap();
                if let Some(vertex_meta) = metas.get(&vertex_shader.0) {
                    info!("Vertex meta is valid");
                    info!("Checking entry point {entry_point:}");
                    if !vertex_meta.entry_points.contains(&entry_point) {
                        warn!("Missing entry point {entry_point:}");
                        apply = false;
                    }
                } else {
                    apply = false;
                }
            }

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
                    })
                    .unwrap();
            };

            if apply {
                info!("Applying vertex shader and entry point");
                descriptor.vertex.shader = vertex_shader.0;
                descriptor.vertex.entry_point = entry_point.into();

                // Clear shader defs to satify ShaderProcessor
                descriptor.vertex.shader_defs.clear();
            } else {
                warn!("Falling back to default vertex shader.");
            }
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                info!("Fragment shader is present, aggregating defs");

                let entry_point = M::Fragment::build(&fragment_descriptor.shader_defs);
                info!("Built fragment entrypoint {entry_point:}");

                #[allow(unused_mut)]
                let mut apply = true;

                #[cfg(feature = "hot-reload")]
                {
                    info!("Fragment meta is present");
                    let metas = crate::prelude::SHADER_META.read().unwrap();
                    if let Some(fragment_meta) = metas.get(&fragment_shader.0) {
                        info!("Fragment meta is valid");
                        info!("Checking entry point {entry_point:}");
                        if !fragment_meta.entry_points.contains(&entry_point) {
                            apply = false;
                            warn!("Missing entry point {entry_point:}, falling back to default fragment shader.");
                        }
                    } else {
                        apply = false;
                    }
                }

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
                        })
                        .unwrap();
                };

                if apply {
                    info!("Applying fragment shader and entry point");
                    fragment_descriptor.shader = fragment_shader.0;
                    fragment_descriptor.entry_point = entry_point.into();

                    // Clear shader defs to satify ShaderProcessor
                    fragment_descriptor.shader_defs.clear();
                } else {
                    warn!("Falling back to default fragment shader.");
                }
            }
        }

        if let Some(label) = &mut descriptor.label {
            *label = format!("rust_gpu_{}", *label).into();
        }

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
