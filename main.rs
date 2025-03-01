use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Time;
use bevy::window::{PrimaryWindow, Window};
use rand::Rng;

// Constants for gameplay tuning.
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 30.0);
const PLAYER_SPEED: f32 = 200.0;
const PLAYER_JUMP_VELOCITY: f32 = 300.0;
const ENEMY_SIZE: Vec2 = Vec2::new(30.0, 30.0);
const ENEMY_SPEED_RANGE: (f32, f32) = (50.0, 150.0);
const OBSTACLE_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const GROUND_HEIGHT: f32 = 20.0;
const GRAVITY_FORCE: f32 = -500.0;

#[derive(Resource)]
pub struct Gravity(pub f32);

#[derive(Resource)]
pub struct Score(i32);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct ScoreText;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Resource)]
pub struct GroundData {
    pub center_y: f32,
    pub top_y: f32,
    pub height: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Gravity(GRAVITY_FORCE))
        .insert_resource(Score(0))
        .insert_resource(GroundData {
            center_y: 0.0,
            top_y: GROUND_HEIGHT / 2.0,
            height: GROUND_HEIGHT,
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_enemies.after(setup))
        .add_systems(Startup, spawn_obstacles.after(setup))
        .add_systems(Update, player_input_system)
        .add_systems(Update, apply_gravity_system)
        .add_systems(Update, movement_system)
        .add_systems(Update, player_wrap_system) // wrap-around for player
        .add_systems(Update, enemy_wrap_system)  // wrap-around for enemies
        // NEW: Enemy-obstacle collision system
        .add_systems(Update, enemy_obstacle_collision_system)
        .add_systems(Update, collision_system)
        .add_systems(Update, enemy_collision_system)
        .add_systems(Update, obstacle_collision_system)
        .add_systems(Update, update_score_system)
        .add_systems(Update, check_end_game_system)
        .run();
}


//
// SETUP SYSTEMS
//

/// Initializes the camera, ground, UI text, and player.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();

    // Calculate ground positions.
    let ground_center_y = 0.0;
    let ground_top_y = ground_center_y + GROUND_HEIGHT / 2.0;

    // Update the GroundData resource.
    commands.insert_resource(GroundData {
        center_y: ground_center_y,
        top_y: ground_top_y,
        height: GROUND_HEIGHT,
    });

    // Spawn the 2D camera.
    commands.spawn(Camera2dBundle::default());

    // Spawn the ground.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(window.width(), GROUND_HEIGHT)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Ground,
    ));

    // Spawn score UI.
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Score: 0",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                ..default()
            },
            ..default()
        },
        ScoreText,
    ));

    // Spawn the player so its bottom touches the ground.
    // Center is ground top + half the player height.
    let player_y = ground_top_y + PLAYER_SIZE.y / 2.0;
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("player.png"),
            sprite: Sprite {
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, player_y, 0.0)),
            ..default()
        },
        Player,
        Velocity(Vec2::ZERO),
    ));
}

/// Spawns a random number of enemies with random horizontal velocities.
fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ground_data: Res<GroundData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let mut rng = rand::thread_rng();
    let enemy_count = rng.gen_range(2..5);
    let enemy_y = ground_data.top_y + ENEMY_SIZE.y / 2.0;

    for _ in 0..enemy_count {
        let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
        let enemy_pos = Vec3::new(x, enemy_y, 0.0);

        // Random horizontal speed and direction.
        let speed = rng.gen_range(ENEMY_SPEED_RANGE.0..ENEMY_SPEED_RANGE.1);
        let direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };

        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("enemy.png"),
                sprite: Sprite {
                    custom_size: Some(ENEMY_SIZE),
                    ..default()
                },
                transform: Transform::from_translation(enemy_pos),
                ..default()
            },
            Enemy,
            Velocity(Vec2::new(direction * speed, 0.0)),
        ));
    }
}

