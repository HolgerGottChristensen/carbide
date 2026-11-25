use std::collections::HashMap;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

use carbide_3d::{Mesh, MeshId, WeakMesh};
use carbide_core::environment::{EnvironmentKey, Exclusive};

use crate::vertex::WgpuVertex;

/// The GPU copies of meshes, shared by every view the wgpu 3D renderer draws. Each mesh is
/// uploaded once, into buffers of its own, the first time a view draws it, and freed once every
/// handle to the mesh is gone.
///
/// Registered in the environment by the renderer's plugin.
#[derive(Debug, Default)]
pub struct MeshCache {
    meshes: HashMap<MeshId, CachedMesh>,
    uploads: u64,
}

/// A mesh's GPU copy.
#[derive(Debug)]
pub struct CachedMesh {
    /// Whether the mesh still has handles; when it hasn't, the buffers are freed.
    mesh: WeakMesh,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
}

impl MeshCache {
    pub fn new() -> MeshCache {
        MeshCache::default()
    }

    /// Upload `mesh` unless it already is. A mesh with no triangles is not uploaded.
    pub fn upload(&mut self, mesh: &Mesh, device: &Device) {
        if mesh.indices().is_empty() || self.meshes.contains_key(&mesh.id()) {
            return;
        }

        let vertices = mesh.vertices().iter().map(WgpuVertex::from_vertex).collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("carbide_3d_mesh_vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("carbide_3d_mesh_indices"),
            contents: bytemuck::cast_slice(mesh.indices()),
            usage: BufferUsages::INDEX,
        });

        self.meshes.insert(mesh.id(), CachedMesh {
            mesh: mesh.downgrade(),
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices().len() as u32,
        });
        self.uploads += 1;
    }

    /// The GPU copy of the mesh with `id`, if it has been uploaded.
    pub fn get(&self, id: MeshId) -> Option<&CachedMesh> {
        self.meshes.get(&id)
    }

    /// Free the GPU copies of meshes that have no handles left. Returns how many were freed.
    pub fn free_dropped(&mut self) -> usize {
        let before = self.meshes.len();
        self.meshes.retain(|_, cached| cached.mesh.is_alive());
        before - self.meshes.len()
    }

    /// How many meshes have a GPU copy.
    pub fn len(&self) -> usize {
        self.meshes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.meshes.is_empty()
    }

    /// How many uploads the cache has made, in total.
    pub fn uploads(&self) -> u64 {
        self.uploads
    }
}

impl EnvironmentKey for MeshCache {
    type Value = MeshCache;
    type Access = Exclusive;
}
