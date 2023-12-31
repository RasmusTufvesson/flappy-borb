use bevy::{
    prelude::*,
    sprite::collide_aabb::collide,
};
use std::f32::consts::PI;
use rand::random;
use crate::{GameState, SCALE, SCREEN_SIZE, despawn_screen, GameOverCause, MenuState, HALF_SCREEN_SIZE, TEXT_COLOR};

// consts
const PIPE_SPEED: f32 = 50.0;
const HALF_PIPE_SPACE: f32 = 50.0;
const MAX_PIPE_HOLE_Y: f32 = 80.;
const NUM_PIPES: u32 = 3;
const PIPE_SIZE: Vec2 = Vec2 { x: SCALE.x * 16., y: SCALE.y * 64. };
const PIPE_COLLIDER: Vec2 = Vec2 { x: SCALE.x * 12., y: SCALE.y * 64. };
const MIDDLE_PIPE_COOLLIDER: Vec2 = Vec2 { x: SCALE.x * 12., y: SCALE.y * 48. };

const BORB_START_POS: Vec3 = Vec3 { x: -200.0, y: 0.0, z: 0.0 };
const BORB_COLLIDER: Vec2 = Vec2 { x: SCALE.x * 10., y: SCALE.y * 10. };
const BORB_SIZE: Vec2 = Vec2 { x: SCALE.x * 12., y: SCALE.y * 12. };
const BORB_HALF_HEIGHT: f32 = BORB_SIZE.y / 2.;
const DEGREES_PER_GRAVITY: f32 = 0.006381360077604268;

const GRAVITY: f32 = 140.0;
const MAX_GRAVITY: f32 = -140.0;
const JUMP_FORCE: f32 = 110.0;

const SCREEN_WIDTH_WITH_PIPE: f32 = SCREEN_SIZE.x + PIPE_SIZE.x;
const HALF_SCREEN_WIDTH_WITH_HALF_PIPE: f32 = SCREEN_WIDTH_WITH_PIPE / 2.;

const MAX_UPGRADES: u32 = 5;
const MAX_CHAOS: u32 = 5;
const PIPES_PER_UPGRADE: u32 = 5;

const FAST_FALL_SPEED: f32 = -160.;
const FAST_PIPE_SPEED: f32 = 75.;

const NOTIFICATION_TEXT_SIZE: f32 = 25.;
const NOTIFICATION_START_X: f32 = 10.;
const NOTIFICATION_START_Y: f32 = 10.;
const NOTIFICATION_SPEED: f32 = 20.;
const NOTIFICATION_ALPHA_SPEED: f32 = 1.;

const PARTICLE_DECEL: f32 = 170.0;
const PARTICLE_START_SPEED: f32 = 70.0;
const PARTICLE_ALPHA_SPEED: f32 = 3.0;
const PARTICLE_MAX_ROTATION: f32 = 0.15;
const PARTICLE_ADD_ROTATION: f32 = 0.2;

const JUMP_PARTICLE_RELATIVE_START_POS: Vec2 = Vec2 { x: -BORB_COLLIDER.x / 2., y: -BORB_COLLIDER.y / 2. };
const JUMP_PARTICLE_NUM: u32 = 4;
const JUMP_PARTICLE_DISTANCE: Vec2 = Vec2 { x: BORB_COLLIDER.x / (JUMP_PARTICLE_NUM - 1) as f32, y: 0.0 };
const JUMP_PARTICLE_DIRECTION: Vec2 = Vec2::NEG_Y;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Game), setup)
            .add_systems(Update, (
                (
                    jump,
                    apply_gravity,
                    move_pipes,
                    check_for_collisions,
                    check_out_of_bounds,
                    update_borb_rotation,
                ).chain(),
                update_notifications,
                update_particles,
            ).run_if(in_state(GameState::Game)))
            .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
    }
}

//enums
enum PipeSide {
    Top,
    Bottom,
    Center,
}

