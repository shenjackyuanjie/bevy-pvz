//! 强类型内容目录：植物、僵尸与弹丸的唯一运行时定义来源。

use std::ops::RangeInclusive;
use std::time::Duration;

use bevy::prelude::*;

/// 单位的色块表现定义。
#[derive(Debug, Clone, Copy)]
pub struct UnitVisualDefinition {
    pub color: Color,
    pub size: Vec2,
}

/// 植物种类。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum PlantKind {
    Sunflower,
    Peashooter,
    WallNut,
}

impl PlantKind {
    pub const ALL: [Self; 3] = [Self::Sunflower, Self::Peashooter, Self::WallNut];
}

/// 植物行为参数；系统负责执行，目录只描述数据。
#[derive(Debug, Clone, Copy)]
pub enum PlantBehavior {
    SunProducer {
        interval: Duration,
        value: u32,
        spawn_offset: Vec2,
    },
    Shooter {
        interval: Duration,
        projectile: ProjectileKind,
        muzzle_offset: Vec2,
    },
    Blocker,
}

#[derive(Debug, Clone, Copy)]
pub struct PlantDefinition {
    pub kind: PlantKind,
    pub display_name: &'static str,
    pub price: u32,
    pub card_cooldown: Duration,
    pub health: f32,
    pub visual: UnitVisualDefinition,
    pub collider_half_size: Vec2,
    pub behavior: PlantBehavior,
}

/// 僵尸种类。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Deserialize)]
pub enum ZombieKind {
    Basic,
}

impl ZombieKind {
    pub const ALL: [Self; 1] = [Self::Basic];
}

#[derive(Debug, Clone)]
pub struct ZombieDefinition {
    pub kind: ZombieKind,
    pub display_name: &'static str,
    pub scene_label: &'static str,
    pub health: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub attack_interval: Duration,
    pub engage_range: RangeInclusive<f32>,
    pub spawn_offset_x: f32,
    pub visual: UnitVisualDefinition,
    pub collider_half_size: Vec2,
}

/// 弹丸种类。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ProjectileKind {
    Pea,
    PhysicsPea,
}

