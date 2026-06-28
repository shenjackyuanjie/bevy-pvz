//! 植物系统
//!
//! 定义植物种类（向日葵、豌豆射手、坚果墙）及其属性（价格、冷却、生命值、颜色），
//! 处理植物放置逻辑（资源消耗、冷却触发、格子占用）、豌豆射手自动射击以及向日葵产太阳。
//!
//! 调度阶段：
//! - `Spawn`：读取 [`PlantRequest`] 消息，验证条件后生成植物实体
//! - `LogicMovement`：豌豆射手检测前方僵尸并发射豌豆、向日葵定时产出太阳
//! - `DeathAndCleanup`：死亡植物释放占用的格子

use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use bevy_rapier2d::prelude::*;

use crate::game::combat::{Dead, Health, Team};
use crate::game::lawn::{CellOccupancy, GridCell, Lane, LawnLayout};
use crate::game::level::{PlantCards, SpawnSun, SunBank};
use crate::game::physics::plant_groups;
use crate::game::projectile::{ProjectileKind, SpawnProjectile};
use crate::game::schedule::GameSet;
use crate::game::state::GameState;
use crate::game::zombie::Zombie;

/// 植物插件，注册生成、行为逻辑（射击/产太阳）和死亡释放格子的系统。
pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlantRequest>()
            .add_systems(
                FixedUpdate,
                place_plants
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (fire_ready_shooters, produce_sun)
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                release_dead_plant_cells
                    .in_set(GameSet::DeathAndCleanup)
                    .before(crate::game::combat::cleanup_dead_entities)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 植物种类枚举。
///
/// 每种植物有对应的显示名称、价格（太阳）、冷却时间、生命值和颜色。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlantKind {
    /// 向日葵：产太阳，生命值低，价格 50 太阳。
    Sunflower,
    /// 豌豆射手：发射豌豆攻击前方僵尸，价格 100 太阳。
    Peashooter,
    /// 坚果墙：高生命值阻挡僵尸，价格 50 太阳。
    WallNut,
}

impl PlantKind {
    /// 所有植物种类的列表，用于遍历注册。
    pub const ALL: [Self; 3] = [Self::Sunflower, Self::Peashooter, Self::WallNut];

    /// 返回植物在场景名牌和卡片中使用的简短中文名称。
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Sunflower => "向日葵",
            Self::Peashooter => "豌豆",
            Self::WallNut => "坚果",
        }
    }

    /// 植物的太阳价格。
    pub fn price(self) -> u32 {
        match self {
            Self::Sunflower | Self::WallNut => 50,
            Self::Peashooter => 100,
        }
    }

    /// 使用后卡片冷却时间。
    pub fn card_cooldown(self) -> Duration {
        match self {
            Self::Sunflower => Duration::from_secs(5),
            Self::Peashooter => Duration::from_secs(4),
            Self::WallNut => Duration::from_secs(8),
        }
    }

    /// 植物的初始生命值。
    fn health(self) -> f32 {
        match self {
            Self::Sunflower => 100.0,
            Self::Peashooter => 120.0,
            Self::WallNut => 600.0,
        }
    }

    /// 植物的精灵颜色。
    fn color(self) -> Color {
        match self {
            Self::Sunflower => Color::srgb(0.98, 0.72, 0.12),
            Self::Peashooter => Color::srgb(0.12, 0.72, 0.20),
            Self::WallNut => Color::srgb(0.55, 0.30, 0.12),
        }
    }
}

/// 植物标记组件，用于查询区分植物与其他实体。
#[derive(Component, Debug)]
pub struct Plant;

/// 内部组件：动作计时器，用于驱动周期性行为（射击间隔、产太阳间隔）。
#[derive(Component, Debug)]
struct ActionTimer(Timer);

/// 放置植物的请求消息。
///
/// 由鼠标点击处理系统（`handle_world_clicks`）发送，
/// [`place_plants`] 系统消费并执行实际放置。
#[derive(Message, Debug, Clone, Copy)]
pub struct PlantRequest {
    /// 要放置的植物种类。
    pub kind: PlantKind,
    /// 目标格子位置。
    pub cell: GridCell,
}

