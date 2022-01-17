use bevy::{
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use smooth_bevy_cameras::LookTransformPlugin;
use smooth_bevy_cameras::controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin};

mod height_map;

use height_map::loader::HeightmapMeshLoader;

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
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_asset_loader(HeightmapMeshLoader)
        .add_startup_system(setup)
        .add_startup_system(setup_camera)
        .run();
}


fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrbitCameraBundle::new(
        OrbitCameraController::default(),
        PerspectiveCameraBundle::default(),
        Vec3::new(-2.0, 5.0, 5.0),
        Vec3::new(0., 0., 0.),
    ));
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    //let texture_handle: Handle<Mesh> = asset_server.load("Heightmap5_DISP.hm.png");
    let texture_handle: Handle<Mesh> = asset_server.load("Sc2wB.hm.jpg");
    commands.spawn_bundle(PbrBundle {
        //mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        //mesh: meshes.add(create_mesh(HeightMap::create(ThreadLocalRngHeightSource::new(), 2000, 5.0))),
        mesh: texture_handle,
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6).into(),
            metallic: 0.5,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
    // This enables wireframe drawing on this entity
    // .insert(Wireframe)

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        point_light: PointLight {
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(-4.0, 8.0, -4.0),
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 400.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
