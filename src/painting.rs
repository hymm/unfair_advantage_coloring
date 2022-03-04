use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use bevy_prototype_lyon::{prelude::*, shapes};
use rand::{thread_rng, Rng};

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
                    .with_system(setup_score)
                    .with_system(setup_ui),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Painting)
                    .with_system(track_cursor.label("track_cursor"))
                    .with_system(paint.after("track_cursor"))
                    .with_system(handle_done_clicked)
                    .with_system(calculate_score),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::Painting)
                    .with_system(despawn_painting)
                    .with_system(generate_paintbrush_texture),
            );
    }
}

const CANVAS_WIDTH: usize = 600;
const CANVAS_HEIGHT: usize = 600;

#[derive(Component)]
struct PaintingScene;

#[derive(Component)]
struct Paintbrush {
    extents: Vec2,
}

#[derive(Component)]
struct BrushParent;

const BRUSH_RECT_MIN: f32 = 20.;
const BRUSH_RECT_MAX: f32 = 100.;
const BRUSH_MAX_OFFSET: f32 = 75.;

fn setup_brush(mut commands: Commands) {
    let mut rng = thread_rng();
    let parent_id = commands
        .spawn()
        .insert(BrushParent)
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .id();
    for _ in 0..3 {
        let width = rng.gen_range(BRUSH_RECT_MIN..BRUSH_RECT_MAX);
        let height = rng.gen_range(BRUSH_RECT_MIN..BRUSH_RECT_MAX);
        let extents = Vec2::new(width, height);

        let offset_x = rng.gen_range(-BRUSH_MAX_OFFSET..BRUSH_MAX_OFFSET);
        let offset_y = rng.gen_range(-BRUSH_MAX_OFFSET..BRUSH_MAX_OFFSET);

        commands.entity(parent_id).with_children(|parent| {
            parent
                .spawn_bundle(GeometryBuilder::build_as(
                    &shapes::Rectangle {
                        extents,
                        origin: RectangleOrigin::Center,
                    },
                    DrawMode::Fill(FillMode::color(Color::rgb_u8(200, 140, 50))),
                    Transform::from_xyz(offset_x, offset_y, 0.0),
                ))
                .insert(Paintbrush { extents })
                .insert(PaintingScene);
        });
    }
}

fn track_cursor(
    mut cursor_pos: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut brush: Query<&mut Transform, With<BrushParent>>,
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
        .insert(PaintingArea)
        .insert(PaintingScene);
}

#[derive(Component)]
struct DoneButton;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(200.0), Val::Px(50.0)),
                // center button
                margin: Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Auto,
                    bottom: Val::Percent(0.0),
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..Default::default()
        })
        .insert(DoneButton)
        .insert(PaintingScene)
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Done",
                        TextStyle {
                            font: asset_server.load("fonts/Archivo-Black.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                        Default::default(),
                    ),
                    ..Default::default()
                })
                .insert(PaintingScene);
        });
}

fn handle_done_clicked(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<DoneButton>)>,
    mut state: ResMut<State<GameState>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Clicked {
            state.set(GameState::Results).unwrap();
            mouse_button.clear();
        }
    }
}

#[derive(Component)]
struct TargetImage;

fn setup_target_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/Unfair_Duck-01.png"),
            ..SpriteBundle::default()
        })
        .insert(TargetImage)
        .insert(PaintingScene);
}

fn paint(
    q: Query<&Handle<Image>, With<PaintingArea>>,
    mut images: ResMut<Assets<Image>>,
    mouse_button: Res<Input<MouseButton>>,
    brush: Query<(&mut GlobalTransform, &Paintbrush)>,
    mut ready: Local<bool>,
) {
    if !*ready {
        if mouse_button.just_released(MouseButton::Left) {
            *ready = true;
        }
        return;
    }
    if mouse_button.pressed(MouseButton::Left) {
        for (t, Paintbrush { extents }) in brush.iter() {
            let cursor_pos = get_canvas_position_from_translation(t);
            let brush_top_left = cursor_pos - *extents / 2.0;
            let handle = q.single();
            let image = images.get_mut(handle).unwrap();

            paint_rect(extents, brush_top_left, image);
        }
    }
}