/// Spawns a random number of obstacles at ground level.
fn spawn_obstacles(
    mut commands: Commands,
    ground_data: Res<GroundData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let mut rng = rand::thread_rng();
    let obstacle_count = rng.gen_range(3..7);
    let obstacle_y = ground_data.top_y + OBSTACLE_SIZE.y / 2.0;

    for _ in 0..obstacle_count {
        let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
        let obstacle_pos = Vec3::new(x, obstacle_y, 0.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    custom_size: Some(OBSTACLE_SIZE),
                    ..default()
                },
                transform: Transform::from_translation(obstacle_pos),
                ..default()
            },
            Obstacle,
        ));
    }
}

//
// GAMEPLAY SYSTEMS
//

/// Processes player input for movement and jumping.
fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
    ground_data: Res<GroundData>,
) {
    for (mut velocity, mut transform) in query.iter_mut() {
        // Horizontal movement.
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += 1.0;
        }
        velocity.x = direction * PLAYER_SPEED;

        // Flip sprite based on direction.
        if direction != 0.0 {
            transform.scale.x = transform.scale.x.abs() * direction.signum();
        }

        // Jump if on the ground.
        if (keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::Key2))
            && transform.translation.y <= ground_data.top_y + PLAYER_SIZE.y / 2.0
        {
            velocity.y = PLAYER_JUMP_VELOCITY;
        }
    }
}

/// Applies gravity to the player.
fn apply_gravity_system(
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    for mut velocity in query.iter_mut() {
        velocity.y += gravity.0 * time.delta_seconds();
    }
}

/// Moves all entities based on their velocity.
fn movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);
    }
}

/// Wraps the player around the screen horizontally.
fn player_wrap_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let window = window_query.single();
    let half_width = window.width() / 2.0;
    for mut transform in query.iter_mut() {
        if transform.translation.x > half_width {
            transform.translation.x = -half_width;
        } else if transform.translation.x < -half_width {
            transform.translation.x = half_width;
        }
    }
}

/// Wraps enemies around the screen horizontally.
fn enemy_wrap_system(
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<&mut Transform, With<Enemy>>,
) {
    let window = window_query.single();
    let half_width = window.width() / 2.0;
    for mut transform in query.iter_mut() {
        if transform.translation.x > half_width {
            transform.translation.x = -half_width;
        } else if transform.translation.x < -half_width {
            transform.translation.x = half_width;
        }
    }
}

/// Helper function for AABB collision detection.
fn is_colliding(pos_a: Vec3, half_a: Vec2, pos_b: Vec3, half_b: Vec2) -> bool {
    (pos_a.x - half_a.x < pos_b.x + half_b.x)
        && (pos_a.x + half_a.x > pos_b.x - half_b.x)
        && (pos_a.y - half_a.y < pos_b.y + half_b.y)
        && (pos_a.y + half_a.y > pos_b.y - half_b.y)
}

