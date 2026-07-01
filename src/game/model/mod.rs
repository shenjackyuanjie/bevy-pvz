//! 由简单色块组成的单位轮廓，供场景实体和拖拽预览共用。

use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::*, render::render_resource::PrimitiveTopology,
};

mod plant;
mod zombie;

pub use plant::plant_model_parts;
pub use zombie::{ZombieModelDetail, zombie_model_parts, zombie_model_parts_with_detail};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelBounds {
    pub center: Vec2,
    pub half_size: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelPart {
    pub color: Color,
    pub size: Vec2,
    pub offset: Vec2,
    pub rotation: f32,
    pub z: f32,
    pub name: &'static str,
    pub is_equipment: bool,
}

/// Builds one vertex-colored mesh from multiple flat shapes.
///
/// Keeping the colors in vertices lets all model parts share one material and
/// one render entity while preserving the existing layered appearance.
pub(crate) struct ColoredMeshBuilder {
    positions: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl ColoredMeshBuilder {
    pub(crate) fn new(shape_capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(shape_capacity * 4),
            colors: Vec::with_capacity(shape_capacity * 4),
            indices: Vec::with_capacity(shape_capacity * 6),
        }
    }

    pub(crate) fn add_rectangle(
        &mut self,
        size: Vec2,
        offset: Vec2,
        rotation: f32,
        z: f32,
        color: Color,
    ) {
        let first = self.positions.len() as u32;
        let half = size * 0.5;
        let (sin, cos) = rotation.sin_cos();
        for corner in [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ] {
            let rotated = Vec2::new(
                corner.x * cos - corner.y * sin,
                corner.x * sin + corner.y * cos,
            );
            let position = offset + rotated;
            self.positions.push([position.x, position.y, z]);
        }
        self.push_color(color, 4);
        self.indices
            .extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
    }

    pub(crate) fn add_circle(
        &mut self,
        radius: f32,
        offset: Vec2,
        z: f32,
        color: Color,
        segments: u32,
    ) {
        let segments = segments.max(3);
        let first = self.positions.len() as u32;
        self.positions.push([offset.x, offset.y, z]);
        for index in 0..segments {
            let angle = std::f32::consts::TAU * index as f32 / segments as f32;
            let (sin, cos) = angle.sin_cos();
            self.positions
                .push([offset.x + cos * radius, offset.y + sin * radius, z]);
        }
        self.push_color(color, segments as usize + 1);
        for index in 0..segments {
            self.indices.extend_from_slice(&[
                first,
                first + 1 + index,
                first + 1 + (index + 1) % segments,
            ]);
        }
    }

    pub(crate) fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }

    fn push_color(&mut self, color: Color, count: usize) {
        let color = LinearRgba::from(color).to_f32_array();
        self.colors.extend(std::iter::repeat_n(color, count));
    }
}

pub(crate) fn model_parts_mesh(parts: &[ModelPart]) -> Mesh {
    let mut sorted_parts: Vec<_> = parts.iter().collect();
    sorted_parts.sort_by(|left, right| left.z.total_cmp(&right.z));

    let mut builder = ColoredMeshBuilder::new(parts.len());
    for part in sorted_parts {
        builder.add_rectangle(part.size, part.offset, part.rotation, part.z, part.color);
    }
    builder.build()
}

pub fn model_bounds(parts: &[ModelPart]) -> ModelBounds {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for part in parts {
        let half_size = part.size * 0.5;
        let (sin, cos) = part.rotation.sin_cos();
        let rotated_half_size = Vec2::new(
            cos.abs() * half_size.x + sin.abs() * half_size.y,
            sin.abs() * half_size.x + cos.abs() * half_size.y,
        );
        min = min.min(part.offset - rotated_half_size);
        max = max.max(part.offset + rotated_half_size);
    }

    ModelBounds {
        center: (min + max) * 0.5,
        half_size: (max - min) * 0.5,
    }
}

pub(super) fn part(
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
        is_equipment: false,
    }
}

pub(super) fn equipment_part(
    name: &'static str,
    color: Color,
    size: Vec2,
    offset: Vec2,
    rotation: f32,
    z: f32,
    alpha: f32,
) -> ModelPart {
    ModelPart {
        is_equipment: true,
        ..part(name, color, size, offset, rotation, z, alpha)
    }
}
