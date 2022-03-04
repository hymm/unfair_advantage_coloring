use async_channel::{Receiver, Sender};
use bevy::prelude::{App, Commands, Plugin};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use std::future::Future;

use crate::comm::create_drawings::DrawingsInput;

pub struct CommPlugin;
impl Plugin for CommPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_comm);
    }
}

// taken from https://github.com/mvlabat/muddle-run/blob/507a95a9e728be2723536ee84031f05e490fcb4b/libs/client_lib/src/net/mod.rs#L136-L220
#[cfg(not(target_arch = "wasm32"))]
pub fn run_async<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Cannot start tokio runtime");

        rt.block_on(async move {
            let local = tokio::task::LocalSet::new();
            local
                .run_until(async move {
                    tokio::task::spawn_local(future)
                        .await
                        .unwrap_or_else(|e| println!("{}", e));
                })
                .await;
        });
    });
}

#[cfg(target_arch = "wasm32")]
pub fn run_async<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(async move {
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                tokio::task::spawn_local(future).await.unwrap();
            })
            .await;
    });
}

pub struct CommChannels {
    pub result_req_tx: Sender<DrawingsInput>,
    pub result_res_rx: Receiver<Result<create_drawings::CreateDrawingsCreateDrawings, String>>,
    pub all_drawings_req_tx: Sender<()>,
    pub all_drawings_res_rx: Receiver<Result<all_drawings::AllDrawingsAllDrawings, String>>,
}

fn setup_comm(mut commands: Commands) {
    let (result_req_tx, result_req_rx) = async_channel::bounded(1);
    let (result_res_tx, result_res_rx) = async_channel::bounded(1);
    run_async(async move {
        post_result_task(result_req_rx, result_res_tx).await;
    });

    let (all_drawings_req_tx, all_drawings_req_rx) = async_channel::bounded(1);
    let (all_drawings_res_tx, all_drawings_res_rx) = async_channel::bounded(1);

    run_async(async move { get_drawings_task(all_drawings_req_rx, all_drawings_res_tx).await });

    commands.insert_resource(CommChannels {
        result_req_tx,
        result_res_rx,
        all_drawings_req_tx,
        all_drawings_res_rx,
    });
}

// graphql query to write name to database
#[allow(non_camel_case_types)] // must match name in graphql file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_entry.graphql"
)]
pub struct createDrawings;

async fn post_result_task(
    result_req_rx: Receiver<DrawingsInput>,
    result_res_tx: Sender<Result<create_drawings::CreateDrawingsCreateDrawings, String>>,
) {
    while let Ok(new_drawing) = result_req_rx.recv().await {
        let result = async move {
            let variables = create_drawings::Variables { new_drawing };

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

            let res = post_graphql::<createDrawings, _>(
                &client,
                "https://graphql.fauna.com/graphql",
                variables,
            )
            .await
            .map_err(|e| e.to_string())?;

            if let Some(errors) = res.errors {
                return Err(errors[0].to_string());
            }

            Ok(res.data.unwrap().create_drawings)
        }
        .await;

        result_res_tx.send(result).await.unwrap();
    }
}

#[allow(non_camel_case_types)] // must match name in graphql file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/all_drawings.graphql"
)]
pub struct allDrawings;

async fn get_drawings_task(
    all_drawings_req_rx: Receiver<()>,
    all_drawing_res_tx: Sender<Result<all_drawings::AllDrawingsAllDrawings, String>>,
) {
    while all_drawings_req_rx.recv().await.is_ok() {
        let result = async move {
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

            let res = post_graphql::<allDrawings, _>(
                &client,
                "https://graphql.fauna.com/graphql",
                all_drawings::Variables {
                    size: 1000
                },
            )
            .await
            .map_err(|e| e.to_string())?;

            if let Some(errors) = res.errors {
                return Err(errors[0].to_string());
            }

            Ok(res.data.unwrap().all_drawings)
        }
        .await;

        all_drawing_res_tx.send(result).await.unwrap();
    }
}
