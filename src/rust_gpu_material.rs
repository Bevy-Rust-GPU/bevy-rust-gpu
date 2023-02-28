use std::marker::PhantomData;

use bevy::{
    pbr::StandardMaterialUniform,
    prelude::{default, info, warn, Handle, Image, Material, Shader, StandardMaterial},
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, AsBindGroupShaderType, Face, ShaderType},
    utils::Uuid,
};

use crate::prelude::EntryPoint;

#[cfg(feature = "entry-point-export")]
use crate::prelude::{EntryPointSender, Export};

#[cfg(feature = "shader-meta")]
use crate::prelude::SHADER_META;

#[derive(ShaderType)]
pub struct BaseMaterial {
    base: StandardMaterialUniform,
}

#[derive(Debug, Default, Clone)]
pub struct ShaderMaterialKey {
    vertex_shader: Option<Handle<Shader>>,
    vertex_defs: Vec<String>,
    fragment_shader: Option<Handle<Shader>>,
    fragment_defs: Vec<String>,
    normal_map: bool,
    cull_mode: Option<Face>,
    #[cfg(feature = "entry-point-export")]
    sender: Option<EntryPointSender>,
    iteration: usize,
}

impl PartialEq for ShaderMaterialKey {
    fn eq(&self, other: &Self) -> bool {
        self.vertex_shader.eq(&other.vertex_shader)
            && self.vertex_defs.eq(&other.vertex_defs)
            && self.fragment_shader.eq(&other.fragment_shader)
            && self.fragment_defs.eq(&other.fragment_defs)
            && self.normal_map.eq(&other.normal_map)
            && self.cull_mode.eq(&other.cull_mode)
            && self.iteration.eq(&other.iteration)
    }
}

impl std::hash::Hash for ShaderMaterialKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vertex_shader.hash(state);
        self.vertex_defs.hash(state);
        self.fragment_shader.hash(state);
        self.fragment_defs.hash(state);
        self.normal_map.hash(state);
        self.cull_mode.hash(state);
        self.iteration.hash(state);
    }
}

impl Eq for ShaderMaterialKey {}

impl<V, F> From<&RustGpuMaterial<V, F>> for ShaderMaterialKey {
    fn from(value: &RustGpuMaterial<V, F>) -> Self {
        ShaderMaterialKey {
            vertex_shader: value.vertex_shader.clone(),
            vertex_defs: value.vertex_defs.clone(),
            fragment_shader: value.fragment_shader.clone(),
            fragment_defs: value.fragment_defs.clone(),
            normal_map: value.normal_map_texture.is_some(),
            cull_mode: value.base.cull_mode,
            #[cfg(feature = "entry-point-export")]
            sender: value.export.clone(),
            iteration: value.iteration.clone(),
        }
    }
}

#[derive(Debug, AsBindGroup)]
#[bind_group_data(ShaderMaterialKey)]
#[uniform(0, BaseMaterial)]
pub struct RustGpuMaterial<V, F> {
    pub base: StandardMaterial,

    pub vertex_shader: Option<Handle<Shader>>,
    pub vertex_defs: Vec<String>,
    pub fragment_shader: Option<Handle<Shader>>,
    pub fragment_defs: Vec<String>,

    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub emissive_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_roughness_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,

    #[texture(9)]
    #[sampler(10)]
    pub normal_map_texture: Option<Handle<Image>>,

    #[cfg(feature = "entry-point-export")]
    pub export: Option<EntryPointSender>,
    pub iteration: usize,
    pub _phantom: PhantomData<(V, F)>,
}

impl<V, F> Default for RustGpuMaterial<V, F> {
    fn default() -> Self {
        RustGpuMaterial {
            base: default(),
            vertex_shader: default(),
            vertex_defs: default(),
            fragment_shader: default(),
            fragment_defs: default(),
            base_color_texture: default(),
            emissive_texture: default(),
            metallic_roughness_texture: default(),
            occlusion_texture: default(),
            normal_map_texture: default(),
            #[cfg(feature = "entry-point-export")]
            export: default(),
            iteration: default(),
            _phantom: default(),
        }
    }
}

impl<V, F> Clone for RustGpuMaterial<V, F> {
    fn clone(&self) -> Self {
        RustGpuMaterial {
            base: self.base.clone(),
            vertex_shader: self.vertex_shader.clone(),
            vertex_defs: self.vertex_defs.clone(),
            fragment_shader: self.fragment_shader.clone(),
            fragment_defs: self.fragment_defs.clone(),
            base_color_texture: self.base_color_texture.clone(),
            emissive_texture: self.emissive_texture.clone(),
            metallic_roughness_texture: self.metallic_roughness_texture.clone(),
            occlusion_texture: self.occlusion_texture.clone(),
            normal_map_texture: self.occlusion_texture.clone(),
            #[cfg(feature = "entry-point-export")]
            export: self.export.clone(),
            iteration: self.iteration.clone(),
            _phantom: default(),
        }
    }
}

impl<V, F> TypeUuid for RustGpuMaterial<V, F> {
    const TYPE_UUID: bevy::utils::Uuid = Uuid::from_fields(
        0x3bb0b1c8,
        0x5ff8,
        0x4085,
        &[0xa4, 0x48, 0x19, 0xda, 0xa3, 0x36, 0xc1, 0x0c],
    );
}

impl<V, F> AsBindGroupShaderType<BaseMaterial> for RustGpuMaterial<V, F> {
    fn as_bind_group_shader_type(
        &self,
        images: &bevy::render::render_asset::RenderAssets<bevy::prelude::Image>,
    ) -> BaseMaterial {
        BaseMaterial {
            base: self.base.as_bind_group_shader_type(images),
        }
    }
}

impl<V, F> Material for RustGpuMaterial<V, F>
where
    V: EntryPoint,
    F: EntryPoint,
    RustGpuMaterial<V, F>: AsBindGroup<Data = ShaderMaterialKey>,
{
    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        self.base.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.base.depth_bias
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        info!("Specializing RustGpuMaterial");
        if let Some(vertex_shader) = key.bind_group_data.vertex_shader {
            info!("Vertex shader is present, aggregating defs");

            let shader_defs: Vec<_> = descriptor
                .vertex
                .shader_defs
                .iter()
                .cloned()
                .chain(key.bind_group_data.vertex_defs.iter().cloned())
                .collect();

            info!("Building vertex entrypoint");
            let entry_point = V::build(&shader_defs);

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
                        shader: V::NAME,
                        permutation: V::permutation(&shader_defs),
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
            if key.bind_group_data.normal_map {
                fragment_descriptor
                    .shader_defs
                    .push(String::from("STANDARDMATERIAL_NORMAL_MAP"));
            }

            if let Some(fragment_shader) = key.bind_group_data.fragment_shader {
                info!("Fragment shader is present, aggregating defs");

                let shader_defs: Vec<_> = fragment_descriptor
                    .shader_defs
                    .iter()
                    .cloned()
                    .chain(key.bind_group_data.fragment_defs.iter().cloned())
                    .collect();

                info!("Building fragment entrypoint");
                let entry_point = F::build(&shader_defs);

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
                            shader: F::NAME,
                            permutation: F::permutation(&shader_defs),
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

        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        if let Some(label) = &mut descriptor.label {
            *label = format!("shader_{}", *label).into();
        }

        Ok(())
    }
}