/// 处理 [`PlantRequest`] 消息，在满足条件时生成植物实体。
///
/// 验证条件（按顺序）：
/// 1. 格子未被占用且在棋盘范围内
/// 2. 该植物卡片冷却已结束（ready）
/// 3. 太阳库存充足
///
/// 条件通过后：扣除太阳、触发卡片冷却、生成精灵/碰撞体/标记组件、更新格子占用表。
fn place_plants(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut requests: MessageReader<PlantRequest>,
    layout: Res<LawnLayout>,
    mut occupancy: ResMut<CellOccupancy>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
) {
    let label_font = asset_server.load("fonts/NotoSansSC-VF.ttf");

    for request in requests.read() {
        if !occupancy.is_available(request.cell, &layout)
            || !cards.ready(request.kind)
            || !sun.try_spend(request.kind.price())
        {
            continue;
        }

        cards.trigger(request.kind);
        let position = layout.cell_center(request.cell);
        // 色块继续承担碰撞与阵营辨识；子级中文名牌直接说明植物身份。
        let mut entity = commands.spawn((
            Sprite::from_color(request.kind.color(), Vec2::new(58.0, 68.0)),
            Transform::from_translation(position.extend(1.0)),
            Plant,
            request.kind,
            request.cell,
            Lane(request.cell.row),
            Health::new(request.kind.health()),
            Team::Plants,
            RigidBody::Fixed,
            Collider::cuboid(29.0, 34.0),
            plant_groups(),
            crate::game::state::LevelEntity,
            Name::new(request.kind.display_name()),
            children![(
                Text2d::new(request.kind.display_name()),
                TextFont {
                    font: label_font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.98, 0.88)),
                TextBackgroundColor(Color::srgba(0.05, 0.08, 0.04, 0.72)),
                TextLayout::new_with_justify(Justify::Center),
                Text2dShadow {
                    offset: Vec2::new(1.5, -1.5),
                    color: Color::srgba(0.0, 0.0, 0.0, 0.9),
                },
                Transform::from_xyz(0.0, -3.0, 3.0),
                Name::new("植物名称"),
            )],
        ));

        // 坚果墙没有周期性动作，不加 ActionTimer。
        if request.kind != PlantKind::WallNut {
            let seconds = match request.kind {
                PlantKind::Sunflower => 7.0,
                PlantKind::Peashooter => 1.35,
                PlantKind::WallNut => unreachable!(),
            };
            entity.insert(ActionTimer(Timer::from_seconds(
                seconds,
                TimerMode::Repeating,
            )));
        }
        match request.kind {
            PlantKind::Sunflower => {
                entity.insert(Sunflower);
            }
            PlantKind::Peashooter => {
                entity.insert(Peashooter);
            }
            PlantKind::WallNut => {}
        }
        let id = entity.id();
        occupancy.0.insert(request.cell, id);
    }
}

/// 豌豆射手行为：检测前方同行的僵尸，发射豌豆。
///
/// 只有当射手的 `ActionTimer` 归零（射击间隔结束）且前方存在僵尸时才会发射。
fn fire_ready_shooters(
    time: Res<Time<Fixed>>,
    mut shooters: Query<(Entity, &Transform, &Lane, &mut ActionTimer), With<Peashooter>>,
    zombies: Query<(&Transform, &Lane), With<Zombie>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    for (entity, transform, lane, mut timer) in &mut shooters {
        timer.0.tick(time.delta());
        let has_target = zombies.iter().any(|(zombie_transform, zombie_lane)| {
            zombie_lane == lane && zombie_transform.translation.x > transform.translation.x
        });
        if has_target && timer.0.just_finished() {
            spawn.write(SpawnProjectile {
                owner: entity,
                origin: transform.translation.truncate() + Vec2::new(36.0, 12.0),
                lane: *lane,
                kind: ProjectileKind::Pea,
            });
        }
    }
}

/// 向日葵行为：定时产出太阳。
///
/// 每隔 7 秒在向日葵上方产生一个太阳拾取物。
fn produce_sun(
    time: Res<Time<Fixed>>,
    mut sunflowers: Query<(&Transform, &mut ActionTimer), With<Sunflower>>,
    mut spawn: MessageWriter<SpawnSun>,
) {
    for (transform, mut timer) in &mut sunflowers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn.write(SpawnSun {
                position: transform.translation.truncate() + Vec2::new(18.0, 24.0),
                value: 25,
            });
        }
    }
}

/// 释放已死亡植物占用的格子，使该位置可以重新放置植物。
///
/// 在死亡实体清理之前运行，确保格子先释放再清理实体。
fn release_dead_plant_cells(
    dead_plants: Query<&GridCell, (With<Plant>, With<Dead>)>,
    mut occupancy: ResMut<CellOccupancy>,
) {
    for cell in &dead_plants {
        occupancy.0.remove(cell);
    }
}

// 标记组件让行为查询变得明确且高效。
/// 向日葵标记组件，用于 Query 过滤。
#[derive(Component)]
struct Sunflower;
/// 豌豆射手标记组件，用于 Query 过滤。
#[derive(Component)]
struct Peashooter;
