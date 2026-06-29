//! 草坪网格系统
//!
//! 定义 PvZ 单路草坪（9 个种植格）的布局、坐标换算以及格子占用检测。
//!
//! 核心类型：
//! - [`LawnLayout`]：道路几何信息（列数、格子大小、原点坐标）
//! - [`GridCell`]：一维种植格坐标，也用作组件标记植物位置
//! - [`CellOccupancy`]：资源，记录每个格子被哪个实体占用

use std::collections::HashMap;

use bevy::prelude::*;

use crate::game::level::LevelSetupSet;
use crate::game::state::{GameState, LevelEntity};

/// 草坪列数：9 列。
pub const LAWN_COLUMNS: u8 = 9;

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
    /// 列数（默认 9）。
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
            origin: Vec2::new(-405.0, LAWN_PATH_Y - cell_size.y * 0.5),
        }
    }
}

impl LawnLayout {
    /// 将世界坐标转换为格子坐标。
    ///
    /// 返回 `Some(GridCell)` 如果坐标在棋盘范围内，否则返回 `None`。
    pub fn world_to_cell(&self, world: Vec2) -> Option<GridCell> {
        let local = world - self.origin;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }
        let column = (local.x / self.cell_size.x).floor() as u8;
        (local.y < self.cell_size.y && column < self.columns).then_some(GridCell { column })
    }

    /// 计算指定格子的世界坐标中心点。
    pub fn cell_center(&self, cell: GridCell) -> Vec2 {
        self.origin
            + Vec2::new(
                (f32::from(cell.column) + 0.5) * self.cell_size.x,
                self.cell_size.y * 0.5,
            )
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

/// 一维格子坐标组件，标记植物所在的列。
///
/// 也用于 [`CellOccupancy`] 的键，以及世界坐标 ↔ 格子坐标的换算。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GridCell {
    /// 列号（0 到 LAWN_COLUMNS - 1）。
    pub column: u8,
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
        cell.column < layout.columns && !self.0.contains_key(&cell)
    }
}

/// 内部标记组件，用于标识草坪视觉格子实体，便于调试。
#[derive(Component)]
struct LawnVisual;

/// 在最下面的道路上绘制草地格子，以及左侧的房子边界线。
///
/// 格子颜色为深浅交替的绿色，模拟原版 PvZ 草坪风格。
/// 房子边线是一条深红色竖线，表示僵尸突破即失败的边界。
fn draw_lawn_placeholders(mut commands: Commands, layout: Res<LawnLayout>) {
    for column in 0..layout.columns {
        let cell = GridCell { column };
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
            let cell = GridCell { column };
            assert_eq!(layout.world_to_cell(layout.cell_center(cell)), Some(cell));
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
        let cell = GridCell { column: 3 };
        assert!(occupancy.is_available(cell, &layout));
        occupancy.0.insert(cell, Entity::PLACEHOLDER);
        assert!(!occupancy.is_available(cell, &layout));
    }
}
