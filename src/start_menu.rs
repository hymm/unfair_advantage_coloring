use async_compat::Compat;
use futures_lite::future;
use bevy::log::info;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use graphql_client::{GraphQLQuery, Response};

use crate::{game_state::GameState, start_menu::create_drawings::DrawingsInput};

pub struct StartMenuPlugin;
impl Plugin for StartMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::StartMenu).with_system(setup_button))
            .add_system_set(
                SystemSet::on_update(GameState::StartMenu)
                    .with_system(button_system)
                    .with_system(create_drawing_task),
            );
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

#[derive(Component)]
struct CreateDrawingsTask(pub Task<Result<(), String>>);

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Button",
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

fn button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    task_pool: ResMut<IoTaskPool>,
) {
    for (interaction, mut color, children) in interaction_query.iter_mut() {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Clicked => {
                text.sections[0].value = "Press".to_string();
                let task = task_pool.spawn(Compat::new(async {
                    let request_body = createDrawings::build_query(create_drawings::Variables {
                        new_drawing: Some(DrawingsInput {
                            name: "test_user".to_string(),
                            score: None,
                            brush: None,
                            shape: None,
                            drawing: None,
                        }),
                    });
                    let client = reqwest::Client::new();
                    let res = client
                        .post("https://graphql.fauna.com/graphql")
                        .json(&request_body)
                        .send()
                        .await.map_err(|e| e.to_string())?;
                    let response_body: Response<create_drawings::ResponseData> =
                        res.json().await.map_err(|e| e.to_string())?;
                    if let Some(errors) = response_body.errors {
                        return Err(errors[0].to_string());
                    }
                    Ok(())
                }));

                commands.spawn().insert(CreateDrawingsTask(task));
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn create_drawing_task(mut commands: Commands, mut q: Query<(Entity, &mut CreateDrawingsTask)>) {
    for (e, mut task) in q.iter_mut() {
        let status = future::block_on(future::poll_once(&mut task.0));
        if let Some(res) = status {
            match res {
                Ok(_) => {},
                Err(e) => info!("{}", e),
            }
            commands.entity(e).despawn();
        }
    }
}
