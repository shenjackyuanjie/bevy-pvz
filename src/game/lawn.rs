//! 草坪网格系统
//!
//! 定义 PvZ 单路草坪、空中种植格的布局、坐标换算以及格子占用检测。
//!
//! 核心类型：
//! - [`LawnLayout`]：道路几何信息（列数、格子大小、原点坐标）
//! - [`GridCell`]：二维种植格坐标，也用作组件标记植物位置
//! - [`CellOccupancy`]：资源，记录每个格子被哪个实体占用

use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::level::LevelSetupSet;
use crate::game::state::{GameState, LevelEntity};

/// 草坪列数：12 列。
pub const LAWN_COLUMNS: u8 = 12;

/// 空中种植格所在的两行；底层草坪为第 0 行，第 1 行刻意留空。
pub const AIR_ROWS: [u8; 2] = [2, 3];

/// 草坪相对世界中心向左偏移 50 像素。
pub const LAWN_CENTER_X: f32 = -50.0;

/// 保留原五行草坪最下面一条道路的中心 Y 坐标。
pub const LAWN_PATH_Y: f32 = -180.0;

/// 草坪插件，初始化 [`LawnLayout`] 和 [`CellOccupancy`] 资源，
/// 并在进入 Playing 状态时绘制棋盘视觉格子。
pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LawnLayout>()
            .init_resource::<CellOccupancy>()
            .add_systems(
                OnEnter(GameState::Playing),
                draw_lawn_placeholders.after(LevelSetupSet::Reset),
            );
    }
}

/// 草坪棋盘布局资源。
///
/// 存储道路的列数、每个格子的尺寸以及左下角的世界坐标原点。
/// 提供 `world_to_cell`（世界坐标 → 格子坐标）和 `cell_center`（格子坐标 → 世界坐标）等换算方法。
#[derive(Resource, Debug, Clone)]
pub struct LawnLayout {
    /// 底层草坪列数（默认 12）。
    pub columns: u8,
    /// 每个格子的尺寸（默认 90×90 像素）。
    pub cell_size: Vec2,
    /// 棋盘左下角的世界坐标。
    pub origin: Vec2,
}

impl Default for LawnLayout {
    fn default() -> Self {
        let cell_size = Vec2::new(90.0, 90.0);
        Self {
            columns: LAWN_COLUMNS,
            cell_size,
            origin: Vec2::new(
                LAWN_CENTER_X - f32::from(LAWN_COLUMNS) * cell_size.x * 0.5,
                LAWN_PATH_Y - cell_size.y * 0.5,
            ),
        }
    }
}

impl LawnLayout {
    /// 将世界坐标转换为格子坐标。
    ///
    /// 返回 `Some(GridCell)` 如果坐标落在底层或六个空中格内，否则返回 `None`。
    pub fn world_to_cell(&self, world: Vec2) -> Option<GridCell> {
        let local = world - self.origin;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }
        let column = (local.x / self.cell_size.x).floor() as u8;
        let row = (local.y / self.cell_size.y).floor() as u8;
        let cell = GridCell { column, row };
        self.contains(cell).then_some(cell)
    }

    /// 计算指定格子的世界坐标中心点。
    pub fn cell_center(&self, cell: GridCell) -> Vec2 {
        self.origin
            + Vec2::new(
                (f32::from(cell.column) + 0.5) * self.cell_size.x,
                (f32::from(cell.row) + 0.5) * self.cell_size.y,
            )
    }

    /// 检查格子是否属于底层草坪或六个空中种植格。
    pub fn contains(&self, cell: GridCell) -> bool {
        if cell.column >= self.columns {
            return false;
        }
        if cell.row == 0 {
            return true;
        }
        AIR_ROWS.contains(&cell.row)
            && (cell.column == 0 || cell.column.saturating_add(2) >= self.columns)
    }

    /// 获取唯一道路的中心 Y 坐标。
    pub fn path_y(&self) -> f32 {
        self.origin.y + self.cell_size.y * 0.5
    }

    /// 棋盘右侧边界的 X 坐标。
    pub fn right(&self) -> f32 {
        self.origin.x + f32::from(self.columns) * self.cell_size.x
    }
}

/// 二维格子坐标组件，标记植物所在的列与行。
///
/// 也用于 [`CellOccupancy`] 的键，以及世界坐标 ↔ 格子坐标的换算。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GridCell {
    /// 列号（0 到 `columns - 1`）。
    pub column: u8,
    /// 行号；0 是僵尸道路，2 和 3 是空中种植位。
    pub row: u8,
}