#[derive(Component, PartialEq, PartialOrd)]
enum PipeType {
    Normal,
    Middle,
}

// resources
#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct Scoreboard {
    pub score: u32,
    was_last_upgrade_good: bool,
}

impl Scoreboard {
    fn add(
        &mut self, score: u32,
        upgrades: &mut ResMut<BorbUpgrades>,
        chaos: &mut ResMut<WorldChaos>,
        commands: &mut Commands,
    ) {
        let num_upgrades = self.score / PIPES_PER_UPGRADE;
        self.score += score;
        for _ in 0..(self.score / PIPES_PER_UPGRADE - num_upgrades) {
            self.upgrade(upgrades, chaos, commands);
        }
    }

    fn upgrade(
        &mut self,
        upgrades: &mut ResMut<BorbUpgrades>,
        chaos: &mut ResMut<WorldChaos>,
        commands: &mut Commands,
    ) {
        if self.was_last_upgrade_good {
            chaos.upgrade(commands);
        } else {
            upgrades.upgrade(commands);
        }
        self.was_last_upgrade_good = !self.was_last_upgrade_good
    }
}

#[derive(Resource)]
struct BorbUpgrades {
    num_upgrades: u32,
    fast_fall: bool,
}

impl Default for BorbUpgrades {
    fn default() -> Self {
        Self {
            num_upgrades: 0,
            fast_fall: false,
        }
    }
}

impl BorbUpgrades {
    fn upgrade(&mut self, commands: &mut Commands) {
        if self.num_upgrades != MAX_UPGRADES {
            self.fast_fall = true;
            self.num_upgrades += 1;
            create_notification("fast fall", commands);
        }
    }
}

#[derive(Resource)]
struct WorldChaos {
    num_chaos: u32,
    fast_pipes: bool,
    different_pipes: bool,
    world_speed: f32,
}

impl Default for WorldChaos {
    fn default() -> Self {
        Self {
            num_chaos: 0,
            fast_pipes: false,
            different_pipes: false,
            world_speed: PIPE_SPEED,
        }
    }
}

impl WorldChaos {
    fn upgrade(&mut self, commands: &mut Commands) {
        if self.num_chaos != MAX_CHAOS {
            self.num_chaos += 1;
            if self.fast_pipes {
                self.different_pipes = true;
                create_notification("different pipes", commands);
            } else {
                self.fast_pipes = true;
                self.world_speed = FAST_PIPE_SPEED;
                create_notification("fast pipes", commands);
            }
        }
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
struct PipeParent(bool, PipeType);

#[derive(Component)]
struct Notification(f32);

#[derive(Component)]
struct Particle {
    speed: f32,
    direction: Vec3,
    rotation_speed: f32,
}

// bundles
#[derive(Bundle)]
struct PipeBundle {
    sprite: SpriteBundle,
    collider: Collider,
    enemy: Obstacle,
    pipe_type: PipeType,
}

impl PipeBundle {
    fn new(position: Vec2, texture: Handle<Image>, side: PipeSide, pipe_type: PipeType) -> PipeBundle {
        PipeBundle {
            sprite: SpriteBundle {
                transform: Transform {
                    translation: position.extend(0.0),
                    scale: Vec3 { x: 1.0, y: 1.0, z: 1.0 },
                    ..default()
                },
                texture,
                visibility: match pipe_type {
                    PipeType::Normal => Visibility::Visible,
                    PipeType::Middle => Visibility::Hidden,
                },
                sprite: Sprite {
                    flip_y: match side {
                        PipeSide::Bottom | PipeSide::Center => false,
                        PipeSide::Top => true,
                    },
                    ..default()
                },
                ..default()
            },
            collider: Collider(match pipe_type {
                PipeType::Normal => PIPE_COLLIDER,
                PipeType::Middle => MIDDLE_PIPE_COOLLIDER,
            }),
            enemy: Obstacle(GameOverCause::HitPipe),
            pipe_type,
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

fn create_notification(
    text: &str,
    commands: &mut Commands,
) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                text,
                TextStyle {
                    font_size: NOTIFICATION_TEXT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(NOTIFICATION_START_Y),
            left: Val::Px(NOTIFICATION_START_X),
            ..default()
        }),
        OnGameScreen,
        Notification(NOTIFICATION_START_Y),
    ));
}

fn create_particle(
    position: Vec2,
    direction: Vec2,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let particle_texture = asset_server.load("sprites/particle.png");
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: position.extend(0.0),
                rotation: Quat::from_rotation_z(PI * random::<f32>()),
                scale: SCALE,
            },
            texture: particle_texture,
            ..default()
        },
        Particle {
            speed: PARTICLE_START_SPEED,
            direction: direction.extend(0.0),
            rotation_speed: (random::<f32>() * PARTICLE_MAX_ROTATION + PARTICLE_ADD_ROTATION) * match random::<bool>() {
                true => 1.,
                false => -1.,
            }
        },
    ));
}

