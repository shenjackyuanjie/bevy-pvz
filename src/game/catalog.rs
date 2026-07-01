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
    TwinSunflower,
    Peashooter,
    SnowPea,
    Repeater,
    GatlingPea,
    WallNut,
    Torchwood,
}

impl PlantKind {
    pub const ALL: [Self; 8] = [
        Self::Sunflower,
        Self::TwinSunflower,
        Self::Peashooter,
        Self::SnowPea,
        Self::Repeater,
        Self::GatlingPea,
        Self::WallNut,
        Self::Torchwood,
    ];
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
        shots_per_burst: u8,
        burst_interval: Duration,
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
    Flag,
    Conehead,
    PoleVaulting,
    Buckethead,
    Newspaper,
    ScreenDoor,
    Football,
    Dancing,
    BackupDancer,
    Snorkel,
    Zomboni,
    BobsledTeam,
    DolphinRider,
    JackInTheBox,
    Balloon,
    Digger,
    Pogo,
    Yeti,
    Bungee,
    Ladder,
    Catapult,
    Gargantuar,
    GigaGargantuar,
    Imp,
    IZombieImp,
    PeashooterZombie,
    WallNutZombie,
    JalapenoZombie,
    GatlingPeaZombie,
    SquashZombie,
    TallNutZombie,
}

impl ZombieKind {
    pub const ALL: [Self; 32] = [
        Self::Basic,
        Self::Flag,
        Self::Conehead,
        Self::PoleVaulting,
        Self::Buckethead,
        Self::Newspaper,
        Self::ScreenDoor,
        Self::Football,
        Self::Dancing,
        Self::BackupDancer,
        Self::Snorkel,
        Self::Zomboni,
        Self::BobsledTeam,
        Self::DolphinRider,
        Self::JackInTheBox,
        Self::Balloon,
        Self::Digger,
        Self::Pogo,
        Self::Yeti,
        Self::Bungee,
        Self::Ladder,
        Self::Catapult,
        Self::Gargantuar,
        Self::GigaGargantuar,
        Self::Imp,
        Self::IZombieImp,
        Self::PeashooterZombie,
        Self::WallNutZombie,
        Self::JalapenoZombie,
        Self::GatlingPeaZombie,
        Self::SquashZombie,
        Self::TallNutZombie,
    ];
}

#[derive(Debug, Clone)]
pub struct ZombieDefinition {
    pub kind: ZombieKind,
    pub display_name: &'static str,
    pub scene_label: &'static str,
    pub health: f32,
    pub equipment_health: Option<f32>,
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
    IcePea,
    FirePea,
    PhysicsPea,
}

