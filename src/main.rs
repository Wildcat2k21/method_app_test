use bevy::prelude::*;

use snake_game::game::view::{ViewConfig, primary_window};
use snake_game::game::{GameConfig, SnakeGamePlugin};

fn main() {
    let conf = GameConfig {
        best_score_file: Some("best_score.txt".into()),
        ..default()
    };
    let view = ViewConfig::default();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(primary_window(&conf.board, &view)),
            ..default()
        }))
        .insert_resource(conf)
        .insert_resource(view)
        .add_plugins(SnakeGamePlugin)
        .run();
}
