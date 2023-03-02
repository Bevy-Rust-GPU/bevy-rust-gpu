use std::path::PathBuf;

use bevy::prelude::{Shader, Handle, AssetServer};

use crate::prelude::SHADER_META_MAP;

/// Loads a `Shader` asset,
/// and optionally its metadata if the corresponding feature flag is enabled.
pub trait LoadRustGpuShader {
    fn load_rust_gpu_shader<'a, P: Into<PathBuf>>(&self, path: P) -> Handle<Shader>;
}

impl LoadRustGpuShader for AssetServer {
    fn load_rust_gpu_shader<'a, P: Into<PathBuf>>(&self, path: P) -> Handle<Shader> {
        let path = path.into();

        let mut meta_path: PathBuf;
        #[cfg(feature = "shader-meta")]
        {
            meta_path = path.clone();
            let last = meta_path.file_name().unwrap().to_str().unwrap().to_string();
            meta_path.pop();
            meta_path.push(last + ".json");
        }

        #[allow(unused_variables)]
        let shader = self.load(path);

        #[cfg(feature = "shader-meta")]
        {
            let mut shader_meta_map = SHADER_META_MAP.write().unwrap();
            shader_meta_map.add(
                shader.clone_weak(),
                self.load::<crate::prelude::ModuleMeta, _>(meta_path),
            );
        }

        shader
    }
}

