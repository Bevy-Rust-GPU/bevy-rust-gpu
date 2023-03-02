//! Wrapper for extending a `Material` with `rust-gpu` shader functionality.

use std::marker::PhantomData;

use bevy::{
    pbr::MaterialPipelineKey,
    prelude::{
        default, info, warn, CoreStage, Handle, Image, IntoSystemDescriptor, Material,
        MaterialPlugin, Plugin, Shader,
    },
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, PreparedBindGroup},
};

use crate::{
    prelude::{EntryPoint, Export, ExportHandle, RustGpuMaterial, SHADER_META},
    systems::{reload_materials, shader_events},
};

const SHADER_DEFS: &[&'static str] = &[
    "NO_STORAGE_BUFFERS_SUPPORT",
    #[cfg(feature = "webgl")]
    "NO_TEXTURE_ARRAYS_SUPPORT",
    #[cfg(feature = "webgl")]
    "SIXTEEN_BYTE_ALIGNMENT",
];

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
        app.add_system_to_stage(CoreStage::Last, reload_materials::<M>);
        app.add_system_to_stage(
            CoreStage::Last,
            shader_events::<M>.before(reload_materials::<M>),
        );

        app.add_plugin(MaterialPlugin::<RustGpu<M>>::default());

        #[cfg(feature = "shader-meta")]
        app.add_plugin(crate::prelude::ShaderMetaPlugin::<M>::default());
    }
}

/// [`RustGpu`] pipeline key.
pub struct RustGpuKey<M>
where
    M: AsBindGroup,
{
    pub base: M::Data,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub iteration: usize,
    #[cfg(feature = "entry-point-export")]
    pub sender: Option<ExportHandle>,
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
            sender: self.sender.clone(),
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
    pub vertex_shader: Option<Handle<Shader>>,

    /// If `Some`, overrides [`Material::fragment_shader`] during specialization.
    pub fragment_shader: Option<Handle<Shader>>,

    /// Current reload iteration, used to drive hot-reloading.
    pub iteration: usize,

    /// If `Some`, active entry points will be reported to this handle.
    #[cfg(feature = "entry-point-export")]
    pub sender: Option<ExportHandle>,
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
        bevy::render::render_resource::PreparedBindGroup<Self>,
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
                    #[cfg(feature = "entry-point-export")]
                    sender: self.sender.clone(),
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
        M::vertex_shader()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        M::fragment_shader()
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

            let shader_defs: Vec<_> = descriptor
                .vertex
                .shader_defs
                .iter()
                .cloned()
                .chain(SHADER_DEFS.iter().map(ToString::to_string))
                .collect();

            info!("Building vertex entrypoint");
            let entry_point = M::Vertex::build(&shader_defs);

            #[allow(unused_mut)]
            let mut apply = true;

            #[cfg(feature = "shader-meta")]
            {
                let metas = SHADER_META.read().unwrap();
                if let Some(vertex_meta) = metas.get(&vertex_shader) {
                    info!("Vertex meta is valid");
                    if !vertex_meta.entry_points.contains(&entry_point) {
                        warn!("Missing entry point {entry_point:}");
                        apply = false;
                    }
                }
            }

            #[cfg(feature = "entry-point-export")]
            if let Some(sender) = &key.bind_group_data.sender {
                info!("Entrypoint sender is valid");
                sender
                    .send(Export {
                        shader: M::Vertex::NAME,
                        permutation: M::Vertex::permutation(&shader_defs),
                    })
                    .unwrap();
            }

            if apply {
                info!("Applying vertex shader and entry point");
                descriptor.vertex.shader = vertex_shader;
                descriptor.vertex.entry_point = entry_point.into();
            } else {
                warn!("Falling back to default vertex shader.");
            }
        }

        if let Some(fragment_descriptor) = descriptor.fragment.as_mut() {
            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                info!("Fragment shader is present, aggregating defs");

                let shader_defs: Vec<_> = fragment_descriptor
                    .shader_defs
                    .iter()
                    .cloned()
                    .chain(SHADER_DEFS.iter().map(ToString::to_string))
                    .collect();

                info!("Building fragment entrypoint");
                let entry_point = M::Fragment::build(&shader_defs);

                #[allow(unused_mut)]
                let mut apply = true;

                #[cfg(feature = "shader-meta")]
                {
                    info!("Fragment meta is present");
                    let metas = SHADER_META.read().unwrap();
                    if let Some(fragment_meta) = metas.get(&fragment_shader) {
                        info!("Fragment meta is valid");
                        if !fragment_meta.entry_points.contains(&entry_point) {
                            apply = false;
                            warn!("Missing entry point {entry_point:}, falling back to default fragment shader.");
                        }
                    }
                }

                #[cfg(feature = "entry-point-export")]
                if let Some(sender) = &key.bind_group_data.sender {
                    sender
                        .send(Export {
                            shader: M::Fragment::NAME,
                            permutation: M::Fragment::permutation(&shader_defs),
                        })
                        .unwrap();
                }

                if apply {
                    info!("Applying fragment shader and entry point");
                    fragment_descriptor.shader = fragment_shader;
                    fragment_descriptor.entry_point = entry_point.into();
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
