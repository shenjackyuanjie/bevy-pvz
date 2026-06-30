//! 植物组合色块模型。

use bevy::prelude::*;

use super::{ModelPart, part};
use crate::game::catalog::PlantKind;

pub fn plant_model_parts(kind: PlantKind, alpha: f32) -> Vec<ModelPart> {
    let green = Color::srgb(0.13, 0.70, 0.13);
    let dark_green = Color::srgb(0.03, 0.28, 0.04);
    let mut parts = vec![
        part(
            "植物茎",
            green,
            Vec2::new(9.0, 35.0),
            Vec2::new(0.0, -14.0),
            0.0,
            0.0,
            alpha,
        ),
        part(
            "左叶",
            green,
            Vec2::new(24.0, 9.0),
            Vec2::new(-10.0, -22.0),
            -0.38,
            0.1,
            alpha,
        ),
        part(
            "右叶",
            green,
            Vec2::new(24.0, 9.0),
            Vec2::new(10.0, -19.0),
            0.38,
            0.1,
            alpha,
        ),
    ];

    match kind {
        PlantKind::Sunflower | PlantKind::TwinSunflower => {
            if kind == PlantKind::TwinSunflower {
                parts.extend([
                    part(
                        "双头向日葵左分茎",
                        green,
                        Vec2::new(7.0, 27.0),
                        Vec2::new(-8.0, -2.0),
                        0.32,
                        0.05,
                        alpha,
                    ),
                    part(
                        "双头向日葵右分茎",
                        green,
                        Vec2::new(7.0, 27.0),
                        Vec2::new(9.0, -2.0),
                        -0.32,
                        0.05,
                        alpha,
                    ),
                ]);
                add_sunflower_head(&mut parts, Vec2::new(-12.0, 16.0), 0.72, alpha);
                add_sunflower_head(&mut parts, Vec2::new(13.0, 16.0), 0.72, alpha);
            } else {
                add_sunflower_head(&mut parts, Vec2::new(0.0, 15.0), 1.0, alpha);
            }
        }
        PlantKind::Peashooter
        | PlantKind::SnowPea
        | PlantKind::Repeater
        | PlantKind::GatlingPea => {
            let (head_color, barrel_color, rim_color, highlight_color, barrel_count) = match kind {
                PlantKind::Peashooter => (
                    Color::srgb(0.20, 0.88, 0.18),
                    Color::srgb(0.15, 0.74, 0.12),
                    dark_green,
                    Color::srgb(0.68, 1.0, 0.46),
                    1,
                ),
                PlantKind::SnowPea => (
                    Color::srgb(0.28, 0.86, 1.0),
                    Color::srgb(0.20, 0.70, 0.94),
                    Color::srgb(0.05, 0.28, 0.38),
                    Color::srgb(0.86, 1.0, 1.0),
                    1,
                ),
                PlantKind::Repeater => (
                    Color::srgb(0.10, 0.74, 0.14),
                    Color::srgb(0.07, 0.62, 0.09),
                    dark_green,
                    Color::srgb(0.54, 0.96, 0.34),
                    2,
                ),
                PlantKind::GatlingPea => (
                    Color::srgb(0.06, 0.56, 0.11),
                    Color::srgb(0.04, 0.44, 0.07),
                    Color::srgb(0.02, 0.18, 0.05),
                    Color::srgb(0.32, 0.78, 0.22),
                    4,
                ),
                _ => unreachable!(),
            };
            parts.push(part(
                "豌豆头部阴影",
                dark_green,
                Vec2::new(34.0, 28.0),
                Vec2::new(-4.0, 11.0),
                0.0,
                0.15,
                alpha,
            ));
            parts.push(part(
                "豌豆头",
                head_color,
                Vec2::new(32.0, 29.0),
                Vec2::new(-2.0, 14.0),
                0.0,
                0.2,
                alpha,
            ));
            let barrel_offsets: &[Vec2] = match barrel_count {
                1 => &[Vec2::new(21.0, 15.0)],
                2 => &[Vec2::new(19.0, 21.0), Vec2::new(22.0, 10.0)],
                _ => &[
                    Vec2::new(18.0, 24.0),
                    Vec2::new(25.0, 18.0),
                    Vec2::new(18.0, 10.0),
                    Vec2::new(25.0, 4.0),
                ],
            };
            for offset in barrel_offsets {
                parts.push(part(
                    "豌豆炮管",
                    barrel_color,
                    Vec2::new(23.0, 11.0),
                    *offset,
                    0.0,
                    0.3,
                    alpha,
                ));
                parts.push(part(
                    "豌豆炮口",
                    rim_color,
                    Vec2::new(5.0, 8.0),
                    *offset + Vec2::new(10.0, 0.0),
                    0.0,
                    0.4,
                    alpha,
                ));
            }
            parts.push(part(
                "豌豆眼睛",
                Color::WHITE,
                Vec2::splat(7.0),
                Vec2::new(2.0, 20.0),
                0.0,
                0.4,
                alpha,
            ));
            parts.push(part(
                "豌豆瞳孔",
                Color::srgb(0.02, 0.04, 0.01),
                Vec2::splat(3.0),
                Vec2::new(4.0, 20.0),
                0.0,
                0.5,
                alpha,
            ));
            parts.push(part(
                "豌豆头部高光",
                highlight_color,
                Vec2::new(9.0, 4.0),
                Vec2::new(-10.0, 24.0),
                -0.2,
                0.4,
                alpha * 0.72,
            ));
            if kind == PlantKind::SnowPea {
                parts.extend([
                    part(
                        "寒冰豌豆冰冠",
                        Color::srgb(0.82, 0.98, 1.0),
                        Vec2::new(15.0, 5.0),
                        Vec2::new(-6.0, 31.0),
                        -0.28,
                        0.45,
                        alpha * 0.88,
                    ),
                    part(
                        "寒冰豌豆冰刺",
                        Color::srgb(0.64, 0.90, 1.0),
                        Vec2::new(5.0, 13.0),
                        Vec2::new(7.0, 30.0),
                        0.55,
                        0.45,
                        alpha * 0.82,
                    ),
                ]);
            }
            if kind == PlantKind::Repeater || kind == PlantKind::GatlingPea {
                parts.push(part(
                    "豌豆加固颈叶",
                    dark_green,
                    Vec2::new(24.0, 7.0),
                    Vec2::new(-4.0, 0.0),
                    0.08,
                    0.18,
                    alpha * 0.82,
                ));
            }
            if kind == PlantKind::GatlingPea {
                parts.push(part(
                    "机枪豌豆炮箍",
                    Color::srgb(0.03, 0.22, 0.06),
                    Vec2::new(15.0, 32.0),
                    Vec2::new(18.0, 14.0),
                    0.0,
                    0.38,
                    alpha * 0.92,
                ));
            }
        }
        PlantKind::WallNut => {
            parts.clear();
            parts.extend([
                part(
                    "坚果身体",
                    Color::srgb(0.55, 0.30, 0.12),
                    Vec2::new(48.0, 62.0),
                    Vec2::new(0.0, -1.0),
                    0.0,
                    0.1,
                    alpha,
                ),
                part(
                    "坚果高光",
                    Color::srgb(0.70, 0.42, 0.18),
                    Vec2::new(7.0, 48.0),
                    Vec2::new(-17.0, 1.0),
                    0.0,
                    0.2,
                    alpha,
                ),
                part(
                    "坚果左眼",
                    Color::WHITE,
                    Vec2::new(8.0, 11.0),
                    Vec2::new(-9.0, 10.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "坚果右眼",
                    Color::WHITE,
                    Vec2::new(8.0, 11.0),
                    Vec2::new(9.0, 10.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "坚果嘴",
                    Color::srgb(0.20, 0.08, 0.03),
                    Vec2::new(18.0, 5.0),
                    Vec2::new(0.0, -12.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "坚果左瞳孔",
                    Color::srgb(0.08, 0.03, 0.01),
                    Vec2::new(3.0, 5.0),
                    Vec2::new(-8.0, 9.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "坚果右瞳孔",
                    Color::srgb(0.08, 0.03, 0.01),
                    Vec2::new(3.0, 5.0),
                    Vec2::new(10.0, 9.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "坚果裂纹",
                    Color::srgb(0.28, 0.11, 0.03),
                    Vec2::new(17.0, 3.0),
                    Vec2::new(8.0, -25.0),
                    0.55,
                    0.3,
                    alpha,
                ),
            ]);
        }
        PlantKind::Torchwood => {
            parts.clear();
            parts.extend([
                part(
                    "树桩身体",
                    Color::srgb(0.46, 0.21, 0.07),
                    Vec2::new(46.0, 58.0),
                    Vec2::new(0.0, -5.0),
                    0.0,
                    0.1,
                    alpha,
                ),
                part(
                    "树桩左边缘",
                    Color::srgb(0.29, 0.11, 0.03),
                    Vec2::new(7.0, 52.0),
                    Vec2::new(-20.0, -4.0),
                    -0.05,
                    0.2,
                    alpha,
                ),
                part(
                    "树桩右边缘",
                    Color::srgb(0.64, 0.33, 0.10),
                    Vec2::new(6.0, 48.0),
                    Vec2::new(19.0, -5.0),
                    0.04,
                    0.2,
                    alpha,
                ),
                part(
                    "树桩顶部",
                    Color::srgb(0.72, 0.39, 0.13),
                    Vec2::new(48.0, 12.0),
                    Vec2::new(0.0, 24.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "火焰外层",
                    Color::srgb(0.96, 0.20, 0.03),
                    Vec2::new(20.0, 28.0),
                    Vec2::new(0.0, 38.0),
                    0.0,
                    0.2,
                    alpha,
                ),
                part(
                    "火焰内层",
                    Color::srgb(1.0, 0.74, 0.08),
                    Vec2::new(10.0, 20.0),
                    Vec2::new(0.0, 35.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "树桩左眼",
                    Color::srgb(1.0, 0.86, 0.52),
                    Vec2::new(8.0, 10.0),
                    Vec2::new(-10.0, 5.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "树桩右眼",
                    Color::srgb(1.0, 0.86, 0.52),
                    Vec2::new(8.0, 10.0),
                    Vec2::new(10.0, 5.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "树桩嘴",
                    Color::srgb(0.13, 0.04, 0.01),
                    Vec2::new(18.0, 7.0),
                    Vec2::new(0.0, -10.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "树纹",
                    Color::srgb(0.31, 0.12, 0.03),
                    Vec2::new(22.0, 4.0),
                    Vec2::new(6.0, -25.0),
                    -0.12,
                    0.3,
                    alpha,
                ),
            ]);
        }
    }
    parts
}

fn add_sunflower_head(parts: &mut Vec<ModelPart>, flower_center: Vec2, scale: f32, alpha: f32) {
    let yellow = Color::srgb(1.0, 0.76, 0.08);
    for index in 0..8 {
        let rotation = index as f32 * std::f32::consts::FRAC_PI_4;
        let offset = flower_center + Vec2::from_angle(rotation) * 16.0 * scale;
        parts.push(part(
            "向日葵花瓣",
            yellow,
            Vec2::new(22.0, 10.0) * scale,
            offset,
            rotation,
            0.2,
            alpha,
        ));
    }
    parts.extend([
        part(
            "向日葵外花盘",
            Color::srgb(0.38, 0.16, 0.03),
            Vec2::splat(29.0 * scale),
            flower_center,
            0.0,
            0.3,
            alpha,
        ),
        part(
            "向日葵内花盘",
            Color::srgb(0.66, 0.34, 0.08),
            Vec2::splat(21.0 * scale),
            flower_center,
            0.0,
            0.4,
            alpha * 0.92,
        ),
        part(
            "向日葵左眼",
            Color::srgb(0.10, 0.04, 0.01),
            Vec2::new(3.0, 5.0) * scale,
            flower_center + Vec2::new(-5.0, 2.0) * scale,
            0.0,
            0.5,
            alpha,
        ),
        part(
            "向日葵右眼",
            Color::srgb(0.10, 0.04, 0.01),
            Vec2::new(3.0, 5.0) * scale,
            flower_center + Vec2::new(5.0, 2.0) * scale,
            0.0,
            0.5,
            alpha,
        ),
        part(
            "向日葵微笑",
            Color::srgb(0.18, 0.05, 0.01),
            Vec2::new(9.0, 3.0) * scale,
            flower_center + Vec2::new(0.0, -5.0) * scale,
            0.0,
            0.5,
            alpha,
        ),
    ]);
}
