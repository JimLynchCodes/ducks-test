use bevy::prelude::*;

#[derive(Component)]
struct _CrackerComponent;

#[derive(Component)]
struct _CrackerText;

pub(super) fn plugin(_app: &mut App) {

    // WIP - cracker stuff
    // _app.add_systems(Startup, _create_cracker);
    // _app.add_systems(Startup, _create_cracker_text);
}

fn _create_cracker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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

fn _create_cracker_text(mut commands: Commands, asset_server: Res<AssetServer>,) {

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
                translation: Vec3::new(0.0, 25.0, 5.0), // Position the text above the crackers & in front of the background
                scale: Vec3::new(1., 1., 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(_CrackerText);
}
