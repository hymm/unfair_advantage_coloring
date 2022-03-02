use async_channel::{Receiver, Sender};
use bevy::prelude::{info, App, Commands, Plugin};
use graphql_client::{GraphQLQuery, reqwest::post_graphql};
use std::future::Future;

use crate::comm::create_drawings::DrawingsInput;

use self::create_drawings::ResponseData;

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
                    tokio::task::spawn_local(future).await.unwrap();
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
}

fn setup_comm(mut commands: Commands) {
    let (result_req_tx, result_req_rx) = async_channel::bounded(1);
    let (result_res_tx, result_res_rx) = async_channel::bounded(1);
    run_async(async move {
        post_result_task(result_req_rx, result_res_tx).await;
    });

    commands.insert_resource(CommChannels {
        result_req_tx,
        result_res_rx,
    });
}

// graphql query to write name to database
#[allow(non_camel_case_types)] // must match name in graphql file
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/schema.graphql",
    query_path = "graphql/create_entry.graphql",
    response_derives = "Debug"
)]
pub struct createDrawings;

unsafe impl Send for DrawingsInput {}
unsafe impl Send for ResponseData {}

async fn post_result_task(
    result_req_rx: Receiver<DrawingsInput>,
    result_res_tx: Sender<Result<create_drawings::CreateDrawingsCreateDrawings, String>>,
) {
    info!("sending");
    let result = async move {
        let new_drawing = result_req_rx.recv().await.unwrap();
        let variables = create_drawings::Variables { new_drawing };

        const FAUNA_API_TOKEN: &str = env!("UNFAIR_ADVANTAGE_PUBLIC_FAUNA_CLIENT_KEY");

        let client = reqwest::Client::builder()
            .default_headers(
                std::iter::once((
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&format!("Bearer {}", FAUNA_API_TOKEN,))
                        .unwrap(),
                ))
                .collect(),
            )
            .build()
            .unwrap();

        let res = post_graphql::<createDrawings, _>(&client, "https://graphql.fauna.com/graphql", variables)
            .await
            .map_err(|e| e.to_string())?;

        info!("sent");
        Ok(res.data.unwrap().create_drawings)
    }
    .await;

    result_res_tx.send(result).await.unwrap();
}
