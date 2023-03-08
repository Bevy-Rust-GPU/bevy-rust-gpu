//! `bevy-pbr-rust`-backed `RustGpuMaterial` implementation for `StandardMaterial`.

use bevy::{prelude::StandardMaterial, render::render_resource::ShaderDefVal};

use crate::{
    prelude::{EntryPoint, EntryPointName, EntryPointParameters, RustGpuMaterial},
    EntryPointConstants,
};

/// `bevy_rust_gpu::mesh::entry_points::vertex`
pub enum MeshVertex {}

impl EntryPoint for MeshVertex {
    const NAME: EntryPointName = "mesh::entry_points::vertex";
    const PARAMETERS: EntryPointParameters = &[
        (&[("VERTEX_TANGENTS", "some")], "none"),
        (&[("VERTEX_COLORS", "some")], "none"),
        (&[("SKINNED", "some")], "none"),
    ];
    const CONSTANTS: EntryPointConstants = &[];
}

/// `bevy_rust_gpu::mesh::entry_points::fragment`
pub enum MeshFragment {}

impl EntryPoint for MeshFragment {
    const NAME: EntryPointName = "mesh::entry_points::fragment";
    const PARAMETERS: EntryPointParameters = &[];
    const CONSTANTS: EntryPointConstants = &[];
}

/// `bevy_rust_gpu::pbr::entry_points::fragment`
pub enum PbrFragment {}

impl EntryPoint for PbrFragment {
    const NAME: EntryPointName = "pbr::entry_points::fragment";
    const PARAMETERS: EntryPointParameters = &[
        (&[("NO_TEXTURE_ARRAYS_SUPPORT", "texture")], "array"),
        (&[("VERTEX_UVS", "some")], "none"),
        (&[("VERTEX_TANGENTS", "some")], "none"),
        (&[("VERTEX_COLORS", "some")], "none"),
        (&[("STANDARDMATERIAL_NORMAL_MAP", "some")], "none"),
        (&[("SKINNED", "some")], "none"),
        (&[("TONEMAP_IN_SHADER", "some")], "none"),
        (&[("DEBAND_DITHER", "some")], "none"),
        (
            &[
                ("BLEND_MULTIPLY", "multiply"),
                ("BLEND_PREMULTIPLIED_ALPHA", "blend_premultiplied_alpha"),
            ],
            "none",
        ),
        (&[("ENVIRONMENT_MAP", "some")], "none"),
        (&[("PREMULTIPLY_ALPHA", "some")], "none"),
        (
            &[
                ("CLUSTERED_FORWARD_DEBUG_Z_SLICES", "debug_z_slices"),
                (
                    "CLUSTERED_FORWARD_DEBUG_CLUSTER_LIGHT_COMPLEXITY",
                    "debug_cluster_light_complexity",
                ),
                (
                    "CLUSTERED_FORWARD_DEBUG_CLUSTER_COHERENCY",
                    "debug_cluster_coherency",
                ),
            ],
            "none",
        ),
        (
            &[("DIRECTIONAL_LIGHT_SHADOW_MAP_DEBUG_CASCADES", "some")],
            "none",
        ),
    ];
    const CONSTANTS: EntryPointConstants = &["MAX_DIRECTIONAL_LIGHTS", "MAX_CASCADES_PER_LIGHT"];

    fn permutation(shader_defs: &Vec<ShaderDefVal>) -> Vec<String> {
        let mut permutation = vec![];

        for (defined, undefined) in Self::PARAMETERS.iter() {
            if let Some(mapping) = defined.iter().find_map(|(def, mapping)| {
                if shader_defs.contains(&ShaderDefVal::Bool(def.to_string(), true)) {
                    Some(mapping)
                } else {
                    None
                }
            }) {
                permutation.push(mapping.to_string());
            } else {
                permutation.push(undefined.to_string())
            };
        }

        if let Some(ge) = shader_defs.iter().find_map(|def| match def {
            bevy::render::render_resource::ShaderDefVal::UInt(key, value) => {
                if key.as_str() == "AVAILABLE_STORAGE_BUFFER_BINDINGS" {
                    Some(*value >= 3)
                } else {
                    None
                }
            }
            _ => None,
        }) {
            if ge {
                permutation.insert(1, "storage".to_string())
            } else {
                permutation.insert(1, "uniform".to_string())
            }
        }

        permutation
    }
}

/// `StandardMaterial` implementation
impl RustGpuMaterial for StandardMaterial {
    type Vertex = MeshVertex;
    type Fragment = PbrFragment;
}
