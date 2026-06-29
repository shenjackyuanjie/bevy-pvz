//! 由简单色块组成的单位轮廓，供场景实体和拖拽预览共用。

use bevy::prelude::*;

use crate::game::catalog::{PlantKind, ZombieKind};

#[derive(Debug, Clone, Copy)]
pub struct ModelPart {
    pub color: Color,
    pub size: Vec2,
    pub offset: Vec2,
    pub rotation: f32,
    pub z: f32,
    pub name: &'static str,
}

fn part(
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

pub fn plant_model_parts(kind: PlantKind, alpha: f32) -> Vec<ModelPart> {
    let green = Color::srgb(0.10, 0.58, 0.16);
    let dark_green = Color::srgb(0.05, 0.30, 0.08);
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
        PlantKind::Sunflower => {
            let yellow = Color::srgb(1.0, 0.76, 0.08);
            let flower_center = Vec2::new(0.0, 15.0);
            for index in 0..8 {
                let rotation = index as f32 * std::f32::consts::FRAC_PI_4;
                let offset = flower_center + Vec2::from_angle(rotation) * 16.0;
                parts.push(part(
                    "向日葵花瓣",
                    yellow,
                    Vec2::new(22.0, 10.0),
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
                    Vec2::splat(29.0),
                    flower_center,
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "向日葵内花盘",
                    Color::srgb(0.66, 0.34, 0.08),
                    Vec2::splat(21.0),
                    flower_center,
                    0.0,
                    0.4,
                    alpha * 0.92,
                ),
                part(
                    "向日葵左眼",
                    Color::srgb(0.10, 0.04, 0.01),
                    Vec2::new(3.0, 5.0),
                    flower_center + Vec2::new(-5.0, 2.0),
                    0.0,
                    0.5,
                    alpha,
                ),
                part(
                    "向日葵右眼",
                    Color::srgb(0.10, 0.04, 0.01),
                    Vec2::new(3.0, 5.0),
                    flower_center + Vec2::new(5.0, 2.0),
                    0.0,
                    0.5,
                    alpha,
                ),
                part(
                    "向日葵微笑",
                    Color::srgb(0.18, 0.05, 0.01),
                    Vec2::new(9.0, 3.0),
                    flower_center + Vec2::new(0.0, -5.0),
                    0.0,
                    0.5,
                    alpha,
                ),
            ]);
        }
        PlantKind::Peashooter | PlantKind::Repeater | PlantKind::GatlingPea => {
            let (head_color, barrel_count) = match kind {
                PlantKind::Peashooter => (Color::srgb(0.16, 0.72, 0.20), 1),
                PlantKind::Repeater => (Color::srgb(0.08, 0.58, 0.16), 2),
                PlantKind::GatlingPea => (Color::srgb(0.05, 0.42, 0.12), 4),
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
                    head_color,
                    Vec2::new(23.0, 11.0),
                    *offset,
                    0.0,
                    0.3,
                    alpha,
                ));
                parts.push(part(
                    "豌豆炮口",
                    dark_green,
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
                Color::srgb(0.42, 0.88, 0.35),
                Vec2::new(9.0, 4.0),
                Vec2::new(-10.0, 24.0),
                -0.2,
                0.4,
                alpha * 0.72,
            ));
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
        _ => {}
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_plant_has_a_composite_model() {
        for kind in PlantKind::ALL {
            assert!(plant_model_parts(kind, 1.0).len() >= 5, "{kind:?}");
        }
    }

    #[test]
    fn armored_zombies_have_visible_headgear() {
        let basic = zombie_model_parts(ZombieKind::Basic, 1.0);
        let cone = zombie_model_parts(ZombieKind::Conehead, 1.0);
        let bucket = zombie_model_parts(ZombieKind::Buckethead, 1.0);

        assert!(cone.len() > basic.len());
        assert!(bucket.len() > basic.len());
        assert!(cone.iter().any(|part| part.name == "路障头盔"));
        assert!(bucket.iter().any(|part| part.name == "铁桶"));
    }
}
