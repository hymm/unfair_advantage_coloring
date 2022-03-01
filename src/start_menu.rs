use bevy::prelude::*;

use crate::game_state::GameState;

pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::StartMenu).with_system(setup_button))
            .add_system(button_hover_system)
            .add_system_set(
                SystemSet::on_update(GameState::StartMenu).with_system(handle_start_clicked),
            )
            .add_system_set(SystemSet::on_exit(GameState::StartMenu).with_system(despawn_button));
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct StartButton;

fn setup_button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
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
        .insert(StartButton)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
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
            });
        });
}

fn despawn_button(mut commands: Commands, query: Query<Entity, With<StartButton>>) {
    let e = query.single();
    commands.entity(e).despawn_recursive();
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
