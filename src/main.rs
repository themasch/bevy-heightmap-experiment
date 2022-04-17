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
        .add_startup_system(setup)
        .add_startup_system(setup_camera)
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
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_mesh: Handle<Mesh> = asset_server.load("linear_gradient.hm.png");
    commands
        .spawn_bundle(PbrBundle {
            mesh: terrain_mesh.clone(),
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
