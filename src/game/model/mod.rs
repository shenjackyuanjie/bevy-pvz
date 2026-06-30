//! 由简单色块组成的单位轮廓，供场景实体和拖拽预览共用。

use bevy::prelude::*;

mod plant;
mod zombie;

pub use plant::plant_model_parts;
pub use zombie::zombie_model_parts;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelBounds {
    pub center: Vec2,
    pub half_size: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelPart {
    pub color: Color,
    pub size: Vec2,
    pub offset: Vec2,
    pub rotation: f32,
    pub z: f32,
    pub name: &'static str,
    pub is_equipment: bool,
}

pub fn model_bounds(parts: &[ModelPart]) -> ModelBounds {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for part in parts {
        let half_size = part.size * 0.5;
        let (sin, cos) = part.rotation.sin_cos();
        let rotated_half_size = Vec2::new(
            cos.abs() * half_size.x + sin.abs() * half_size.y,
            sin.abs() * half_size.x + cos.abs() * half_size.y,
        );
        min = min.min(part.offset - rotated_half_size);
        max = max.max(part.offset + rotated_half_size);
    }

    ModelBounds {
        center: (min + max) * 0.5,
        half_size: (max - min) * 0.5,
    }
}

pub(super) fn part(
    name: &'static str,
    color: Color,
    size: Vec2,
    offset: Vec2,
    rotation: f32,
    z: f32,
    alpha: f32,
) -> ModelPart {
    ModelPart {
        color: color.with_alpha(alpha),
        size,
        offset,
        rotation,
        z,
        name,
        is_equipment: false,
    }
}

pub(super) fn equipment_part(
    name: &'static str,
    color: Color,
    size: Vec2,
    offset: Vec2,
    rotation: f32,
    z: f32,
    alpha: f32,
) -> ModelPart {
    ModelPart {
        is_equipment: true,
        ..part(name, color, size, offset, rotation, z, alpha)
    }
}
