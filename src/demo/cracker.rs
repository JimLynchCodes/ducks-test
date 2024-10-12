use bevy::{prelude::*, utils::info};

use super::websocket_connect::MoveCrackersBevyEvent;

#[derive(Component)]
struct _CrackerComponent;

#[derive(Component)]
struct _CrackerText;

const CRACKER_TEXT_OFFSET: f32 = 22.;

pub(super) fn plugin(_app: &mut App) {
    // WIP - cracker stuff
    _app.add_systems(Startup, _create_cracker);
    _app.add_systems(Startup, _create_cracker_text);
    _app.add_systems(Update, listen_for_move_cracker_bevy_event);
}

fn listen_for_move_cracker_bevy_event(
    mut bevy_move_crackers_event_reader: EventReader<MoveCrackersBevyEvent>,
    mut param_set: ParamSet<(
        Query<&mut Transform, With<_CrackerComponent>>,
        Query<&mut Transform, With<_CrackerText>>,
        Query<&mut Text, With<_CrackerText>>,
    )>,
) {
    for e in bevy_move_crackers_event_reader.read() {

        info!(
            "Moving crackers! x: {:?}, y: {:?}",
            e.x_position, e.y_position
        );

        for mut transform in param_set.p0().iter_mut() {
            // Mutate the transform, e.g., moving the cracker component upwards
            transform.translation.x = e.x_position;
            transform.translation.y = e.y_position;
        }

        for mut transform in param_set.p1().iter_mut() {
            // Mutate the transform, e.g., moving the cracker component upwards
            transform.translation.x = e.x_position;
            transform.translation.y = e.y_position + CRACKER_TEXT_OFFSET;
        }

        for mut transform in param_set.p2().iter_mut() {
            transform.sections[0].value = e.points.to_string();
        }
    }
}

fn _create_cracker(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle = asset_server.load("images/cracker-v1.png");

    println!("creating cracker...");

    commands
        .spawn(SpriteBundle {
            texture: texture_handle,
            transform: Transform {
                scale: Vec3::new(0.075, 0.075, 0.0),
                translation: Vec3::new(0.0, 0.0, 5.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(_CrackerComponent);
}

fn _create_cracker_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("inserting text!!");

    commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "10".to_string(), // The text to display
                TextStyle {
                    font: asset_server.load("FiraSans-Bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform {
                translation: Vec3::new(0.0, CRACKER_TEXT_OFFSET, 5.0), // Position the text above the crackers & in front of the background
                scale: Vec3::new(1., 1., 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(_CrackerText);
}
