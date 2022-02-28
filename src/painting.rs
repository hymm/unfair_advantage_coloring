use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_prototype_lyon::{prelude::*, shapes};

use crate::game_state::GameState;

pub struct PaintingPlugin;
impl Plugin for PaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ShapePlugin)
            .add_system_set(
                SystemSet::on_enter(GameState::Painting)
                    .with_system(setup_brush)
                    .with_system(setup_painting_area)
                    .with_system(setup_target_image)
                    .with_system(setup_score),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Painting)
                    .with_system(track_cursor.label("track_cursor"))
                    .with_system(paint.after("track_cursor"))
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Painting)
                .with_system(calculate_score)
            );
    }
}

const CANVAS_WIDTH: usize = 600;
const CANVAS_HEIGHT: usize = 600;

#[derive(Component)]
struct Paintbrush {
    extents: Vec2,
}

fn setup_brush(mut commands: Commands) {
    let extents = Vec2::new(50.0, 50.0);
    commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Rectangle {
                extents,
                origin: RectangleOrigin::Center,
            },
            DrawMode::Fill(FillMode::color(Color::rgb_u8(200, 140, 50))),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .insert(Paintbrush { extents });
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
        t.translation = (position.position - size / 2.0).extend(2.0);
    }
}

#[derive(Component)]
struct PaintingArea;

fn setup_painting_area(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let texture = images.add(Image::new_fill(
        Extent3d {
            width: CANVAS_WIDTH as u32,
            height: CANVAS_HEIGHT as u32,
            ..Default::default()
        },
        TextureDimension::D2,
        &[255, 255, 255, 0],
        TextureFormat::Rgba8Unorm,
    ));
    commands
        .spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(CANVAS_WIDTH as f32, CANVAS_HEIGHT as f32)),
                ..Sprite::default()
            },
            texture,
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            ..SpriteBundle::default()
        })
        .insert(PaintingArea);
}

#[derive(Component)]
struct TargetImage;

fn setup_target_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/test.png"),
            ..SpriteBundle::default()
        })
        .insert(TargetImage);
}

fn paint(
    q: Query<&Handle<Image>, With<PaintingArea>>,
    mut images: ResMut<Assets<Image>>,
    mouse_button: Res<Input<MouseButton>>,
    brush: Query<(&mut Transform, &Paintbrush)>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        // convert transform to top left of paint area
        let (t, Paintbrush { extents }) = brush.single();
        let cursor_pos = get_canvas_position_from_translation(t);
        let brush_top_left = cursor_pos - *extents / 2.0;
        // TODO: discard pixels outside of canvas
        let handle = q.single();
        let image = images.get_mut(handle).unwrap();

        for x in 0..(extents.x as u32) {
            for y in 0..(extents.y as u32) {
                let pos = brush_top_left + Vec2::new(x as f32, y as f32);
                color_pixel(image, pos);
            }
        }
    }
}

fn get_start_byte(x: usize, y: usize) -> usize {
    (y * CANVAS_WIDTH as usize + x) * 4
}

fn get_canvas_position_from_translation(t: &Transform) -> Vec2 {
    let mut canvas_pos =
        t.translation.truncate() + Vec2::new(CANVAS_WIDTH as f32, -(CANVAS_HEIGHT as f32)) / 2.0;
    canvas_pos.y = -canvas_pos.y;
    canvas_pos
}

fn color_pixel(image: &mut Image, pos: Vec2) {
    if pos.x < 0.0 || pos.y < 0.0 || pos.x >= CANVAS_WIDTH as f32 || pos.y >= CANVAS_HEIGHT as f32 {
        return;
    }
    let start_byte = get_start_byte(pos.x as usize, pos.y as usize);
    let new = [255, 0, 0, 255];
    let splice_range = start_byte..(start_byte + 4);
    image.data.splice(splice_range, new);
}

#[derive(Default)]
struct Score(pub f32);

fn setup_score(mut commands: Commands) {
    commands.insert_resource(Score(0.0));
}

fn calculate_score(
    target_image: Query<&Handle<Image>, With<TargetImage>>,
    player_image: Query<&Handle<Image>, With<PaintingArea>>,
    images: Res<Assets<Image>>,
    mut score: ResMut<Score>,
    mouse_button: Res<Input<MouseButton>>,
) {
    let target_image = images.get(target_image.single());
    if mouse_button.pressed(MouseButton::Left) || target_image.is_none() {
        return;
    }
    let target_image = target_image.unwrap();
    let player_image = images.get(player_image.single()).unwrap();

    let mut sum_good = 0;
    let mut sum_bad = 0;
    for x in 0..CANVAS_WIDTH {
        for y in 0..CANVAS_HEIGHT {
            let start_byte = get_start_byte(x, y);
            // use if not white
            let should_color = target_image.data[start_byte] != 255;
            // use alpha transparency
            let is_colored = player_image.data[start_byte + 3] == 255;
            if is_colored {
                if should_color {
                    sum_good += 1;
                } else {
                    sum_bad += 1;
                }
            }
        }
    }

    score.0 = (sum_good - sum_bad) as f32;
}
