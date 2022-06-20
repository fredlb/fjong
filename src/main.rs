use std::f32::consts::PI;

use bevy::{
    core::FixedTimestep,
    math::{const_vec2, const_vec3},
    input::gamepad::{GamepadEvent, GamepadEventType},
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
const BALL_SPEED_X: f32 = 400.0;
const BALL_SPEED_Y: f32 = 50.0;
const INITIAL_BALL_DIRECTION: Vec2 = const_vec2!([-0.5, 0.1]);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const SCOREBOARD_FONT_SIZE: f32 = 32.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(15.0);

const BACKGROUND_COLOR: Color = Color::BLACK;
const FOREGROUND_COLOR: Color = Color::WHITE;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Thingies>()
        .insert_resource(Scoreboard {
            p1_score: 0,
            p2_score: 0,
            fjongs: 0,
        })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_system(gamepad_connections)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(ai2.before(check_for_collisions))
                .with_system(move_p1_paddle.before(check_for_collisions))
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

#[derive(Default)]
struct Thingies {
    score_cooldown: Timer,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut thingies: ResMut<Thingies>) {
    thingies.score_cooldown = Timer::from_seconds(0.7, false);
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
        .insert(Velocity(const_vec2!([0.0, 0.0])))
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
        .insert(Velocity(const_vec2!([
            INITIAL_BALL_DIRECTION.normalize().x * BALL_SPEED_X,
            INITIAL_BALL_DIRECTION.normalize().y * BALL_SPEED_Y,
        ])));

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
                        value: "P1: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
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
                        value: "P2: ".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: FOREGROUND_COLOR,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: asset_server.load("fonts/PressStart2P-Regular.ttf"),
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

/// Simple resource to store the ID of the connected gamepad.
/// We need to know which gamepad to use for player input.
struct MyGamepad(Gamepad);

fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<MyGamepad>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        match kind {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(MyGamepad(*id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                // if it's the one we previously associated with the player,
                // disassociate it:
                if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                    if old_id == id {
                        commands.remove_resource::<MyGamepad>();
                    }
                }
            }
            // other events are irrelevant
            _ => {}
        }
    }
}

fn move_p1_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<P1Paddle>>,
    axes: Res<Axis<GamepadAxis>>,
    my_gamepad: Option<Res<MyGamepad>>,
) {
    if let Some(gp) = my_gamepad {
        let axis_ly = GamepadAxis(gp.0, GamepadAxisType::LeftStickY);
        let mut paddle_transform = query.single_mut();

        if let Some(y) = axes.get(axis_ly) {
            let new_paddle_position = y * 250.0;
            let top_bound = TOP_WALL - PADDLE_SIZE.y + PADDLE_PADDING;
            let bottom_bound = BOTTOM_WALL + PADDLE_SIZE.y - PADDLE_PADDING;

            paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
        }
    } else {
        let mut paddle_transform = query.single_mut();
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::S) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::W) {
            direction += 1.0;
        }

        let new_paddle_position = paddle_transform.translation.y + direction * PADDLE_SPEED * TIME_STEP;
        let top_bound = TOP_WALL - PADDLE_SIZE.y + PADDLE_PADDING;
        let bottom_bound = BOTTOM_WALL + PADDLE_SIZE.y - PADDLE_PADDING;

        paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
    };

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

