use bevy::{app::AppExit, pbr::wireframe::Wireframe, prelude::*};

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct TerrainMarker(pub Handle<Mesh>);

/// test/example on how to get access to a mesh and update its indices.
/// we might be able to use this later to update the lod of a mesh?
#[allow(dead_code)]
fn update_terrain_lod(
    keyboard_input: Res<Input<KeyCode>>,
    mut assets: ResMut<Assets<Mesh>>,
    terrain_bundles: Query<&TerrainMarker>,
) {
    if keyboard_input.just_released(KeyCode::H) {
        for terrain in terrain_bundles.iter() {
            assets.get_mut(terrain.0.clone()).unwrap().set_indices(None);
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct ToggleWireframe;

pub fn toggle_wireframe(
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

/// exit the .. gam... thing when Q or ESC is released
pub fn exit_from_keypress(
    keyboard_input: Res<Input<KeyCode>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    if keyboard_input.just_released(KeyCode::Q) || keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
