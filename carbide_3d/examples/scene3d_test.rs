use carbide_3d::camera::camera_projection::CameraProjection;
use carbide_3d::camera::SimpleCamera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::{Handedness, Mesh, Object, Scene3d, Vertex};
use carbide_core::color::{ColorExt, BLUE, DARK_CHARCOAL, ORANGE, WHITE};
use carbide_core::draw::{Color, Dimension};
use carbide_core::environment::EnvironmentColor;
use carbide_core::state::{GlobalState, LocalState, ReadState};
use carbide_core::widget::WidgetExt;
use carbide_wgpu::{Application, Window};
use std::collections::HashMap;
use tobj::GPU_LOAD_OPTIONS;
use carbide_core::math::{InnerSpace, Matrix4, Point3, Vector3};

fn main() {
    carbide_wgpu_3d::init();

    let (models, materials) =
        tobj::load_obj(
            concat!(env!("CARGO_MANIFEST_DIR"), "/ressources/renderball.obj"),
            &GPU_LOAD_OPTIONS
        )
            .expect("Failed to OBJ load file");

    // Note: If you don't mind missing the materials, you can generate a default.
    let materials = materials.expect("Failed to load MTL file");

    println!("Number of models          = {}", models.len());
    println!("Number of materials       = {}", materials.len());

    let mut application = Application::new();

    let material = PbrMaterial::new().color(ORANGE);
    let material2 = PbrMaterial::new().color(DARK_CHARCOAL);


    /*let object = Object::new(mesh, material);
    let object2 = Object::new(mesh2, material2);*/

    //let object = Object::new(create_mesh(), material);
    //let object = Object::new(create_icosahedron(5), material);
    let object = Object::new(Mesh::from(models[0].clone()), material.clone());
    let object2 = Object::new(Mesh::from(models[1].clone()), material2);

    //let view = Matrix4::look_at_lh(Point3::new(0.0, 1.0, 2.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
    let view = Matrix4::look_at_lh(Point3::new(0.0, 0.5, 1.2), Point3::new(0.0, 0.5, 0.0), Vector3::unit_y());

    let camera = SimpleCamera {
        projection: CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
        //projection: CameraProjection::Orthographic { size: Vector3::new(3.0, 3.0, 100.0) },
        view,
    };

    //let light = DirectionalLight::new(WHITE, 10.0, Vector3::new(-1.0, -1.0, -2.0));
    let light = DirectionalLight::new(Vector3::new(-1.0, -1.0, -1.0))
        .color(WHITE)
        .intensity(5.0);

    let color = LocalState::new(WHITE).hue();
    let color2 = BLUE.hex();
    let color3 = LocalState::new(EnvironmentColor::Red).red();
    let color4 = GlobalState::new(EnvironmentColor::Red).red();
    let color5 = LocalState::new(BLUE).complement().invert().hex();
    let color6 = BLUE.complement().invert();

    println!("{}", color5.value());

    application.set_scene(
        Window::new(
            "Cube example",
            Dimension::new(600.0, 600.0),
            //Scene3d::new((object, light), camera)
            Scene3d::new((object, object2, light), camera)
        )
    );

    application.launch()
}


fn vertex(pos: [f32; 3]) -> Vertex {
    Vertex::new(Vector3::from(pos))
}

