use bevy::prelude::*;

use super::websocket_connect::UpdateYourScoreBevyEvent;

#[derive(Component)]
struct ScoreText;

const TABLE_OFFSET: u32 = 30;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, create_score_text);
    app.add_systems(Startup, setup_leaderboard_table);
    app.add_systems(Update, bevy_event_listener_update_your_score_text);
}

fn create_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("creating Score text...");

    let text = TextBundle {
        style: Style {
            position_type: PositionType::Absolute, // Absolute positioning
            left: Val::Percent(3.),
            top: Val::Percent(3.),
            ..Default::default()
        },
        text: Text::from_section(
            "Score: 0".to_string(),
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"), // Load your font here
                font_size: 25.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    };

    commands.spawn(text).insert(ScoreText);
}

fn bevy_event_listener_update_your_score_text(
    mut event_reader: EventReader<UpdateYourScoreBevyEvent>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    for e in event_reader.read() {
        info!("heard the update score event!");

        for mut text in score_text.iter_mut() {
            text.sections[0].value = format!("Score: {}", e.new_score);
        }
    }
}

struct Score {
    name: String,
    value: u32,
}

fn setup_leaderboard_table(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Sample data for the table
    let scores = vec![
        Score {
            name: "Player 1".to_string(),
            value: 100,
        },
        Score {
            name: "Player 2".to_string(),
            value: 150,
        },
        Score {
            name: "Player 3".to_string(),
            value: 120,
        },
    ];

    // Create a parent node for the table
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                // align_items: AlignItems::Center,
                // justify_content: JustifyContent::FlexStart,
                position_type: PositionType::Absolute, // Absolute positioning
                row_gap: Val::Px(7.),
                // column_gap: Val::Px(50.),
                right: Val::Percent(3.),
                top: Val::Percent(3.),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            // Header Row
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        flex_grow: 1.,
                        // min_width: Val::Px(400.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Leaderboard",
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 25.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..Default::default()
                    });
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "",
                            TextStyle {
                                font: asset_server.load("FiraSans-Bold.ttf"),
                                font_size: 25.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..Default::default()
                    });
                });

            // Create rows for each score
            let y_offset = 30.0; // Adjust this value for the desired spacing
            for score in scores {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                format!("{}    ", &score.name), // super hacky way to add right padding... 😅
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..Default::default()
                        });
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                score.value.to_string(),
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..Default::default()
                        });
                    });
            }
        });
}
