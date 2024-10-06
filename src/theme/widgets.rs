//! Helper traits for creating common widgets.

use bevy::{ecs::system::EntityCommands, prelude::*, ui::Val::*};
use bevy_simple_text_input::TextInputBundle;

use crate::theme::{interaction::InteractionPalette, palette::*};

// Component to hold the current text input
#[derive(Component)]
struct TextInput {
    value: String,
}

/// An extension trait for spawning UI widgets.
pub trait Widgets {
    /// Spawn a simple button with text.
    fn button(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple header label. Bigger than [`Widgets::label`].
    fn header(&mut self, text: impl Into<String>) -> EntityCommands;

    /// Spawn a simple text label.
    fn label(&mut self, text: impl Into<String>) -> EntityCommands;

    // Spawn a text input.
    // fn text_input(&mut self, text: impl Into<String>) -> EntityCommands;
}

impl<T: Spawn> Widgets for T {
    fn button(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Button"),
            ButtonBundle {
                style: Style {
                    width: Px(200.0),
                    height: Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(NODE_BACKGROUND),
                ..default()
            },
            InteractionPalette {
                none: NODE_BACKGROUND,
                hovered: BUTTON_HOVERED_BACKGROUND,
                pressed: BUTTON_PRESSED_BACKGROUND,
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Button Text"),
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: 40.0,
                        color: BUTTON_TEXT,
                        ..default()
                    },
                ),
            ));
        });

        entity
    }

    fn header(&mut self, text: impl Into<String>) -> EntityCommands {
        let mut entity = self.spawn((
            Name::new("Header"),
            NodeBundle {
                style: Style {
                    width: Px(500.0),
                    height: Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(NODE_BACKGROUND),
                ..default()
            },
        ));
        entity.with_children(|children| {
            children.spawn((
                Name::new("Header Text"),
                TextBundle::from_section(
                    text,
                    TextStyle {
                        font_size: 40.0,
                        color: HEADER_TEXT,
                        ..default()
                    },
                ),
            ));
        });
        entity
    }

    fn label(&mut self, text: impl Into<String>) -> EntityCommands {
        let entity = self.spawn((
            Name::new("Label"),
            TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 24.0,
                    color: LABEL_TEXT,
                    ..default()
                },
            )
            .with_style(Style {
                width: Px(500.0),
                ..default()
            }),
        ));
        entity
    }
}

// fn text_input(&mut self, text: impl Into<String>) -> EntityCommands {
//     let entity = self.spawn(

//         NodeBundle {
//             style: Style {
//                 // scale: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
//                 justify_content: JustifyContent::Center,
//                 align_items: AlignItems::Center,
//                 ..Default::default()
//             },
//             background_color: BackgroundColor(Color::rgb(0.2, 0.2, 0.2)),
//             ..Default::default()
//         })
//         .with_children(|parent| {
//             // Create a text input field
//             parent.spawn(TextBundle {
//                 text: Text::from_section(
//                     "Enter text: ".to_string(),
//                     TextStyle {
//                         // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                         font_size: 30.0,
//                         color: Color::WHITE,
//                         ..Default::default()
//                     },
//                 ),
//                 ..Default::default()
//             })
//             .insert(TextInput {
//                 value: String::new(),
//             });
//         });
// TextBundle {
// text: Text::with_section(
//     "Enter text: ".to_string(),
//     TextStyle {
//         font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//         font_size: 30.0,
//         color: Color::WHITE,
//     },
//     Default::default(),
// ),
// ..Default::default()
// })
// .insert(TextInput { value: String::new() });
// Name::new("Label"),
// TextBundle::from_section(
//     text,
//     TextStyle {
//         font_size: 24.0,
//         color: LABEL_TEXT,
//         ..default()
//     },
// )
// .with_style(Style {
//     width: Px(500.0),
//     ..default()
// }),
//         // ));
//         *entity
//     }
// }

/// An extension trait for spawning UI containers.
pub trait Containers {
    /// Spawns a root node that covers the full screen
    /// and centers its content horizontally and vertically.
    fn ui_root(&mut self) -> EntityCommands;
}

impl Containers for Commands<'_, '_> {
    fn ui_root(&mut self) -> EntityCommands {
        self.spawn((
            Name::new("UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
    }
}

/// An internal trait for types that can spawn entities.
/// This is here so that [`Widgets`] can be implemented on all types that
/// are able to spawn entities.
/// Ideally, this trait should be [part of Bevy itself](https://github.com/bevyengine/bevy/issues/14231).
trait Spawn {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands;
}

impl Spawn for Commands<'_, '_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}

impl Spawn for ChildBuilder<'_> {
    fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityCommands {
        self.spawn(bundle)
    }
}
