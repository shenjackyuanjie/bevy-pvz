//! Zombie appearance variants.
use super::super::{ModelPart, part};
use super::ZombiePalette;
use crate::game::catalog::ZombieKind;
use bevy::prelude::*;
pub(super) fn apply(
    kind: ZombieKind,
    parts: &mut Vec<ModelPart>,
    palette: ZombiePalette,
    alpha: f32,
) -> bool {
    let skin = palette.skin;
    let jacket = palette.jacket;
    match kind {
        ZombieKind::BobsledTeam => {
            parts.clear();
            parts.extend([
                part(
                    "雪橇",
                    Color::srgb(0.12, 0.28, 0.52),
                    Vec2::new(103.0, 18.0),
                    Vec2::new(0.0, -29.0),
                    0.0,
                    0.1,
                    alpha,
                ),
                part(
                    "雪橇前翘",
                    Color::srgb(0.18, 0.42, 0.72),
                    Vec2::new(24.0, 8.0),
                    Vec2::new(-51.0, -22.0),
                    0.45,
                    0.2,
                    alpha,
                ),
                part(
                    "雪橇队身体左",
                    Color::srgb(0.72, 0.08, 0.10),
                    Vec2::new(28.0, 33.0),
                    Vec2::new(-24.0, -3.0),
                    -0.05,
                    0.25,
                    alpha,
                ),
                part(
                    "雪橇队身体右",
                    Color::srgb(0.72, 0.08, 0.10),
                    Vec2::new(28.0, 33.0),
                    Vec2::new(20.0, -3.0),
                    0.05,
                    0.25,
                    alpha,
                ),
                part(
                    "part",
                    skin,
                    Vec2::new(26.0, 24.0),
                    Vec2::new(-24.0, 22.0),
                    0.04,
                    0.35,
                    alpha,
                ),
                part(
                    "part",
                    skin,
                    Vec2::new(26.0, 24.0),
                    Vec2::new(20.0, 22.0),
                    -0.04,
                    0.35,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.94, 0.94, 0.88),
                    Vec2::new(29.0, 10.0),
                    Vec2::new(-24.0, 34.0),
                    0.04,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.94, 0.94, 0.88),
                    Vec2::new(29.0, 10.0),
                    Vec2::new(20.0, 34.0),
                    -0.04,
                    0.45,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Zomboni => {
            parts.clear();
            parts.extend([
                part(
                    "冰车车体",
                    Color::srgb(0.30, 0.58, 0.76),
                    Vec2::new(88.0, 38.0),
                    Vec2::new(0.0, -12.0),
                    0.0,
                    0.2,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.50, 0.76, 0.86),
                    Vec2::new(45.0, 31.0),
                    Vec2::new(13.0, 18.0),
                    -0.08,
                    0.3,
                    alpha,
                ),
                part(
                    "冰车挡风玻璃",
                    Color::srgb(0.68, 0.90, 0.94),
                    Vec2::new(26.0, 20.0),
                    Vec2::new(23.0, 22.0),
                    -0.08,
                    0.4,
                    alpha * 0.78,
                ),
                part(
                    "part",
                    Color::srgb(0.13, 0.18, 0.20),
                    Vec2::new(24.0, 30.0),
                    Vec2::new(-38.0, -24.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "冰车后轮",
                    Color::srgb(0.13, 0.18, 0.20),
                    Vec2::new(22.0, 25.0),
                    Vec2::new(30.0, -24.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "冰车警灯",
                    Color::srgb(0.92, 0.12, 0.06),
                    Vec2::new(13.0, 8.0),
                    Vec2::new(7.0, 38.0),
                    0.0,
                    0.5,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Catapult => {
            parts.clear();
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.38, 0.22, 0.12),
                    Vec2::new(94.0, 30.0),
                    Vec2::new(0.0, -16.0),
                    0.0,
                    0.2,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.10, 0.09, 0.08),
                    Vec2::splat(26.0),
                    Vec2::new(-32.0, -34.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.10, 0.09, 0.08),
                    Vec2::splat(30.0),
                    Vec2::new(32.0, -34.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.62, 0.38, 0.18),
                    Vec2::new(10.0, 78.0),
                    Vec2::new(8.0, 13.0),
                    -0.55,
                    0.35,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.84, 0.22, 0.06),
                    Vec2::new(34.0, 9.0),
                    Vec2::new(-23.0, 50.0),
                    -0.2,
                    0.45,
                    alpha,
                ),
                part(
                    "篮球",
                    Color::srgb(0.90, 0.42, 0.08),
                    Vec2::splat(20.0),
                    Vec2::new(-27.0, 66.0),
                    0.0,
                    0.5,
                    alpha,
                ),
                part(
                    "part",
                    skin,
                    Vec2::new(28.0, 25.0),
                    Vec2::new(31.0, 18.0),
                    0.10,
                    0.4,
                    alpha,
                ),
                part(
                    "驾驶僵尸身体",
                    jacket,
                    Vec2::new(25.0, 29.0),
                    Vec2::new(31.0, -3.0),
                    0.0,
                    0.35,
                    alpha,
                ),
            ]);
            true
        }
        _ => false,
    }
}
