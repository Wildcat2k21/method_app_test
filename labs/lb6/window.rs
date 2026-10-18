//! Окно и раскладка поля.

use snake_game::core::{Board, Cell};
use snake_game::game::view::{ViewConfig, primary_window};

#[test]
fn window_fits_board_and_hud_exactly() {
    let view = ViewConfig { cell_px: 28.0, hud_px: 48.0 };

    let window = primary_window(&Board::new(20, 15), &view);

    assert_eq!((window.resolution.width(), window.resolution.height()), (560.0, 468.0));
    assert!(!window.resizable, "размер окна фиксирован: поле не масштабируется");
    assert_eq!(window.title, "Змейка");
}

/// Каждая клетка поля целиком внутри окна и ниже полосы HUD.
#[test]
fn every_cell_is_inside_window_below_hud() {
    let view = ViewConfig::default();
    for board in [Board::new(20, 15), Board::new(7, 3), Board::new(1, 1), Board::new(40, 30)] {
        let size = view.window_size(&board).as_vec2();
        let (half_w, half_h) = (size.x / 2.0, size.y / 2.0);
        let half_cell = view.cell_px / 2.0;
        for cell in board.cells() {
            let p = view.cell_to_world(&board, cell);
            assert!(p.x - half_cell >= -half_w - 0.01 && p.x + half_cell <= half_w + 0.01, "{board:?} {cell:?}: x");
            assert!(p.y - half_cell >= -half_h - 0.01, "{board:?} {cell:?}: ниже окна");
            assert!(p.y + half_cell <= half_h - view.hud_px + 0.01, "{board:?} {cell:?}: залезает под HUD");
        }
    }
}

#[test]
fn neighbour_cells_are_one_cell_apart() {
    let view = ViewConfig::default();
    let board = Board::new(20, 15);
    let a = view.cell_to_world(&board, Cell::new(3, 4));
    let b = view.cell_to_world(&board, Cell::new(4, 5));
    assert_eq!(b - a, bevy::math::Vec2::splat(view.cell_px));
}
