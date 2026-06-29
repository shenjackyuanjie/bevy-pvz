//! 跨系统玩法参数。可由草坪布局推导的边界不再散落为绝对坐标。

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct GameplaySettings {
    pub sun_pickup_radius: f32,
    pub defeat_offset_x: f32,
    pub physics_side_margins: Vec2,
    pub physics_boundary_thickness: f32,
    pub physics_wall_half_height: f32,
    pub physics_floor_half_width_scale: f32,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        Self {
            sun_pickup_radius: 28.0,
            defeat_offset_x: 16.0,
            physics_side_margins: Vec2::new(120.0, 220.0),
            physics_boundary_thickness: 10.0,
            physics_wall_half_height: 400.0,
            physics_floor_half_width_scale: 0.75,
        }
    }
}

impl GameplaySettings {
    pub fn validate(&self) -> Result<(), String> {
        let values = [
            ("sun_pickup_radius", self.sun_pickup_radius),
            ("defeat_offset_x", self.defeat_offset_x),
            ("physics_side_margins.x", self.physics_side_margins.x),
            ("physics_side_margins.y", self.physics_side_margins.y),
            (
                "physics_boundary_thickness",
                self.physics_boundary_thickness,
            ),
            ("physics_wall_half_height", self.physics_wall_half_height),
            (
                "physics_floor_half_width_scale",
                self.physics_floor_half_width_scale,
            ),
        ];
        for (name, value) in values {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "{name} must be finite and non-negative, got {value}"
                ));
            }
        }
        for (name, value) in [
            ("sun_pickup_radius", self.sun_pickup_radius),
            (
                "physics_boundary_thickness",
                self.physics_boundary_thickness,
            ),
            ("physics_wall_half_height", self.physics_wall_half_height),
            (
                "physics_floor_half_width_scale",
                self.physics_floor_half_width_scale,
            ),
        ] {
            if value == 0.0 {
                return Err(format!("{name} must be positive"));
            }
        }
        Ok(())
    }
}
