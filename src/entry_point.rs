pub type EntryPointName = &'static str;
pub type EntryPointMappings =
    &'static [(&'static [(&'static str, &'static str)], &'static str)];

pub trait EntryPoint: 'static + Send + Sync {
    const NAME: &'static str;
    const PARAMETERS: EntryPointMappings;

    fn is_defined(shader_defs: &Vec<String>, def: &String) -> bool {
        let def = def.into();
        shader_defs.contains(def)
    }

    fn permutation(shader_defs: &Vec<String>) -> Vec<String> {
        let mut permutation = vec![];

        for (defined, undefined) in Self::PARAMETERS.iter() {
            if let Some(mapping) = defined.iter().find_map(|(def, mapping)| {
                if Self::is_defined(shader_defs, &def.to_string()) {
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
    const PARAMETERS: EntryPointMappings = &[];
}

/// Manually compose `bevy_render` shader defs that aren't available during specialization
pub fn rust_gpu_shader_defs() -> Vec<String> {
    // NO_STORAGE_BUFFERS_SUPPORT is implied for now,
    // since `rust-gpu` can't bind read-only storage buffers yet
    let extra_defs = vec!["NO_STORAGE_BUFFERS_SUPPORT".to_string()];

    // Same webgl logic as `bevy_render/src/render_resource/pipeline_cache.rs`
    #[cfg(feature = "webgl")]
    let extra_defs = extra_defs + vec!["NO_TEXTURE_ARRAYS_SUPPORT", "SIXTEEN_BYTE_ALIGNMENT"];

    extra_defs
}
