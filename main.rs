use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Time;
use rand::Rng; // Add rand to your Cargo.toml dependencies

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Gravity(-500.0))
        .insert_resource(Score(0))
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_enemies)
        .add_systems(Startup, spawn_obstacles)
        .add_systems(Update, player_input_system)
        .add_systems(Update, apply_gravity_system)
        .add_systems(Update, movement_system)
        .add_systems(Update, collision_system)
        .add_systems(Update, enemy_collision_system)
        .add_systems(Update, obstacle_collision_system)
        .add_systems(Update, update_score_system)
        .add_systems(Update, check_end_game_system)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the 2D camera.
    commands.spawn(Camera2dBundle::default());

    // Spawn the score UI text at the top-right corner.
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

    // Spawn the player.
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 100.0, 0.0),
                scale: Vec3::ONE,
                ..default()
            },
            ..default()
        },
        Player,
        Velocity(Vec2::ZERO),
    ));

    // Spawn the ground platform.
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -100.0, 0.0),
                scale: Vec3::new(500.0, 20.0, 1.0),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                ..default()
            },
            ..default()
        },
        Ground,
    ));
}

/// Spawn a random number of enemies with random positions.
fn spawn_enemies(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut rng = rand::thread_rng();
    let enemy_count = rng.gen_range(2..5);
    for _ in 0..enemy_count {
        let x = rng.gen_range(-300.0..300.0);
        let y = rng.gen_range(-50.0..150.0);
        commands.spawn((
            SpriteBundle {
                texture: asset_server.load("enemy.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    ..default()
                },
                ..default()
            },
            Enemy,
        ));
    }
}

/// Spawn a random number of obstacles at ground level.
fn spawn_obstacles(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let obstacle_count = rng.gen_range(3..7);
    for _ in 0..obstacle_count {
        let x = rng.gen_range(-300.0..300.0);
        // Place obstacles near the ground.
        let y = rng.gen_range(-90.0..-50.0);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    ..default()
                },
                ..default()
            },
            Obstacle,
        ));
    }
}

fn update_score_system(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if score.is_changed() {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("Score: {}", score.0);
        }
    }
}

fn player_input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
    for (mut velocity, mut transform) in query.iter_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction += 1.0;
        }
        let speed = 200.0;
        velocity.x = direction * speed;

        // Flip the sprite based on movement direction.
        if direction < 0.0 {
            transform.scale.x = transform.scale.x.abs() * -1.0;
        } else if direction > 0.0 {
            transform.scale.x = transform.scale.x.abs();
        }

        // Jump when pressing Space or Key2 if the player is on the ground.
        if (keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::Key2))
            && transform.translation.y <= -75.0
        {
            velocity.y = 300.0;
        }
    }
}

fn apply_gravity_system(
    time: Res<Time>,
    gravity: Res<Gravity>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    for mut velocity in query.iter_mut() {
        velocity.y += gravity.0 * time.delta_seconds();
    }
}

fn movement_system(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn collision_system(mut query: Query<(&mut Transform, &mut Velocity), With<Player>>) {
    for (mut transform, mut velocity) in query.iter_mut() {
        let ground_y = -90.0;
        if transform.translation.y - 15.0 < ground_y {
            transform.translation.y = ground_y + 15.0;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

/// Detects collisions between the player and enemies.
/// If the collision is from above (stomp), the enemy is removed and the score increases.
/// Otherwise, the game ends.
fn enemy_collision_system(
    mut commands: Commands,
    mut score: ResMut<Score>,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    asset_server: Res<AssetServer>,
    player_entity_query: Query<Entity, With<Player>>,
) {
    for (player_transform, player_sprite) in player_query.iter() {
        let player_pos = player_transform.translation;
        let player_half = if let Some(size) = player_sprite.custom_size {
            size / 2.0
        } else {
            Vec2::new(15.0, 15.0)
        };

        for (enemy_entity, enemy_transform, enemy_sprite) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation;
            let enemy_half = if let Some(size) = enemy_sprite.custom_size {
                size / 2.0
            } else {
                Vec2::new(15.0, 15.0)
            };

            let collision = (player_pos.x - player_half.x < enemy_pos.x + enemy_half.x)
                && (player_pos.x + player_half.x > enemy_pos.x - enemy_half.x)
                && (player_pos.y - player_half.y < enemy_pos.y + enemy_half.y)
                && (player_pos.y + player_half.y > enemy_pos.y - enemy_half.y);

            if collision {
                // If collision is from above (stomp kill)
                if player_pos.y - player_half.y >= enemy_pos.y + enemy_half.y - 5.0 {
                    commands.entity(enemy_entity).despawn();
                    score.0 += 100;
                    info!("Enemy defeated! Score: {}", score.0);
                } else {
                    // Side collision: game over.
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

/// Checks collisions between the player and obstacles. Any collision ends the game.
fn obstacle_collision_system(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &Sprite), With<Player>>,
    obstacle_query: Query<(Entity, &Transform, &Sprite), With<Obstacle>>,
    asset_server: Res<AssetServer>,
) {
    for (player_entity, player_transform, player_sprite) in player_query.iter() {
        let player_pos = player_transform.translation;
        let player_half = if let Some(size) = player_sprite.custom_size {
            size / 2.0
        } else {
            Vec2::new(15.0, 15.0)
        };

        for (_obstacle_entity, obstacle_transform, obstacle_sprite) in obstacle_query.iter() {
            let obstacle_pos = obstacle_transform.translation;
            let obstacle_half = if let Some(size) = obstacle_sprite.custom_size {
                size / 2.0
            } else {
                Vec2::new(20.0, 20.0)
            };

            let collision = (player_pos.x - player_half.x < obstacle_pos.x + obstacle_half.x)
                && (player_pos.x + player_half.x > obstacle_pos.x - obstacle_half.x)
                && (player_pos.y - player_half.y < obstacle_pos.y + obstacle_half.y)
                && (player_pos.y + player_half.y > obstacle_pos.y - obstacle_half.y);

            if collision {
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
                commands.entity(player_entity).despawn();
                info!("Game Over: Hit an obstacle!");
            }
        }
    }
}

/// Checks if there are no enemies left or if the player has been despawned, and exits the app.
fn check_end_game_system(
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
    mut exit: EventWriter<AppExit>,
) {
    if enemy_query.is_empty() || player_query.is_empty() {
        exit.send(AppExit);
    }
}
