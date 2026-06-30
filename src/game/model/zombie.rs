//! 僵尸通用身体与种类差异配件。

use bevy::prelude::*;

use super::{ModelPart, equipment_part, part};
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
        }
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
                    "撑杆手",
                    skin,
                    Vec2::new(12.0, 10.0),
                    Vec2::new(23.0, 20.0),
                    -0.18,
                    0.55,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Buckethead => {
            parts.extend([
                equipment_part(
                    "铁桶",
                    Color::srgb(0.56, 0.60, 0.64),
                    Vec2::new(39.0, 29.0),
                    Vec2::new(0.0, 48.0),
                    0.06,
                    0.5,
                    alpha,
                ),
                equipment_part(
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
                equipment_part(
                    "橄榄球护甲",
                    Color::srgb(0.62, 0.05, 0.05),
                    Vec2::new(46.0, 45.0),
                    Vec2::new(0.0, 2.0),
                    -0.04,
                    0.5,
                    alpha,
                ),
                equipment_part(
                    "左护肩",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(-24.0, 14.0),
                    -0.2,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "右护肩",
                    Color::srgb(0.82, 0.12, 0.08),
                    Vec2::new(20.0, 14.0),
                    Vec2::new(24.0, 14.0),
                    0.2,
                    0.6,
                    alpha,
                ),
                equipment_part(
                    "橄榄球头盔",
                    Color::srgb(0.70, 0.06, 0.05),
                    Vec2::new(42.0, 28.0),
                    Vec2::new(0.0, 45.0),
                    0.05,
                    0.6,
                    alpha,
                ),
                equipment_part(
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
        }
        ZombieKind::JackInTheBox => {
            parts.extend([
                part(
                    "小丑帽",
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
                    "玩偶盒",
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
                    "玩偶盒摇柄",
                    Color::srgb(0.16, 0.13, 0.10),
                    Vec2::new(18.0, 5.0),
                    Vec2::new(45.0, -3.0),
                    0.0,
                    0.7,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Balloon => {
            parts.extend([
                part(
                    "气球绳",
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
                    "矿镐柄",
                    Color::srgb(0.47, 0.25, 0.10),
                    Vec2::new(8.0, 52.0),
                    Vec2::new(31.0, 6.0),
                    0.65,
                    0.55,
                    alpha,
                ),
                part(
                    "矿镐头",
                    Color::srgb(0.64, 0.66, 0.63),
                    Vec2::new(34.0, 7.0),
                    Vec2::new(43.0, 29.0),
                    0.65,
                    0.65,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Pogo => {
            parts.extend([
                part(
                    "跳跳弹簧杆",
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
        ZombieKind::Catapult => {
            parts.clear();
            parts.extend([
                part(
                    "投篮车车体",
                    Color::srgb(0.38, 0.22, 0.12),
                    Vec2::new(94.0, 30.0),
                    Vec2::new(0.0, -16.0),
                    0.0,
                    0.2,
                    alpha,
                ),
                part(
                    "投篮车前轮",
                    Color::srgb(0.10, 0.09, 0.08),
                    Vec2::splat(26.0),
                    Vec2::new(-32.0, -34.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "投篮车后轮",
                    Color::srgb(0.10, 0.09, 0.08),
                    Vec2::splat(30.0),
                    Vec2::new(32.0, -34.0),
                    0.0,
                    0.3,
                    alpha,
                ),
                part(
                    "投篮臂",
                    Color::srgb(0.62, 0.38, 0.18),
                    Vec2::new(10.0, 78.0),
                    Vec2::new(8.0, 13.0),
                    -0.55,
                    0.35,
                    alpha,
                ),
                part(
                    "篮球筐",
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
                    "驾驶僵尸头",
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
        }
        ZombieKind::Gargantuar | ZombieKind::GigaGargantuar => {
            let giga = kind == ZombieKind::GigaGargantuar;
            let skin = if giga {
                Color::srgb(0.55, 0.28, 0.28)
            } else {
                Color::srgb(0.42, 0.53, 0.38)
            };
            let jacket = if giga {
                Color::srgb(0.38, 0.08, 0.08)
            } else {
                Color::srgb(0.30, 0.19, 0.16)
            };
            parts.clear();
            parts.extend([
                part(
                    "巨人左腿",
                    trousers,
                    Vec2::new(18.0, 48.0),
                    Vec2::new(-16.0, -41.0),
                    -0.05,
                    0.1,
                    alpha,
                ),
                part(
                    "巨人右腿",
                    trousers,
                    Vec2::new(18.0, 48.0),
                    Vec2::new(14.0, -39.0),
                    0.08,
                    0.1,
                    alpha,
                ),
                part(
                    "巨人躯干",
                    jacket,
                    Vec2::new(58.0, 68.0),
                    Vec2::new(0.0, 5.0),
                    -0.04,
                    0.2,
                    alpha,
                ),
                part(
                    "巨人左臂",
                    skin,
                    Vec2::new(16.0, 66.0),
                    Vec2::new(-40.0, 5.0),
                    -0.45,
                    0.2,
                    alpha,
                ),
                part(
                    "巨人右臂",
                    skin,
                    Vec2::new(18.0, 70.0),
                    Vec2::new(43.0, 7.0),
                    0.55,
                    0.2,
                    alpha,
                ),
                part(
                    "电线杆",
                    Color::srgb(0.34, 0.20, 0.10),
                    Vec2::new(13.0, 110.0),
                    Vec2::new(51.0, -5.0),
                    0.58,
                    0.15,
                    alpha,
                ),
                part(
                    "巨人头",
                    skin,
                    Vec2::new(44.0, 40.0),
                    Vec2::new(0.0, 56.0),
                    0.05,
                    0.3,
                    alpha,
                ),
                part(
                    "巨人眼睛",
                    if giga {
                        Color::srgb(1.0, 0.18, 0.08)
                    } else {
                        Color::WHITE
                    },
                    Vec2::new(19.0, 8.0),
                    Vec2::new(1.0, 62.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "巨人口",
                    Color::srgb(0.12, 0.04, 0.03),
                    Vec2::new(22.0, 6.0),
                    Vec2::new(5.0, 46.0),
                    0.05,
                    0.4,
                    alpha,
                ),
                part(
                    "背包小鬼",
                    Color::srgb(0.42, 0.52, 0.34),
                    Vec2::new(26.0, 34.0),
                    Vec2::new(-27.0, 30.0),
                    -0.2,
                    0.45,
                    alpha,
                ),
            ]);
        }
        ZombieKind::Imp | ZombieKind::IZombieImp => {
            parts.clear();
            parts.extend([
                part(
                    "小鬼左腿",
                    trousers,
                    Vec2::new(8.0, 19.0),
                    Vec2::new(-7.0, -18.0),
                    -0.08,
                    0.1,
                    alpha,
                ),
                part(
                    "小鬼右腿",
                    trousers,
                    Vec2::new(8.0, 19.0),
                    Vec2::new(7.0, -17.0),
                    0.10,
                    0.1,
                    alpha,
                ),
                part(
                    "小鬼身体",
                    jacket,
                    Vec2::new(24.0, 28.0),
                    Vec2::new(0.0, 0.0),
                    -0.04,
                    0.2,
                    alpha,
                ),
                part(
                    "小鬼左臂",
                    skin,
                    Vec2::new(7.0, 24.0),
                    Vec2::new(-16.0, 3.0),
                    -0.48,
                    0.2,
                    alpha,
                ),
                part(
                    "小鬼右臂",
                    skin,
                    Vec2::new(7.0, 25.0),
                    Vec2::new(17.0, 4.0),
                    0.58,
                    0.2,
                    alpha,
                ),
                part(
                    "小鬼头",
                    skin,
                    Vec2::new(24.0, 22.0),
                    Vec2::new(0.0, 24.0),
                    0.07,
                    0.3,
                    alpha,
                ),
                part(
                    "小鬼眼睛",
                    Color::WHITE,
                    Vec2::new(15.0, 6.0),
                    Vec2::new(1.0, 27.0),
                    0.0,
                    0.4,
                    alpha,
                ),
                part(
                    "小鬼嘴",
                    Color::srgb(0.14, 0.05, 0.04),
                    Vec2::new(12.0, 4.0),
                    Vec2::new(3.0, 17.0),
                    0.08,
                    0.4,
                    alpha,
                ),
            ]);
        }
        _ => {}
    }
    parts
}
