use async_compat::Compat;
use bevy::log::info;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;
use graphql_client::{GraphQLQuery, Response};

use crate::{game_state::GameState, start_menu::create_drawings::DrawingsInput};

pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::StartMenu).with_system(setup_button))
            .add_system(button_hover_system)
            .add_system_set(
                SystemSet::on_update(GameState::StartMenu)
                    .with_system(handle_start_clicked),
            )
            .add_system_set(SystemSet::on_exit(GameState::StartMenu).with_system(despawn_button));
    }
}

// graphql query to write name to database
#[allow(non_camel_case_types)] // must match name in graphql file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_entry.graphql"
)]
struct createDrawings;


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

fn write_name_system(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    task_pool: ResMut<IoTaskPool>,
) {
    for (interaction, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        if *interaction == Interaction::Clicked {
            text.sections[0].value = "Press".to_string();
            let task = task_pool.spawn(Compat::new(async {
                let request_body = createDrawings::build_query(create_drawings::Variables {
                    new_drawing: DrawingsInput {
                        name: "test_user".to_string(),
                        score: None,
                        brush: None,
                        shape: None,
                        drawing: None,
                    },
                });

                const FAUNA_API_TOKEN: &str = env!("UNFAIR_ADVANTAGE_PUBLIC_FAUNA_CLIENT_KEY");

                let client = reqwest::Client::builder()
                    .default_headers(
                        std::iter::once((
                            reqwest::header::AUTHORIZATION,
                            reqwest::header::HeaderValue::from_str(&format!(
                                "Bearer {}",
                                FAUNA_API_TOKEN,
                            ))
                            .unwrap(),
                        ))
                        .collect(),
                    )
                    .build()
                    .unwrap();
                let res = client
                    .post("https://graphql.fauna.com/graphql")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?;
                let response_body: Response<create_drawings::ResponseData> =
                    res.json().await.map_err(|e| e.to_string())?;
                if let Some(errors) = response_body.errors {
                    return Err(errors[0].to_string());
                }
                Ok(())
            }));
            task.detach();
        }
    }
}
