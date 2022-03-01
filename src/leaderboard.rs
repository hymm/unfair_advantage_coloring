use async_compat::Compat;
use bevy::{prelude::*, tasks::IoTaskPool};
use graphql_client::{GraphQLQuery, Response};

use crate::{
    constants::NORMAL_BUTTON, game_state::GameState, leaderboard::create_drawings::DrawingsInput,
    painting::Score,
};

pub struct LeaderBoardPlugin;
impl Plugin for LeaderBoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::LeaderBoard).with_system(setup_ui))
            .add_system_set(
                SystemSet::on_update(GameState::LeaderBoard).with_system(write_name_system),
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
            let task = task_pool.spawn(Compat::new(async {
                info!("sending");
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
                info!("sent");
                Ok(())
            }));
            task.detach();
        }
    }
}
