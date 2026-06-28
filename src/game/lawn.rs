use std::collections::HashMap;

use bevy::prelude::*;

use super::state::{GameState, LevelEntity};

pub const LAWN_ROWS: u8 = 5;
pub const LAWN_COLUMNS: u8 = 9;

pub struct LawnPlugin;

impl Plugin for LawnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LawnLayout>()
            .init_resource::<CellOccupancy>()
            .add_systems(OnEnter(GameState::Playing), draw_lawn_placeholders);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct LawnLayout {
    pub rows: u8,
    pub columns: u8,
    pub cell_size: Vec2,
    /// Bottom-left corner of the board.
    pub origin: Vec2,
}

impl Default for LawnLayout {
    fn default() -> Self {
        let cell_size = Vec2::new(90.0, 90.0);
        Self {
            rows: LAWN_ROWS,
            columns: LAWN_COLUMNS,
            cell_size,
            origin: Vec2::new(-405.0, -225.0),
        }
    }
}

impl LawnLayout {
    pub fn world_to_cell(&self, world: Vec2) -> Option<GridCell> {
        let local = world - self.origin;
        if local.x < 0.0 || local.y < 0.0 {
            return None;
        }
        let column = (local.x / self.cell_size.x).floor() as u8;
        let row = (local.y / self.cell_size.y).floor() as u8;
        (row < self.rows && column < self.columns).then_some(GridCell { row, column })
    }

    pub fn cell_center(&self, cell: GridCell) -> Vec2 {
        self.origin
            + Vec2::new(
                (f32::from(cell.column) + 0.5) * self.cell_size.x,
                (f32::from(cell.row) + 0.5) * self.cell_size.y,
            )
    }

    pub fn lane_y(&self, lane: Lane) -> f32 {
        self.origin.y + (f32::from(lane.0) + 0.5) * self.cell_size.y
    }

    pub fn right(&self) -> f32 {
        self.origin.x + f32::from(self.columns) * self.cell_size.x
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct GridCell {
    pub row: u8,
    pub column: u8,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Lane(pub u8);

#[derive(Resource, Debug, Default)]
pub struct CellOccupancy(pub HashMap<GridCell, Entity>);

impl CellOccupancy {
    pub fn is_available(&self, cell: GridCell, layout: &LawnLayout) -> bool {
        cell.row < layout.rows && cell.column < layout.columns && !self.0.contains_key(&cell)
    }
}

#[derive(Component)]
struct LawnVisual;

fn draw_lawn_placeholders(mut commands: Commands, layout: Res<LawnLayout>) {
    for row in 0..layout.rows {
        for column in 0..layout.columns {
            let cell = GridCell { row, column };
            let color = if (row + column) % 2 == 0 {
                Color::srgb(0.24, 0.55, 0.22)
            } else {
                Color::srgb(0.29, 0.62, 0.25)
            };
            commands.spawn((
                Sprite::from_color(color, layout.cell_size - Vec2::splat(2.0)),
                Transform::from_translation(layout.cell_center(cell).extend(-10.0)),
                LawnVisual,
                LevelEntity,
                Name::new(format!("Lawn cell {row}:{column}")),
            ));
        }
    }

    // The house edge makes the loss boundary obvious.
    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.45, 0.18, 0.12),
            Vec2::new(18.0, layout.cell_size.y * f32::from(layout.rows)),
        ),
        Transform::from_xyz(layout.origin.x - 22.0, 0.0, -5.0),
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
        for row in 0..layout.rows {
            for column in 0..layout.columns {
                let cell = GridCell { row, column };
                assert_eq!(layout.world_to_cell(layout.cell_center(cell)), Some(cell));
            }
        }
    }

    #[test]
    fn grid_rejects_boundaries_and_occupancy() {
        let layout = LawnLayout::default();
        assert_eq!(layout.world_to_cell(layout.origin - Vec2::ONE), None);
        assert_eq!(
            layout.world_to_cell(layout.origin + layout.cell_size * Vec2::new(9.0, 5.0)),
            None
        );

        let mut occupancy = CellOccupancy::default();
        let cell = GridCell { row: 2, column: 3 };
        assert!(occupancy.is_available(cell, &layout));
        occupancy.0.insert(cell, Entity::PLACEHOLDER);
        assert!(!occupancy.is_available(cell, &layout));
    }
}