fn ai2(
    mut ball_query: Query<(&Velocity, &Transform), With<Ball>>,
    mut paddle_2: Query<(&mut Velocity, &Transform), (With<P2Paddle>, Without<Ball>)>
) {
    let (ball_velocity, ball_transform) = ball_query.single_mut();
    let (mut p2_velocity, p2_transform) = paddle_2.single_mut();


    if (ball_velocity.x > 0.0) && ((ball_transform.translation.x + (BALL_SIZE.x/2.0)) > ((LEFT_WALL - RIGHT_WALL)/2.0)) {
        if (ball_transform.translation.y + (BALL_SIZE.x/2.0)) != (p2_transform.translation.y + (PADDLE_SIZE.y / 2.0)) {

            let time_til_collision = (((RIGHT_WALL - LEFT_WALL)/2.0 - PADDLE_PADDING - PADDLE_SIZE.x) - ball_transform.translation.x) / ball_velocity.x;

            let distance_wanted = (p2_transform.translation.y ) - (ball_transform.translation.y + (BALL_SIZE.x/2.0));

            let velocity_wanted = -distance_wanted / time_til_collision;

            let top_bound = TOP_WALL - PADDLE_SIZE.y + PADDLE_PADDING;
            let bottom_bound = BOTTOM_WALL + PADDLE_SIZE.y - PADDLE_PADDING;

            // TODO: Condition so it can't clip top and bottom walls
            if velocity_wanted > 800.0 {
                p2_velocity.y = 800.0
            } else if velocity_wanted < -800.0  {
                p2_velocity.y = -800.0
            } else {
                p2_velocity.y = velocity_wanted;
            }

        } else {
            p2_velocity.y = 0.0;
        }
    } else {
        p2_velocity.y = 0.0;
    }
}

fn apply_velocity(
    mut thingies: ResMut<Thingies>,
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    if thingies.score_cooldown.tick(time.delta()).finished() {
        for (mut transform, velocity) in query.iter_mut() {
            transform.translation.x += velocity.x * TIME_STEP;
            transform.translation.y += velocity.y * TIME_STEP;
        }
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
    mut thingies: ResMut<Thingies>,
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
    for (
        collider_entity,
        transform,
        maybe_p1_goal,
        maybe_p2_goal,
        maybe_p1_paddle,
        maybe_p2_paddle,
    ) in collider_query.iter()
    {
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
                if scoreboard.fjongs >= 5 {
                    scoreboard.fjongs = 2;
                }
                scoreboard.p2_score += 1;
                ball_transform.translation.x = BALL_STARTING_POSITION.x;
                ball_transform.translation.y = BALL_STARTING_POSITION.y;
                ball_transform.translation.z = BALL_STARTING_POSITION.z;
                ball_velocity.x = BALL_SPEED_X;
                ball_velocity.y = BALL_SPEED_Y;
                thingies.score_cooldown.reset();
            }

            if maybe_p2_goal.is_some() {
                if scoreboard.fjongs >= 5 {
                    scoreboard.fjongs = 2;
                }
                scoreboard.p1_score += 1;
                ball_transform.translation.x = BALL_STARTING_POSITION.x;
                ball_transform.translation.y = BALL_STARTING_POSITION.y;
                ball_transform.translation.z = BALL_STARTING_POSITION.z;
                ball_velocity.x = BALL_SPEED_X;
                ball_velocity.y = BALL_SPEED_Y;
                thingies.score_cooldown.reset();
            }

            if maybe_p1_paddle.is_some() {
                scoreboard.fjongs += 1;
                let relative_intersect_y = transform.translation.y - ball_transform.translation.y;
                let normalized_relative_intersection_y = relative_intersect_y/(PADDLE_SIZE.y / 2.0);
                let bounce_angle = normalized_relative_intersection_y * (PI/2.0 - (PI/4.0));

                ball_velocity.x = BALL_SPEED * bounce_angle.cos() + (scoreboard.fjongs as f32 * 4.0);
                ball_velocity.y = BALL_SPEED * (-bounce_angle.sin()) + (scoreboard.fjongs as f32 * 4.0);
            }

            if maybe_p2_paddle.is_some() {
                scoreboard.fjongs += 1;
                let relative_intersect_y = transform.translation.y - ball_transform.translation.y;
                let normalized_relative_intersection_y = relative_intersect_y/(PADDLE_SIZE.y / 2.0);
                let bounce_angle = normalized_relative_intersection_y * (PI/2.0 - (PI/4.0));

                ball_velocity.x = ((BALL_SPEED * bounce_angle.cos()) + (scoreboard.fjongs as f32 * 4.0)) * -1.0;
                ball_velocity.y = ((BALL_SPEED * bounce_angle.sin()) + (scoreboard.fjongs as f32 * 4.0)) * -1.0;
            }

        }
    }
}

