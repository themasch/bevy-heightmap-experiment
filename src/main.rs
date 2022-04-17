use std::path::PathBuf;
use bevy::asset::AssetServerSettings;
use bevy::diagnostic::EntityCountDiagnosticsPlugin;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{render_resource::WgpuFeatures, settings::WgpuSettings},
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

use systems::{TerrainMarker, ToggleWireframe};
use venture::debug_ui::DebugUiPlugin;
use venture::height_map::loader::HeightmapMeshLoader;

mod systems;

#[derive(Debug, Clone, Default)]
struct LoadTerrainMapPath(Option<PathBuf>);

fn main() {
    let assets = std::env::current_dir()
        .unwrap()
        .join("assets")
        .into_os_string()
        .into_string()
        .unwrap();

    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuSettings {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..default()
        })
        .insert_resource(AssetServerSettings {
            asset_folder: assets,
            watch_for_changes: false,
        })
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_plugin(DebugUiPlugin)
        .init_asset_loader::<HeightmapMeshLoader>()
        .insert_resource(LoadTerrainMapPath::default())
        .add_startup_system(setup)
        .add_startup_system(setup_camera)
        .add_system(file_drag_and_drop_system)
        .add_system(load_new_terrain)
        .add_system(systems::exit_from_keypress)
        .add_system(systems::toggle_wireframe)
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_mesh = meshes.add(Mesh::from(shape::Quad::default()));
    commands
        .spawn_bundle(PbrBundle {
            mesh: terrain_mesh.clone_weak(),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.8, 0.8),
                metallic: 0.25,
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(TerrainMarker(terrain_mesh))
        .insert(ToggleWireframe);

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

fn load_new_terrain(mut terrain: Query<(&mut Handle<Mesh>, &mut TerrainMarker)>, mut to_load: ResMut<LoadTerrainMapPath>, asset_server: Res<AssetServer>) {
    if to_load.0.is_none() {
        return;
    }

    // we can unwrap here, as we know that to_load.0 is Some(..)
    let path = to_load.0.take().unwrap();
    println!("loading new map from {:?}", path.as_os_str());
    let terrain_mesh: Handle<Mesh> = asset_server.load(path);

    let (mut current_mesh, mut marker) = terrain.single_mut();

    *current_mesh = terrain_mesh.clone_weak();
    marker.0 = terrain_mesh;
}

fn file_drag_and_drop_system(mut events: EventReader<FileDragAndDrop>, mut to_load: ResMut<LoadTerrainMapPath>) {
    for event in events.iter() {
        if let FileDragAndDrop::DroppedFile { id: _, path_buf } = event {
            dbg!(&path_buf);
            to_load.0 = Some(path_buf.clone());
            // we only want one update and do not care about additional files dropped
            return;
        }
    }
}