impl ProjectileKind {
    pub const ALL: [Self; 4] = [Self::Pea, Self::IcePea, Self::FirePea, Self::PhysicsPea];
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileVisualDefinition {
    pub fill_color: Color,
    pub border_color: Color,
    pub border_width: f32,
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
    pub visual: ProjectileVisualDefinition,
    pub radius: f32,
    pub motion: ProjectileMotionDefinition,
    pub hit_policy: HitPolicyDefinition,
}

/// 实体碰撞半尺寸。实际 Rapier 碰撞体和逻辑扫掠检测共同使用此值。
#[derive(Component, Debug, Clone, Copy)]
pub struct ColliderHalfSize(pub Vec2);

/// 碰撞箱中心相对逻辑实体原点的偏移。
#[derive(Component, Debug, Clone, Copy)]
pub struct ColliderCenterOffset(pub Vec2);

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
                    kind: PlantKind::TwinSunflower,
                    display_name: "双头向日葵",
                    price: 150,
                    card_cooldown: Duration::from_secs(7),
                    health: 120.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(1.0, 0.72, 0.10),
                        size: Vec2::new(68.0, 74.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::SunProducer {
                        interval: Duration::from_secs(7),
                        value: 50,
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
                        color: Color::srgb(0.20, 0.88, 0.18),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Shooter {
                        interval: Duration::from_millis(1350),
                        projectile: ProjectileKind::Pea,
                        muzzle_offset: Vec2::new(36.0, 12.0),
                        shots_per_burst: 1,
                        burst_interval: Duration::from_millis(150),
                    },
                },
                PlantDefinition {
                    kind: PlantKind::Repeater,
                    display_name: "多发射手",
                    price: 200,
                    card_cooldown: Duration::from_secs(5),
                    health: 120.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.10, 0.74, 0.14),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Shooter {
                        interval: Duration::from_millis(1350),
                        projectile: ProjectileKind::Pea,
                        muzzle_offset: Vec2::new(36.0, 12.0),
                        shots_per_burst: 2,
                        burst_interval: Duration::from_millis(150),
                    },
                },
                PlantDefinition {
                    kind: PlantKind::SnowPea,
                    display_name: "寒冰豌豆",
                    price: 175,
                    card_cooldown: Duration::from_secs(5),
                    health: 120.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.28, 0.86, 1.0),
                        size: Vec2::new(58.0, 68.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Shooter {
                        interval: Duration::from_millis(1350),
                        projectile: ProjectileKind::IcePea,
                        muzzle_offset: Vec2::new(36.0, 12.0),
                        shots_per_burst: 1,
                        burst_interval: Duration::from_millis(150),
                    },
                },
                PlantDefinition {
                    kind: PlantKind::GatlingPea,
                    display_name: "机枪射手",
                    price: 250,
                    card_cooldown: Duration::from_secs(7),
                    health: 140.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.06, 0.56, 0.11),
                        size: Vec2::new(62.0, 70.0),
                    },
                    collider_half_size: Vec2::new(29.0, 34.0),
                    behavior: PlantBehavior::Shooter {
                        interval: Duration::from_millis(1350),
                        projectile: ProjectileKind::Pea,
                        muzzle_offset: Vec2::new(38.0, 12.0),
                        shots_per_burst: 4,
                        burst_interval: Duration::from_millis(120),
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
                PlantDefinition {
                    kind: PlantKind::Torchwood,
                    display_name: "火炬树桩",
                    price: 175,
                    card_cooldown: Duration::from_secs(5),
                    health: 300.0,
                    visual: UnitVisualDefinition {
                        color: Color::srgb(0.48, 0.22, 0.07),
                        size: Vec2::new(58.0, 72.0),
                    },
                    collider_half_size: Vec2::new(29.0, 36.0),
                    behavior: PlantBehavior::Blocker,
                },
            ],
            zombies: [
                (ZombieKind::Basic, "普通僵尸", 200.0),
                (ZombieKind::Flag, "旗帜僵尸", 200.0),
                (ZombieKind::Conehead, "路障僵尸", 560.0),
                (ZombieKind::PoleVaulting, "撑杆僵尸", 340.0),
                (ZombieKind::Buckethead, "铁桶僵尸", 1300.0),
                (ZombieKind::Newspaper, "读报僵尸", 360.0),
                (ZombieKind::ScreenDoor, "铁门僵尸", 1300.0),
                (ZombieKind::Football, "橄榄球僵尸", 1600.0),
                (ZombieKind::Dancing, "舞王僵尸", 340.0),
                (ZombieKind::BackupDancer, "伴舞僵尸", 200.0),
                (ZombieKind::Snorkel, "潜水僵尸", 200.0),
                (ZombieKind::Zomboni, "冰车僵尸", 1160.0),
                (ZombieKind::BobsledTeam, "雪橇僵尸小队", 1080.0),
                (ZombieKind::DolphinRider, "海豚僵尸", 340.0),
                (ZombieKind::JackInTheBox, "小丑僵尸", 340.0),
                (ZombieKind::Balloon, "气球僵尸", 200.0),
                (ZombieKind::Digger, "矿工僵尸", 300.0),
                (ZombieKind::Pogo, "跳跳僵尸", 340.0),
                (ZombieKind::Yeti, "雪人僵尸", 920.0),
                (ZombieKind::Bungee, "蹦极僵尸", 460.0),
                (ZombieKind::Ladder, "梯子僵尸", 840.0),
                (ZombieKind::Catapult, "投篮僵尸", 660.0),
                (ZombieKind::Gargantuar, "巨人僵尸", 3000.0),
                (ZombieKind::GigaGargantuar, "红眼巨人僵尸", 6000.0),
                (ZombieKind::Imp, "小鬼僵尸", 200.0),
                (ZombieKind::IZombieImp, "我是僵尸模式小鬼", 60.0),
                (ZombieKind::PeashooterZombie, "豌豆僵尸", 200.0),
                (ZombieKind::WallNutZombie, "坚果僵尸", 1300.0),
                (ZombieKind::JalapenoZombie, "辣椒僵尸", 340.0),
                (ZombieKind::GatlingPeaZombie, "机枪僵尸", 200.0),
                (ZombieKind::SquashZombie, "窝瓜僵尸", 200.0),
                (ZombieKind::TallNutZombie, "高坚果僵尸", 2400.0),
            ]
            .into_iter()
            .map(|(kind, display_name, health)| zombie_definition(kind, display_name, health))
            .collect(),
            projectiles: vec![
                ProjectileDefinition {
                    kind: ProjectileKind::Pea,
                    damage: 20.0,
                    visual: ProjectileVisualDefinition {
                        fill_color: Color::srgb(0.45, 1.0, 0.14),
                        border_color: Color::srgb(0.04, 0.26, 0.03),
                        border_width: 2.0,
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
                    kind: ProjectileKind::IcePea,
                    damage: 20.0,
                    visual: ProjectileVisualDefinition {
                        fill_color: Color::srgb(0.35, 0.85, 0.95),
                        border_color: Color::srgb(0.05, 0.28, 0.34),
                        border_width: 2.0,
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
                    kind: ProjectileKind::FirePea,
                    damage: 40.0,
                    visual: ProjectileVisualDefinition {
                        fill_color: Color::srgb(1.0, 0.38, 0.08),
                        border_color: Color::srgb(0.45, 0.08, 0.02),
                        border_width: 2.0,
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
                    damage: 20.0,
                    visual: ProjectileVisualDefinition {
                        fill_color: Color::srgb(0.35, 0.85, 0.95),
                        border_color: Color::srgb(0.05, 0.28, 0.34),
                        border_width: 2.0,
                    },
                    radius: 9.0,
                    motion: ProjectileMotionDefinition::Physics {
                        initial_velocity: Vec2::new(330.0, 300.0),
                        gravity_scale: 1.0,
                        restitution: 0.45,
                        friction: 0.35,
                        ccd: true,
                    },
                    hit_policy: HitPolicyDefinition {
                        destroy_on_hit: true,
                        max_pierces: 0,
                    },
                },
            ],
        }
    }
}

