use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::time::Time;
use rand::Rng; // Ensure this is added to your Cargo.toml dependencies
use bevy::window::{PrimaryWindow, Window};

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
        .insert_resource(GroundData {
            center_y: 0.0,
            top_y: 0.0,
            height: 0.0,
        })
        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_enemies.after(setup))
        .add_systems(Startup, spawn_obstacles.after(setup))
        .add_systems(Update, player_input_system)
        .add_systems(Update, apply_gravity_system)
        .add_systems(Update, movement_system)
        .add_systems(Update, player_wrap_system) // wrap-around system
        .add_systems(Update, collision_system)
        .add_systems(Update, enemy_collision_system)
        .add_systems(Update, obstacle_collision_system)
        .add_systems(Update, update_score_system)
        .add_systems(Update, check_end_game_system)
        .run();
}

#[derive(Resource)]
pub struct GroundY(pub f32);

#[derive(Resource)]
pub struct GroundData {
    pub center_y: f32,
    pub top_y: f32,
    pub height: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let ground_height = 20.0;
    // Calculate ground center and top positions.
    let ground_center_y = -window.height() / 2.0 + ground_height / 2.0;
    let ground_top_y = ground_center_y + ground_height / 2.0;

    commands.insert_resource(GroundData {
        center_y: ground_center_y,
        top_y: ground_top_y,
        height: ground_height,
    });

    // Spawn the camera.
    commands.spawn(Camera2dBundle::default());

    // Spawn the ground.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.8, 0.2),
                custom_size: Some(Vec2::new(window.width(), ground_height)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, ground_center_y, 0.0),
                ..default()
            },
            ..default()
        },
        Ground,
    ));

    // Spawn the score UI text.
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

    // Spawn the player so its bottom rests on the ground.
    // For a 30x30 sprite, center = ground_top_y + 15.
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("player.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, ground_top_y + 15.0, 0.0),
                ..default()
            },
            ..default()
        },
        Player,
        Velocity(Vec2::ZERO),
    ));
}

/// Spawn a random number of enemies at ground level.
fn spawn_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ground_data: Res<GroundData>,
    obstacle_query: Query<&Transform, With<Obstacle>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let mut rng = rand::thread_rng();
    let enemy_count = rng.gen_range(2..5);

    let ground_top = ground_data.top_y;
    let enemy_height = 30.0;
    let enemy_half_height = enemy_height / 2.0;

    // Gather obstacle positions and sizes.
    let obstacle_data: Vec<(Vec3, Vec2)> = obstacle_query
        .iter()
        .map(|t| (t.translation, Vec2::new(40.0, 40.0)))
        .collect();

    for _ in 0..enemy_count {
        let mut spawn_tries = 0;
        let max_tries = 200;
        let mut spawned = false;

        while spawn_tries < max_tries && !spawned {
            let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
            let candidate_pos = Vec3::new(x, ground_top + enemy_half_height, 0.0);

            // Check for collision with obstacles.
            let mut valid_position = true;
            for (obstacle_pos, obstacle_size) in &obstacle_data {
                let obstacle_half = *obstacle_size / 2.0;
                let enemy_half = Vec2::new(enemy_half_height, enemy_half_height);
                let collision_x = (candidate_pos.x - enemy_half.x)
                    < (obstacle_pos.x + obstacle_half.x)
                    && (candidate_pos.x + enemy_half.x)
                        > (obstacle_pos.x - obstacle_half.x);
                let collision_y = (candidate_pos.y - enemy_half.y)
                    < (obstacle_pos.y + obstacle_half.y)
                    && (candidate_pos.y + enemy_half.y)
                        > (obstacle_pos.y - obstacle_half.y);

                if collision_x && collision_y {
                    valid_position = false;
                    break;
                }
            }

            if valid_position {
                commands.spawn((
                    SpriteBundle {
                        texture: asset_server.load("enemy.png"),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(30.0, 30.0)),
                            ..default()
                        },
                        transform: Transform {
                            translation: candidate_pos,
                            ..default()
                        },
                        ..default()
                    },
                    Enemy,
                ));
                spawned = true;
            }
            spawn_tries += 1;
        }
    }
}

