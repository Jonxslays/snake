use std::ops::Neg;

use bevy::core::FixedTimestep;
use bevy::prelude::*;
use rand::prelude::random;

const WIN_HEIGHT: f32 = 600.;
const WIN_WIDTH: f32 = WIN_HEIGHT + 100.;
const BG_COLOR: Color = Color::rgb(0.07, 0.07, 0.07);
const FOOD_COLOR: Color = Color::rgb(0.7, 0.0, 0.0);
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.0, 0.7, 0.0);
const SNAKE_BODY_COLOR: Color = Color::rgb(0.3, 0.3, 0.3);
const GRID_HEIGHT: u32 = 30;
const GRID_WIDTH: u32 = 35;
const FOOD_WIN_AMOUNT: u32 = 50;
const FALL_BEHIND_LOSS_AMOUNT: u32 = 15;

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Component)]
struct SnakePart;

fn spawn_snake_part(mut commands: Commands, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_BODY_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(SnakePart)
        .insert(position)
        .insert(Size::square(0.7))
        .insert(UiFixedZ(99.0))
        .id()
}

#[derive(Default, Component)]
struct SnakeBody(Vec<Entity>);

#[derive(Component)]
struct RenderedFood(u32);

#[derive(Component)]
struct DevouredFood(u32);

#[derive(Component, Debug, Clone)]
enum GameStatus {
    InProgress,
    Won,
    Lost,
}

#[derive(Bundle)]
struct GameState {
    status: GameStatus,
    devoured: DevouredFood,
    rendered: RenderedFood,
}

impl GameState {
    fn new() -> Self {
        Self {
            status: GameStatus::InProgress,
            devoured: DevouredFood(0),
            rendered: RenderedFood(0),
        }
    }
}

fn setup_game_state(mut commands: Commands) {
    commands.spawn_bundle(GameState::new());
}

#[derive(Component)]
struct ScoreText;

fn setup_score_text(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "Score: 0",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            transform: Transform {
                translation: Vec3::new(0.0, 275.0, 0.0),
                ..default()
            },
            ..default()
        })
        .insert(ScoreText)
        .insert(UiFixedZ(101.0));
}

fn score_update_system(
    mut text_query: Query<&mut Text, With<ScoreText>>,
    devoured: Query<&DevouredFood>,
) {
    for mut text in text_query.iter_mut() {
        if let Some(count) = devoured.iter().next() {
            text.sections[0].value = format!("Score: {}", count.0);
        }
    }
}

fn update_game_status(mut query: Query<(&mut GameStatus, &RenderedFood, &DevouredFood)>) {
    if let Some((mut status, rendered, devoured)) = query.iter_mut().next() {
        if devoured.0 >= FOOD_WIN_AMOUNT {
            *status = GameStatus::Won;
        } else if rendered.0 >= FALL_BEHIND_LOSS_AMOUNT {
            *status = GameStatus::Lost;
        }

        // println!("Rendered: {}", rendered.0);
        // println!("Devoured: {}", devoured.0);
        // println!("Status: {:?}", *status);
    }
}

#[derive(Component)]
struct UiFixedZ(f32);

fn ui_apply_fixed_z(mut query: Query<(&mut Transform, &mut GlobalTransform, &UiFixedZ)>) {
    for (mut transform, mut global_transform, fixed) in query.iter_mut() {
        transform.translation.z = fixed.0;
        global_transform.translation.z = fixed.0;
    }
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
}

impl Neg for Direction {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Component)]
struct Food;

fn food_spawner(
    mut commands: Commands,
    mut render_event: EventWriter<RenderFoodEvent>,
    query: Query<&GameStatus>,
) {
    let mut should_draw = true;
    if let Some(status) = query.iter().next() {
        match *status {
            GameStatus::InProgress => (),
            _ => should_draw = false,
        }
    }

    if !should_draw {
        return;
    }

    let get_random_pos = |bound: u32| (random::<f32>() * bound as f32) as i32;
    let x = get_random_pos(GRID_WIDTH);
    let y = get_random_pos(GRID_HEIGHT);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Food)
        .insert(Position::new(x, y))
        .insert(Size::square(0.8))
        .insert(UiFixedZ(1.0));

    render_event.send(RenderFoodEvent);
}

fn handle_render_event(
    mut rendered: Query<&mut RenderedFood>,
    mut render_event: EventReader<RenderFoodEvent>,
) {
    if render_event.iter().next().is_some() {
        if let Some(mut count) = rendered.iter_mut().next() {
            count.0 += 1;
        }
    }
}

struct GrowthEvent;
struct RenderFoodEvent;
struct GameOverEvent(GameStatus);

#[derive(Component)]
struct GameOverText;

fn show_end_game_text(mut commands: Commands, status: &GameStatus, asset_server: Res<AssetServer>) {
    let message: &str;
    let color: Color;

    match *status {
        GameStatus::Won => {
            message = "You won!";
            color = Color::GREEN;
        }
        GameStatus::Lost => {
            message = "You lost!";
            color = Color::RED;
        }
        _ => unreachable!(),
    }

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                message,
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 65.0,
                    color,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            ..default()
        })
        .insert(GameOverText)
        .insert(UiFixedZ(102.0));
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    segments_res: ResMut<SnakeBody>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeBody>>,
    asset_server: Res<AssetServer>,
) {
    if let Some(event) = reader.iter().next() {
        for ent in food.iter().chain(segments.iter()) {
            commands.entity(ent).despawn();
        }

        for part in segments_res.0.iter() {
            commands.entity(*part).despawn();
        }

        show_end_game_text(commands, &event.0, asset_server);
    }
}

#[derive(Component)]
struct SnakeHead {
    direction: Direction,
}

