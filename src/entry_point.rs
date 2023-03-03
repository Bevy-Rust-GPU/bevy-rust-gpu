//! Trait representation of a `rust-gpu` entry point.

/// An entry point name for use with the [`EntryPoint`] trait.
pub type EntryPointName = &'static str;

/// A set of entry compile parameters for use with the [`EntryPoint`] trait.
pub type EntryPointParameters =
    &'static [(&'static [(&'static str, &'static str)], &'static str)];

/// A `rust-gpu` entry point for use with [`RustGpuMaterial`](crate::rust_gpu_material::RustGpuMaterial).
pub trait EntryPoint: 'static + Send + Sync {
    /// The entry point's base function name, including module path
    ///
    /// ```
    /// # use bevy_rust_gpu::prelude::EntryPointName;
    /// const NAME: EntryPointName = "mesh::entry_points::vertex";
    /// ```
    const NAME: &'static str;

    /// Mapping from bevy shader defs to `permutate-macro` parameters.
    ///
    /// ```
    /// # use bevy_rust_gpu::prelude::EntryPointParameters;
    /// const PARAMETERS: EntryPointParameters = &[
    ///     (&[("VERTEX_TANGENTS", "some")], "none"),
    ///     (&[("VERTEX_COLORS", "some")], "none"),
    ///     (&[("SKINNED", "some")], "none"),
    /// ];
    /// ```
    const PARAMETERS: EntryPointParameters;

    /// Constructs a permutation set from the provided shader defs
    fn permutation(shader_defs: &Vec<String>) -> Vec<String> {
        let mut permutation = vec![];

        for (defined, undefined) in Self::PARAMETERS.iter() {
            if let Some(mapping) = defined.iter().find_map(|(def, mapping)| {
                if shader_defs.contains(&def.to_string()) {
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

    /// Build an entry point name from the provided shader defs
    fn build(shader_defs: &Vec<String>) -> String {
        std::iter::once(Self::NAME.to_string())
            .chain(
                Self::permutation(shader_defs)
                    .into_iter()
                    .map(|variant| "__".to_string() + &variant),
            )
            .collect::<String>()
    }
}

impl EntryPoint for () {
    const NAME: &'static str = "";
    const PARAMETERS: EntryPointParameters = &[];
}