// systems
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(Scoreboard { score: 0, was_last_upgrade_good: true });
    commands.insert_resource(WorldChaos::default());
    commands.insert_resource(BorbUpgrades::default());

    let game_over_sound = asset_server.load("sounds/game_over.wav");
    commands.insert_resource(CollisionSound(game_over_sound));

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
    let middle_pipe = asset_server.load("sprites/middle_pipe.png");
    let x_diff = SCREEN_WIDTH_WITH_PIPE / NUM_PIPES as f32;
    for pipe_num in 0..NUM_PIPES {
        let x = -HALF_SCREEN_WIDTH_WITH_HALF_PIPE + x_diff * (pipe_num as f32 + 1.);
        let y = random_pipe_hole_y();
        commands
            .spawn((
                PipeParent(false, PipeType::Normal),
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
                parent.spawn(PipeBundle::new(Vec2 { x: 0., y: -HALF_PIPE_SPACE }, pipe.clone(), PipeSide::Bottom, PipeType::Normal));
                parent.spawn(PipeBundle::new(Vec2 { x: 0., y: HALF_PIPE_SPACE }, pipe.clone(), PipeSide::Top, PipeType::Normal));
                parent.spawn(PipeBundle::new(Vec2 { x: 0., y: 0. }, middle_pipe.clone(), PipeSide::Center, PipeType::Middle));
            });
    }
}

