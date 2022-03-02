#![allow(clippy::type_complexity)]
// disable console opening on windows
// #![windows_subsystem = "windows"]

use bevy::prelude::*;
mod comm;
mod constants;
mod game_state;
mod leaderboard;
mod painting;
mod start_menu;

use crate::game_state::GameState;

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Unfair Painting Competition".to_string(),
            width: 600.0,
            height: 800.0,
            resizable: false,
            ..Default::default()
        })
        .add_state(GameState::Loading)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_plugin(crate::start_menu::StartMenuPlugin)
        .add_plugin(crate::painting::PaintingPlugin)
        .add_plugin(crate::leaderboard::LeaderBoardPlugin)
        .add_plugin(crate::comm::CommPlugin)
        .add_startup_system(finish_loading)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// stub function to move out of loading state
fn finish_loading(mut state: ResMut<State<GameState>>) {
    state.set(GameState::StartMenu).unwrap();
}
