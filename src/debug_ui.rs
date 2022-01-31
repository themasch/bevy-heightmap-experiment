use bevy::app::App;
use bevy::prelude::*;
use bevy::diagnostic::{Diagnostics, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};

pub struct DebugUiPlugin;

impl Plugin for DebugUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_ui);
        app.add_system(update_fps_text);
        app.add_system(update_entity_count_text);
    }
}

#[derive(Component)]
struct FpsText;

#[derive(Component)]
struct EntityCountText;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("font/FiraSans-Book.otf");

    commands.spawn_bundle(UiCameraBundle::default());

    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Auto, Val::Auto),
            flex_direction: FlexDirection::ColumnReverse,
            position_type: PositionType::Absolute,
            padding: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                right: Val::Px(5.0),
                bottom: Val::Px(5.0),
            },
            position: Rect {
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        color: Color::rgba(0.3, 0.3, 0.3, 0.95).into(),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                flex_shrink: 0.0,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 15.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "000".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 15.0,
                            color: Color::YELLOW,
                        },
                    },
                ],
                alignment: Default::default(),
            },
            ..Default::default()
        })
            .insert(FpsText);

        parent.spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                flex_shrink: 0.0,
                ..Default::default()
            },
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Entities: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 15.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "000".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 15.0,
                            color: Color::YELLOW,
                        },
                    },
                ],
                alignment: Default::default(),
            },
            ..Default::default()
        })
            .insert(EntityCountText);
    });
}

fn update_fps_text(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        let fps = if let Some(fps_from_diagnostics) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(fps_avg) = fps_from_diagnostics.average() {
                fps_avg
            } else {
                0.0
            }
        } else {
            0.0
        };

        text.sections[1].value = format!("{:.1}", fps);
    }
}


fn update_entity_count_text(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<EntityCountText>>) {
    for mut text in query.iter_mut() {
        let fps = if let Some(entity_count_dia) = diagnostics.get(EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
            if let Some(fps_avg) = entity_count_dia.average() {
                fps_avg
            } else {
                0.0
            }
        } else {
            0.0
        };

        text.sections[1].value = format!("{:.1}", fps);
    }
}