fn create_mesh() -> Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 0.5)
        vertex([-0.5, -0.5, 0.5]),
        vertex([0.5, -0.5, 0.5]),
        vertex([0.5, 0.5, 0.5]),
        vertex([-0.5, 0.5, 0.5]),
        // near side (0.0, 0.0, -0.5)
        vertex([-0.5, 0.5, -0.5]),
        vertex([0.5, 0.5, -0.5]),
        vertex([0.5, -0.5, -0.5]),
        vertex([-0.5, -0.5, -0.5]),
        // right side (0.5, 0.0, 0.0)
        vertex([0.5, -0.5, -0.5]),
        vertex([0.5, 0.5, -0.5]),
        vertex([0.5, 0.5, 0.5]),
        vertex([0.5, -0.5, 0.5]),
        // left side (-0.5, 0.0, 0.0)
        vertex([-0.5, -0.5, 0.5]),
        vertex([-0.5, 0.5, 0.5]),
        vertex([-0.5, 0.5, -0.5]),
        vertex([-0.5, -0.5, -0.5]),
        // top (0.0, 0.5, 0.0)
        vertex([0.5, 0.5, -0.5]),
        vertex([-0.5, 0.5, -0.5]),
        vertex([-0.5, 0.5, 0.5]),
        vertex([0.5, 0.5, 0.5]),
        // bottom (0.0, -0.5, 0.0)
        vertex([0.5, -0.5, 0.5]),
        vertex([-0.5, -0.5, 0.5]),
        vertex([-0.5, -0.5, -0.5]),
        vertex([0.5, -0.5, -0.5]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // far
        4, 5, 6, 6, 7, 4, // near
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    Mesh::new(vertex_positions.to_vec(), index_data.to_vec())
        .calculate_normals(Handedness::Left)
}

const X: f32 = 0.525731112119133606;
const Z: f32 = 0.850650808352039932;
const N: f32 = 0.0;

const VERTICES: [[f32; 3]; 12] = [
    [-X,N,Z], [X,N,Z], [-X,N,-Z], [X,N,-Z],
    [N,Z,X], [N,Z,-X], [N,-Z,X], [N,-Z,-X],
    [Z,X,N], [-Z,X, N], [Z,-X,N], [-Z,-X, N]
];

const INDICES: [[u32; 3]; 20] = [
    [0,4,1], [0,9,4], [9,5,4], [4,5,8], [4,8,1],
    [8,10,1], [8,3,10], [5,3,8], [5,2,3], [2,7,3],
    [7,10,3], [7,6,10], [7,11,6], [11,0,6], [0,1,6],
    [6,1,10], [9,0,11], [9,11,2], [9,2,5], [7,2,11]
];

// https://schneide.blog/2016/07/15/generating-an-icosphere-in-c/
fn create_icosahedron(splits: u32) -> Mesh {

    let mut lookup: HashMap<(u32, u32), u32> = HashMap::new();

    fn vertex_for_edge(map: &mut HashMap<(u32, u32), u32>, vertices: &mut Vec<Vertex>, first: u32, second: u32) -> u32 {
        if let Some(index) = map.get(&(first, second)) {
            return *index;
        }

        let new_index = vertices.len() as u32;
        map.insert((first, second), new_index);

        let first_vertex = vertices[first as usize];
        let second_vertex = vertices[second as usize];

        let position = (first_vertex.position + second_vertex.position).normalize();
        let mut vertex = Vertex::new(position);
        vertex.color_0 = Color::random();
        vertices.push(vertex);

        new_index
    }

    let mut current_vertices = VERTICES.into_iter().map(|a| vertex(a)).collect::<Vec<_>>();
    let mut current_indices = INDICES.into_iter().collect::<Vec<_>>();

    for _ in 0..splits {
        let mut new_indices = vec![];

        for current_vertex in current_indices {
            let mut mid = [0u32; 3];

            for i in 0..3 {
                mid[i] = vertex_for_edge(&mut lookup, &mut current_vertices, current_vertex[i], current_vertex[(i + 1) % 3]);
            }

            new_indices.push([current_vertex[0], mid[0], mid[2]]);
            new_indices.push([current_vertex[1], mid[1], mid[0]]);
            new_indices.push([current_vertex[2], mid[2], mid[1]]);
            new_indices.push([mid[0], mid[1], mid[2]]);

        }
        current_indices = new_indices;
    }

    //println!("{:?}", &current_vertices);


    let vertices = current_indices.into_iter().flatten().map(|i| current_vertices[i as usize]).collect::<Vec<_>>();

    let indices = (0..vertices.len() as u32).collect::<Vec<u32>>();

    Mesh::new(
        vertices,
        indices
    ).calculate_normals(Handedness::Right)
}