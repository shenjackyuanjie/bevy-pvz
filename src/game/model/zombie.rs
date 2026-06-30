//! 僵尸通用身体与种类差异配件。

use bevy::prelude::*;

use super::{ModelPart, model_bounds, part};
use crate::game::catalog::ZombieKind;

mod armored;
mod boss;
mod mobility;
mod plant_like;
mod vehicles;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZombieModelDetail {
    Full,
    Simplified,
}

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
    zombie_model_parts_with_detail(kind, alpha, ZombieModelDetail::Full)
}

pub fn zombie_model_parts_with_detail(
    kind: ZombieKind,
    alpha: f32,
    detail: ZombieModelDetail,
) -> Vec<ModelPart> {
    let palette = ZombiePalette::default();
    let mut parts = match detail {
        ZombieModelDetail::Full => base_zombie_parts(palette, alpha),
        ZombieModelDetail::Simplified => simplified_zombie_parts(palette, alpha),
    };
    let base_bounds = model_bounds(&parts);
    let ground_y = base_bounds.center.y - base_bounds.half_size.y;
    let _ = armored::apply(kind, &mut parts, palette, alpha)
        || mobility::apply(kind, &mut parts, palette, alpha)
        || vehicles::apply(kind, &mut parts, palette, alpha)
        || boss::apply(kind, &mut parts, palette, alpha)
        || plant_like::apply(kind, &mut parts, palette, alpha);
    let bounds = model_bounds(&parts);
    let offset_y = ground_y - (bounds.center.y - bounds.half_size.y);
    for part in &mut parts {
        part.offset.y += offset_y;
    }
    if detail == ZombieModelDetail::Simplified {
        retain_key_model_parts(&mut parts, 7);
    }
    parts
}

fn retain_key_model_parts(parts: &mut Vec<ModelPart>, maximum_parts: usize) {
    fn retain_index(retained: &mut Vec<usize>, index: usize) {
        if !retained.contains(&index) {
            retained.push(index);
        }
    }

    if parts.len() <= maximum_parts {
        return;
    }

    let rotated_half_size = |part: &ModelPart| {
        let half_size = part.size * 0.5;
        let (sin, cos) = part.rotation.sin_cos();
        Vec2::new(
            cos.abs() * half_size.x + sin.abs() * half_size.y,
            sin.abs() * half_size.x + cos.abs() * half_size.y,
        )
    };
    let mut retained = Vec::with_capacity(maximum_parts);

    let extrema = [
        parts.iter().enumerate().min_by(|(_, left), (_, right)| {
            let left_edge = left.offset.x - rotated_half_size(left).x;
            let right_edge = right.offset.x - rotated_half_size(right).x;
            left_edge.total_cmp(&right_edge)
        }),
        parts.iter().enumerate().max_by(|(_, left), (_, right)| {
            let left_edge = left.offset.x + rotated_half_size(left).x;
            let right_edge = right.offset.x + rotated_half_size(right).x;
            left_edge.total_cmp(&right_edge)
        }),
        parts.iter().enumerate().min_by(|(_, left), (_, right)| {
            let left_edge = left.offset.y - rotated_half_size(left).y;
            let right_edge = right.offset.y - rotated_half_size(right).y;
            left_edge.total_cmp(&right_edge)
        }),
        parts.iter().enumerate().max_by(|(_, left), (_, right)| {
            let left_edge = left.offset.y + rotated_half_size(left).y;
            let right_edge = right.offset.y + rotated_half_size(right).y;
            left_edge.total_cmp(&right_edge)
        }),
    ];
    for (index, _) in extrema.into_iter().flatten() {
        retain_index(&mut retained, index);
    }

    if let Some((index, _)) = parts
        .iter()
        .enumerate()
        .filter(|(_, part)| part.is_equipment)
        .max_by(|(_, left), (_, right)| {
            (left.size.x * left.size.y).total_cmp(&(right.size.x * right.size.y))
        })
    {
        retain_index(&mut retained, index);
    }

    let mut by_area: Vec<_> = (0..parts.len()).collect();
    by_area.sort_unstable_by(|left, right| {
        let left_area = parts[*left].size.x * parts[*left].size.y;
        let right_area = parts[*right].size.x * parts[*right].size.y;
        right_area.total_cmp(&left_area)
    });
    for index in by_area {
        if retained.len() == maximum_parts {
            break;
        }
        retain_index(&mut retained, index);
    }

    retained.sort_unstable();
    *parts = retained.into_iter().map(|index| parts[index]).collect();
}

