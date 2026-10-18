//! Графика: клетки поля → пиксели, спрайты для сущностей логики.
//!
//! Логика спрайтов не создаёт. Этот плагин сам дополняет новые сегменты и еду
//! спрайтами и переводит [`GridPos`] в `Transform`.

use bevy::prelude::*;
use bevy::window::WindowResolution;

use crate::core::{Board, Cell};

use super::components::*;
use super::config::GameConfig;
use super::SnakeSet;

const BG: Color = Color::srgb(0.06, 0.07, 0.09);
const CELL_LIGHT: Color = Color::srgb(0.13, 0.15, 0.18);
const CELL_DARK: Color = Color::srgb(0.11, 0.13, 0.16);
const HEAD: Color = Color::srgb(0.55, 0.95, 0.45);
const BODY: Color = Color::srgb(0.25, 0.70, 0.32);
const FOOD: Color = Color::srgb(0.95, 0.32, 0.30);

const Z_BOARD: f32 = 0.0;
const Z_FOOD: f32 = 1.0;
const Z_SNAKE: f32 = 2.0;

/// Размеры в пикселях. Поле занимает окно под полосой HUD высотой `hud_px`.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewConfig {
    pub cell_px: f32,
    pub hud_px: f32,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self { cell_px: 28.0, hud_px: 48.0 }
    }
}

impl ViewConfig {
    pub fn window_size(&self, board: &Board) -> UVec2 {
        let w = board.width as f32 * self.cell_px;
        let h = board.height as f32 * self.cell_px + self.hud_px;
        UVec2::new(w as u32, h as u32)
    }

    /// Центр клетки в мировых координатах; камера смотрит в центр окна.
    pub fn cell_to_world(&self, board: &Board, cell: Cell) -> Vec2 {
        let x = (cell.x as f32 - (board.width - 1) as f32 / 2.0) * self.cell_px;
        let y = (cell.y as f32 - (board.height - 1) as f32 / 2.0) * self.cell_px - self.hud_px / 2.0;
        Vec2::new(x, y)
    }
}

/// Главное окно: размер точно под поле и HUD, без изменения размера.
pub fn primary_window(board: &Board, view: &ViewConfig) -> Window {
    let size = view.window_size(board);
    Window {
        title: "Змейка".into(),
        resolution: WindowResolution::new(size.x, size.y),
        resizable: false,
        ..default()
    }
}

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ViewConfig>()
            .insert_resource(ClearColor(BG))
            .add_systems(Startup, (spawn_camera, spawn_board))
            .add_systems(
                Update,
                (decorate_segments, decorate_food, sync_transforms).chain().after(SnakeSet::Outcome),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_board(mut commands: Commands, cfg: Res<GameConfig>, view: Res<ViewConfig>) {
    for cell in cfg.board.cells() {
        let color = if (cell.x + cell.y) % 2 == 0 { CELL_LIGHT } else { CELL_DARK };
        commands.spawn((
            Sprite::from_color(color, Vec2::splat(view.cell_px)),
            Transform::from_translation(view.cell_to_world(&cfg.board, cell).extend(Z_BOARD)),
        ));
    }
}

fn decorate_segments(
    mut commands: Commands,
    cfg: Res<GameConfig>,
    view: Res<ViewConfig>,
    added: Query<(Entity, &GridPos, Has<SnakeHead>), Added<SnakeSegment>>,
) {
    for (e, pos, is_head) in &added {
        let color = if is_head { HEAD } else { BODY };
        commands.entity(e).insert((
            Sprite::from_color(color, Vec2::splat(view.cell_px * 0.88)),
            Transform::from_translation(view.cell_to_world(&cfg.board, pos.0).extend(Z_SNAKE)),
        ));
    }
}

fn decorate_food(
    mut commands: Commands,
    cfg: Res<GameConfig>,
    view: Res<ViewConfig>,
    added: Query<(Entity, &GridPos), Added<Food>>,
) {
    for (e, pos) in &added {
        commands.entity(e).insert((
            Sprite::from_color(FOOD, Vec2::splat(view.cell_px * 0.6)),
            Transform::from_translation(view.cell_to_world(&cfg.board, pos.0).extend(Z_FOOD)),
        ));
    }
}

fn sync_transforms(
    cfg: Res<GameConfig>,
    view: Res<ViewConfig>,
    mut moved: Query<(&GridPos, &mut Transform), Changed<GridPos>>,
) {
    for (pos, mut transform) in &mut moved {
        let z = transform.translation.z;
        transform.translation = view.cell_to_world(&cfg.board, pos.0).extend(z);
    }
}
