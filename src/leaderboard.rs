use bevy::prelude::*;

use crate::{game_state::GameState, painting::Score};

pub struct LeaderBoardPlugin;
impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::LeaderBoard).with_system(setup_ui));
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, score: Res<Score>) {
    commands.spawn_bundle(TextBundle {
        text: Text::with_section(
            format!("Score: {}", score.0),
            TextStyle {
                font: asset_server.load("fonts/Archivo-Black.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.5, 0.5, 1.0),
            },
            Default::default(),
        ),
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
