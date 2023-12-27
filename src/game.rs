use bevy::{
    prelude::*,
    sprite::collide_aabb::collide,
};
use std::f32::consts::PI;
use rand::random;
use crate::{GameState, SCALE, SCREEN_SIZE, despawn_screen, GameOverCause, MenuState, HALF_SCREEN_SIZE};

// consts
const PIPE_SPEED: f32 = 50.0;
const HALF_PIPE_SPACE: f32 = 50.0;
const MAX_PIPE_HOLE_Y: f32 = 80.;
const NUM_PIPES: u32 = 3;
const PIPE_SIZE: Vec2 = Vec2 { x: SCALE.x * 16., y: SCALE.y * 64. };
const PIPE_COLLIDER: Vec2 = Vec2 { x: SCALE.x * 12., y: SCALE.y * 64. };

const BORB_START_POS: Vec3 = Vec3 { x: -200.0, y: 0.0, z: 0.0 };
const BORB_COLLIDER: Vec2 = Vec2 { x: SCALE.x * 10., y: SCALE.y * 10. };
const BORB_SIZE: Vec2 = Vec2 { x: SCALE.x * 12., y: SCALE.y * 12. };
const BORB_HALF_HEIGHT: f32 = BORB_SIZE.y / 2.;

const GRAVITY: f32 = 140.0;
const MAX_GRAVITY: f32 = -140.0;
const JUMP_FORCE: f32 = 110.0;

const SCREEN_WIDTH_WITH_PIPE: f32 = SCREEN_SIZE.x + PIPE_SIZE.x;
const HALF_SCREEN_WIDTH_WITH_HALF_PIPE: f32 = SCREEN_WIDTH_WITH_PIPE / 2.;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Scoreboard(0))
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(Update, (
                (
                    jump,
                    apply_gravity,
                    move_pipes,
                    check_for_collisions,
                    check_out_of_bounds,
                ).chain(),
                bevy::window::close_on_esc,
            ).run_if(in_state(GameState::Game)))
            .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
    }
}

//enums
enum PipeSide {
    Top,
    Bottom,
}

// resources
#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct Scoreboard(pub u32);

impl Scoreboard {
    fn add(&mut self, score: u32) {
        self.0 += score;
    }

    fn reset(&mut self) {
        self.0 = 0;
    }
}

// components
#[derive(Component)]
struct OnGameScreen;

#[derive(Component)]
struct Borb;

#[derive(Component, Deref, DerefMut)]
struct Gravity(f32);

#[derive(Component)]
struct Collider(Vec2);

#[derive(Component)]
struct Obstacle(GameOverCause);

#[derive(Component)]
struct PipeParent(bool);

// bundles
#[derive(Bundle)]
struct PipeBundle {
    sprite: SpriteBundle,
    collider: Collider,
    enemy: Obstacle,
}

impl PipeBundle {
    fn new(position: Vec2, texture: Handle<Image>, side: PipeSide) -> PipeBundle {
        PipeBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.0),
                    rotation: Quat::from_rotation_z(match side {
                        PipeSide::Bottom => 0.0,
                        PipeSide::Top => PI,
                    }),
                    scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                    ..default()
                },
                texture,
                ..default()
            },
            collider: Collider(PIPE_COLLIDER),
            enemy: Obstacle(GameOverCause::HitPipe),
        }
    }
}

// functions
fn random_pipe_hole_y() -> f32 {
    random::<f32>() * MAX_PIPE_HOLE_Y * 2.0 - MAX_PIPE_HOLE_Y
}

fn game_over(
    cause: GameOverCause,
    game_state: &mut ResMut<NextState<GameState>>,
    menu_state: &mut ResMut<NextState<MenuState>>,
) {
    game_state.set(GameState::Menu);
    menu_state.set(MenuState::GameOver(cause));
}

