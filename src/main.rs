use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use bevy::{
    pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin},
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })        // Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup)
        .run();
}

fn create_mesh() -> Mesh {
    let start = Instant::now();
    let mut rng = rand::thread_rng();

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // we want {resolution} by {resolution} tiles
    let resolution = 2000;
    // we want the terrain to occupy size * size units
    let size = 5;

    let res_scale = size as f32 / resolution as f32;

    for x in 0..=resolution {
        for y in 0..=resolution {
            let lx = (x as f32 - (resolution as f32 / 2.0)) * res_scale;
            let ly = (y as f32 - (resolution as f32 / 2.0)) * res_scale;

            let height = rng.gen_range(-2..2) as f32 / 100.0;
            positions.push([lx, height, ly]);
            uvs.push([lx, ly]);
            normals.push([0.0, 1.0, 0.0]);
        }
    }

    let res_plus1 = resolution + 1;
    for py in 0..resolution {
        for px in 0..resolution {
            for off in [0, 1, res_plus1, 1, 1 + res_plus1, res_plus1] {
                indices.push(px + (py * res_plus1) + off);
            }
        }
    }

    let indices = Indices::U32(indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(indices));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let duration = start.elapsed();
    println!("terrain generation took {duration:?}");

    mesh
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // To draw the wireframe on all entities, set this to 'true'
    wireframe_config.global = false;
    // plane
    /*commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });*/
    // cube
    commands.spawn_bundle(PbrBundle {
        // mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        mesh: meshes.add(create_mesh()),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6).into(),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
    // This enables wireframe drawing on this entity
    //.insert(Wireframe);
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        point_light: PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
