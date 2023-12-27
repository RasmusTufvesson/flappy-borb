use bevy::{
    prelude::*,
    window::{PresentMode, WindowTheme},
    core::FrameCount,
};

mod game;
mod menu;

const BACKGROUND_COLOR: Color = Color::rgb(0.157, 0.8, 0.875);
const TEXT_COLOR: Color = Color::rgb(0.188, 0.173, 0.18);

const SCALE: Vec3 = Vec3 { x: 3., y: 3., z: 1. };

const SCREEN_SIZE: Vec2 = Vec2 { x: 500., y: 300. };
const HALF_SCREEN_SIZE: Vec2 = Vec2 { x: SCREEN_SIZE.x / 2., y: SCREEN_SIZE.y / 2. };

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Game,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
enum GameOverCause {
    TooHigh,
    TooLow,
    HitPipe,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    MainMenu,
    GameOver(GameOverCause),
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Flappy Borb".into(),
                resolution: SCREEN_SIZE.into(),
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                visible: false,
                ..default()
            }),
            ..Default::default()
        }).set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, make_visible)
        .add_state::<GameState>()
        .add_state::<MenuState>()
        .add_plugins((game::GamePlugin, menu::MenuPlugin))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.single_mut().visible = true;
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}