/// Spawn a random number of obstacles at ground level.
fn spawn_obstacles(
    mut commands: Commands,
    ground_data: Res<GroundData>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single();
    let mut rng = rand::thread_rng();
    let obstacle_count = rng.gen_range(3..7);

    let ground_top = ground_data.top_y;

    for _ in 0..obstacle_count {
        let obstacle_height = 40.0;
        let obstacle_half_height = obstacle_height / 2.0;
        let obstacle_center_y = ground_top + obstacle_half_height;

        let x = rng.gen_range(-window.width() / 2.0..window.width() / 2.0);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x, obstacle_center_y, 0.0),
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
    ground_data: Res<GroundData>,
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

        // Flip sprite based on movement direction.
        if direction < 0.0 {
            transform.scale.x = transform.scale.x.abs() * -1.0;
        } else if direction > 0.0 {
            transform.scale.x = transform.scale.x.abs();
        }

        // Allow jumping if the player's bottom is on or below the ground.
        if (keyboard_input.just_pressed(KeyCode::Space)
            || keyboard_input.just_pressed(KeyCode::Key2))
            && transform.translation.y <= ground_data.top_y + 15.0
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

/// Wraps the player around the screen horizontally.
/// If the player moves beyond one side, it reappears on the opposite side.
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

fn collision_system(
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>, 
    ground_data: Res<GroundData>
) {
    for (mut transform, mut velocity) in query.iter_mut() {
        let player_half_height = 15.0;
        let ground_top = ground_data.top_y;

        // Snap the player to the ground if falling below it.
        if transform.translation.y - player_half_height < ground_top {
            transform.translation.y = ground_top + player_half_height;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

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
                // If collision is from above (stomp), despawn the enemy and update score.
                if player_pos.y - player_half.y >= enemy_pos.y + enemy_half.y - 5.0 {
                    commands.entity(enemy_entity).despawn();
                    score.0 += 100;
                    info!("Enemy defeated! Score: {}", score.0);
                } else {
                    // Otherwise, show game over and despawn the player.
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

fn obstacle_collision_system(
    mut param_set: ParamSet<(
        Query<(&mut Transform, &mut Velocity, &Sprite), With<Player>>,
        Query<&Transform, With<Obstacle>>,
    )>,
) {
    // Collect obstacle positions.
    let obstacles: Vec<Vec3> = param_set.p1().iter().map(|t| t.translation).collect();

    for (mut player_transform, mut player_velocity, player_sprite) in param_set.p0().iter_mut() {
        let player_pos = player_transform.translation;
        let player_half = if let Some(size) = player_sprite.custom_size {
            size / 2.0
        } else {
            Vec2::new(15.0, 15.0)
        };

        for &obstacle_pos in &obstacles {
            let obstacle_half = Vec2::new(20.0, 20.0);

            let collision_x = (player_pos.x - player_half.x < obstacle_pos.x + obstacle_half.x)
                && (player_pos.x + player_half.x > obstacle_pos.x - obstacle_half.x);
            let collision_y = (player_pos.y - player_half.y < obstacle_pos.y + obstacle_half.y)
                && (player_pos.y + player_half.y > obstacle_pos.y - obstacle_half.y);

            if collision_x && collision_y {
                if player_pos.x < obstacle_pos.x {
                    player_transform.translation.x =
                        obstacle_pos.x - obstacle_half.x - player_half.x;
                } else {
                    player_transform.translation.x =
                        obstacle_pos.x + obstacle_half.x + player_half.x;
                }

                player_velocity.x = 0.0;

                if player_pos.y > obstacle_pos.y {
                    player_transform.translation.y =
                        obstacle_pos.y + obstacle_half.y + player_half.y;
                }
            }
        }
    }
}

fn check_end_game_system(
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<Entity, With<Player>>,
    mut exit: EventWriter<AppExit>,
) {
    if enemy_query.is_empty() || player_query.is_empty() {
        exit.send(AppExit);
    }
}
