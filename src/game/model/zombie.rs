//! 僵尸通用身体与种类差异配件。

use bevy::prelude::*;

use super::{ModelPart, part};
use crate::game::catalog::ZombieKind;

pub fn zombie_model_parts(kind: ZombieKind, alpha: f32) -> Vec<ModelPart> {
    let skin = Color::srgb(0.48, 0.58, 0.42);
    let jacket = Color::srgb(0.30, 0.24, 0.20);
    let trousers = Color::srgb(0.18, 0.25, 0.30);
    let mut parts = vec![
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
    ];

    match kind {
        ZombieKind::Conehead => {
            for (size, offset) in [
                (Vec2::new(42.0, 8.0), Vec2::new(0.0, 48.0)),
                (Vec2::new(30.0, 9.0), Vec2::new(0.0, 56.0)),
                (Vec2::new(18.0, 11.0), Vec2::new(0.0, 65.0)),
            ] {
                parts.push(part(
                    "路障头盔",
                    Color::srgb(0.96, 0.40, 0.06),
                    size,
                    offset,
                    0.08,
                    0.5,
                    alpha,
                ));
            }
        }
        ZombieKind::Buckethead => {
            parts.extend([
                part(
                    "铁桶",
                    Color::srgb(0.56, 0.60, 0.64),
                    Vec2::new(39.0, 29.0),
                    Vec2::new(0.0, 48.0),
                    0.06,
                    0.5,
                    alpha,
                ),
                part(
                    "铁桶边",
                    Color::srgb(0.78, 0.81, 0.84),
                    Vec2::new(45.0, 7.0),
                    Vec2::new(0.0, 35.0),
                    0.06,
                    0.6,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Football => {
            parts.extend([
                part(
                    "橄榄球护甲",
                    Color::srgb(0.62, 0.05, 0.05),
                    Vec2::new(46.0, 45.0),
                    Vec2::new(0.0, 2.0),
                    -0.04,
                    0.5,
                    alpha,
                ),
                part(
                    "左护肩",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(-24.0, 14.0),
                    -0.2,
                    0.6,
                    alpha,
                ),
                part(
                    "右护肩",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(24.0, 14.0),
                    0.2,
                    0.6,
                    alpha,
                ),
                part(
                    "橄榄球头盔",
                    Color::srgb(0.70, 0.06, 0.05),
                    Vec2::new(42.0, 28.0),
                    Vec2::new(0.0, 45.0),
                    0.05,
                    0.6,
                    alpha,
                ),
                part(
                    "头盔白条",
                    Color::srgb(0.92, 0.90, 0.80),
                    Vec2::new(7.0, 29.0),
                    Vec2::new(0.0, 45.0),
                    0.05,
                    0.7,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Newspaper => {
            parts.extend([
                part(
                    "报纸",
                    Color::srgb(0.82, 0.80, 0.70),
                    Vec2::new(48.0, 37.0),
                    Vec2::new(7.0, -1.0),
                    -0.08,
                    0.6,
                    alpha,
                ),
                part(
                    "报纸头条",
                    Color::srgb(0.15, 0.13, 0.11),
                    Vec2::new(35.0, 5.0),
                    Vec2::new(7.0, 9.0),
                    -0.08,
                    0.7,
                    alpha,
                ),
                part(
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
        }
        ZombieKind::ScreenDoor => {
            parts.extend([
                part(
                    "铁门网面",
                    Color::srgb(0.40, 0.43, 0.42),
                    Vec2::new(55.0, 72.0),
                    Vec2::new(18.0, 2.0),
                    -0.04,
                    0.55,
                    alpha * 0.68,
                ),
                part(
                    "铁门左框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(6.0, 76.0),
                    Vec2::new(-9.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
                part(
                    "铁门右框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(6.0, 76.0),
                    Vec2::new(45.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
                part(
                    "铁门横框",
                    Color::srgb(0.68, 0.70, 0.68),
                    Vec2::new(58.0, 6.0),
                    Vec2::new(18.0, 2.0),
                    -0.04,
                    0.7,
                    alpha,
                ),
            ]);
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
                    "冰车驾驶舱",
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
                    "冰车前滚轮",
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
        }
        _ => {}
    }
    parts
}
