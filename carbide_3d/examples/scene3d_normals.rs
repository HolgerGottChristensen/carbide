use std::collections::HashMap;
use std::env::current_dir;
use std::f32::consts::PI;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tobj::GPU_LOAD_OPTIONS;
use carbide_3d::{Handedness, Mesh, Object, Scene3d, Vertex};
use carbide_3d::camera::camera_projection::CameraProjection;
use carbide_3d::camera::SimpleCamera;
use carbide_3d::light::DirectionalLight;
use carbide_3d::material::Material;
use carbide_3d::material::pbr_material::PbrMaterial;
use carbide_3d::node3d::Node3dExt;
use carbide_core::animation::ease_in_out;
use carbide_core::color::{BLACK, BLUE, ColorExt, DARK_BLUE, DARK_CHARCOAL, DARK_GREEN, GREEN, GREY, ORANGE, RED, WHITE};
use carbide_core::draw::{Color, Dimension, ImageId};
use carbide_core::environment::{EnvironmentColor, IntoColorReadState};
use carbide_core::render::matrix::{Deg, Euler, InnerSpace, Matrix3, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3, Vector4, Zero};
use carbide_core::state::{AnimatedState, GlobalState, LocalState, Map1, Map2, ReadState, ReadStateExtNew};
use carbide_core::widget::WidgetExt;
use carbide_wgpu::{Application, Window};

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

    let animated_color = AnimatedState::linear().duration(Duration::from_secs_f64(40.0)).repeat().range(0.0, 2.0*PI).map(|h| Color::Hsla(*h, 1.0, 0.5, 1.0));

    let material = PbrMaterial::new()
        .color(animated_color.clone());
    let material2 = PbrMaterial::new()
        .color(animated_color)
        //.color(ImageId::new("materials/ground068/Ground068.color.png"))
        //.normal(ImageId::new("materials/ground068/Ground068.normal.dx.png"))
        .normal(ImageId::new("images/normalmap.png"))
        ;
        //.color(ImageId::new("materials/plaster/Plaster001.color.png"));

    let material_green = PbrMaterial::new().color(DARK_GREEN);
    let material_red = PbrMaterial::new().color(RED);
    let material_blue = PbrMaterial::new().color(DARK_BLUE);
    let material_white = PbrMaterial::new().color(WHITE);


    /*let object = Object::new(mesh, material);
    let object2 = Object::new(mesh2, material2);*/

    let object = Object::new(create_plane(Vector3::new(0.0, 0.0, -0.05), 0.4), material2);
    let object2 = Object::new(create_plane(Vector3::new(0.0, 0.0, -0.06), 0.45), material);
    //let object = Object::new(create_mesh(), material2);
    //let object = Object::new(create_icosahedron(5), material);
    //let object = Object::new(Mesh::from(models[0].clone()), material);
    //let object2 = Object::new(Mesh::from(models[1].clone()), material2);
    let center = Object::new(create_cube(Vector3::zero(), 0.05), material_white);
    let x = Object::new(create_cube(Vector3::new(0.05, 0.0, 0.0), 0.03), material_red);
    let y = Object::new(create_cube(Vector3::new(0.0, 0.05, 0.0), 0.03), material_green);
    let z = Object::new(create_cube(Vector3::new(0.0, 0.0, 0.05), 0.03), material_blue);

    let animated = AnimatedState::custom(ease_in_out).duration(Duration::from_secs_f64(3.0)).repeat_alternate().range(0.0f32, 0.25f32);
    //let animated = AnimatedState::linear(None).duration(Duration::from_secs_f64(3.0)).repeat().range(0.0f32, 1.0f32);
    let rotation = Map1::read_map(animated, |t| Matrix4::<f32>::from(Euler::new(Deg(0.0), Deg(*t * 360.0), Deg(0.0))));
    let view = Matrix4::look_at_lh(Point3::new(1.0, 0.0, 1.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
    let view2 = Matrix4::look_at_lh(Point3::new(1.0, 1.0, 1.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
    let view3 = Matrix4::look_at_lh(Point3::new(0.0, 0.0, 1.0), Point3::new(0.0, 0.0, 0.0), Vector3::unit_y());
    let rotating_view = Map2::read_map(rotation, view, |rotation, view| {
         *view * *rotation
    });

    //let view = Matrix4::look_at_lh(Point3::new(0.0, 0.5, 1.2), Point3::new(0.0, 0.5, 0.0), Vector3::unit_y());

    let camera = SimpleCamera {
        //projection: CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
        projection: CameraProjection::Orthographic { size: Vector3::new(0.5, 0.5, 100.0) },
        //view: rotating_view,
        //view: view2,
        view: view3,
    };

    //let light = DirectionalLight::new(WHITE, 10.0, Vector3::new(-1.0, -1.0, -2.0));
    /*let light = DirectionalLight::new(Vector3::new(-1.0, -1.0, -1.0))
        .color(WHITE)
        .intensity(5.0);*/


    let animated = AnimatedState::custom(ease_in_out).duration(Duration::from_secs_f64(3.0)).repeat_alternate().range(-0.15f32, 0.15f32);
    let rotation = Map1::read_map(animated, |t| Matrix3::<f32>::from(Euler::new(Deg(-20.0), Deg(*t * 360.0), Deg(0.0))));

    let direction = Map1::read_map(rotation, |rotation| {
        *rotation * Vector3::new(0.0, 0.0, 1.0)
    });

    //let light = DirectionalLight::new(Vector3::new(0.0, 0.0, 1.0))
    let light = DirectionalLight::new(direction)
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
            "Cube example - Carbide",
            Dimension::new(600.0, 600.0),
            Scene3d::new((center, x, y, z, object, object2, light), camera)
            //Scene3d::new((object, light), camera)
            //Scene3d::new((object, object2, light), camera)
        )
    );

    application.launch()
}


fn vertex(pos: [f32; 3]) -> Vertex {
    Vertex::new(Vector3::from(pos))
}

fn vertex_coord(pos: [f32; 3], coord: [f32; 2]) -> Vertex {
    Vertex::new(Vector3::from(pos))
        .texture_coords_0(Vector2::from(coord))
}

fn create_plane(origin: Vector3<f32>, width: f32) -> Mesh {
    let vertex_positions = [
        vertex_coord([-0.5 * width + origin.x, 0.5 * width + origin.y, origin.z], [0.0, 0.0]),
        vertex_coord([0.5 * width + origin.x, 0.5 * width + origin.y, origin.z], [1.0, 0.0]),
        vertex_coord([0.5 * width + origin.x, -0.5 * width + origin.y, origin.z], [1.0, 1.0]),
        vertex_coord([-0.5 * width + origin.x, -0.5 * width + origin.y, origin.z], [0.0, 1.0]),
    ];

    let index_data: &[u32] = &[
        0, 2, 1, 2, 0, 3,
    ];

    Mesh::new(vertex_positions.to_vec(), index_data.to_vec())
        .calculate_normals(Handedness::Left)
        .calculate_tangents()
}

fn create_cube(origin: Vector3<f32>, width: f32) -> Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 0.5)
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        // near side (0.0, 0.0, -0.5 * width)
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
        // right side (0.5 * width, 0.0, 0.0)
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        // left side (-0.5 * width, 0.0, 0.0)
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
        // top (0.0, 0.5 * width, 0.0)
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, 0.5 * width + origin.y, 0.5 * width + origin.z]),
        // bottom (0.0, -0.5 * width, 0.0)
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, 0.5 * width + origin.z]),
        vertex([-0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
        vertex([0.5 * width + origin.x, -0.5 * width + origin.y, -0.5 * width + origin.z]),
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