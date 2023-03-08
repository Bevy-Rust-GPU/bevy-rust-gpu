//! Adds support for exporting the active entry point set to a `.json` file.
//!
//! This can be used in conjunction with `rust-gpu-builder` and `permutate-macro` to drive hot-recompiles.

use std::{
    fs::File,
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender},
};

use bevy::{
    prelude::{default, info, CoreSet, Deref, DerefMut, IntoSystemConfig, NonSendMut, Plugin},
    render::render_resource::ShaderDefVal,
    tasks::IoTaskPool,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "hot-rebuild")]
pub(crate) static EXPORT_HANDLES: once_cell::sync::Lazy<
    std::sync::RwLock<HashMap<PathBuf, ExportHandle>>,
> = once_cell::sync::Lazy::new(default);

#[cfg(feature = "hot-rebuild")]
pub(crate) static MATERIAL_EXPORTS: once_cell::sync::Lazy<
    std::sync::RwLock<HashMap<std::any::TypeId, PathBuf>>,
> = once_cell::sync::Lazy::new(default);

/// Handles exporting known `RustGpuMaterial` permutations to a JSON file for static compilation.
pub struct EntryPointExportPlugin;

impl Plugin for EntryPointExportPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.world.init_non_send_resource::<EntryPointExport>();

        app.add_systems((
            EntryPointExport::create_export_containers_system.in_base_set(CoreSet::Update),
            EntryPointExport::receive_entry_points_system.in_base_set(CoreSet::Last),
            EntryPointExport::export_entry_points_system
                .in_base_set(CoreSet::Last)
                .after(EntryPointExport::receive_entry_points_system),
        ));
    }
}

/// Handle to an entry point file export
pub type ExportHandle = SyncSender<Export>;

/// MPSC reciever carrying entry points for export.
type EntryPointReceiver = Receiver<Export>;

/// MPSC message describing an entry point.
#[derive(Debug, Default, Clone)]
pub struct Export {
    pub shader: &'static str,
    pub permutation: Vec<String>,
    pub constants: Vec<ShaderDefVal>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged)]
enum PermutationConstant {
    Bool(bool),
    Uint(u32),
    Int(i32),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref, DerefMut)]
struct PermutationConstants {
    #[serde(flatten)]
    constants: HashMap<String, PermutationConstant>,
}

impl From<Vec<ShaderDefVal>> for PermutationConstants {
    fn from(value: Vec<ShaderDefVal>) -> Self {
        PermutationConstants {
            constants: value
                .into_iter()
                .map(|def| match def {
                    ShaderDefVal::Bool(key, value) => (key, PermutationConstant::Bool(value)),
                    ShaderDefVal::Int(key, value) => (key, PermutationConstant::Int(value)),
                    ShaderDefVal::UInt(key, value) => (key, PermutationConstant::Uint(value)),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Permutation {
    parameters: Vec<String>,
    constants: PermutationConstants,
}

/// Serializable container for a single entry point
#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
struct EntryPoints {
    #[serde(flatten)]
    entry_points: HashMap<String, Vec<Permutation>>,
}

/// Container for a set of entry points, with MPSC handles and change tracking
#[derive(Debug)]
struct EntryPointExportContainer {
    rx: EntryPointReceiver,
    entry_points: EntryPoints,
    changed: bool,
}

/// Non-send resource used to register export files and aggregate their entry points.
#[derive(Debug, Default, Deref, DerefMut)]
struct EntryPointExport {
    exports: HashMap<PathBuf, EntryPointExportContainer>,
}

impl EntryPointExport {
    /// System used to populate export containers for registered materials
    pub fn create_export_containers_system(mut exports: NonSendMut<Self>) {
        let material_exports = MATERIAL_EXPORTS.read().unwrap();
        for (_, path) in material_exports.iter() {
            if !exports.contains_key(path) {
                let (tx, rx) = std::sync::mpsc::sync_channel::<Export>(32);

                EXPORT_HANDLES.write().unwrap().insert(path.clone(), tx);

                let container = EntryPointExportContainer {
                    rx,
                    entry_points: default(),
                    changed: default(),
                };

                exports.insert(path.clone(), container);
            }
        }
    }

    /// System used to receive and store entry points sent from materials.
    pub fn receive_entry_points_system(mut exports: NonSendMut<Self>) {
        for (_, export) in exports.exports.iter_mut() {
            while let Ok(entry_point) = export.rx.try_recv() {
                if !export.entry_points.contains_key(entry_point.shader) {
                    info!("New entry point: {}", entry_point.shader);
                    export
                        .entry_points
                        .insert(entry_point.shader.to_string(), default());
                    export.changed = true;
                }

                let entry = &export.entry_points[entry_point.shader];
                let permutation = Permutation {
                    parameters: entry_point.permutation,
                    constants: entry_point.constants.into(),
                };

                if !entry.contains(&permutation) {
                    info!("New permutation: {:?}", permutation);
                    export
                        .entry_points
                        .get_mut(entry_point.shader)
                        .unwrap()
                        .push(permutation);
                    export.changed = true;
                }
            }
        }
    }

    /// System used to write active entry point sets to their respective files on change via the IO task pool.
    pub fn export_entry_points_system(mut exports: NonSendMut<Self>) {
        for (path, export) in exports.exports.iter_mut() {
            if export.changed {
                let entry_points = export.entry_points.clone();
                let path = path.clone();
                let io_pool = IoTaskPool::get();
                io_pool
                    .spawn(async move {
                        info!("Exporting entry points to {:}", path.to_str().unwrap());
                        let writer = File::create(path).unwrap();
                        serde_json::to_writer_pretty(writer, &entry_points).unwrap();
                    })
                    .detach();
                export.changed = false;
            }
        }
    }
}