fn zombie_definition(
    kind: ZombieKind,
    display_name: &'static str,
    total_health: f32,
) -> ZombieDefinition {
    let (speed, visual_size, collider_half_size) = match kind {
        ZombieKind::Flag => (19.0, Vec2::new(62.0, 88.0), Vec2::new(29.0, 41.0)),
        ZombieKind::Conehead => (16.5, Vec2::new(60.0, 94.0), Vec2::new(29.0, 41.0)),
        ZombieKind::PoleVaulting => (26.0, Vec2::new(74.0, 100.0), Vec2::new(31.0, 41.0)),
        ZombieKind::Buckethead => (15.5, Vec2::new(62.0, 96.0), Vec2::new(29.0, 41.0)),
        ZombieKind::Newspaper => (22.0, Vec2::new(62.0, 82.0), Vec2::new(31.0, 41.0)),
        ZombieKind::ScreenDoor => (13.5, Vec2::new(76.0, 90.0), Vec2::new(36.0, 43.0)),
        ZombieKind::Football => (30.0, Vec2::new(70.0, 92.0), Vec2::new(33.0, 44.0)),
        ZombieKind::Dancing => (21.0, Vec2::new(68.0, 90.0), Vec2::new(31.0, 41.0)),
        ZombieKind::BackupDancer => (20.0, Vec2::new(60.0, 84.0), Vec2::new(29.0, 41.0)),
        ZombieKind::Snorkel => (16.0, Vec2::new(66.0, 84.0), Vec2::new(31.0, 41.0)),
        ZombieKind::Zomboni => (11.0, Vec2::new(96.0, 66.0), Vec2::new(48.0, 33.0)),
        ZombieKind::BobsledTeam => (18.0, Vec2::new(108.0, 70.0), Vec2::new(54.0, 35.0)),
        ZombieKind::DolphinRider => (27.0, Vec2::new(88.0, 92.0), Vec2::new(38.0, 41.0)),
        ZombieKind::JackInTheBox => (23.0, Vec2::new(66.0, 88.0), Vec2::new(31.0, 41.0)),
        ZombieKind::Balloon => (18.0, Vec2::new(64.0, 128.0), Vec2::new(29.0, 41.0)),
        ZombieKind::Digger => (24.0, Vec2::new(70.0, 92.0), Vec2::new(31.0, 41.0)),
        ZombieKind::Pogo => (24.0, Vec2::new(68.0, 108.0), Vec2::new(31.0, 48.0)),
        ZombieKind::Yeti => (18.5, Vec2::new(76.0, 98.0), Vec2::new(36.0, 47.0)),
        ZombieKind::Bungee => (20.0, Vec2::new(70.0, 112.0), Vec2::new(32.0, 50.0)),
        ZombieKind::Ladder => (18.0, Vec2::new(84.0, 92.0), Vec2::new(39.0, 43.0)),
        ZombieKind::Catapult => (10.5, Vec2::new(108.0, 76.0), Vec2::new(54.0, 38.0)),
        ZombieKind::Gargantuar => (9.0, Vec2::new(104.0, 140.0), Vec2::new(52.0, 70.0)),
        ZombieKind::GigaGargantuar => (8.0, Vec2::new(110.0, 146.0), Vec2::new(55.0, 73.0)),
        ZombieKind::Imp | ZombieKind::IZombieImp => {
            (29.0, Vec2::new(42.0, 58.0), Vec2::new(21.0, 29.0))
        }
        ZombieKind::PeashooterZombie => (17.0, Vec2::new(64.0, 86.0), Vec2::new(31.0, 41.0)),
        ZombieKind::WallNutZombie => (12.0, Vec2::new(70.0, 88.0), Vec2::new(35.0, 43.0)),
        ZombieKind::JalapenoZombie => (24.0, Vec2::new(64.0, 88.0), Vec2::new(31.0, 41.0)),
        ZombieKind::GatlingPeaZombie => (16.0, Vec2::new(70.0, 88.0), Vec2::new(34.0, 43.0)),
        ZombieKind::SquashZombie => (18.0, Vec2::new(70.0, 86.0), Vec2::new(34.0, 42.0)),
        ZombieKind::TallNutZombie => (11.0, Vec2::new(76.0, 108.0), Vec2::new(38.0, 54.0)),
        _ => (17.0, Vec2::new(58.0, 82.0), Vec2::new(29.0, 41.0)),
    };
    let equipment_health = zombie_equipment_health(kind, total_health);
    ZombieDefinition {
        kind,
        display_name,
        scene_label: zombie_scene_label(kind),
        health: total_health - equipment_health.unwrap_or(0.0),
        equipment_health,
        speed,
        attack_damage: 20.0,
        attack_interval: Duration::from_secs(1),
        engage_range: -12.0..=62.0,
        spawn_offset_x: 75.0,
        visual: UnitVisualDefinition {
            color: Color::srgb(0.42, 0.48, 0.38),
            size: visual_size,
        },
        collider_half_size,
    }
}

