use bevy::app::AppExit;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::pbr::wireframe::{Wireframe, WireframePlugin};
use bevy::prelude::Entity;
use bevy::{
    prelude::*,
    render::{options::WgpuOptions, render_resource::WgpuFeatures},
};

use smooth_bevy_cameras::controllers::orbit::{
    OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin,
};
use smooth_bevy_cameras::LookTransformPlugin;

mod height_map;

use height_map::loader::HeightmapMeshLoader;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        }) // Adds frame time diagnostics
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // Adds a system that prints diagnostics to the console
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(WireframePlugin)
        .add_asset_loader(HeightmapMeshLoader)
        .add_startup_system(setup)
        .add_startup_system(setup_camera)
        .add_system(exit)
        .add_system(toggle_wireframe)
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

/// exit the .. gam... thing when Q or ESC is released
fn exit(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_released(KeyCode::Q) || keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

fn toggle_wireframe(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    entities: Query<(Entity, With<ToggleWireframe>, Option<With<Wireframe>>)>,
) {
    if keyboard_input.just_released(KeyCode::W) {
        for (entity, _, has_wf) in entities.iter() {
            if has_wf.is_some() {
                commands.entity(entity).remove::<Wireframe>();
            } else {
                commands.entity(entity).insert(Wireframe);
            }
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
struct ToggleWireframe;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    let terrain_mesh: Handle<Mesh> = asset_server.load("Heightmap5_DISP.hm.png");
    //let terrain_mesh: Handle<Mesh> = asset_server.load("Sc2wB.hm.jpg");
    //let terrain_mesh: Handle<Mesh> = asset_server.load("dereth-2015-07-27-height.hm.png");
    commands
        .spawn_bundle(PbrBundle {
            mesh: terrain_mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.8, 0.8),
                metallic: 0.25,
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
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
