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

/// Handles exporting known `RustGpuMaterial` permutations to a JSON file for static compilation
pub struct EntryPointExportPlugin;

impl Plugin for EntryPointExportPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.world.init_non_send_resource::<EntryPointExport>();

        app.add_system_to_stage(CoreStage::Last, EntryPointExport::receive_entry_points)
            .add_system_to_stage(
                CoreStage::Last,
                EntryPointExport::export_entry_points.after(EntryPointExport::receive_entry_points),
            );
    }
}

pub type EntryPointSender = SyncSender<Export>;
pub type EntryPointReceiver = Receiver<Export>;

#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct EntryPoints {
    pub entry_points: HashMap<String, Vec<Vec<String>>>,
}

#[derive(Debug, Default, Clone)]
pub struct Export {
    pub shader: &'static str,
    pub permutation: Vec<String>,
}

#[derive(Debug)]
pub struct EntryPointExportContainer {
    pub tx: EntryPointSender,
    pub rx: EntryPointReceiver,
    pub entry_points: EntryPoints,
    pub changed: bool,
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

#[derive(Debug, Default, Deref, DerefMut)]
pub struct EntryPointExport {
    pub exports: HashMap<PathBuf, EntryPointExportContainer>,
}

impl EntryPointExport {
    pub fn export<T: Into<PathBuf>>(&mut self, path: T) -> EntryPointSender {
        self.exports.entry(path.into()).or_default().tx.clone()
    }

    pub fn receive_entry_points(mut exports: NonSendMut<Self>) {
        for (_, export) in exports.iter_mut() {
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

    pub fn export_entry_points(mut exports: NonSendMut<Self>) {
        for (path, export) in exports.iter_mut() {
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
