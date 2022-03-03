use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    comm::{create_drawings::DrawingsInput, CommChannels},
    game_state::GameState,
    painting::{PaintbrushImageHandle, Score},
};

pub struct ResultsPlugin;
impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Results).with_system(set_egui_image))
            .add_system_set(
                SystemSet::on_update(GameState::Results)
                    .with_system(egui_ui)
                    .with_system(check_done),
            )
            .insert_resource(UserNick::default())
            .insert_resource(ResultCommStatus::Waiting);
    }
}

#[derive(Component)]
struct SendButton;

#[derive(Default)]
struct UserNick(pub String);

#[derive(PartialEq, Eq, Clone)]
enum ResultCommStatus {
    Waiting,
    Sending,
    Done,
    Error(String),
}

fn egui_ui(
    mut egui_ctx: ResMut<EguiContext>,
    mut user_nick: ResMut<UserNick>,
    comm_channels: ResMut<CommChannels>,
    mut result_comm_status: ResMut<ResultCommStatus>,
    score: Res<Score>,
) {
    egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Nickname: ");
            ui.text_edit_singleline(&mut user_nick.0);

            if ui.add(egui::Button::new("Send Result")).clicked() {
                comm_channels
                    .result_req_tx
                    .try_send(DrawingsInput {
                        name: user_nick.0.clone(),
                        score: Some(score.0),
                        brush: None,
                        shape: None,
                        drawing: None,
                    })
                    .unwrap();
                *result_comm_status = ResultCommStatus::Sending;
            };
        });

        if let ResultCommStatus::Error(e) = result_comm_status.clone() {
            ui.horizontal(|ui| {
                ui.label(e);
            });
        }

        ui.vertical_centered(|ui| {
            ui.image(egui::TextureId::User(0), [50., 50.]);
        });
    });
}

fn check_done(
    comm_channels: ResMut<CommChannels>,
    mut result_comm_status: ResMut<ResultCommStatus>,
    mut state: ResMut<State<GameState>>,
) {
    if let Ok(res) = comm_channels.result_res_rx.try_recv() {
        if let Err(e) = res {
            *result_comm_status = ResultCommStatus::Error(e);
        } else {
            *result_comm_status = ResultCommStatus::Done;
            state.set(GameState::LeaderBoard).unwrap();
        }
    }
}

fn set_egui_image(handle: Res<PaintbrushImageHandle>, mut egui_context: ResMut<EguiContext>) {
    egui_context.set_egui_texture(0, handle.0.clone());
}