// systems
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Scoreboard>,
) {
    let game_over_sound = asset_server.load("sounds/game_over.wav");
    commands.insert_resource(CollisionSound(game_over_sound));

    score.reset();

    // Borb
    let borb_texture = asset_server.load("sprites/borb.png");
    commands.spawn((
        SpriteBundle {
            texture: borb_texture,
            transform: Transform::from_translation(BORB_START_POS).with_scale(SCALE),
            ..default()
        },
        Borb,
        Gravity(0.0),
        Collider(BORB_COLLIDER),
        OnGameScreen,
    ));

    // Pipes
    let pipe = asset_server.load("sprites/pipe.png");
    let x_diff = SCREEN_WIDTH_WITH_PIPE / NUM_PIPES as f32;
    for pipe_num in 0..NUM_PIPES {
        let x = -HALF_SCREEN_WIDTH_WITH_HALF_PIPE + x_diff * (pipe_num as f32 + 1.);
        let y = random_pipe_hole_y();
        commands
            .spawn((
                PipeParent(false),
                SpatialBundle {
                    transform: Transform {
                        translation: Vec3 { x, y, z: 0.0 },
                        scale: SCALE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                parent.spawn(PipeBundle::new(Vec2 { x: 0., y: -HALF_PIPE_SPACE }, pipe.clone(), PipeSide::Bottom));
                parent.spawn(PipeBundle::new(Vec2 { x: 0., y: HALF_PIPE_SPACE }, pipe.clone(), PipeSide::Top));
            });
    }
}

fn move_pipes(
    mut query: Query<(&mut Transform, &mut PipeParent)>,
    borb_query: Query<&Transform, (With<Borb>, Without<PipeParent>)>,
    time: Res<Time>,
    mut score: ResMut<Scoreboard>,
) {
    let borb_transform = borb_query.single();
    for (mut transform, mut pipe) in &mut query {
        transform.translation.x -= PIPE_SPEED * time.delta_seconds();
        if transform.translation.x < -HALF_SCREEN_WIDTH_WITH_HALF_PIPE {
            transform.translation.y = random_pipe_hole_y();
            transform.translation.x = HALF_SCREEN_WIDTH_WITH_HALF_PIPE;
            pipe.0 = false
        } else if !pipe.0 && transform.translation.x < borb_transform.translation.x {
            score.add(1);
            pipe.0 = true;
        }
    }
}

fn jump(mut query: Query<(&mut Gravity, With<Borb>)>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut gravity, _) in &mut query {
            gravity.0 = JUMP_FORCE;
        }
    }
}

fn apply_gravity(mut query: Query<(&mut Transform, &mut Gravity)>, time: Res<Time>) {
    for (mut transform, mut gravity) in &mut query {
        gravity.0 -= GRAVITY * time.delta_seconds();
        gravity.0 = gravity.0.max(MAX_GRAVITY);
        transform.translation.y += gravity.0 * time.delta_seconds();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    borb_query: Query<(&Transform, &Collider), With<Borb>>,
    collider_query: Query<(&GlobalTransform, &Collider, &Obstacle)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    sound: Res<CollisionSound>,
) {
    let (borb_transform, borb_collider) = borb_query.single();

    for (transform, collider, enemy) in &collider_query {
        let collision = collide(
            borb_transform.translation,
            borb_collider.0,
            transform.translation(),
            collider.0,
        );
        if let Some(_) = collision {
            game_over(enemy.0, &mut game_state, &mut menu_state);
            commands.spawn(AudioBundle {
                source: sound.0.clone(),
                settings: PlaybackSettings::DESPAWN,
            });
        }
    }
}

fn check_out_of_bounds(
    mut commands: Commands,
    mut borb_query: Query<&Transform, With<Borb>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    sound: Res<CollisionSound>,
) {
    let borb_transform = borb_query.single_mut();
    let y = borb_transform.translation.y;
    if y < -HALF_SCREEN_SIZE.y - BORB_HALF_HEIGHT {
        game_over(GameOverCause::TooLow, &mut game_state, &mut menu_state);
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    } else if y > HALF_SCREEN_SIZE.y + BORB_HALF_HEIGHT {
        game_over(GameOverCause::TooHigh, &mut game_state, &mut menu_state);
        commands.spawn(AudioBundle {
            source: sound.0.clone(),
            settings: PlaybackSettings::DESPAWN,
        });
    }
}