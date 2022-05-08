use bevy::{
    core::FixedTimestep,
    math::{const_vec2, const_vec3},
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};

const TIME_STEP: f32 = 1.0 / 60.0;

const PADDLE_SIZE: Vec3 = const_vec3!([20.0, 120.0, 0.0]);
const GAP_BETWEEN_PADDLE_AND_GOAL: f32 = 60.0;
const PADDLE_PADDING: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = const_vec3!([0.0, 0.0, 1.0]);
const BALL_SIZE: Vec3 = const_vec3!([30.0, 30.0, 0.0]);
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = const_vec2!([-0.5, 0.1]);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::BLACK;
const FOREGROUND_COLOR: Color = Color::WHITE;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard {
            p1_score: 0,
            p2_score: 0,
            fjongs: 0,
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(move_p1_paddle.before(check_for_collisions))
                .with_system(move_p2_paddle.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions)),
        )
        .add_system(update_p1_scoreboard)
        .add_system(update_p2_scoreboard)
        .run();
}

#[derive(Component)]
struct P1Paddle;

#[derive(Component)]
struct P2Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct P1Goal;

#[derive(Component)]
struct P2Goal;

#[derive(Component)]
struct P1GoalText;

#[derive(Component)]
struct P2GoalText;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Bundle)]
struct WallBundle {
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

enum WallLocation {
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_width = RIGHT_WALL - LEFT_WALL;

        match self {
            WallLocation::Bottom => Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS),
            WallLocation::Top => Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS),
        }
    }
}

impl WallBundle {
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: FOREGROUND_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

struct Scoreboard {
    p1_score: usize,
    p2_score: usize,
    fjongs: usize,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    let p1_paddle_x = LEFT_WALL + GAP_BETWEEN_PADDLE_AND_GOAL;
    let p2_paddle_x = RIGHT_WALL - GAP_BETWEEN_PADDLE_AND_GOAL;

    let arena_height = TOP_WALL - BOTTOM_WALL;
    // P1 paddle
    commands
        .spawn()
        .insert(P1Paddle)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(p1_paddle_x, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: FOREGROUND_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);
    //
    // P2 paddle
    commands
        .spawn()
        .insert(P2Paddle)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(p2_paddle_x, 0.0, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: FOREGROUND_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);

    // Ball
    commands
        .spawn()
        .insert(Ball)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                scale: BALL_SIZE,
                translation: BALL_STARTING_POSITION,
                ..default()
            },
            sprite: Sprite {
                color: FOREGROUND_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED));

    commands.spawn_bundle(WallBundle::new(WallLocation::Bottom));
    commands.spawn_bundle(WallBundle::new(WallLocation::Top));

    commands
        .spawn()
        .insert(P1Goal)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(LEFT_WALL, 0.0, 0.0),
                scale: Vec3::new(WALL_THICKNESS, arena_height + WALL_THICKNESS, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: BACKGROUND_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);

    commands
        .spawn()
        .insert(P2Goal)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(RIGHT_WALL, 0.0, 0.0),
                scale: Vec3::new(WALL_THICKNESS, arena_height + WALL_THICKNESS, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: BACKGROUND_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "P1 score: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    top: SCOREBOARD_TEXT_PADDING,
                    left: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(P1GoalText);

    commands
        .spawn_bundle(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "P2 score: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                ],
                ..default()
            },
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: SCOREBOARD_TEXT_PADDING,
                    right: SCOREBOARD_TEXT_PADDING,
                    ..default()
                },
                ..default()
            },
            ..default()
        })
        .insert(P2GoalText);
}

fn move_p1_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<P1Paddle>>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;
    if keyboard_input.pressed(KeyCode::A) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Q) {
        direction += 1.0;
    }

    let new_paddle_position = paddle_transform.translation.y + direction * PADDLE_SPEED * TIME_STEP;
    let top_bound = TOP_WALL - PADDLE_SIZE.y + PADDLE_PADDING;
    let bottom_bound = BOTTOM_WALL + PADDLE_SIZE.y - PADDLE_PADDING;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

fn move_p2_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<P2Paddle>>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;
    if keyboard_input.pressed(KeyCode::L) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::O) {
        direction += 1.0;
    }

    let new_paddle_position = paddle_transform.translation.y + direction * PADDLE_SPEED * TIME_STEP;
    let top_bound = TOP_WALL - PADDLE_SIZE.y + PADDLE_PADDING;
    let bottom_bound = BOTTOM_WALL + PADDLE_SIZE.y - PADDLE_PADDING;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn update_p1_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<P1GoalText>>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.p1_score);
}

fn update_p2_scoreboard(
    scoreboard: Res<Scoreboard>,
    mut query: Query<&mut Text, With<P2GoalText>>,
) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.p2_score);
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    collider_query: Query<
        (
            Entity,
            &Transform,
            Option<&P1Goal>,
            Option<&P2Goal>,
            Option<&P1Paddle>,
            Option<&P2Paddle>,
        ),
        (With<Collider>, Without<Ball>),
    >,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, mut ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    // wall collision
    for (collider_entity, transform, maybe_p1_goal, maybe_p2_goal, maybe_p1_paddle, maybe_p2_paddle) in collider_query.iter() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );

        if let Some(collision) = collision {
            collision_events.send_default();

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => { /* do nothing */ }
            }

            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }

            if maybe_p1_goal.is_some() {
                // increase p2 score, reset ball and serve
                scoreboard.p2_score += 1;
                ball_transform.translation.x = BALL_STARTING_POSITION.x;
                ball_transform.translation.y = BALL_STARTING_POSITION.y;
                ball_transform.translation.z = BALL_STARTING_POSITION.z;
            }

            if maybe_p2_goal.is_some() {
                scoreboard.p1_score += 1;
                ball_transform.translation.x = BALL_STARTING_POSITION.x;
                ball_transform.translation.y = BALL_STARTING_POSITION.y;
                ball_transform.translation.z = BALL_STARTING_POSITION.z;
            }

            if maybe_p1_paddle.is_some() {
                scoreboard.fjongs += 1;
            }

            if maybe_p2_paddle.is_some() {
                scoreboard.fjongs += 1;
            }
            if ball_velocity.x > 0.0 {
                ball_velocity.x += (scoreboard.fjongs as f32) * 1.5;
                if ball_velocity.x >= 1000.0 {
                    ball_velocity.x = 1000.0;
                }
            } else {
                ball_velocity.x -= (scoreboard.fjongs as f32) * 1.5;
                if ball_velocity.x <= -1000.0 {
                    ball_velocity.x = -1000.0;
                }
            }

            if ball_velocity.y > 0.0 {
                ball_velocity.y += (scoreboard.fjongs as f32) * 1.5;
                if ball_velocity.y >= 200.0 {
                    ball_velocity.y = 200.0;
                }
            } else {
                ball_velocity.y -= (scoreboard.fjongs as f32) * 1.5;
                if ball_velocity.y <= -200.0 {
                    ball_velocity.y = -200.0;
                }
            }
            
        }
    }
}