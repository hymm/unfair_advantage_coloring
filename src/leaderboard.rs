use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use std::cmp::PartialOrd;

use crate::{
    comm::{all_drawings, CommChannels},
    game_state::GameState,
};

pub struct LeaderboardPlugin;
impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AllDrawings::default())
            .add_system_set(
                SystemSet::on_enter(GameState::LeaderBoard).with_system(start_poll_leaderboard),
            )
            .add_system_set(
                SystemSet::on_update(GameState::LeaderBoard)
                    .with_system(egui_ui)
                    .with_system(check_poll_leaderboard),
            );
    }
}

#[derive(Default)]
struct AllDrawings(pub Option<Vec<all_drawings::AllDrawingsAllDrawingsData>>);

fn egui_ui(mut egui_ctx: ResMut<EguiContext>, mut all: ResMut<AllDrawings>) {
    egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("leaderboard").show(ui, |ui| {
                ui.label("rank");
                ui.label("name");
                ui.label("score");
                ui.end_row();

                if let Some(mut drawings) = all.0.take() {
                    drawings.sort_by(|a, b| {
                        b.score
                            .unwrap()
                            .partial_cmp(&a.score.unwrap())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    for (rank, result) in drawings.iter().enumerate() {
                        ui.label(format!("{}", rank + 1));
                        ui.label(result.name.clone());
                        ui.label(format!("{:.1}", result.score.unwrap().clone()));
                        ui.end_row();
                    }
                    all.0 = Some(drawings);
                }
            });
        });
    });
}

fn start_poll_leaderboard(comm_channels: ResMut<CommChannels>) {
    comm_channels.all_drawings_req_tx.try_send(()).unwrap();
}

fn check_poll_leaderboard(comm_channels: ResMut<CommChannels>, mut all: ResMut<AllDrawings>) {
    if let Ok(result) = comm_channels.all_drawings_res_rx.try_recv() {
        match result {
            Ok(all_drawings) => all.0 = Some(all_drawings.data.into_iter().flatten().collect()),
            Err(e) => info!("{}", e),
        }
    }
}
