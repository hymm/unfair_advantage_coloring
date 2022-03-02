use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    comm::{create_drawings::DrawingsInput, CommChannels},
    constants::NORMAL_BUTTON,
    game_state::GameState,
    painting::Score,
};

pub struct ResultsPlugin;
impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(GameState::Results))
            .add_system_set(
                SystemSet::on_update(GameState::Results)
                    .with_system(egui_ui),
            )
            .insert_resource(UserNick::default());
    }
}

#[derive(Component)]
struct SendButton;

#[derive(Default)]
struct UserNick(pub String);

fn egui_ui(
    mut egui_ctx: ResMut<EguiContext>,
    mut user_nick: ResMut<UserNick>,
    comm_channels: ResMut<CommChannels>,
    score: Res<Score>,
) {
    egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("Nickname: ");
            ui.text_edit_singleline(&mut user_nick.0);
            if ui.button("Send Result").clicked() {
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
            };
        });
    });
}
