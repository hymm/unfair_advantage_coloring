use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use bevy_egui::{egui, EguiContext};
use std::cmp::PartialOrd;

use crate::{
    comm::{
        all_drawings::{self, AllDrawingsAllDrawingsData},
        CommChannels,
    },
    game_state::GameState,
};

pub struct LeaderboardPlugin;
impl Plugin for LeaderboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AllDrawings::default())
            .insert_resource(BrushHashmap(HashMap::default()))
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

fn egui_ui(
    mut egui_ctx: ResMut<EguiContext>,
    mut all: ResMut<AllDrawings>,
    brush_hashmap: Res<BrushHashmap>,
    mut state: ResMut<State<GameState>>,
) {
    egui::CentralPanel::default().show(egui_ctx.ctx_mut(), |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("leaderboard").show(ui, |ui| {
                ui.label("rank");
                ui.label("name");
                ui.label("score");
                ui.label("brush");
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
                        if let Some(image) = brush_hashmap.0.get(&result.name) {
                            ui.image(egui::TextureId::User(image.egui_id), [50., 50.]);
                        }
                        ui.end_row();
                    }
                    all.0 = Some(drawings);
                }
            });
        });
    });

    
    egui::SidePanel::right("side_panel")
        .default_width(100.)
        .show(egui_ctx.ctx_mut(), |ui| {
            if ui.button("Back to Start Menu").clicked() {
                state.set(GameState::StartMenu).unwrap();
            }
        });
}

fn start_poll_leaderboard(comm_channels: ResMut<CommChannels>) {
    comm_channels.all_drawings_req_tx.try_send(()).unwrap();
}

struct BrushEguiImage {
    #[allow(dead_code)] // handle is saved here to prevent unload
    handle: Handle<Image>,
    egui_id: u64,
}
struct BrushHashmap(pub HashMap<String, BrushEguiImage>);

fn check_poll_leaderboard(
    comm_channels: ResMut<CommChannels>,
    mut all: ResMut<AllDrawings>,
    mut images: ResMut<Assets<Image>>,
    mut egui_ctx: ResMut<EguiContext>,
    mut brush_hashmap: ResMut<BrushHashmap>,
) {
    if let Ok(result) = comm_channels.all_drawings_res_rx.try_recv() {
        match result {
            Ok(all_drawings) => {
                let mut temp: Vec<AllDrawingsAllDrawingsData> =
                    all_drawings.data.into_iter().flatten().collect();
                for (n, entry) in temp.iter_mut().enumerate() {
                    if let Some(brush) = entry.brush.take() {
                        let image = Image::new(
                            Extent3d {
                                width: 250,
                                height: 250,
                                ..Default::default()
                            },
                            TextureDimension::D2,
                            base64::decode(&brush).unwrap(),
                            TextureFormat::Rgba8Unorm,
                        );
                        let handle = images.add(image);
                        brush_hashmap.0.insert(
                            entry.name.clone(),
                            BrushEguiImage {
                                handle: handle.clone(),
                                egui_id: (n + 1) as u64,
                            },
                        );
                        egui_ctx.set_egui_texture((n + 1) as u64, handle);
                    }
                }
                all.0 = Some(temp);
            }
            Err(e) => info!("{}", e),
        }
    }
}