/// Keeps the player on the ground if falling below it.
fn collision_system(
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    ground_data: Res<GroundData>,
) {
    for (mut transform, mut velocity) in query.iter_mut() {
        let player_half = PLAYER_SIZE.y / 2.0;
        if transform.translation.y - player_half < ground_data.top_y {
            transform.translation.y = ground_data.top_y + player_half;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

/// Handles collisions between the player and enemies.
fn enemy_collision_system(
    mut commands: Commands,
    mut score: ResMut<Score>,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    asset_server: Res<AssetServer>,
    player_entity_query: Query<Entity, With<Player>>,
) {
    for (player_transform, player_sprite) in player_query.iter() {
        let player_half = player_sprite
            .custom_size
            .unwrap_or(PLAYER_SIZE)
            / 2.0;
        for (enemy_entity, enemy_transform, enemy_sprite) in enemy_query.iter() {
            let enemy_half = enemy_sprite
                .custom_size
                .unwrap_or(ENEMY_SIZE)
                / 2.0;
            if is_colliding(
                player_transform.translation,
                Vec2::splat(player_half.x),
                enemy_transform.translation,
                Vec2::splat(enemy_half.x),
            ) {
                // Stomp enemy if player is above.
                if player_transform.translation.y - player_half.y
                    >= enemy_transform.translation.y + enemy_half.y - 5.0
                {
                    commands.entity(enemy_entity).despawn();
                    score.0 += 100;
                    info!("Enemy defeated! Score: {}", score.0);
                } else {
                    // Game over scenario.
                    commands.spawn(TextBundle {
                        text: Text::from_section(
                            "Game Over",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 80.0,
                                color: Color::RED,
                            },
                        ),
                        style: Style {
                            position_type: PositionType::Absolute,
                            top: Val::Percent(40.0),
                            left: Val::Percent(35.0),
                            ..default()
                        },
                        ..default()
                    });
                    for player_entity in player_entity_query.iter() {
                        commands.entity(player_entity).despawn();
                    }
                    info!("Game Over!");
                }
            }
        }
    }
}

fn enemy_obstacle_collision_system(
    mut enemy_query: Query<(&Transform, &mut Velocity), With<Enemy>>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
) {
    for (enemy_transform, mut enemy_velocity) in enemy_query.iter_mut() {
        // Define enemy half size (assuming enemy sprite uses ENEMY_SIZE)
        let enemy_half = ENEMY_SIZE / 2.0;
        for obstacle_transform in obstacle_query.iter() {
            // Define obstacle half size (assuming obstacle sprite uses OBSTACLE_SIZE)
            let obstacle_half = OBSTACLE_SIZE / 2.0;
            let enemy_pos = enemy_transform.translation;
            let obstacle_pos = obstacle_transform.translation;
            // Basic AABB collision detection
            let collision = (enemy_pos.x - enemy_half.x < obstacle_pos.x + obstacle_half.x)
                && (enemy_pos.x + enemy_half.x > obstacle_pos.x - obstacle_half.x)
                && (enemy_pos.y - enemy_half.y < obstacle_pos.y + obstacle_half.y)
                && (enemy_pos.y + enemy_half.y > obstacle_pos.y - obstacle_half.y);
            if collision {
                // Invert the horizontal velocity if a collision is detected.
                enemy_velocity.x = -enemy_velocity.x;
            }
        }
    }
}


/// Handles collisions between the player and obstacles.
fn obstacle_collision_system(
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Velocity, &Sprite), With<Player>>,
        Query<&Transform, With<Obstacle>>,
    )>,
) {
    let obstacles: Vec<Vec3> = param_set.p1().iter().map(|t| t.translation).collect();

    for (mut player_transform, mut player_velocity, player_sprite) in param_set.p0().iter_mut() {
        let player_half = player_sprite.custom_size.unwrap_or(PLAYER_SIZE) / 2.0;
        for &obstacle_pos in &obstacles {
            let obstacle_half = OBSTACLE_SIZE / 2.0;
            if is_colliding(player_transform.translation, player_half, obstacle_pos, obstacle_half) {
                // Prevent horizontal overlap.
                if player_transform.translation.x < obstacle_pos.x {
                    player_transform.translation.x =
                        obstacle_pos.x - obstacle_half.x - player_half.x;
                } else {
                    player_transform.translation.x =
                        obstacle_pos.x + obstacle_half.x + player_half.x;
                }
                player_velocity.x = 0.0;
                // Adjust vertical position if needed.
                if player_transform.translation.y > obstacle_pos.y {
                    player_transform.translation.y =
                        obstacle_pos.y + obstacle_half.y + player_half.y;
                }
            }
        }
    }
}

/// Updates the UI score text when the score changes.
fn update_score_system(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("Score: {}", score.0);
        }
    }
}

/// Ends the game when either all enemies are defeated or the player is gone.
fn check_end_game_system(
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut exit: EventWriter<AppExit>,
) {
    if enemy_query.is_empty() {
        // Spawn a win title if no enemies remain.
        commands.spawn(TextBundle {
            text: Text::from_section(
                "You Win!",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::GREEN,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(40.0),
                left: Val::Percent(35.0),
                ..default()
            },
            ..default()
        });
        exit.send(AppExit);
    } else if player_query.is_empty() {
        // Spawn a game over title if the player is gone.
        commands.spawn(TextBundle {
            text: Text::from_section(
                "Game Over",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 80.0,
                    color: Color::RED,
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Percent(40.0),
                left: Val::Percent(35.0),
                ..default()
            },
            ..default()
        });
        exit.send(AppExit);
    }
}

