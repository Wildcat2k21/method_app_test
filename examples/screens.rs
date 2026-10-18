//! Обход экранов игры со снимками в `docs/screens/`: меню, игра, пауза, конец партии.
//!
//! Запуск из папки snake_game: `cargo run --example screens`.
//! Во время игры змейкой управляет автопилот: жадно идёт к еде, не врезаясь.

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use snake_game::core::{Direction, rules};
use snake_game::game::view::{ViewConfig, primary_window};
use snake_game::game::*;

const OUT: &str = "docs/screens";

#[derive(Clone, Copy)]
enum Action {
    Shot(&'static str),
    Go(GameState),
    SetPause(Pause),
    Autopilot(bool),
    Exit,
}

/// Пауза перед шагом (с) и сам шаг.
const SCRIPT: &[(f32, Action)] = &[
    (1.0, Action::Shot("1_menu")),
    (0.2, Action::Go(GameState::Playing)),
    (0.0, Action::Autopilot(true)),
    (6.0, Action::Shot("2_playing")),
    (0.2, Action::SetPause(Pause::Paused)),
    (0.5, Action::Shot("3_pause")),
    (0.2, Action::SetPause(Pause::Running)),
    (0.0, Action::Autopilot(false)),
    (4.0, Action::Shot("4_game_over")),
    (1.0, Action::Exit),
];

#[derive(Resource)]
struct Script {
    step: usize,
    wait: Timer,
    autopilot: bool,
}

fn main() {
    std::fs::create_dir_all(OUT).expect("папка для снимков");
    let conf = GameConfig { seed: Some(7), ..default() };
    let view = ViewConfig::default();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(&conf.board, &view)),
            ..default()
        }))
        .insert_resource(conf)
        .insert_resource(view)
        .add_plugins(SnakeGamePlugin)
        .insert_resource(Script { step: 0, wait: Timer::from_seconds(SCRIPT[0].0, TimerMode::Once), autopilot: false })
        .add_systems(Update, (run_script, autopilot.before(SnakeSet::Tick)))
        .run();
}

fn run_script(
    mut commands: Commands,
    time: Res<Time>,
    mut script: ResMut<Script>,
    mut next_game: ResMut<NextState<GameState>>,
    next_pause: Option<ResMut<NextState<Pause>>>,
    mut exit: MessageWriter<AppExit>,
) {
    if script.step >= SCRIPT.len() || !script.wait.tick(time.delta()).is_finished() {
        return;
    }
    match SCRIPT[script.step].1 {
        Action::Shot(name) => {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(format!("{OUT}/{name}.png")));
        }
        Action::Go(state) => next_game.set(state),
        Action::SetPause(pause) => {
            if let Some(mut next_pause) = next_pause {
                next_pause.set(pause);
            }
        }
        Action::Autopilot(on) => script.autopilot = on,
        Action::Exit => {
            exit.write(AppExit::Success);
        }
    }
    script.step += 1;
    if let Some(&(delay, _)) = SCRIPT.get(script.step) {
        script.wait = Timer::from_seconds(delay, TimerMode::Once);
    }
}

/// Из безопасных направлений выбирает ближайшее к еде; при равенстве - текущее.
fn autopilot(
    script: Res<Script>,
    cfg: Res<GameConfig>,
    body: Res<SnakeBody>,
    segments: Query<&GridPos, With<SnakeSegment>>,
    food: Query<&GridPos, With<Food>>,
    mut heads: Query<&mut Heading, With<SnakeHead>>,
) {
    let (true, Ok(mut heading), Ok(food)) = (script.autopilot, heads.single_mut(), food.single()) else {
        return;
    };
    let cells: Vec<_> = body.0.iter().filter_map(|&e| segments.get(e).ok()).map(|p| p.0).collect();
    let Some(&head) = cells.first() else {
        return;
    };
    let current = heading.current;
    let distance = |d: Direction| {
        let next = head.step(d);
        (next.x - food.0.x).abs() + (next.y - food.0.y).abs()
    };
    let best = Direction::ALL
        .into_iter()
        .filter(|&d| d == current || rules::turn(current, d).is_some())
        .filter(|&d| rules::check_move(&cfg.board, &cells, head.step(d)).is_ok())
        .min_by_key(|&d| (distance(d), d != current));
    if let Some(dir) = best {
        heading.queued = (dir != current).then_some(dir);
    }
}
