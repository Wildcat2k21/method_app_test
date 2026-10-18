//! Рекорд в файле: переживает перезапуск, не ломается от плохого файла.

use std::fs;
use std::path::PathBuf;

use snake_game::core::Cell;
use snake_game::game::{GameConfig, GameState};

use crate::common::{Harness, config};

/// Отдельная временная папка на тест; удаляется при выходе из теста.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("snake_lb6_{}_{name}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn file(&self) -> PathBuf {
        self.0.join("best_score.txt")
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn with_file(path: PathBuf) -> GameConfig {
    GameConfig { best_score_file: Some(path), ..config(20, 15) }
}

/// Партия, в которой съедено `eat` единиц еды (по ряду перед змейкой), затем стена.
fn play_round(h: &mut Harness, eat: i32) {
    h.start();
    h.ticks(11);
    h.frame();
    assert_eq!(h.state(), GameState::GameOver);
    assert_eq!(h.score(), eat as u32);
}

fn row_food(eat: i32) -> Vec<Cell> {
    (10..10 + eat).map(|x| Cell::new(x, 7)).chain([Cell::new(0, 0)]).collect()
}

#[test]
fn best_score_survives_restart() {
    let tmp = TempDir::new("restart");

    let mut first = Harness::new(with_file(tmp.file()), row_food(3));
    play_round(&mut first, 3);
    drop(first);
    assert_eq!(fs::read_to_string(tmp.file()).unwrap(), "3\n");

    let second = Harness::new(with_file(tmp.file()), []);
    assert_eq!(second.best(), 3, "второй запуск прочитал рекорд");
}

#[test]
fn lower_score_does_not_overwrite_record() {
    let tmp = TempDir::new("lower");
    fs::write(tmp.file(), "10\n").unwrap();

    let mut h = Harness::new(with_file(tmp.file()), row_food(2));
    play_round(&mut h, 2);

    assert_eq!(h.best(), 10);
    assert_eq!(fs::read_to_string(tmp.file()).unwrap(), "10\n");
}

#[test]
fn corrupted_file_counts_as_zero_and_is_repaired() {
    let tmp = TempDir::new("corrupted");
    fs::write(tmp.file(), "не число").unwrap();

    let mut h = Harness::new(with_file(tmp.file()), row_food(1));
    assert_eq!(h.best(), 0);
    play_round(&mut h, 1);

    assert_eq!(fs::read_to_string(tmp.file()).unwrap(), "1\n");
}

/// Папки нет - сохранить нельзя. Игра продолжает работать, рекорд остаётся в памяти.
#[test]
fn unwritable_location_does_not_crash() {
    let tmp = TempDir::new("unwritable");
    let path = tmp.0.join("нет_такой_папки").join("best_score.txt");

    let mut h = Harness::new(with_file(path.clone()), row_food(2));
    play_round(&mut h, 2);

    assert_eq!(h.best(), 2);
    assert!(!path.exists());
}
