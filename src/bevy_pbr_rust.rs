//! `bevy-pbr-rust`-backed `RustGpuMaterial` implementation for `StandardMaterial`.

use bevy::prelude::StandardMaterial;

use crate::prelude::{EntryPoint, EntryPointName, EntryPointParameters, RustGpuMaterial};

/// `bevy_rust_gpu::mesh::entry_points::vertex`
pub enum MeshVertex {}

impl EntryPoint for MeshVertex {
    const NAME: EntryPointName = "mesh::entry_points::vertex";
    const PARAMETERS: EntryPointParameters = &[
        (&[("VERTEX_TANGENTS", "some")], "none"),
        (&[("VERTEX_COLORS", "some")], "none"),
        (&[("SKINNED", "some")], "none"),
    ];
}

/// `bevy_rust_gpu::mesh::entry_points::fragment`
pub enum MeshFragment {}

impl EntryPoint for MeshFragment {
    const NAME: EntryPointName = "mesh::entry_points::fragment";
    const PARAMETERS: EntryPointParameters = &[];
}

/// `bevy_rust_gpu::pbr::entry_points::fragment`
pub enum PbrFragment {}

impl EntryPoint for PbrFragment {
    const NAME: EntryPointName = "pbr::entry_points::fragment";
    const PARAMETERS: EntryPointParameters = &[
        (&[("NO_TEXTURE_ARRAYS_SUPPORT", "texture")], "array"),
        (&[("NO_STORAGE_BUFFERS_SUPPORT", "uniform")], "storage"),
        (&[("VERTEX_POSITIONS", "some")], "none"),
        (&[("VERTEX_NORMALS", "some")], "none"),
        (&[("VERTEX_UVS", "some")], "none"),
        (&[("VERTEX_TANGENTS", "some")], "none"),
        (&[("VERTEX_COLORS", "some")], "none"),
        (&[("STANDARDMATERIAL_NORMAL_MAP", "some")], "none"),
        (&[("SKINNED", "some")], "none"),
        (&[("TONEMAP_IN_SHADER", "some")], "none"),
        (&[("DEBAND_DITHER", "some")], "none"),
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
    ];
}

/// `StandardMaterial` implementation
impl RustGpuMaterial for StandardMaterial {
    type Vertex = MeshVertex;
    type Fragment = PbrFragment;
}

