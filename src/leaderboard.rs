use async_compat::Compat;
use bevy::{prelude::*, tasks::IoTaskPool};

use crate::{constants::NORMAL_BUTTON, game_state::GameState, painting::Score};

pub struct LeaderBoardPlugin;
impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::LeaderBoard).with_system(setup_ui))
            .add_system_set(
                SystemSet::on_update(GameState::LeaderBoard).with_system(write_name_system),
            );
    }
}

#[derive(Component)]
struct SendButton;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(SendButton)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Send",
                    TextStyle {
                        font: asset_server.load("fonts/Archivo-Black.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

fn write_name_system(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<SendButton>)>,
    task_pool: ResMut<IoTaskPool>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            let task = task_pool.spawn(Compat::new(async {}));
            task.detach();
        }
    }
}