fn zombie_scene_label(kind: ZombieKind) -> &'static str {
    match kind {
        ZombieKind::Basic => "普通\n僵尸",
        ZombieKind::Flag => "旗帜\n僵尸",
        ZombieKind::Conehead => "路障\n僵尸",
        ZombieKind::PoleVaulting => "撑杆\n僵尸",
        ZombieKind::Buckethead => "铁桶\n僵尸",
        ZombieKind::Newspaper => "读报\n僵尸",
        ZombieKind::ScreenDoor => "栅栏\n僵尸",
        ZombieKind::Football => "橄榄球\n僵尸",
        ZombieKind::Dancing => "舞王\n僵尸",
        ZombieKind::BackupDancer => "伴舞\n僵尸",
        ZombieKind::Snorkel => "潜水\n僵尸",
        ZombieKind::Zomboni => "冰车\n僵尸",
        ZombieKind::BobsledTeam => "雪橇小队\n僵尸",
        ZombieKind::DolphinRider => "海豚\n僵尸",
        ZombieKind::JackInTheBox => "小丑\n僵尸",
        ZombieKind::Balloon => "气球\n僵尸",
        ZombieKind::Digger => "矿工\n僵尸",
        ZombieKind::Pogo => "跳跳\n僵尸",
        ZombieKind::Yeti => "雪人\n僵尸",
        ZombieKind::Bungee => "蹦极\n僵尸",
        ZombieKind::Ladder => "梯子\n僵尸",
        ZombieKind::Catapult => "投篮\n僵尸",
        ZombieKind::Gargantuar => "巨人\n僵尸",
        ZombieKind::GigaGargantuar => "红眼巨人\n僵尸",
        ZombieKind::Imp => "小鬼\n僵尸",
        ZombieKind::IZombieImp => "我是小鬼\n僵尸",
        ZombieKind::PeashooterZombie => "豌豆\n僵尸",
        ZombieKind::WallNutZombie => "坚果\n僵尸",
        ZombieKind::JalapenoZombie => "辣椒\n僵尸",
        ZombieKind::GatlingPeaZombie => "机枪\n僵尸",
        ZombieKind::SquashZombie => "窝瓜\n僵尸",
        ZombieKind::TallNutZombie => "高坚果\n僵尸",
    }
}

