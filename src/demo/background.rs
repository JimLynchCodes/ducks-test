//! Plugin handling the player character in particular.
//! Note that this is separate from the `movement` module as that could be used
//! for other characters as well.

use bevy::prelude::*;

// #[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
// #[reflect(Component)]

// #[derive(Asset)]
// pub enum Background {
//     grass,
// };

pub(super) fn plugin(app: &mut App) {
    // app.register_type::<Background>();
    app.add_systems(Startup, create_background);

    // app.register_type::<Player>();
    // app.load_resource::<PlayerAssets>();

    // app.add_plugins(VirtualJoystickPlugin::<String>::default());
    // // Record directional input as movement controls.
    // app.add_systems(Startup, create_joystick_scene);

    // // Handles input from Keyboard AND Joystick as movement controls.
    // app.add_systems(Update, handle_joystick_or_keyboard_input);
}

fn create_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("images/ducks-bg-test.png");

    println!("creating background...");

    // Spawn an entity with the background image as a sprite
    commands.spawn(SpriteBundle {
        texture: texture_handle,
        transform: Transform {
            scale: Vec3::new(1.0, 1.0, 0.0), // Adjust scale if necessary,
            translation: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

// fn create_background(
//     commands: Commands,
//     asset_server: Res<AssetServer>,
//     texture_atlases: ResMut<Assets<TextureAtlas>>,
// ) {
//     let texture_handle = asset_server.load("grass.png");

//     // Spawn a sprite with the background image
//     commands.spawn_bundle(SpriteBundle {
//         texture: texture_handle,
//         transform: Transform {
//             scale: Vec3::new(1.0, 1.0, 0.0), // Adjust scale if needed
//             ..Default::default()
//         },
//         ..Default::default()
//     });

// }
