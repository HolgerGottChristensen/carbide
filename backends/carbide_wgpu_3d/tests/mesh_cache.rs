//! The wgpu 3D renderer's cache of meshes on the GPU.

use carbide_3d::Mesh;
use carbide_wgpu_3d::MeshCache;
use carbide_wgpu_core::WgpuContextNew;

fn device() -> WgpuContextNew {
    futures::executor::block_on(WgpuContextNew::new())
}

#[test]
fn clones_of_a_mesh_share_one_upload() {
    let context = device();
    let mut cache = MeshCache::new();

    let mesh = Mesh::cube(1.0);
    cache.upload(&mesh, &context.device);
    cache.upload(&mesh.clone(), &context.device);

    assert_eq!(cache.uploads(), 1);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(mesh.id()).unwrap().index_count, 36);
}

#[test]
fn a_mesh_is_freed_once_its_last_handle_drops() {
    let context = device();
    let mut cache = MeshCache::new();

    let mesh = Mesh::cube(1.0);
    let clone = mesh.clone();
    cache.upload(&mesh, &context.device);

    drop(mesh);
    assert_eq!(cache.free_dropped(), 0, "a clone still holds the mesh");
    assert_eq!(cache.len(), 1);

    drop(clone);
    assert_eq!(cache.free_dropped(), 1);
    assert!(cache.is_empty());
}

#[test]
fn a_changed_mesh_is_a_new_upload() {
    let context = device();
    let mut cache = MeshCache::new();

    let mesh = Mesh::cube(1.0);
    let changed = mesh.clone().calculate_tangents();
    cache.upload(&mesh, &context.device);
    cache.upload(&changed, &context.device);

    assert_eq!(cache.uploads(), 2);
    assert_ne!(mesh.id(), changed.id());
}

#[test]
fn an_empty_mesh_is_not_uploaded() {
    let context = device();
    let mut cache = MeshCache::new();

    let empty = Mesh::new(vec![], vec![]);
    cache.upload(&empty, &context.device);

    assert_eq!(cache.uploads(), 0);
    assert!(cache.get(empty.id()).is_none());
}
