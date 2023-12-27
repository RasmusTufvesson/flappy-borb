use bevy::{app::AppExit, prelude::*};
use crate::{despawn_screen, GameState, TEXT_COLOR, MenuState, GameOverCause};
use crate::game::Scoreboard;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(MenuState::MainMenu), main_menu_setup)
            .add_systems(OnExit(MenuState::MainMenu), despawn_screen::<OnMainMenuScreen>)

            .add_systems(OnEnter(MenuState::GameOver(GameOverCause::HitPipe)), game_over_menu_setup)
            .add_systems(OnEnter(MenuState::GameOver(GameOverCause::TooHigh)), game_over_menu_setup)
            .add_systems(OnEnter(MenuState::GameOver(GameOverCause::TooLow)), game_over_menu_setup)
            .add_systems(OnExit(MenuState::GameOver(GameOverCause::HitPipe)), despawn_screen::<OnGameOverMenuScreen>)
            .add_systems(OnExit(MenuState::GameOver(GameOverCause::TooHigh)), despawn_screen::<OnGameOverMenuScreen>)
            .add_systems(OnExit(MenuState::GameOver(GameOverCause::TooLow)), despawn_screen::<OnGameOverMenuScreen>)

            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenuScreen>)
            .add_systems(
                Update,
                (menu_action, button_system).run_if(in_state(GameState::Menu)),
            );
    }
}

#[derive(Component)]
struct OnMenuScreen;

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnGameOverMenuScreen;

const NORMAL_BUTTON: Color = Color::rgb(0.157, 0.8, 0.875);
const HOVERED_BUTTON: Color = Color::rgb(0.224, 0.278, 0.471);
// const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.224, 0.278, 0.471);
const PRESSED_BUTTON: Color = Color::rgb(0.157, 0.8, 0.875);
const MENU_BACKGROUND: Color = Color::rgb(0.224, 0.471, 0.659);

// All actions that can be triggered from a button click
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
    ToMainMenu,
}

// This system handles changing all buttons color based on mouse interaction
fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn main_menu_setup(mut commands: Commands) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(140.0),
        height: Val::Px(35.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 30.0,
        color: TEXT_COLOR,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnMainMenuScreen,
            OnMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: MENU_BACKGROUND.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "Flappy Borb",
                            TextStyle {
                                font_size: 40.0,
                                color: TEXT_COLOR,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );

                    // Display two buttons for each action available from the main menu:
                    // - new game
                    // - quit
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Play,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "New Game",
                                button_text_style.clone(),
                            ));
                        });
                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::Quit,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Quit", button_text_style));
                        });
                });
        });
}

fn game_over_menu_setup(
    mut commands: Commands,
    menu_state: Res<State<MenuState>>,
    score: Res<Scoreboard>,
) {
    let button_style = Style {
        width: Val::Px(140.0),
        height: Val::Px(35.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let text_style = TextStyle {
        font_size: 30.0,
        color: TEXT_COLOR,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnGameOverMenuScreen,
            OnMenuScreen,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: MENU_BACKGROUND.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(
                        TextBundle::from_section(
                            "Game Over",
                            TextStyle {
                                font_size: 40.0,
                                color: TEXT_COLOR,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );
                    
                    parent.spawn(TextBundle::from_section(
                        format!("{} score", score.0),
                        text_style.clone(),
                    ));

                    if let MenuState::GameOver(cause) = menu_state.get() {
                        parent.spawn(TextBundle::from_section(
                            match cause {
                                GameOverCause::HitPipe => "Hit by pipe",
                                GameOverCause::TooHigh => "Escaped to heaven",
                                GameOverCause::TooLow => "Fell down to hell",
                            },
                            text_style.clone(),
                        ));
                    }

                    parent
                        .spawn((
                            ButtonBundle {
                                style: button_style,
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            MenuButtonAction::ToMainMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section("Main Menu", text_style));
                        });
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
    mut menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                }
                MenuButtonAction::ToMainMenu => {
                    menu_state.set(MenuState::MainMenu);
                }
            }
        }
    }
}