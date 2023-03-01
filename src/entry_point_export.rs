//! Adds support for exporting the active entrypoint set to a `.json` file.
//!
//! This can be used in conjunction with `rust-gpu-builder` and `permutate-macro` to drive hot-recompiles.

use std::{
    fs::File,
    path::PathBuf,
    sync::mpsc::{Receiver, SyncSender},
};

use bevy::{
    prelude::{
        default, info, CoreStage, Deref, DerefMut, IntoSystemDescriptor, NonSendMut, Plugin,
    },
    tasks::IoTaskPool,
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

/// Handles exporting known `RustGpuMaterial` permutations to a JSON file for static compilation.
pub struct EntryPointExportPlugin;

impl Plugin for EntryPointExportPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.world.init_non_send_resource::<EntryPointExport>();

        app.add_system_to_stage(
            CoreStage::Last,
            EntryPointExport::receive_entry_points_system,
        )
        .add_system_to_stage(
            CoreStage::Last,
            EntryPointExport::export_entry_points_system
                .after(EntryPointExport::receive_entry_points_system),
        );
    }
}

/// MPSC sender carrying entry points for export.
pub type EntryPointSender = SyncSender<Export>;

/// MPSC reciever carrying entry points for export.
type EntryPointReceiver = Receiver<Export>;

/// MPSC message describing an entry point.
#[derive(Debug, Default, Clone)]
pub struct Export {
    pub shader: &'static str,
    pub permutation: Vec<String>,
}

/// Serializable container for a single entry point
#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
struct EntryPoints {
    #[serde(flatten)]
    entry_points: HashMap<String, Vec<Vec<String>>>,
}

/// Container for a set of entry points, with MPSC handles and change tracking
#[derive(Debug)]
struct EntryPointExportContainer {
    tx: EntryPointSender,
    rx: EntryPointReceiver,
    entry_points: EntryPoints,
    changed: bool,
}

impl Default for EntryPointExportContainer {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<Export>(32);

        EntryPointExportContainer {
            tx,
            rx,
            entry_points: default(),
            changed: default(),
        }
    }
}

/// Non-send resource used to register export files and aggregate their entry points.
#[derive(Debug, Default)]
pub struct EntryPointExport {
    exports: HashMap<PathBuf, EntryPointExportContainer>,
}

impl EntryPointExport {
    /// Registers a path to which entrypoints will be exported,
    /// returning a corresponding [`EntryPointSender`] that can be passed to a
    /// [`RustGpuMaterial`](crate::rust_gpu_material::RustGpuMaterial).
    pub fn export<T: Into<PathBuf>>(&mut self, path: T) -> EntryPointSender {
        self.exports.entry(path.into()).or_default().tx.clone()
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
                if !entry.contains(&entry_point.permutation) {
                    info!("New permutation: {:?}", entry_point.permutation);
                    export
                        .entry_points
                        .get_mut(entry_point.shader)
                        .unwrap()
                        .push(entry_point.permutation);
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