impl SnakeHead {
    pub fn new() -> Self {
        Self {
            direction: Direction::Up,
        }
    }
}

fn snake_eating(
    mut commands: Commands,
    mut growth_writer: EventWriter<GrowthEvent>,
    food_positions: Query<(Entity, &Position), With<Food>>,
    head_positions: Query<&Position, With<SnakeHead>>,
) {
    for head_pos in head_positions.iter() {
        for (ent, food_pos) in food_positions.iter() {
            if food_pos == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}

fn inc_and_dec(devoured: &mut Query<&mut DevouredFood>, rendered: &mut Query<&mut RenderedFood>) {
    if let Some(mut eaten) = devoured.iter_mut().next() {
        eaten.0 += 1;
    }

    if let Some(mut to_eat) = rendered.iter_mut().next() {
        to_eat.0 -= 1;
    }
}

fn snake_growth(
    commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut body: ResMut<SnakeBody>,
    mut growth_reader: EventReader<GrowthEvent>,
    mut devoured: Query<&mut DevouredFood>,
    mut rendered: Query<&mut RenderedFood>,
) {
    if growth_reader.iter().next().is_some() {
        body.0
            .push(spawn_snake_part(commands, last_tail_position.0.unwrap()));

        inc_and_dec(&mut devoured, &mut rendered);
    }
}

fn spawn_snake(mut commands: Commands, mut body: ResMut<SnakeBody>) {
    *body = SnakeBody(vec![
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: SNAKE_HEAD_COLOR,
                    ..default()
                },
                ..default()
            })
            .insert(SnakeHead::new())
            .insert(SnakePart)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .insert(UiFixedZ(100.0))
            .id(),
        spawn_snake_part(commands, Position { x: 3, y: 2 }),
    ]);
}

fn snake_movement_input(keyboard_input: Res<Input<KeyCode>>, mut heads: Query<&mut SnakeHead>) {
    if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            Direction::Right
        } else {
            head.direction
        };

        if dir != -head.direction {
            head.direction = dir;
        }
    }
}

fn snake_movement(
    body: ResMut<SnakeBody>,
    mut game_status: Query<&mut GameStatus>,
    mut heads: Query<(Entity, &SnakeHead)>,
    mut positions: Query<&mut Position>,
    mut last_tail_position: ResMut<LastTailPosition>,
    mut game_over_event: EventWriter<GameOverEvent>,
) {
    if let Some((head_entity, head)) = heads.iter_mut().next() {
        let body_positions = body
            .0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        if body_positions[1..].contains(&head_pos) {
            let mut status = game_status.iter_mut().next().unwrap();
            *status = GameStatus::Lost;
            game_over_event.send(GameOverEvent((*status).clone()));
        }

        match &head.direction {
            Direction::Left => {
                if head_pos.x <= 0 {
                    head_pos.x = GRID_WIDTH as i32;
                }

                head_pos.x -= 1;
            }
            Direction::Right => {
                if head_pos.x >= GRID_WIDTH as i32 - 1 {
                    head_pos.x = -1;
                }

                head_pos.x += 1;
            }
            Direction::Up => {
                if head_pos.y >= GRID_HEIGHT as i32 - 1 {
                    head_pos.y = -1;
                }

                head_pos.y += 1;
            }
            Direction::Down => {
                if head_pos.y <= 0 {
                    head_pos.y = GRID_HEIGHT as i32;
                }

                head_pos.y -= 1;
            }
        };

        body_positions
            .iter()
            .zip(body.0.iter().skip(1))
            .for_each(|(pos, segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });

        *last_tail_position = LastTailPosition(Some(*body_positions.last().unwrap()));
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();

    for (sprite_size, mut transform) in q.iter_mut() {
        transform.scale = Vec3::new(
            sprite_size.width / GRID_WIDTH as f32 * window.width() as f32,
            sprite_size.height / GRID_HEIGHT as f32 * window.height() as f32,
            1.0,
        );
    }
}

fn calculate_grid_position(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    let window = windows.get_primary().unwrap();

    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            calculate_grid_position(pos.x as f32, window.width() as f32, GRID_WIDTH as f32),
            calculate_grid_position(pos.y as f32, window.height() as f32, GRID_HEIGHT as f32),
            0.0,
        );
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Component)]
struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn main() {
    App::new()
        .add_startup_system(setup_camera)
        .add_startup_system(setup_score_text)
        .add_startup_system(food_spawner)
        .insert_resource(ClearColor(BG_COLOR))
        .insert_resource(LastTailPosition::default())
        .insert_resource(SnakeBody::default())
        .insert_resource(WindowDescriptor {
            height: WIN_HEIGHT,
            width: WIN_WIDTH,
            title: "Snake!".to_string(),
            resizable: false,
            ..default()
        })
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation)
                .with_system(size_scaling),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(3.0))
                .with_system(update_game_status.before(food_spawner))
                .with_system(food_spawner),
        )
        .add_system(handle_render_event.after(food_spawner))
        .add_system(snake_movement_input.before(snake_movement))
        .add_system(score_update_system.after(snake_movement))
        .add_system(game_over.after(snake_movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.10))
                .with_system(snake_movement)
                .with_system(snake_eating.after(snake_movement))
                .with_system(snake_growth.after(snake_eating)),
        )
        .add_startup_system(spawn_snake)
        .add_startup_system(setup_game_state)
        .add_system_to_stage(CoreStage::Last, ui_apply_fixed_z)
        .add_plugins(DefaultPlugins)
        .add_event::<GrowthEvent>()
        .add_event::<RenderFoodEvent>()
        .add_event::<GameOverEvent>()
        .run()
}
