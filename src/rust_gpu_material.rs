//! Trait

use bevy::prelude::Material;

use crate::prelude::EntryPoint;

/// A [`Material`] type with statically-known `rust-gpu` vertex and fragment entry points.
pub trait RustGpuMaterial: Material {
    type Vertex: EntryPoint;
    type Fragment: EntryPoint;
}