fn paint_rect(extents: &Vec2, top_left: Vec2, image: &mut Image) {
    for x in 0..(extents.x as u32) {
        for y in 0..(extents.y as u32) {
            let pos = top_left + Vec2::new(x as f32, y as f32);
            color_pixel(image, pos);
        }
    }
}

fn get_start_byte(x: usize, y: usize) -> usize {
    (y * CANVAS_WIDTH as usize + x) * 4
}

fn get_canvas_position_from_translation(t: &GlobalTransform) -> Vec2 {
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
pub struct Score(pub f64);

#[derive(Component)]
pub struct ScoreText;

fn setup_score(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Score(0.0));

    commands
        .spawn_bundle(TextBundle {
            text: Text::with_section(
                format!("Score: {:.1}", 0.0),
                TextStyle {
                    font: asset_server.load("fonts/Archivo-Black.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
                Default::default(),
            ),
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ScoreText)
        .insert(PaintingScene);
}

fn calculate_score(
    target_image: Query<&Handle<Image>, With<TargetImage>>,
    player_image: Query<&Handle<Image>, With<PaintingArea>>,
    images: Res<Assets<Image>>,
    mut score: ResMut<Score>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
    mut last_score: Local<Score>,
) {
    let target_image = images.get(target_image.single());
    if target_image.is_none() {
        return;
    }
    let target_image = target_image.unwrap();
    let player_image = images.get(player_image.single()).unwrap();

    let mut sum_good = 0;
    let mut sum_bad = 0;
    let mut max_score = 0;
    for x in 0..CANVAS_WIDTH {
        for y in 0..CANVAS_HEIGHT {
            let start_byte = get_start_byte(x, y);
            // use if not white
            let should_color = target_image.data[start_byte] != 255;
            if should_color {
                max_score += 1;
            }
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

    score.0 = ((sum_good - sum_bad) as f64 / max_score as f64) * 100.0;
    let mut score_text = score_text.single_mut();
    score_text.sections[0].value = format!("Score: {:.1}", score.0);

    if score.0 > last_score.0 {
        score_text.sections[0].style.color = Color::WHITE;
    } else if score.0 < last_score.0 {
        score_text.sections[0].style.color = Color::RED;
    }
    last_score.0 = score.0;
}

fn despawn_painting(mut commands: Commands, q: Query<Entity, With<PaintingScene>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}

pub struct PaintbrushImageHandle(pub Handle<Image>);

fn generate_paintbrush_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    brush: Query<(&mut Transform, &Paintbrush)>,
) {
    let d = BRUSH_RECT_MAX + 2. * BRUSH_MAX_OFFSET;
    let mut image = Image::new_fill(
        Extent3d {
            width: d as u32,
            height: d as u32,
            ..Default::default()
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8Unorm,
    );

    for (t, Paintbrush { extents }) in brush.iter() {
        let mut canvas_pos = t.translation.truncate() + Vec2::new(d, -d) / 2.0;
        canvas_pos.y = -canvas_pos.y;
        let brush_top_left = canvas_pos - *extents / 2.0;
        for x in 0..(extents.x as u32) {
            for y in 0..(extents.y as u32) {
                let pos = brush_top_left + Vec2::new(x as f32, y as f32);

                let start_byte = (pos.y as usize * d as usize + pos.x as usize) * 4;
                let new = [255, 0, 0, 255];
                let splice_range = start_byte..(start_byte + 4);
                image.data.splice(splice_range, new);
            }
        }
    }

    let handle = images.add(image);

    commands.insert_resource(PaintbrushImageHandle(handle));
}
