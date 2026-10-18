//! Интерфейс: HUD и экраны меню, паузы, конца партии.

use bevy::app::AppExit;
use bevy::prelude::*;

use super::config::GameConfig;
use super::resources::{BestScore, LastRound, RoundEnd, Score};
use super::state::{GameState, Pause};
use super::view::ViewConfig;

const TEXT: Color = Color::srgb(0.92, 0.94, 0.96);
const MUTED: Color = Color::srgb(0.60, 0.64, 0.70);
const ACCENT: Color = Color::srgb(0.55, 0.95, 0.45);
const DANGER: Color = Color::srgb(0.95, 0.40, 0.38);
const HUD_BG: Color = Color::srgb(0.09, 0.10, 0.13);
const OVERLAY_BG: Color = Color::srgba(0.03, 0.04, 0.05, 0.82);
const BUTTON: Color = Color::srgb(0.17, 0.19, 0.23);
const BUTTON_HOVER: Color = Color::srgb(0.24, 0.27, 0.32);
const BUTTON_PRESSED: Color = Color::srgb(0.30, 0.55, 0.32);

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFonts>()
            .add_systems(Startup, spawn_hud)
            .add_systems(OnEnter(GameState::Menu), spawn_menu)
            .add_systems(OnEnter(Pause::Paused), spawn_pause)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(Update, (update_hud, button_colors, button_actions));
    }
}

#[derive(Resource)]
struct UiFonts {
    regular: Handle<Font>,
    bold: Handle<Font>,
}

/// Встроенный шрифт Bevy содержит только латиницу, поэтому шрифт с кириллицей
/// лежит в `assets/fonts`.
///
/// Загружается при сборке плагина, а не в `PreStartup`: `OnEnter` начального
/// состояния (экран меню) выполняется раньше любой стартовой системы.
impl FromWorld for UiFonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            regular: assets.load("fonts/NotoSans-Regular.ttf"),
            bold: assets.load("fonts/NotoSans-Bold.ttf"),
        }
    }
}

fn text(fonts: &UiFonts, value: impl Into<String>, size: f32, color: Color, bold: bool) -> impl Bundle {
    let font = if bold { &fonts.bold } else { &fonts.regular };
    (
        Text::new(value),
        TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() },
        TextColor(color),
    )
}

// ---------- HUD ----------

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct LevelText;

#[derive(Component)]
struct BestText;

fn spawn_hud(mut commands: Commands, fonts: Res<UiFonts>, view: Res<ViewConfig>) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: px(view.hud_px),
            padding: UiRect::horizontal(px(16)),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(HUD_BG),
        children![
            (ScoreText, text(&fonts, "Счёт: 0", 20.0, TEXT, true)),
            (LevelText, text(&fonts, "Скорость: 1", 16.0, MUTED, false)),
            (BestText, text(&fonts, "Рекорд: 0", 20.0, ACCENT, true)),
        ],
    ));
}

fn update_hud(
    score: Res<Score>,
    best: Res<BestScore>,
    cfg: Res<GameConfig>,
    mut score_text: Query<&mut Text, (With<ScoreText>, Without<LevelText>, Without<BestText>)>,
    mut level_text: Query<&mut Text, (With<LevelText>, Without<BestText>)>,
    mut best_text: Query<&mut Text, With<BestText>>,
) {
    if score.is_changed() {
        for mut t in &mut score_text {
            **t = format!("Счёт: {}", score.0);
        }
        for mut t in &mut level_text {
            **t = format!("Скорость: {}", cfg.speed.level(score.0));
        }
    }
    if best.is_changed() {
        for mut t in &mut best_text {
            **t = format!("Рекорд: {}", best.0);
        }
    }
}

// ---------- Экраны ----------

/// Кнопка экрана и действие по нажатию.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    Play,
    Resume,
    ToMenu,
    Quit,
}

fn overlay(scope: impl Bundle) -> impl Bundle {
    (
        scope,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(12),
            ..default()
        },
        BackgroundColor(OVERLAY_BG),
        GlobalZIndex(10),
    )
}

fn button(fonts: &UiFonts, label: &str, action: UiAction) -> impl Bundle {
    (
        Button,
        action,
        Node {
            width: px(220),
            height: px(48),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(BUTTON),
        children![text(fonts, label, 20.0, TEXT, true)],
    )
}

fn hint(fonts: &UiFonts, value: &str) -> impl Bundle {
    text(fonts, value, 14.0, MUTED, false)
}

fn spawn_menu(mut commands: Commands, fonts: Res<UiFonts>) {
    commands.spawn((
        overlay(DespawnOnExit(GameState::Menu)),
        children![
            text(&fonts, "ЗМЕЙКА", 48.0, ACCENT, true),
            hint(&fonts, "Стрелки / WASD - поворот, P / Esc - пауза"),
            button(&fonts, "Играть", UiAction::Play),
            button(&fonts, "Выход", UiAction::Quit),
            hint(&fonts, "Enter - начать"),
        ],
    ));
}

fn spawn_pause(mut commands: Commands, fonts: Res<UiFonts>) {
    commands.spawn((
        overlay(DespawnOnExit(Pause::Paused)),
        children![
            text(&fonts, "Пауза", 40.0, TEXT, true),
            button(&fonts, "Продолжить", UiAction::Resume),
            button(&fonts, "В меню", UiAction::ToMenu),
            hint(&fonts, "P / Esc - продолжить, M - в меню"),
        ],
    ));
}

fn spawn_game_over(mut commands: Commands, fonts: Res<UiFonts>, score: Res<Score>, last: Res<LastRound>) {
    let (title, color, reason) = match last.0 {
        Some(RoundEnd::BoardFilled) => ("Победа!", ACCENT, "Змейка заняла всё поле"),
        Some(RoundEnd::HitWall) => ("Игра окончена", DANGER, "Столкновение со стеной"),
        Some(RoundEnd::HitSelf) => ("Игра окончена", DANGER, "Змейка укусила себя"),
        None => ("Игра окончена", DANGER, ""),
    };
    commands.spawn((
        overlay(DespawnOnExit(GameState::GameOver)),
        children![
            text(&fonts, title, 40.0, color, true),
            hint(&fonts, reason),
            text(&fonts, format!("Счёт: {}", score.0), 24.0, TEXT, false),
            button(&fonts, "Ещё раз", UiAction::Play),
            button(&fonts, "В меню", UiAction::ToMenu),
            hint(&fonts, "Enter - ещё раз, Esc - в меню"),
        ],
    ));
}

// ---------- Кнопки ----------

fn button_colors(mut buttons: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>) {
    for (interaction, mut bg) in &mut buttons {
        bg.0 = match interaction {
            Interaction::Pressed => BUTTON_PRESSED,
            Interaction::Hovered => BUTTON_HOVER,
            Interaction::None => BUTTON,
        };
    }
}

fn button_actions(
    buttons: Query<(&Interaction, &UiAction), Changed<Interaction>>,
    mut next_game: ResMut<NextState<GameState>>,
    // Pause - подсостояние: вне Playing ресурса нет.
    mut next_pause: Option<ResMut<NextState<Pause>>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            UiAction::Play => next_game.set(GameState::Playing),
            UiAction::Resume => {
                if let Some(next_pause) = next_pause.as_mut() {
                    next_pause.set(Pause::Running);
                }
            }
            UiAction::ToMenu => next_game.set(GameState::Menu),
            UiAction::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}