fn move_pipes(
    mut commands: Commands,
    mut query: Query<(&Children, &mut Transform, &mut PipeParent)>,
    mut pipe_query: Query<(&mut Visibility, &PipeType), (Without<PipeParent>, Without<Borb>)>,
    time: Res<Time>,
    mut score: ResMut<Scoreboard>,
    mut chaos: ResMut<WorldChaos>,
    mut upgrades: ResMut<BorbUpgrades>,
) {
    for (children, mut transform, mut pipe) in &mut query {
        transform.translation.x -= chaos.world_speed * time.delta_seconds();
        if transform.translation.x < -HALF_SCREEN_WIDTH_WITH_HALF_PIPE {
            if chaos.different_pipes && pipe.1 != PipeType::Middle && random::<f32>() < 0.2 {
                for child in children.iter() {
                    if let Ok((mut visibility, pipe_type)) = pipe_query.get_mut(*child) {
                        if pipe_type == &PipeType::Middle {
                            *visibility = Visibility::Visible;
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                    }
                }
                pipe.1 = PipeType::Middle;
                transform.translation.y = 0.;
            } else {
                for child in children.iter() {
                    if let Ok((mut visibility, pipe_type)) = pipe_query.get_mut(*child) {
                        if pipe_type == &PipeType::Normal {
                            *visibility = Visibility::Visible;
                        } else {
                            *visibility = Visibility::Hidden;
                        }
                    }
                }
                pipe.1 = PipeType::Normal;
                transform.translation.y = random_pipe_hole_y();
            }
            transform.translation.x = HALF_SCREEN_WIDTH_WITH_HALF_PIPE;
            pipe.0 = false
        } else if !pipe.0 && transform.translation.x < BORB_START_POS.x {
            score.add(1, &mut upgrades, &mut chaos, &mut commands);
            pipe.0 = true;
        }
    }
}

fn jump(
    mut commands: Commands,
    mut query: Query<(&mut Gravity, &Transform), With<Borb>>,
    keyboard_input: Res<Input<KeyCode>>,
    upgrades: Res<BorbUpgrades>,
    asset_server: Res<AssetServer>,
) {
    let (mut gravity, transform) = query.single_mut();
    if keyboard_input.just_pressed(KeyCode::Space) {
        gravity.0 = JUMP_FORCE;
        for i in 0..JUMP_PARTICLE_NUM {
            create_particle(
                transform.translation.truncate() + JUMP_PARTICLE_RELATIVE_START_POS + JUMP_PARTICLE_DISTANCE * i as f32,
                JUMP_PARTICLE_DIRECTION,
                &mut commands,
                &asset_server
            );
        }
    } else if upgrades.fast_fall && keyboard_input.just_pressed(KeyCode::ControlLeft) {
        gravity.0 = FAST_FALL_SPEED;
    }
}

fn apply_gravity(
    mut query: Query<(&mut Transform, &mut Gravity)>,
    time: Res<Time>,
    upgrades: Res<BorbUpgrades>,
) {
    for (mut transform, mut gravity) in &mut query {
        gravity.0 -= GRAVITY * time.delta_seconds();
        if upgrades.fast_fall {
            gravity.0 = gravity.0.max(FAST_FALL_SPEED);
        } else {
            gravity.0 = gravity.0.max(MAX_GRAVITY);
        }
        transform.translation.y += gravity.0 * time.delta_seconds();
    }
}

fn check_for_collisions(
    mut commands: Commands,
    borb_query: Query<(&Transform, &Collider), With<Borb>>,
    collider_query: Query<(&GlobalTransform, &Collider, &Obstacle, &Visibility)>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    sound: Res<CollisionSound>,
) {
    let (borb_transform, borb_collider) = borb_query.single();

    for (transform, collider, enemy, visible) in &collider_query {
        if visible == &Visibility::Visible {
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

fn update_notifications(
    mut text_query: Query<(Entity, &mut Style, &mut Text, &mut Notification)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut style, mut text, mut notif) in &mut text_query {
        notif.0 += NOTIFICATION_SPEED * time.delta_seconds();
        style.bottom = Val::Px(notif.0);
        let alpha = text.sections[0].style.color.a() - NOTIFICATION_ALPHA_SPEED * time.delta_seconds();
        if alpha <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            text.sections[0].style.color.set_a(alpha);
        }
    }
}

fn update_borb_rotation(
    mut borb_query: Query<(&mut Transform, &Gravity), With<Borb>>,
) {
    let (mut transform, gravity) = borb_query.single_mut();
    transform.rotation = Quat::from_rotation_z(gravity.0 * DEGREES_PER_GRAVITY);
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
    time: Res<Time>,
    chaos: Res<WorldChaos>,
) {
    for (entity, mut transform, mut sprite, mut particle) in &mut query {
        particle.speed -= PARTICLE_DECEL * time.delta_seconds();
        particle.speed = particle.speed.max(0.);
        transform.translation += particle.direction * particle.speed * time.delta_seconds();
        transform.translation.x -= chaos.world_speed * time.delta_seconds();
        transform.rotate_z(particle.speed * particle.rotation_speed * time.delta_seconds());
        let alpha = sprite.color.a() - PARTICLE_ALPHA_SPEED * time.delta_seconds();
        if alpha <= 0.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            sprite.color.set_a(alpha);
        }
    }
}