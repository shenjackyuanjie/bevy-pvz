//! 僵尸通用身体与种类差异配件。

use bevy::prelude::*;

use super::{ModelPart, part};
use crate::game::catalog::ZombieKind;

mod armored;
mod boss;
mod mobility;
mod plant_like;
mod vehicles;

#[derive(Debug, Clone, Copy)]
pub(super) struct ZombiePalette {
    pub skin: Color,
    pub jacket: Color,
    pub trousers: Color,
}

impl Default for ZombiePalette {
    fn default() -> Self {
        Self {
            skin: Color::srgb(0.48, 0.58, 0.42),
            jacket: Color::srgb(0.30, 0.24, 0.20),
            trousers: Color::srgb(0.18, 0.25, 0.30),
        }
    }
}

pub fn zombie_model_parts(kind: ZombieKind, alpha: f32) -> Vec<ModelPart> {
    let palette = ZombiePalette::default();
    let mut parts = base_zombie_parts(palette, alpha);
    let _ = armored::apply(kind, &mut parts, palette, alpha)
        || mobility::apply(kind, &mut parts, palette, alpha)
        || vehicles::apply(kind, &mut parts, palette, alpha)
        || boss::apply(kind, &mut parts, palette, alpha)
        || plant_like::apply(kind, &mut parts, palette, alpha);
    parts
}

fn base_zombie_parts(palette: ZombiePalette, alpha: f32) -> Vec<ModelPart> {
    let skin = palette.skin;
    let jacket = palette.jacket;
    let trousers = palette.trousers;
    vec![
        part(
            "僵尸左腿",
            trousers,
            Vec2::new(12.0, 32.0),
            Vec2::new(-10.0, -25.0),
            -0.08,
            0.1,
            alpha,
        ),
        part(
            "僵尸右腿",
            trousers,
            Vec2::new(12.0, 32.0),
            Vec2::new(9.0, -24.0),
            0.10,
            0.1,
            alpha,
        ),
        part(
            "僵尸躯干",
            jacket,
            Vec2::new(34.0, 40.0),
            Vec2::new(0.0, 1.0),
            -0.05,
            0.2,
            alpha,
        ),
        part(
            "僵尸左臂",
            skin,
            Vec2::new(10.0, 37.0),
            Vec2::new(-22.0, 3.0),
            -0.55,
            0.1,
            alpha,
        ),
        part(
            "僵尸右臂",
            skin,
            Vec2::new(10.0, 39.0),
            Vec2::new(23.0, 5.0),
            0.68,
            0.1,
            alpha,
        ),
        part(
            "僵尸头",
            skin,
            Vec2::new(33.0, 30.0),
            Vec2::new(0.0, 29.0),
            0.08,
            0.3,
            alpha,
        ),
        part(
            "僵尸左眼",
            Color::WHITE,
            Vec2::new(7.0, 9.0),
            Vec2::new(-8.0, 33.0),
            0.0,
            0.4,
            alpha,
        ),
        part(
            "僵尸右眼",
            Color::WHITE,
            Vec2::new(7.0, 9.0),
            Vec2::new(7.0, 33.0),
            0.0,
            0.4,
            alpha,
        ),
        part(
            "僵尸嘴",
            Color::srgb(0.16, 0.06, 0.05),
            Vec2::new(17.0, 5.0),
            Vec2::new(3.0, 21.0),
            0.08,
            0.4,
            alpha,
        ),
    ]
}
