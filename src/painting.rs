use bevy::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes};

use crate::game_state::GameState;

pub struct PaintingPlugin;
impl Plugin for PaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_system_set(SystemSet::on_enter(GameState::Painting).with_system(setup_brush))
            .add_system_set(SystemSet::on_update(GameState::Painting).with_system(track_cursor));
    }
}

#[derive(Component)]
struct Paintbrush;

fn setup_brush(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Ellipse {
                radii: Vec2::new(100.0, 100.0),
                center: Vec2::new(0.0, 0.0),
            },
            DrawMode::Fill(FillMode::color(Color::rgb_u8(200, 140, 50))),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .insert(Paintbrush);
}

fn track_cursor(
    mut cursor_pos: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut brush: Query<&mut Transform, With<Paintbrush>>,
) {
  let window = windows.get_primary().unwrap();
  for position in cursor_pos.iter() {
    let size = Vec2::new(window.width() as f32, window.height() as f32);
    let mut t = brush.single_mut();
    t.translation = (position.position - size / 2.0).extend(0.0);
  }
}
