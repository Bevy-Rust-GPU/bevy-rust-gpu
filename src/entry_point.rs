//! Trait representation of a `rust-gpu` entry point.

use std::collections::BTreeMap;

use bevy::render::render_resource::ShaderDefVal;

/// An entry point name for use with the [`EntryPoint`] trait.
pub type EntryPointName = &'static str;

/// A set of entry point compile parameters for use with the [`EntryPoint`] trait.
pub type EntryPointParameters =
    &'static [(&'static [(&'static str, &'static str)], &'static str)];

/// A set of entry point constants for use with the [`EntryPoint`] trait.
pub type EntryPointConstants = &'static [&'static str];

/// A set of entry point constants for use with the [`EntryPoint`] trait.
pub type EntryPointTypes = Vec<(String, String)>;

/// A `rust-gpu` entry point for use with [`RustGpuMaterial`](crate::rust_gpu_material::RustGpuMaterial).
pub trait EntryPoint: 'static + Send + Sync {
    /// The entry point's base function name, including module path
    ///
    /// ```
    /// # use bevy_rust_gpu::prelude::EntryPointName;
    /// const NAME: EntryPointName = "mesh::entry_points::vertex";
    /// ```
    const NAME: &'static str;

    fn parameters() -> EntryPointParameters {
        &[]
    }

    fn constants() -> EntryPointConstants {
        &[]
    }

    fn types() -> EntryPointTypes {
        vec![]
    }

    /// Constructs a permutation set from the provided shader defs
    fn permutation(shader_defs: &Vec<ShaderDefVal>) -> Vec<String> {
        let mut permutation = vec![];

        for (defined, undefined) in Self::parameters().iter() {
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

        permutation
    }

    fn filter_constants(shader_defs: &Vec<ShaderDefVal>) -> Vec<ShaderDefVal> {
        shader_defs
            .iter()
            .filter(|def| match def {
                ShaderDefVal::Bool(key, _)
                | ShaderDefVal::Int(key, _)
                | ShaderDefVal::UInt(key, _) => Self::constants().contains(&key.as_str()),
            })
            .cloned()
            .collect()
    }

    /// Build an entry point name from the provided shader defs
    fn build(shader_defs: &Vec<ShaderDefVal>) -> String {
        let constants = Self::filter_constants(shader_defs)
            .into_iter()
            .map(|def| {
                (
                    match &def {
                        ShaderDefVal::Bool(key, _)
                        | ShaderDefVal::Int(key, _)
                        | ShaderDefVal::UInt(key, _) => key.clone(),
                    },
                    match &def {
                        ShaderDefVal::Bool(value, _) => value.to_string(),
                        ShaderDefVal::Int(_, value) => value.to_string(),
                        ShaderDefVal::UInt(_, value) => value.to_string(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        std::iter::once(Self::NAME.to_string())
            .chain(
                Self::permutation(shader_defs)
                    .into_iter()
                    .map(|variant| "__".to_string() + &variant),
            )
            .chain(
                constants
                    .into_iter()
                    .map(|(key, value)| key + "_" + &value)
                    .map(|variant| "__".to_string() + &variant),
            )
            .chain(
                Self::types()
                    .into_iter()
                    .map(|(key, value)| {
                        key.to_string().to_lowercase()
                            + "_"
                            + &value
                                .to_string()
                                .replace(" ", "")
                                .replace("\n", "")
                                .replace("<", "_")
                                .replace(">", "_")
                                .replace("[", "_")
                                .replace("]", "_")
                                .replace("(", "_")
                                .replace(")", "_")
                                .replace("::", "_")
                                .replace(",", "_")
                                .trim_end_matches("_")
                                .to_lowercase()
                    })
                    .map(|variant| "__".to_string() + &variant),
            )
            .collect::<String>()
    }
}

impl EntryPoint for () {
    const NAME: &'static str = "";
}