impl ProjectileKind {
    pub const ALL: [Self; 2] = [Self::Pea, Self::PhysicsPea];
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileVisualDefinition {
    pub color: Color,
    pub size: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub enum ProjectileMotionDefinition {
    Path {
        velocity: Vec2,
    },
    Physics {
        initial_velocity: Vec2,
        gravity_scale: f32,
        restitution: f32,
        friction: f32,
        ccd: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct HitPolicyDefinition {
    pub destroy_on_hit: bool,
    pub max_pierces: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileDefinition {
    pub kind: ProjectileKind,
    pub damage: f32,
    pub lifetime: Duration,
    pub visual: ProjectileVisualDefinition,
    pub radius: f32,
    pub motion: ProjectileMotionDefinition,
    pub hit_policy: HitPolicyDefinition,
}

/// 实体碰撞半尺寸。实际 Rapier 碰撞体和逻辑扫掠检测共同使用此值。
#[derive(Component, Debug, Clone, Copy)]
pub struct ColliderHalfSize(pub Vec2);

/// 内置的、已解析为强类型的内容目录。
#[derive(Resource, Debug, Clone)]
pub struct ContentCatalog {
    pub plants: Vec<PlantDefinition>,
    pub zombies: Vec<ZombieDefinition>,
    pub projectiles: Vec<ProjectileDefinition>,
}

impl Default for ContentCatalog {
    fn default() -> Self {
        Self {
            plants: vec![
                PlantDefinition {
                    kind: PlantKind::Sunflower,
                    display_name: "向日葵",
                    price: 50,
                    card_cooldown: Duration::from_secs(5),
                    health: 100.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.98, 0.72, 0.12),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::SunProducer {
                        interval: Duration::from_secs(7),
                        value: 25,
                        spawn_offset: Vec2::new(18.0, 24.0),
                    },
                },
                PlantDefinition {
                    kind: PlantKind::Peashooter,
                    display_name: "豌豆",
                    price: 100,
                    card_cooldown: Duration::from_secs(4),
                    health: 120.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.12, 0.72, 0.20),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Shooter {
                        interval: Duration::from_millis(1350),
                        projectile: ProjectileKind::Pea,
                        muzzle_offset: Vec2::new(36.0, 12.0),
                    },
                },
                PlantDefinition {
                    kind: PlantKind::WallNut,
                    display_name: "坚果",
                    price: 50,
                    card_cooldown: Duration::from_secs(8),
                    health: 600.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.55, 0.30, 0.12),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Blocker,
                },
            ],
            zombies: vec![ZombieDefinition {
                kind: ZombieKind::Basic,
                display_name: "普通僵尸",
                scene_label: "僵尸",
                health: 100.0,
                speed: 17.0,
                attack_damage: 20.0,
                attack_interval: Duration::from_secs(1),
                engage_range: -12.0..=62.0,
                spawn_offset_x: 75.0,
                visual: UnitVisualDefinition {
                    color: Color::srgb(0.42, 0.48, 0.38),
                    size: Vec2::new(58.0, 82.0),
                },
                collider_half_size: Vec2::new(29.0, 41.0),
            }],
            projectiles: vec![
                ProjectileDefinition {
                    kind: ProjectileKind::Pea,
                    damage: 20.0,
                    lifetime: Duration::from_secs(5),
                    visual: ProjectileVisualDefinition {
                        color: Color::srgb(0.28, 0.92, 0.22),
                        size: Vec2::splat(18.0),
                    },
                    radius: 9.0,
                    motion: ProjectileMotionDefinition::Path {
                        velocity: Vec2::new(430.0, 0.0),
                    },
                    hit_policy: HitPolicyDefinition {
                        destroy_on_hit: true,
                        max_pierces: 0,
                    },
                },
                ProjectileDefinition {
                    kind: ProjectileKind::PhysicsPea,
                    damage: 35.0,
                    lifetime: Duration::from_secs(8),
                    visual: ProjectileVisualDefinition {
                        color: Color::srgb(0.35, 0.85, 0.95),
                        size: Vec2::splat(18.0),
                    },
                    radius: 9.0,
                    motion: ProjectileMotionDefinition::Physics {
                        initial_velocity: Vec2::new(330.0, 300.0),
                        gravity_scale: 1.0,
                        restitution: 0.72,
                        friction: 0.35,
                        ccd: true,
                    },
                    hit_policy: HitPolicyDefinition {
                        destroy_on_hit: false,
                        max_pierces: u8::MAX,
                    },
                },
            ],
        }
    }
}

impl ContentCatalog {
    pub fn contains_plant(&self, kind: PlantKind) -> bool {
        self.plants.iter().any(|item| item.kind == kind)
    }

    pub fn contains_zombie(&self, kind: ZombieKind) -> bool {
        self.zombies.iter().any(|item| item.kind == kind)
    }

    pub fn plant(&self, kind: PlantKind) -> &PlantDefinition {
        self.plants
            .iter()
            .find(|item| item.kind == kind)
            .expect("validated plant kind")
    }

    pub fn zombie(&self, kind: ZombieKind) -> &ZombieDefinition {
        self.zombies
            .iter()
            .find(|item| item.kind == kind)
            .expect("validated zombie kind")
    }

    pub fn projectile(&self, kind: ProjectileKind) -> &ProjectileDefinition {
        self.projectiles
            .iter()
            .find(|item| item.kind == kind)
            .expect("validated projectile kind")
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_kinds(
            "plant",
            &PlantKind::ALL,
            self.plants.iter().map(|item| item.kind),
        )?;
        validate_kinds(
            "zombie",
            &ZombieKind::ALL,
            self.zombies.iter().map(|item| item.kind),
        )?;
        validate_kinds(
            "projectile",
            &ProjectileKind::ALL,
            self.projectiles.iter().map(|item| item.kind),
        )?;

        for plant in &self.plants {
            validate_positive("plant health", plant.health)?;
            validate_size("plant visual size", plant.visual.size)?;
            validate_size("plant collider", plant.collider_half_size)?;
            if let PlantBehavior::Shooter {
                projectile,
                interval,
                ..
            } = plant.behavior
            {
                if interval.is_zero() {
                    return Err("shooter interval must be positive".into());
                }
                self.projectiles
                    .iter()
                    .find(|item| item.kind == projectile)
                    .ok_or_else(|| {
                        format!(
                            "plant {:?} references missing projectile {:?}",
                            plant.kind, projectile
                        )
                    })?;
            }
            if let PlantBehavior::SunProducer {
                interval, value, ..
            } = plant.behavior
                && (interval.is_zero() || value == 0)
            {
                return Err("sun producer interval and value must be positive".into());
            }
        }
        for zombie in &self.zombies {
            validate_positive("zombie health", zombie.health)?;
            validate_positive("zombie speed", zombie.speed)?;
            validate_positive("zombie attack damage", zombie.attack_damage)?;
            validate_size("zombie visual size", zombie.visual.size)?;
            validate_size("zombie collider", zombie.collider_half_size)?;
            if zombie.attack_interval.is_zero() {
                return Err("zombie attack interval must be positive".into());
            }
            if zombie.engage_range.start() > zombie.engage_range.end() {
                return Err("zombie engage range is reversed".into());
            }
        }
        for projectile in &self.projectiles {
            validate_positive("projectile damage", projectile.damage)?;
            validate_positive("projectile radius", projectile.radius)?;
            validate_size("projectile visual size", projectile.visual.size)?;
            if projectile.lifetime.is_zero() {
                return Err("projectile lifetime must be positive".into());
            }
            match projectile.motion {
                ProjectileMotionDefinition::Path { velocity } => {
                    if !velocity.is_finite() || velocity == Vec2::ZERO {
                        return Err("path projectile velocity must be finite and non-zero".into());
                    }
                }
                ProjectileMotionDefinition::Physics {
                    initial_velocity,
                    gravity_scale,
                    restitution,
                    friction,
                    ..
                } => {
                    if !initial_velocity.is_finite()
                        || !gravity_scale.is_finite()
                        || !restitution.is_finite()
                        || restitution < 0.0
                        || !friction.is_finite()
                        || friction < 0.0
                    {
                        return Err("physics projectile motion contains invalid values".into());
                    }
                }
            }
        }
        Ok(())
    }
}

fn validate_kinds<T: Copy + Eq + std::fmt::Debug>(
    label: &str,
    expected: &[T],
    actual: impl Iterator<Item = T>,
) -> Result<(), String> {
    let actual: Vec<_> = actual.collect();
    for kind in expected {
        let count = actual.iter().filter(|item| *item == kind).count();
        if count != 1 {
            return Err(format!("{label} kind {kind:?} has {count} definitions"));
        }
    }
    if actual.len() != expected.len() {
        return Err(format!("{label} catalog contains unknown definitions"));
    }
    Ok(())
}

fn validate_positive(label: &str, value: f32) -> Result<(), String> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{label} must be finite and positive, got {value}"))
    }
}

fn validate_size(label: &str, value: Vec2) -> Result<(), String> {
    validate_positive(label, value.x)?;
    validate_positive(label, value.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_catalog_is_complete_and_valid() {
        ContentCatalog::default().validate().unwrap();
    }
}
