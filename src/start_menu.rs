use bevy::prelude::*;

use crate::{
    constants::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON},
    game_state::GameState,
};

pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::StartMenu)
                .with_system(setup_button)
                .with_system(setup_splash_image),
        )
        .add_system(button_hover_system)
        .add_system_set(
            SystemSet::on_update(GameState::StartMenu)
                .with_system(handle_start_clicked)
                .with_system(handle_leaderboard_clicked),
        )
        .add_system_set(SystemSet::on_exit(GameState::StartMenu).with_system(despawn_button));
    }
}

#[derive(Component)]
struct StartMenuScene;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct LeaderboardButton;

fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(65.0)),
                // center button
                margin: Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Px(26.0),
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(StartButton)
        .insert(StartMenuScene)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Start",
                        TextStyle {
                            font: asset_server.load("fonts/Archivo-Black.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(StartMenuScene);
        });

    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(300.0), Val::Px(65.0)),
                // center button
                margin: Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Px(26.0),
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(LeaderboardButton)
        .insert(StartMenuScene)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Leaderboard",
                        TextStyle {
                            font: asset_server.load("fonts/Archivo-Black.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(StartMenuScene);
        });
}

fn setup_splash_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/Unfair_Start_Screen.png"),
            ..SpriteBundle::default()
        })
        .insert(StartMenuScene);
}

fn despawn_button(mut commands: Commands, query: Query<Entity, With<StartMenuScene>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn handle_start_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    mut state: ResMut<State<GameState>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            state.set(GameState::Painting).unwrap();
            mouse_button.clear();
        }
    }
}

fn handle_leaderboard_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<LeaderboardButton>)>,
    mut state: ResMut<State<GameState>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            state.set(GameState::LeaderBoard).unwrap();
            mouse_button.clear();
        }
    }
}
