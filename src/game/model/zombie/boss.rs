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
    let trousers = palette.trousers;
    match kind {
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
                    "part",
                    Color::srgb(0.34, 0.20, 0.10),
                    Vec2::new(13.0, 110.0),
                    Vec2::new(51.0, -5.0),
                    0.58,
                    0.15,
                    alpha,
                ),
                part(
                    "part",
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
                    "part",
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
            for part in parts.iter_mut() {
                part.offset.y += 18.0;
            }
            true
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
                    "part",
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
                    "part",
                    Color::srgb(0.14, 0.05, 0.04),
                    Vec2::new(12.0, 4.0),
                    Vec2::new(3.0, 17.0),
                    0.08,
                    0.4,
                    alpha,
                ),
            ]);
            true
        }
        _ => false,
    }
}
