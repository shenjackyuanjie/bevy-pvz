//! Zombie appearance variants.
use super::super::{ModelPart, part};
use super::ZombiePalette;
use crate::game::catalog::ZombieKind;
use bevy::prelude::*;
pub(super) fn apply(
    kind: ZombieKind,
    parts: &mut Vec<ModelPart>,
    _palette: ZombiePalette,
    alpha: f32,
) -> bool {
    match kind {
        ZombieKind::PeashooterZombie => {
            parts.extend([
                part(
                    "豌豆僵尸头冠",
                    Color::srgb(0.16, 0.70, 0.18),
                    Vec2::new(34.0, 29.0),
                    Vec2::new(2.0, 47.0),
                    0.04,
                    0.36,
                    alpha,
                ),
                part(
                    "豌豆僵尸炮管",
                    Color::srgb(0.16, 0.70, 0.18),
                    Vec2::new(27.0, 12.0),
                    Vec2::new(25.0, 48.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "豌豆僵尸炮口",
                    Color::srgb(0.05, 0.30, 0.08),
                    Vec2::new(6.0, 9.0),
                    Vec2::new(38.0, 48.0),
                    0.0,
                    0.5,
                    alpha,
                ),
                part(
                    "豌豆僵尸叶领",
                    Color::srgb(0.08, 0.46, 0.10),
                    Vec2::new(34.0, 7.0),
                    Vec2::new(0.0, 20.0),
                    -0.05,
                    0.45,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::JalapenoZombie => {
            parts.extend([
                part(
                    "辣椒身体",
                    Color::srgb(0.86, 0.06, 0.04),
                    Vec2::new(31.0, 58.0),
                    Vec2::new(0.0, 4.0),
                    -0.08,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.96, 0.18, 0.06),
                    Vec2::new(20.0, 20.0),
                    Vec2::new(2.0, 44.0),
                    -0.08,
                    0.46,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.08, 0.42, 0.08),
                    Vec2::new(18.0, 8.0),
                    Vec2::new(-2.0, 59.0),
                    0.25,
                    0.5,
                    alpha,
                ),
                part(
                    "辣椒火花",
                    Color::srgb(1.0, 0.72, 0.05),
                    Vec2::new(14.0, 18.0),
                    Vec2::new(24.0, 17.0),
                    0.35,
                    0.55,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::GatlingPeaZombie => {
            parts.extend([
                part(
                    "机枪豌豆头冠",
                    Color::srgb(0.06, 0.45, 0.12),
                    Vec2::new(36.0, 30.0),
                    Vec2::new(0.0, 47.0),
                    0.04,
                    0.36,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.06, 0.45, 0.12),
                    Vec2::new(26.0, 8.0),
                    Vec2::new(25.0, 56.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.06, 0.45, 0.12),
                    Vec2::new(31.0, 8.0),
                    Vec2::new(28.0, 48.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.06, 0.45, 0.12),
                    Vec2::new(26.0, 8.0),
                    Vec2::new(25.0, 40.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "机枪弹鼓",
                    Color::srgb(0.04, 0.26, 0.08),
                    Vec2::new(18.0, 18.0),
                    Vec2::new(11.0, 31.0),
                    0.0,
                    0.48,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::SquashZombie => {
            parts.extend([
                part(
                    "窝瓜外壳",
                    Color::srgb(0.34, 0.60, 0.13),
                    Vec2::new(52.0, 48.0),
                    Vec2::new(0.0, 6.0),
                    -0.03,
                    0.45,
                    alpha,
                ),
                part(
                    "窝瓜顶部",
                    Color::srgb(0.48, 0.78, 0.20),
                    Vec2::new(37.0, 16.0),
                    Vec2::new(0.0, 35.0),
                    -0.03,
                    0.46,
                    alpha,
                ),
                part(
                    "窝瓜眼眉",
                    Color::srgb(0.08, 0.16, 0.04),
                    Vec2::new(34.0, 5.0),
                    Vec2::new(0.0, 24.0),
                    -0.03,
                    0.55,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.07, 0.10, 0.03),
                    Vec2::new(24.0, 6.0),
                    Vec2::new(4.0, 6.0),
                    -0.03,
                    0.55,
                    alpha,
                ),
            ]);
            true
        }
        _ => false,
    }
}
