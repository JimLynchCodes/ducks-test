use bevy::prelude::*;

use super::websocket_connect::UpdateYourScoreBevyEvent;

#[derive(Component)]
struct ScoreText;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, create_score_text);
    app.add_systems(Update, bevy_event_listener_update_your_score_text);
}

fn create_score_text(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {

    println!("creating Score text...");

    let text = TextBundle {
        style: Style {
            position_type: PositionType::Absolute, // Absolute positioning
            right: Val::Percent(5.),
            top: Val::Percent(5.),
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