impl GridCell {
    /// 是否位于僵尸行进的底层草坪。
    pub fn is_ground(self) -> bool {
        self.row == 0
    }

    /// 是否位于空中种植格所在的行。
    pub fn is_elevated(self) -> bool {
        AIR_ROWS.contains(&self.row)
    }
}

/// 格子占用资源，记录每个格子被哪个实体占用。
///
/// 键为 [`GridCell`]，值为实体 ID。
/// 放置植物时检查并设置占用，植物死亡后释放。
#[derive(Resource, Debug, Default)]
pub struct CellOccupancy(pub HashMap<GridCell, Entity>);

impl CellOccupancy {
    /// 检查指定格子是否可用（在棋盘范围内且未被占用）。
    pub fn is_available(&self, cell: GridCell, layout: &LawnLayout) -> bool {
        layout.contains(cell) && !self.0.contains_key(&cell)
    }
}

/// 内部标记组件，用于标识草坪视觉格子实体，便于调试。
#[derive(Component)]
struct LawnVisual;

/// 绘制底层道路、六个空中种植格，以及左侧的房子边界线。
///
/// 格子颜色为深浅交替的绿色，模拟原版 PvZ 草坪风格。
/// 房子边线是一条深红色竖线，表示僵尸突破即失败的边界。
fn draw_lawn_placeholders(mut commands: Commands, layout: Res<LawnLayout>) {
    for column in 0..layout.columns {
        let cell = GridCell { column, row: 0 };
        let color = if column % 2 == 0 {
            Color::srgb(0.24, 0.55, 0.22)
        } else {
            Color::srgb(0.29, 0.62, 0.25)
        };
        commands.spawn((
            Sprite::from_color(color, layout.cell_size - Vec2::splat(2.0)),
            Transform::from_translation(layout.cell_center(cell).extend(-10.0)),
            LawnVisual,
            LevelEntity,
            Name::new(format!("Lawn cell {column}")),
        ));
    }

    for column in [
        0,
        layout.columns.saturating_sub(2),
        layout.columns.saturating_sub(1),
    ] {
        for row in AIR_ROWS {
            let cell = GridCell { column, row };
            if !layout.contains(cell) {
                continue;
            }
            let color = if (column + row) % 2 == 0 {
                Color::srgb(0.36, 0.68, 0.31)
            } else {
                Color::srgb(0.31, 0.62, 0.27)
            };
            commands.spawn((
                Sprite::from_color(color, layout.cell_size - Vec2::splat(2.0)),
                Transform::from_translation(layout.cell_center(cell).extend(-10.0)),
                LawnVisual,
                LevelEntity,
                Name::new(format!("Air lawn cell {column}:{row}")),
            ));
        }
    }

    // 房子边线（深红色竖线），位于棋盘左侧边界外。
    // 僵尸到达此线即触发失败判定。
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.45, 0.18, 0.12),
            Vec2::new(18.0, layout.cell_size.y),
        ),
        Transform::from_xyz(layout.origin.x - 22.0, layout.path_y(), -5.0),
        LawnVisual,
        LevelEntity,
        Name::new("House breach line"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_world_round_trip() {
        let layout = LawnLayout::default();
        for column in 0..layout.columns {
            let cell = GridCell { column, row: 0 };
            assert_eq!(layout.world_to_cell(layout.cell_center(cell)), Some(cell));
        }
        for column in [0, layout.columns - 2, layout.columns - 1] {
            for row in AIR_ROWS {
                let cell = GridCell { column, row };
                assert_eq!(layout.world_to_cell(layout.cell_center(cell)), Some(cell));
            }
        }
    }

    #[test]
    fn grid_rejects_boundaries_and_occupancy() {
        let layout = LawnLayout::default();
        assert_eq!(layout.world_to_cell(layout.origin - Vec2::ONE), None);
        assert_eq!(
            layout.world_to_cell(
                layout.origin + layout.cell_size * Vec2::new(f32::from(layout.columns), 1.0)
            ),
            None
        );

        let mut occupancy = CellOccupancy::default();
        assert_eq!(
            layout.world_to_cell(layout.origin + layout.cell_size * Vec2::new(0.5, 1.5)),
            None
        );
        assert_eq!(
            layout.world_to_cell(layout.origin + layout.cell_size * Vec2::new(1.5, 2.5)),
            None
        );

        let cell = GridCell { column: 3, row: 0 };
        assert!(occupancy.is_available(cell, &layout));
        occupancy.0.insert(cell, Entity::PLACEHOLDER);
        assert!(!occupancy.is_available(cell, &layout));
    }
}
