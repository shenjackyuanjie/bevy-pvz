//! Zombie appearance variants.
use super::super::{ModelPart, equipment_part, part};
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
        ZombieKind::Flag => {
            parts.extend([
                part(
                    "旗杆",
                    Color::srgb(0.45, 0.25, 0.10),
                    Vec2::new(5.0, 78.0),
                    Vec2::new(29.0, 18.0),
                    -0.08,
                    0.45,
                    alpha,
                ),
                part(
                    "旗帜",
                    Color::srgb(0.82, 0.08, 0.06),
                    Vec2::new(38.0, 24.0),
                    Vec2::new(47.0, 48.0),
                    -0.08,
                    0.5,
                    alpha,
                ),
                part(
                    "旗帜图案",
                    Color::srgb(0.96, 0.86, 0.70),
                    Vec2::new(20.0, 5.0),
                    Vec2::new(48.0, 49.0),
                    -0.08,
                    0.6,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Conehead => {
            for (size, offset) in [
                (Vec2::new(42.0, 8.0), Vec2::new(0.0, 48.0)),
                (Vec2::new(30.0, 9.0), Vec2::new(0.0, 56.0)),
                (Vec2::new(18.0, 11.0), Vec2::new(0.0, 65.0)),
            ] {
                parts.push(equipment_part(
                    "路障头盔",
                    Color::srgb(0.96, 0.40, 0.06),
                    size,
                    offset,
                    0.08,
                    0.5,
                    alpha,
                ));
            }
            true
        }
        ZombieKind::Buckethead => {
            parts.extend([
                equipment_part(
                    "铁桶",
                    Color::srgb(0.56, 0.60, 0.64),
                    Vec2::new(39.0, 24.0),
                    Vec2::new(0.0, 53.0),
                    0.06,
                    0.36,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.78, 0.81, 0.84),
                    Vec2::new(45.0, 7.0),
                    Vec2::new(0.0, 43.0),
                    0.06,
                    0.37,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Football => {
            parts.extend([
                equipment_part(
                    "part",
                    Color::srgb(0.62, 0.05, 0.05),
                    Vec2::new(46.0, 45.0),
                    Vec2::new(0.0, 2.0),
                    -0.04,
                    0.5,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(-24.0, 14.0),
                    -0.2,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(24.0, 14.0),
                    0.2,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.70, 0.06, 0.05),
                    Vec2::new(42.0, 19.0),
                    Vec2::new(0.0, 51.0),
                    0.05,
                    0.36,
                    alpha,
                ),
                equipment_part(
                    "头盔白条",
                    Color::srgb(0.92, 0.90, 0.80),
                    Vec2::new(7.0, 19.0),
                    Vec2::new(0.0, 51.0),
                    0.05,
                    0.37,
                    alpha,
                ),
                equipment_part(
                    "面罩横杆",
                    Color::srgb(0.92, 0.90, 0.80),
                    Vec2::new(31.0, 3.0),
                    Vec2::new(1.0, 36.0),
                    0.04,
                    0.43,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Newspaper => {
            parts.extend([
                equipment_part(
                    "报纸",
                    Color::srgb(0.82, 0.80, 0.70),
                    Vec2::new(48.0, 37.0),
                    Vec2::new(7.0, -1.0),
                    -0.08,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "报纸头条",
                    Color::srgb(0.15, 0.13, 0.11),
                    Vec2::new(35.0, 5.0),
                    Vec2::new(7.0, 9.0),
                    -0.08,
                    0.7,
                    alpha,
                ),
                equipment_part(
                    "报纸正文",
                    Color::srgb(0.32, 0.29, 0.25),
                    Vec2::new(29.0, 3.0),
                    Vec2::new(7.0, 0.0),
                    -0.08,
                    0.7,
                    alpha,
                ),
                part(
                    "眼镜",
                    Color::srgb(0.12, 0.10, 0.08),
                    Vec2::new(24.0, 3.0),
                    Vec2::new(0.0, 34.0),
                    0.0,
                    0.6,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::ScreenDoor => {
            parts.extend([
                equipment_part(
                    "铁门网面",
                    Color::srgb(0.40, 0.43, 0.42),
                    Vec2::new(55.0, 72.0),
                    Vec2::new(18.0, 2.0),
                    -0.04,
                    0.55,
                    alpha * 0.68,
                ),
                equipment_part(
                    "铁门左框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(6.0, 76.0),
                    Vec2::new(-9.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
                equipment_part(
                    "铁门右框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(6.0, 76.0),
                    Vec2::new(45.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
                equipment_part(
                    "铁门横框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(58.0, 6.0),
                    Vec2::new(18.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::JackInTheBox => {
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.92, 0.12, 0.18),
                    Vec2::new(32.0, 14.0),
                    Vec2::new(0.0, 48.0),
                    0.18,
                    0.5,
                    alpha,
                ),
                part(
                    "小丑帽球",
                    Color::srgb(1.0, 0.86, 0.16),
                    Vec2::splat(10.0),
                    Vec2::new(13.0, 57.0),
                    0.0,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.74, 0.10, 0.15),
                    Vec2::new(34.0, 30.0),
                    Vec2::new(24.0, -5.0),
                    -0.1,
                    0.55,
                    alpha,
                ),
                equipment_part(
                    "玩偶盒盖",
                    Color::srgb(0.96, 0.74, 0.18),
                    Vec2::new(38.0, 7.0),
                    Vec2::new(24.0, 12.0),
                    -0.1,
                    0.65,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.16, 0.13, 0.10),
                    Vec2::new(18.0, 5.0),
                    Vec2::new(45.0, -3.0),
                    0.0,
                    0.7,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Ladder => {
            parts.extend([
                equipment_part(
                    "梯子左梁",
                    Color::srgb(0.62, 0.42, 0.20),
                    Vec2::new(7.0, 82.0),
                    Vec2::new(24.0, 1.0),
                    -0.14,
                    0.55,
                    alpha,
                ),
                equipment_part(
                    "梯子右梁",
                    Color::srgb(0.62, 0.42, 0.20),
                    Vec2::new(7.0, 82.0),
                    Vec2::new(54.0, 1.0),
                    -0.14,
                    0.55,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.74, 0.52, 0.28),
                    Vec2::new(38.0, 5.0),
                    Vec2::new(39.0, 23.0),
                    -0.14,
                    0.65,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.74, 0.52, 0.28),
                    Vec2::new(38.0, 5.0),
                    Vec2::new(39.0, 1.0),
                    -0.14,
                    0.65,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.74, 0.52, 0.28),
                    Vec2::new(38.0, 5.0),
                    Vec2::new(39.0, -21.0),
                    -0.14,
                    0.65,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::WallNutZombie => {
            parts.extend([
                equipment_part(
                    "part",
                    Color::srgb(0.55, 0.30, 0.12),
                    Vec2::new(58.0, 62.0),
                    Vec2::new(0.0, 5.0),
                    0.0,
                    0.48,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.72, 0.43, 0.18),
                    Vec2::new(7.0, 46.0),
                    Vec2::new(-20.0, 7.0),
                    0.0,
                    0.55,
                    alpha,
                ),
                equipment_part(
                    "坚果裂纹",
                    Color::srgb(0.26, 0.10, 0.04),
                    Vec2::new(18.0, 3.0),
                    Vec2::new(12.0, -20.0),
                    0.55,
                    0.6,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::TallNutZombie => {
            parts.extend([
                equipment_part(
                    "高坚果壳",
                    Color::srgb(0.58, 0.34, 0.14),
                    Vec2::new(62.0, 90.0),
                    Vec2::new(0.0, 12.0),
                    0.0,
                    0.48,
                    alpha,
                ),
                equipment_part(
                    "part",
                    Color::srgb(0.76, 0.47, 0.20),
                    Vec2::new(8.0, 70.0),
                    Vec2::new(-23.0, 14.0),
                    0.0,
                    0.55,
                    alpha,
                ),
                equipment_part(
                    "高坚果裂纹上",
                    Color::srgb(0.28, 0.12, 0.04),
                    Vec2::new(18.0, 3.0),
                    Vec2::new(14.0, 32.0),
                    0.45,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "高坚果裂纹下",
                    Color::srgb(0.28, 0.12, 0.04),
                    Vec2::new(22.0, 3.0),
                    Vec2::new(-4.0, -20.0),
                    -0.35,
                    0.6,
                    alpha,
                ),
            ]);
            true
        }
        _ => false,
    }
}