fn zombie_equipment_health(kind: ZombieKind, total_health: f32) -> Option<f32> {
    let body_health = 200.0;
    let equipment_health = match kind {
        ZombieKind::Conehead
        | ZombieKind::Buckethead
        | ZombieKind::Newspaper
        | ZombieKind::ScreenDoor
        | ZombieKind::Football
        | ZombieKind::Ladder
        | ZombieKind::WallNutZombie
        | ZombieKind::TallNutZombie => total_health - body_health,
        ZombieKind::JackInTheBox => 140.0,
        _ => return None,
    };
    (equipment_health > 0.0).then_some(equipment_health)
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
                shots_per_burst,
                burst_interval,
                ..
            } = plant.behavior
            {
                if interval.is_zero() || shots_per_burst == 0 {
                    return Err("shooter interval and burst size must be positive".into());
                }
                if shots_per_burst > 1 && burst_interval.is_zero() {
                    return Err("multi-shot burst interval must be positive".into());
                }
                if burst_interval.saturating_mul(u32::from(shots_per_burst.saturating_sub(1)))
                    >= interval
                {
                    return Err("shooter burst must finish before the next firing cycle".into());
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
            if let Some(equipment_health) = zombie.equipment_health {
                validate_positive("zombie equipment health", equipment_health)?;
            }
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
            validate_positive("projectile border width", projectile.visual.border_width)?;
            if projectile.visual.border_width >= projectile.radius {
                return Err("projectile border width must be smaller than its radius".into());
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

    #[test]
    fn armored_zombies_split_equipment_from_body_health() {
        let catalog = ContentCatalog::default();
        assert_eq!(catalog.zombie(ZombieKind::Basic).equipment_health, None);

        let conehead = catalog.zombie(ZombieKind::Conehead);
        assert_eq!(conehead.health, 200.0);
        assert_eq!(conehead.equipment_health, Some(360.0));

        let buckethead = catalog.zombie(ZombieKind::Buckethead);
        assert_eq!(buckethead.health, 200.0);
        assert_eq!(buckethead.equipment_health, Some(1100.0));

        let screen_door = catalog.zombie(ZombieKind::ScreenDoor);
        assert_eq!(screen_door.health, 200.0);
        assert_eq!(screen_door.equipment_health, Some(1100.0));
    }

}
