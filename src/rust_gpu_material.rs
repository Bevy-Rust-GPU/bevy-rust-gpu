//! Trait

use std::marker::PhantomData;

use bevy::{
    prelude::{
        default, CoreStage, Handle, IntoSystemDescriptor, Material, MaterialPlugin, Plugin, Shader,
    },
    render::render_resource::AsBindGroup,
};

use crate::prelude::{reload_materials, shader_events, EntryPoint, RustGpu};

#[cfg(feature = "entry-point-export")]
use crate::prelude::ExportHandle;

/// A [`Material`] type with statically-known `rust-gpu` vertex and fragment entry points.
pub trait RustGpuMaterial: Material {
    type Vertex: EntryPoint;
    type Fragment: EntryPoint;
}