fn simplified_zombie_parts(palette: ZombiePalette, alpha: f32) -> Vec<ModelPart> {
    let skin = palette.skin;
    let jacket = palette.jacket;
    let trousers = palette.trousers;
    let shoe = Color::srgb(0.07, 0.08, 0.07);
    let highlight = Color::srgb(0.73, 0.84, 0.56);
    vec![
        part(
            "僵尸下半身",
            trousers,
            Vec2::new(28.0, 34.0),
            Vec2::new(0.0, -24.0),
            -0.02,
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
            Vec2::new(36.0, 40.0),
            Vec2::new(0.0, 1.0),
            -0.05,
            0.2,
            alpha,
        ),
        part(
            "僵尸胸前高光",
            highlight,
            Vec2::new(6.0, 22.0),
            Vec2::new(-12.0, 6.0),
            -0.12,
            0.28,
            alpha * 0.74,
        ),
        part(
            "僵尸左臂",
            skin,
            Vec2::new(10.0, 36.0),
            Vec2::new(-22.0, 3.0),
            -0.55,
            0.1,
            alpha,
        ),
        part(
            "僵尸右臂",
            skin,
            Vec2::new(10.0, 38.0),
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
            "僵尸眼带",
            Color::WHITE,
            Vec2::new(20.0, 8.0),
            Vec2::new(0.0, 33.0),
            0.0,
            0.4,
            alpha * 0.9,
        ),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn visual_bottom(kind: ZombieKind) -> f32 {
        let parts = zombie_model_parts(kind, 1.0);
        let bounds = model_bounds(&parts);
        bounds.center.y - bounds.half_size.y
    }

    #[test]
    fn every_zombie_model_aligns_with_the_ground() {
        let basic_bottom = visual_bottom(ZombieKind::Basic);
        for kind in ZombieKind::ALL {
            assert!(
                (visual_bottom(kind) - basic_bottom).abs() <= 0.01,
                "{kind:?} is not aligned with the ground"
            );
        }
    }

    fn visual_bottom_with_detail(kind: ZombieKind, detail: ZombieModelDetail) -> f32 {
        let parts = zombie_model_parts_with_detail(kind, 1.0, detail);
        let bounds = model_bounds(&parts);
        bounds.center.y - bounds.half_size.y
    }

    #[test]
    fn every_simplified_zombie_model_aligns_with_the_ground() {
        let basic_bottom =
            visual_bottom_with_detail(ZombieKind::Basic, ZombieModelDetail::Simplified);
        for kind in ZombieKind::ALL {
            assert!(
                (visual_bottom_with_detail(kind, ZombieModelDetail::Simplified) - basic_bottom)
                    .abs()
                    <= 0.01,
                "{kind:?} simplified model is not aligned with the ground"
            );
        }
    }

    #[test]
    fn simplified_zombie_models_reduce_part_count() {
        for kind in ZombieKind::ALL {
            let full = zombie_model_parts_with_detail(kind, 1.0, ZombieModelDetail::Full);
            let simplified =
                zombie_model_parts_with_detail(kind, 1.0, ZombieModelDetail::Simplified);
            assert!(simplified.len() <= 7, "{kind:?} kept too many model parts");
            assert!(
                simplified.len() <= full.len(),
                "{kind:?} simplified model added parts"
            );
            if full.len() > 7 {
                assert!(
                    simplified.len() < full.len(),
                    "{kind:?} simplified model did not reduce part count"
                );
            }
        }
    }

    #[test]
    fn simplified_zombie_models_keep_bounds_close() {
        for kind in ZombieKind::ALL {
            let full = model_bounds(&zombie_model_parts_with_detail(
                kind,
                1.0,
                ZombieModelDetail::Full,
            ));
            let simplified = model_bounds(&zombie_model_parts_with_detail(
                kind,
                1.0,
                ZombieModelDetail::Simplified,
            ));
            let delta_half = (full.half_size - simplified.half_size).abs();
            let delta_center = (full.center - simplified.center).abs();

            assert!(
                delta_half.x <= 8.0 && delta_half.y <= 8.0,
                "{kind:?} simplified bounds drift too far: {delta_half:?}"
            );
            assert!(
                delta_center.x <= 4.0 && delta_center.y <= 4.0,
                "{kind:?} simplified center drift too far: {delta_center:?}"
            );
        }
    }
}
