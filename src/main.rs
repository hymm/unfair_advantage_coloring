#![allow(clippy::type_complexity)]
// disable console opening on windows
// #![windows_subsystem = "windows"]

use bevy::{prelude::*, log::LogPlugin};
mod game_state;
mod start_menu;

use crate::game_state::GameState;

fn main() {
    App::new()
        .add_state(GameState::Loading)
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_plugin(crate::start_menu::StartMenuPlugin)
        .add_startup_system(finish_loading)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

// stub function to move out of loading state
fn finish_loading(mut state: ResMut<State<GameState>>) {
    state.push(GameState::StartMenu).unwrap();
}
