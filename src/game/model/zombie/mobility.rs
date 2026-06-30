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
    match kind {
        ZombieKind::PoleVaulting => {
            parts.extend([
                part(
                    "撑杆夹克条纹",
                    Color::srgb(0.74, 0.13, 0.12),
                    Vec2::new(31.0, 7.0),
                    Vec2::new(1.0, 10.0),
                    -0.05,
                    0.45,
                    alpha,
                ),
                part(
                    "撑杆",
                    Color::srgb(0.62, 0.35, 0.12),
                    Vec2::new(7.0, 98.0),
                    Vec2::new(32.0, 6.0),
                    -0.18,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    skin,
                    Vec2::new(12.0, 10.0),
                    Vec2::new(23.0, 20.0),
                    -0.18,
                    0.55,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Dancing => {
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.05, 0.04, 0.035),
                    Vec2::new(48.0, 31.0),
                    Vec2::new(0.0, 51.0),
                    0.03,
                    0.22,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.56, 0.12, 0.66),
                    Vec2::new(39.0, 42.0),
                    Vec2::new(0.0, 0.0),
                    -0.05,
                    0.45,
                    alpha,
                ),
                part(
                    "舞王金链",
                    Color::srgb(1.0, 0.78, 0.12),
                    Vec2::new(24.0, 5.0),
                    Vec2::new(0.0, 16.0),
                    -0.05,
                    0.55,
                    alpha,
                ),
                part(
                    "舞王喇叭裤左",
                    Color::srgb(0.13, 0.11, 0.18),
                    Vec2::new(18.0, 18.0),
                    Vec2::new(-11.0, -42.0),
                    -0.08,
                    0.2,
                    alpha,
                ),
                part(
                    "舞王喇叭裤右",
                    Color::srgb(0.13, 0.11, 0.18),
                    Vec2::new(18.0, 18.0),
                    Vec2::new(10.0, -41.0),
                    0.08,
                    0.2,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::BackupDancer => {
            parts.extend([
                part(
                    "伴舞头带",
                    Color::srgb(0.95, 0.22, 0.30),
                    Vec2::new(35.0, 6.0),
                    Vec2::new(0.0, 42.0),
                    0.08,
                    0.5,
                    alpha,
                ),
                part(
                    "伴舞背心",
                    Color::srgb(0.16, 0.58, 0.72),
                    Vec2::new(36.0, 31.0),
                    Vec2::new(0.0, 4.0),
                    -0.05,
                    0.45,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.95, 0.22, 0.30),
                    Vec2::new(12.0, 5.0),
                    Vec2::new(-26.0, 10.0),
                    -0.55,
                    0.5,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.95, 0.22, 0.30),
                    Vec2::new(12.0, 5.0),
                    Vec2::new(27.0, 12.0),
                    0.68,
                    0.5,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Snorkel => {
            parts.extend([
                part(
                    "潜水面罩",
                    Color::srgb(0.10, 0.16, 0.18),
                    Vec2::new(35.0, 13.0),
                    Vec2::new(0.0, 34.0),
                    0.04,
                    0.55,
                    alpha * 0.9,
                ),
                part(
                    "潜水镜片",
                    Color::srgb(0.36, 0.76, 0.86),
                    Vec2::new(25.0, 8.0),
                    Vec2::new(0.0, 34.0),
                    0.04,
                    0.6,
                    alpha * 0.78,
                ),
                part(
                    "part",
                    Color::srgb(0.92, 0.32, 0.06),
                    Vec2::new(7.0, 41.0),
                    Vec2::new(22.0, 55.0),
                    0.05,
                    0.55,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.10, 0.32, 0.44),
                    Vec2::new(23.0, 8.0),
                    Vec2::new(-17.0, -43.0),
                    -0.18,
                    0.2,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.10, 0.32, 0.44),
                    Vec2::new(23.0, 8.0),
                    Vec2::new(17.0, -42.0),
                    0.18,
                    0.2,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::DolphinRider => {
            parts.extend([
                part(
                    "海豚身体",
                    Color::srgb(0.32, 0.60, 0.72),
                    Vec2::new(72.0, 22.0),
                    Vec2::new(7.0, -34.0),
                    0.0,
                    0.05,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.36, 0.68, 0.80),
                    Vec2::new(27.0, 19.0),
                    Vec2::new(-37.0, -28.0),
                    -0.10,
                    0.08,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.74, 0.90, 0.92),
                    Vec2::new(25.0, 6.0),
                    Vec2::new(-55.0, -27.0),
                    -0.05,
                    0.1,
                    alpha,
                ),
                part(
                    "海豚背鳍",
                    Color::srgb(0.24, 0.48, 0.62),
                    Vec2::new(14.0, 22.0),
                    Vec2::new(5.0, -16.0),
                    -0.28,
                    0.08,
                    alpha,
                ),
                part(
                    "泳裤",
                    Color::srgb(0.94, 0.20, 0.12),
                    Vec2::new(33.0, 13.0),
                    Vec2::new(0.0, -17.0),
                    -0.05,
                    0.45,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Balloon => {
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.78, 0.78, 0.72),
                    Vec2::new(3.0, 56.0),
                    Vec2::new(6.0, 73.0),
                    0.10,
                    0.05,
                    alpha,
                ),
                part(
                    "气球",
                    Color::srgb(0.78, 0.16, 0.78),
                    Vec2::new(42.0, 50.0),
                    Vec2::new(6.0, 106.0),
                    0.08,
                    0.1,
                    alpha,
                ),
                part(
                    "气球高光",
                    Color::srgb(0.96, 0.64, 0.96),
                    Vec2::new(10.0, 17.0),
                    Vec2::new(-8.0, 119.0),
                    0.18,
                    0.2,
                    alpha * 0.8,
                ),
                part(
                    "吊带",
                    Color::srgb(0.24, 0.18, 0.16),
                    Vec2::new(36.0, 5.0),
                    Vec2::new(0.0, 8.0),
                    0.0,
                    0.45,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Digger => {
            parts.extend([
                part(
                    "矿工头盔",
                    Color::srgb(0.88, 0.70, 0.16),
                    Vec2::new(42.0, 15.0),
                    Vec2::new(0.0, 45.0),
                    0.04,
                    0.5,
                    alpha,
                ),
                part(
                    "矿灯",
                    Color::srgb(1.0, 0.92, 0.42),
                    Vec2::new(12.0, 8.0),
                    Vec2::new(15.0, 47.0),
                    0.04,
                    0.6,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.47, 0.25, 0.10),
                    Vec2::new(8.0, 52.0),
                    Vec2::new(31.0, 6.0),
                    0.65,
                    0.55,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.64, 0.66, 0.63),
                    Vec2::new(34.0, 7.0),
                    Vec2::new(43.0, 29.0),
                    0.65,
                    0.65,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Pogo => {
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.66, 0.68, 0.70),
                    Vec2::new(7.0, 76.0),
                    Vec2::new(0.0, -27.0),
                    0.0,
                    0.05,
                    alpha,
                ),
                part(
                    "跳跳脚踏",
                    Color::srgb(0.16, 0.16, 0.14),
                    Vec2::new(46.0, 7.0),
                    Vec2::new(0.0, -20.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "跳跳把手",
                    Color::srgb(0.16, 0.16, 0.14),
                    Vec2::new(42.0, 7.0),
                    Vec2::new(0.0, 21.0),
                    0.0,
                    0.45,
                    alpha,
                ),
                part(
                    "弹簧底座",
                    Color::srgb(0.36, 0.36, 0.34),
                    Vec2::new(34.0, 9.0),
                    Vec2::new(0.0, -63.0),
                    0.0,
                    0.45,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Yeti => {
            parts.extend([
                part(
                    "雪人毛皮身体",
                    Color::srgb(0.82, 0.90, 0.88),
                    Vec2::new(47.0, 48.0),
                    Vec2::new(0.0, 1.0),
                    -0.04,
                    0.42,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.88, 0.95, 0.93),
                    Vec2::new(43.0, 35.0),
                    Vec2::new(0.0, 34.0),
                    0.05,
                    0.42,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.82, 0.90, 0.88),
                    Vec2::new(13.0, 48.0),
                    Vec2::new(-29.0, 0.0),
                    -0.28,
                    0.35,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.82, 0.90, 0.88),
                    Vec2::new(13.0, 48.0),
                    Vec2::new(30.0, 1.0),
                    0.32,
                    0.35,
                    alpha,
                ),
                part(
                    "雪人蓝脸",
                    Color::srgb(0.38, 0.62, 0.72),
                    Vec2::new(24.0, 15.0),
                    Vec2::new(2.0, 29.0),
                    0.04,
                    0.5,
                    alpha,
                ),
                part(
                    "雪人脚左",
                    Color::srgb(0.70, 0.84, 0.84),
                    Vec2::new(23.0, 10.0),
                    Vec2::new(-13.0, -47.0),
                    -0.05,
                    0.2,
                    alpha,
                ),
                part(
                    "雪人脚右",
                    Color::srgb(0.70, 0.84, 0.84),
                    Vec2::new(23.0, 10.0),
                    Vec2::new(14.0, -46.0),
                    0.05,
                    0.2,
                    alpha,
                ),
            ]);
            true
        }
        ZombieKind::Bungee => {
            parts.extend([
                part(
                    "part",
                    Color::srgb(0.08, 0.08, 0.08),
                    Vec2::new(5.0, 84.0),
                    Vec2::new(0.0, 76.0),
                    0.0,
                    0.05,
                    alpha,
                ),
                part(
                    "蹦极挂钩",
                    Color::srgb(0.72, 0.74, 0.72),
                    Vec2::new(22.0, 8.0),
                    Vec2::new(0.0, 121.0),
                    0.0,
                    0.1,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.10, 0.12, 0.14),
                    Vec2::new(32.0, 10.0),
                    Vec2::new(0.0, 34.0),
                    0.04,
                    0.55,
                    alpha,
                ),
                part(
                    "蹦极背带",
                    Color::srgb(0.92, 0.58, 0.08),
                    Vec2::new(9.0, 42.0),
                    Vec2::new(-8.0, 2.0),
                    0.20,
                    0.48,
                    alpha,
                ),
                part(
                    "part",
                    Color::srgb(0.92, 0.58, 0.08),
                    Vec2::new(9.0, 42.0),
                    Vec2::new(9.0, 2.0),
                    -0.20,
                    0.48,
                    alpha,
                ),
            ]);
            true
        }
        _ => false,
    }
}
