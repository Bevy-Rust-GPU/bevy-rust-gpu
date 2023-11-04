
use ::bevy::utils::HashMap;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

/// Module binary data container.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum RustGpuBuilderModules {
    /// Contains a single unnamed module.
    Single(Vec<u8>),
    /// Contains multiple named modules.
    Multi(HashMap<String, Vec<u8>>),
}

/// Compile output from `rust-gpu-builder`,
/// includes SPIR-V binary modules and entry point metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RustGpuBuilderOutput {
    pub entry_points: Vec<String>,
    pub modules: RustGpuBuilderModules,
}

#[cfg(feature = "bevy")]
mod bevy {
    use super::RustGpuBuilderOutput;
    use bevy::{reflect::TypeUuid, utils::Uuid};

    /// Implementing TypeUuid allows use as a `bevy` asset
    impl TypeUuid for RustGpuBuilderOutput {
        const TYPE_UUID: Uuid =
            Uuid::from_fields(1664188495, 11437, 8530, &[15, 75, 77, 14, 32, 11, 25, 52]);
    }
}
