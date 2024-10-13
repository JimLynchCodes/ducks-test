use bevy::prelude::*;
use serde::Deserialize;

use super::websocket_connect::{UpdateLeaderboardBevyEvent, UpdateYourScoreBevyEvent};

#[derive(Component)]
struct ErrorTextComponent;

#[derive(Component)]
struct YourScoreText;

#[derive(Component)]
struct YourPositionText;

#[derive(Component)]
struct YourLeaderboardPlaceText;

#[derive(Component)]
struct LeaderboardName1stPlaceText;
#[derive(Component)]
struct LeaderboardName2ndPlaceText;
#[derive(Component)]
struct LeaderboardName3rdPlaceText;
#[derive(Component)]
struct LeaderboardName4thPlaceText;
#[derive(Component)]
struct LeaderboardName5thPlaceText;

#[derive(Component)]
struct LeaderboardScore1stPlaceText;
#[derive(Component)]
struct LeaderboardScore2ndPlaceText;
#[derive(Component)]
struct LeaderboardScore3rdPlaceText;
#[derive(Component)]
struct LeaderboardScore4thPlaceText;
#[derive(Component)]
struct LeaderboardScore5thPlaceText;

#[derive(Debug, Deserialize)]
pub struct LeaderboardUpdateData {
    pub your_points: u64,
    pub your_leaderboard_place: u64,

    pub leaderboard_name_1st_place: String,
    pub leaderboard_name_2nd_place: String,
    pub leaderboard_name_3rd_place: String,
    pub leaderboard_name_4th_place: String,
    pub leaderboard_name_5th_place: String,

    pub leaderboard_score_1st_place: u64,
    pub leaderboard_score_2nd_place: u64,
    pub leaderboard_score_3rd_place: u64,
    pub leaderboard_score_4th_place: u64,
    pub leaderboard_score_5th_place: u64,
}

const TABLE_OFFSET: u32 = 30;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, create_score_text);
    app.add_systems(Startup, setup_leaderboard_table);
    app.add_systems(Update, bevy_event_listener_update_your_score_text);
    app.add_systems(Update, bevy_event_listener_update_leaderboard);
}

fn create_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    println!("creating Score text...");

    let score_text = TextBundle {
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


    let position_text = TextBundle {
        style: Style {
            position_type: PositionType::Absolute, // Absolute positioning
            left: Val::Percent(3.),
            top: Val::Percent(6.),
            ..Default::default()
        },
        text: Text::from_section(
            "Position: --".to_string(),
            TextStyle {
                font: asset_server.load("FiraSans-Bold.ttf"), // Load your font here
                font_size: 25.0,
                color: Color::WHITE,
            },
        ),
        ..Default::default()
    };

    commands.spawn(score_text).insert(YourScoreText);
    commands.spawn(position_text).insert(YourPositionText);
}

fn bevy_event_listener_update_your_score_text(
    mut event_reader: EventReader<UpdateYourScoreBevyEvent>,
    mut score_text: Query<&mut Text, With<YourScoreText>>,
) {
    for e in event_reader.read() {
        info!("heard the update score event!");

        for mut text in score_text.iter_mut() {
            text.sections[0].value = format!("Score: {}", e.new_score);
        }
    }
}

