//! 由简单色块组成的单位轮廓，供场景实体和拖拽预览共用。

use bevy::prelude::*;

mod plant;
mod zombie;

pub use plant::plant_model_parts;
pub use zombie::zombie_model_parts;

#[derive(Debug, Clone, Copy)]
pub struct ModelPart {
    pub color: Color,
    pub size: Vec2,
    pub offset: Vec2,
    pub rotation: f32,
    pub z: f32,
    pub name: &'static str,
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
    }
}
