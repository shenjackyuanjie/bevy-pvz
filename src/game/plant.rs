//! 植物系统
//!
//! 定义植物种类（向日葵、豌豆射手、坚果墙）及其属性（价格、冷却、生命值、颜色），
//! 处理植物放置逻辑（资源消耗、冷却触发、格子占用）、豌豆射手自动射击以及向日葵产太阳。
//!
//! 调度阶段：
//! - `Spawn`：读取 [`PlantRequest`] 消息，验证条件后生成植物实体
//! - `LogicMovement`：豌豆射手检测前方僵尸并发射豌豆、向日葵定时产出太阳
//! - `DeathAndCleanup`：死亡植物释放占用的格子

use bevy::sprite::Text2dShadow;
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::catalog::{ColliderHalfSize, ContentCatalog, PlantBehavior};
use crate::game::combat::{Dead, Health, Team};
use crate::game::lawn::{CellOccupancy, GridCell, Lane, LawnLayout};
use crate::game::level::{PlantCards, SpawnSun, SunBank};
use crate::game::physics::plant_groups;
use crate::game::projectile::{ProjectileKind, SpawnProjectile};
use crate::game::schedule::GameSet;
use crate::game::state::GameState;
use crate::game::theme::UiTheme;
use crate::game::zombie::Zombie;

pub use crate::game::catalog::PlantKind;

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
#[derive(SystemParam)]
struct PlacePlantParams<'w, 's> {
    commands: Commands<'w, 's>,
    assets: Res<'w, GameAssets>,
    theme: Res<'w, UiTheme>,
    catalog: Res<'w, ContentCatalog>,
    layout: Res<'w, LawnLayout>,
    occupancy: ResMut<'w, CellOccupancy>,
    sun: ResMut<'w, SunBank>,
    cards: ResMut<'w, PlantCards>,
}

fn place_plants(mut params: PlacePlantParams, mut requests: MessageReader<PlantRequest>) {
    for request in requests.read() {
        let definition = params.catalog.plant(request.kind);
        if !params.occupancy.is_available(request.cell, &params.layout)
            || !params.cards.ready(request.kind)
            || !params.sun.try_spend(definition.price)
        {
            continue;
        }

        params.cards.trigger(request.kind, &params.catalog);
        let position = params.layout.cell_center(request.cell);
        let label = &params.theme.plant_label;
        // 色块继续承担碰撞与阵营辨识；子级中文名牌直接说明植物身份。
        let mut entity = params.commands.spawn((
            Sprite::from_color(definition.visual.color, definition.visual.size),
            Transform::from_translation(position.extend(1.0)),
            Plant,
            request.kind,
            request.cell,
            Lane(request.cell.row),
            Health::new(definition.health),
            Team::Plants,
            RigidBody::Fixed,
            Collider::cuboid(
                definition.collider_half_size.x,
                definition.collider_half_size.y,
            ),
            ColliderHalfSize(definition.collider_half_size),
            plant_groups(),
            crate::game::state::LevelEntity,
            Name::new(definition.display_name),
            children![(
                Text2d::new(definition.display_name),
                TextFont {
                    font: params.assets.chinese_font.clone(),
                    font_size: label.font_size,
                    ..default()
                },
                TextColor(label.text),
                TextBackgroundColor(label.background),
                TextLayout::new_with_justify(Justify::Center),
                Text2dShadow {
                    offset: label.shadow_offset,
                    color: label.shadow,
                },
                Transform::from_xyz(label.offset.x, label.offset.y, 3.0),
                Name::new("植物名称"),
            )],
        ));

        match definition.behavior {
            PlantBehavior::SunProducer {
                interval,
                value,
                spawn_offset,
            } => {
                entity.insert((
                    ActionTimer(Timer::new(interval, TimerMode::Repeating)),
                    SunProducer {
                        value,
                        spawn_offset,
                    },
                ));
            }
            PlantBehavior::Shooter {
                interval,
                projectile,
                muzzle_offset,
            } => {
                entity.insert((
                    ActionTimer(Timer::new(interval, TimerMode::Repeating)),
                    Shooter {
                        projectile,
                        muzzle_offset,
                    },
                ));
            }
            PlantBehavior::Blocker => {}
        }
        let id = entity.id();
        params.occupancy.0.insert(request.cell, id);
    }
}

/// 豌豆射手行为：检测前方同行的僵尸，发射豌豆。
///
/// 只有当射手的 `ActionTimer` 归零（射击间隔结束）且前方存在僵尸时才会发射。
fn fire_ready_shooters(
    time: Res<Time<Fixed>>,
    mut shooters: Query<(Entity, &Transform, &Lane, &Shooter, &mut ActionTimer)>,
    zombies: Query<(&Transform, &Lane), With<Zombie>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    for (entity, transform, lane, shooter, mut timer) in &mut shooters {
        timer.0.tick(time.delta());
        let has_target = zombies.iter().any(|(zombie_transform, zombie_lane)| {
            zombie_lane == lane && zombie_transform.translation.x > transform.translation.x
        });
        if has_target && timer.0.just_finished() {
            spawn.write(SpawnProjectile {
                owner: entity,
                origin: transform.translation.truncate() + shooter.muzzle_offset,
                lane: *lane,
                kind: shooter.projectile,
            });
        }
    }
}

/// 向日葵行为：定时产出太阳。
///
/// 按实体上的目录参数定时在植物附近产生太阳拾取物。
fn produce_sun(
    time: Res<Time<Fixed>>,
    mut sunflowers: Query<(&Transform, &SunProducer, &mut ActionTimer)>,
    mut spawn: MessageWriter<SpawnSun>,
) {
    for (transform, producer, mut timer) in &mut sunflowers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn.write(SpawnSun {
                position: transform.translation.truncate() + producer.spawn_offset,
                value: producer.value,
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

#[derive(Component, Debug)]
struct SunProducer {
    value: u32,
    spawn_offset: Vec2,
}

#[derive(Component, Debug)]
struct Shooter {
    projectile: ProjectileKind,
    muzzle_offset: Vec2,
}