fn bevy_event_listener_update_leaderboard(
    mut event_reader: EventReader<UpdateLeaderboardBevyEvent>,
    // mut score_text: Query<&mut Text, With<YourScoreText>>,

    mut your_position_text: Query<&mut Text, With<YourPositionText>>,
    mut leaderboard_name_1st_place_text: Query<&mut Text, With<LeaderboardName1stPlaceText>>,
    mut leaderboard_name_2nd_place_text: Query<&mut Text, With<LeaderboardName2ndPlaceText>>,
    mut leaderboard_name_3rd_place_text: Query<&mut Text, With<LeaderboardName3rdPlaceText>>,
    mut leaderboard_name_4th_place_text: Query<&mut Text, With<LeaderboardName4thPlaceText>>,
    mut leaderboard_name_5th_place_text: Query<&mut Text, With<LeaderboardName5thPlaceText>>,
    mut leaderboard_score_1st_place_text: Query<&mut Text, With<LeaderboardScore1stPlaceText>>,
    mut leaderboard_score_2nd_place_text: Query<&mut Text, With<LeaderboardScore2ndPlaceText>>,
    mut leaderboard_score_3rd_place_text: Query<&mut Text, With<LeaderboardScore3rdPlaceText>>,
    mut leaderboard_score_4th_place_text: Query<&mut Text, With<LeaderboardScore4thPlaceText>>,
    mut leaderboard_score_5th_place_text: Query<&mut Text, With<LeaderboardScore5thPlaceText>>,
) {
    for e in event_reader.read() {
    
        info!("heard the update score event!");

        let default_leaderboard_data = LeaderboardUpdateData {
            your_points: 0,
            your_leaderboard_place: 0,

            leaderboard_name_1st_place: "--".to_string(),
            leaderboard_name_2nd_place: "--".to_string(),
            leaderboard_name_3rd_place: "--".to_string(),
            leaderboard_name_4th_place: "--".to_string(),
            leaderboard_name_5th_place: "--".to_string(),

            leaderboard_score_1st_place: 0,
            leaderboard_score_2nd_place: 0,
            leaderboard_score_3rd_place: 0,
            leaderboard_score_4th_place: 0,
            leaderboard_score_5th_place: 0,
        };

        let update_leaderboard_msg_data = serde_json::from_value(e.data.clone()).unwrap_or_else(|op| {
            info!("Failed to parse incoming websocket message: {}", op);
            default_leaderboard_data
        });

        // Update your position
        for mut text in your_position_text.iter_mut() {
            text.sections[0].value = format!("Position: {:?}", update_leaderboard_msg_data.your_leaderboard_place.clone());
        }

        // update the 5 name fields
        for mut text in leaderboard_name_1st_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_name_1st_place.clone();
        }

        for mut text in leaderboard_name_2nd_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_name_2nd_place.clone();
        }

        for mut text in leaderboard_name_3rd_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_name_3rd_place.clone();
        }

        for mut text in leaderboard_name_4th_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_name_4th_place.clone();
        }

        for mut text in leaderboard_name_5th_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_name_5th_place.clone();

        // update the 5 score fields
        for mut text in leaderboard_score_1st_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_score_1st_place.to_string();
        }

        for mut text in leaderboard_score_2nd_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_score_2nd_place.to_string();
        }

        for mut text in leaderboard_score_3rd_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_score_3rd_place.to_string();
        }

        for mut text in leaderboard_score_4th_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_score_4th_place.to_string();
        }

        for mut text in leaderboard_score_5th_place_text.iter_mut() {
            text.sections[0].value = update_leaderboard_msg_data.leaderboard_score_5th_place.to_string();
        }
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
        Score {
            name: "Player 4".to_string(),
            value: 120,
        },
        Score {
            name: "Player 5".to_string(),
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
            for (index, score) in scores.into_iter().enumerate() {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        let leaderboard_name_text_bundle = TextBundle {
                            text: Text::from_section(
                                format!("{}    ", &score.name), // super hacky way to add right padding... 😅
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..Default::default()
                        };

                        match index {
                            0 => {
                                parent
                                    .spawn(leaderboard_name_text_bundle)
                                    .insert(LeaderboardName1stPlaceText);
                            }
                            1 => {
                                parent
                                    .spawn(leaderboard_name_text_bundle)
                                    .insert(LeaderboardName2ndPlaceText);
                            }
                            2 => {
                                parent
                                    .spawn(leaderboard_name_text_bundle)
                                    .insert(LeaderboardName3rdPlaceText);
                            }
                            3 => {
                                parent
                                    .spawn(leaderboard_name_text_bundle)
                                    .insert(LeaderboardName4thPlaceText);
                            }
                            4 => {
                                parent
                                    .spawn(leaderboard_name_text_bundle)
                                    .insert(LeaderboardName5thPlaceText);
                            }
                            _ => (),
                        }

                        let leaderboard_score_text_bundle = TextBundle {
                            text: Text::from_section(
                                score.value.to_string(),
                                TextStyle {
                                    font: asset_server.load("FiraSans-Bold.ttf"),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..Default::default()
                        };

                        match index {
                            0 => {
                                parent
                                    .spawn(leaderboard_score_text_bundle)
                                    .insert(LeaderboardScore1stPlaceText);
                            }
                            1 => {
                                parent
                                    .spawn(leaderboard_score_text_bundle)
                                    .insert(LeaderboardScore2ndPlaceText);
                            }
                            2 => {
                                parent
                                    .spawn(leaderboard_score_text_bundle)
                                    .insert(LeaderboardScore3rdPlaceText);
                            }
                            3 => {
                                parent
                                    .spawn(leaderboard_score_text_bundle)
                                    .insert(LeaderboardScore4thPlaceText);
                            }
                            4 => {
                                parent
                                    .spawn(leaderboard_score_text_bundle)
                                    .insert(LeaderboardScore5thPlaceText);
                            }
                            _ => (),
                        }
                    });
            }
        });
}
