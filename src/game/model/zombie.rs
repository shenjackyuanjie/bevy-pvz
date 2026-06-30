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
    let shirt = Color::srgb(0.82, 0.78, 0.64);
    let shoe = Color::srgb(0.07, 0.08, 0.07);
    let pupil = Color::srgb(0.04, 0.05, 0.04);
    let outline = Color::srgb(0.05, 0.07, 0.045);
    let highlight = Color::srgb(0.73, 0.84, 0.56);
    vec![
        part(
            "僵尸左腿轮廓",
            outline,
            Vec2::new(16.0, 36.0),
            Vec2::new(-10.0, -25.0),
            -0.08,
            0.05,
            alpha * 0.78,
        ),
        part(
            "僵尸右腿轮廓",
            outline,
            Vec2::new(16.0, 36.0),
            Vec2::new(9.0, -24.0),
            0.10,
            0.05,
            alpha * 0.78,
        ),
        part(
            "僵尸左臂轮廓",
            outline,
            Vec2::new(14.0, 41.0),
            Vec2::new(-22.0, 3.0),
            -0.55,
            0.05,
            alpha * 0.78,
        ),
        part(
            "僵尸右臂轮廓",
            outline,
            Vec2::new(14.0, 43.0),
            Vec2::new(23.0, 5.0),
            0.68,
            0.05,
            alpha * 0.78,
        ),
        part(
            "僵尸躯干轮廓",
            outline,
            Vec2::new(40.0, 46.0),
            Vec2::new(0.0, 1.0),
            -0.05,
            0.18,
            alpha * 0.76,
        ),
        part(
            "僵尸头轮廓",
            outline,
            Vec2::new(39.0, 36.0),
            Vec2::new(0.0, 29.0),
            0.08,
            0.28,
            alpha * 0.76,
        ),
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
            "僵尸左鞋",
            shoe,
            Vec2::new(20.0, 8.0),
            Vec2::new(-13.0, -43.0),
            -0.08,
            0.15,
            alpha,
        ),
        part(
            "僵尸右鞋",
            shoe,
            Vec2::new(20.0, 8.0),
            Vec2::new(13.0, -42.0),
            0.10,
            0.15,
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
            "僵尸衬衫",
            shirt,
            Vec2::new(15.0, 34.0),
            Vec2::new(2.0, 1.0),
            -0.05,
            0.31,
            alpha,
        ),
        part(
            "僵尸左衣领",
            Color::srgb(0.92, 0.88, 0.72),
            Vec2::new(14.0, 7.0),
            Vec2::new(-7.0, 19.0),
            0.48,
            0.32,
            alpha,
        ),
        part(
            "僵尸右衣领",
            Color::srgb(0.92, 0.88, 0.72),
            Vec2::new(14.0, 7.0),
            Vec2::new(9.0, 18.0),
            -0.56,
            0.32,
            alpha,
        ),
        part(
            "僵尸领带",
            Color::srgb(0.45, 0.06, 0.05),
            Vec2::new(7.0, 27.0),
            Vec2::new(3.0, -4.0),
            -0.04,
            0.33,
            alpha,
        ),
        part(
            "僵尸外套破边",
            Color::srgb(0.12, 0.10, 0.09),
            Vec2::new(9.0, 6.0),
            Vec2::new(-14.0, -20.0),
            -0.24,
            0.34,
            alpha,
        ),
        part(
            "僵尸肩部高光",
            highlight,
            Vec2::new(5.0, 30.0),
            Vec2::new(-15.0, 5.0),
            -0.12,
            0.35,
            alpha * 0.82,
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
            "僵尸头部高光",
            highlight,
            Vec2::new(5.0, 19.0),
            Vec2::new(-11.0, 33.0),
            0.13,
            0.46,
            alpha * 0.78,
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
            "僵尸左瞳孔",
            pupil,
            Vec2::new(3.0, 5.0),
            Vec2::new(-7.0, 32.0),
            0.0,
            0.45,
            alpha,
        ),
        part(
            "僵尸右瞳孔",
            pupil,
            Vec2::new(3.0, 5.0),
            Vec2::new(8.0, 32.0),
            0.0,
            0.45,
            alpha,
        ),
        part(
            "僵尸鼻子",
            Color::srgb(0.38, 0.48, 0.34),
            Vec2::new(5.0, 8.0),
            Vec2::new(2.0, 27.0),
            0.12,
            0.42,
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
        part(
            "僵尸牙齿",
            Color::srgb(0.92, 0.88, 0.74),
            Vec2::new(9.0, 3.0),
            Vec2::new(3.0, 22.0),
            0.08,
            0.45,
            alpha,
        ),
    ]
}
