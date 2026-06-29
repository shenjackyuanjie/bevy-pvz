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
use crate::game::lawn::{CellOccupancy, GridCell, LawnLayout};
use crate::game::level::{PlantCards, SpawnSun, SunBank};
use crate::game::model::plant_model_parts;
use crate::game::physics::{plant_groups, torchwood_groups};
use crate::game::projectile::{ProjectileKind, ProjectileRoute, SpawnProjectile};
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
            )
            .add_systems(
                Update,
                update_plant_debug_visibility.run_if(in_state(GameState::Playing)),
            );
    }
}

/// 植物标记组件，用于查询区分植物与其他实体。
#[derive(Component, Debug)]
pub struct Plant;

/// 火炬树桩上半部的路径弹丸判定矩形。
#[derive(Component, Debug, Clone, Copy)]
pub struct TorchwoodFlameZone {
    pub offset: Vec2,
    pub half_size: Vec2,
}

/// Rapier 使用的火炬点燃 Sensor 标记。
#[derive(Component, Debug)]
pub struct TorchwoodFlameCollider;

/// 内部组件：动作计时器，用于驱动周期性行为（射击间隔、产太阳间隔）。
#[derive(Component, Debug)]
struct ActionTimer(Timer);

/// 植物名称标签标记组件，用于在 debug 模式下控制名称文字的显示/隐藏。
#[derive(Component, Debug)]
struct PlantNameText;

/// 放置植物的请求消息。
///
/// 由植物拖拽结束系统发送，
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
/// 1. 向日葵只能放在空中格，专用行只接受普通豌豆射手
/// 2. 格子未被占用且在棋盘范围内
/// 3. 该植物卡片冷却已结束（ready）
/// 4. 太阳库存充足
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
        if !plant_can_occupy(request.kind, request.cell)
            || !params.occupancy.is_available(request.cell, &params.layout)
            || !params.cards.ready(request.kind)
            || !params.sun.try_spend(definition.price)
        {
            continue;
        }

        params.cards.trigger(request.kind, &params.catalog);
        let position = params.layout.cell_center(request.cell);
        let label = &params.theme.plant_label;
        let font = params.assets.chinese_font.clone();
        let model_parts = plant_model_parts(request.kind, 1.0);
        let model_facing = if request.cell.is_peashooter_row()
            && matches!(
                request.kind,
                PlantKind::Peashooter | PlantKind::Repeater | PlantKind::GatlingPea
            ) {
            -1.0
        } else {
            1.0
        };
        // 透明根节点承担碰撞与逻辑，子级色块组成植物轮廓。
        let mut entity = params.commands.spawn((
            Sprite::from_color(
                definition.visual.color.with_alpha(0.0),
                definition.visual.size,
            ),
            Transform::from_translation(position.extend(1.0)),
            Plant,
            request.kind,
            request.cell,
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
        ));
        entity.with_children(|parent| {
            for part in model_parts {
                parent.spawn((
                    Sprite::from_color(part.color, part.size),
                    Transform::from_xyz(part.offset.x * model_facing, part.offset.y, part.z)
                        .with_rotation(Quat::from_rotation_z(part.rotation * model_facing)),
                    Name::new(part.name),
                ));
            }
            parent.spawn((
                Text2d::new(definition.display_name),
                TextFont {
                    font,
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
                Visibility::Hidden,
                PlantNameText,
                Name::new("植物名称"),
            ));
        });

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
                shots_per_burst,
                burst_interval,
            } => {
                entity.insert((
                    ActionTimer(Timer::new(interval, TimerMode::Repeating)),
                    Shooter {
                        projectile,
                        muzzle_offset,
                        shots_per_burst,
                        remaining_burst_shots: 0,
                        burst_timer: Timer::new(burst_interval, TimerMode::Repeating),
                    },
                ));
            }
            PlantBehavior::Blocker => {}
        }
        if request.kind == PlantKind::Torchwood {
            let zone = TorchwoodFlameZone {
                offset: Vec2::new(0.0, 18.0),
                half_size: Vec2::new(24.0, 15.0),
            };
            entity.insert(zone);
            entity.with_child((
                Collider::cuboid(zone.half_size.x, zone.half_size.y),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                torchwood_groups(),
                Transform::from_xyz(zone.offset.x, zone.offset.y, 0.0),
                TorchwoodFlameCollider,
                Name::new("火炬树桩点燃区"),
            ));
        }
        let id = entity.id();
        params.occupancy.0.insert(request.cell, id);
    }
}

/// 空中格只接受向日葵，row 0 只接受坚果/火炬，row -2 只接受豌豆/火炬。
fn plant_can_occupy(kind: PlantKind, cell: GridCell) -> bool {
    match kind {
        PlantKind::Sunflower => cell.is_elevated(),
        PlantKind::Peashooter | PlantKind::Repeater | PlantKind::GatlingPea => {
            cell.is_peashooter_row()
        }
        PlantKind::WallNut => cell.is_ground(),
        PlantKind::Torchwood => cell.is_ground() || cell.is_peashooter_row(),
    }
}

/// 物理 debug 渲染开启时，显示植物名称标签。
fn update_plant_debug_visibility(
    debug: Res<DebugRenderContext>,
    mut texts: Query<&mut Visibility, With<PlantNameText>>,
) {
    let visibility = if debug.enabled {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for mut v in &mut texts {
        *v = visibility;
    }
}

/// 豌豆射手行为：检测道路前方的僵尸，发射豌豆。
///
/// 只有当射手的 `ActionTimer` 归零（射击间隔结束）且前方存在僵尸时才会发射。
fn fire_ready_shooters(
    time: Res<Time<Fixed>>,
    layout: Res<LawnLayout>,
    mut shooters: Query<(
        Entity,
        &Transform,
        &GridCell,
        &mut Shooter,
        &mut ActionTimer,
    )>,
    zombies: Query<&Transform, With<Zombie>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    for (entity, transform, cell, mut shooter, mut timer) in &mut shooters {
        let (origin, route) = shooter_projectile_route(
            &layout,
            *cell,
            transform.translation.truncate(),
            shooter.muzzle_offset,
        );
        timer.0.tick(time.delta());
        if shooter.remaining_burst_shots > 0 {
            shooter.burst_timer.tick(time.delta());
            if shooter.burst_timer.just_finished() {
                spawn.write(SpawnProjectile {
                    owner: entity,
                    origin,
                    kind: shooter.projectile,
                    route,
                });
                shooter.remaining_burst_shots -= 1;
            }
            continue;
        }
        let target_origin_x = match route {
            ProjectileRoute::Direct => origin.x,
            ProjectileRoute::LeftEdgePortal { exit, .. } => exit.x,
        };
        let has_target = zombies
            .iter()
            .any(|zombie_transform| zombie_transform.translation.x > target_origin_x);
        if has_target && timer.0.just_finished() {
            spawn.write(SpawnProjectile {
                owner: entity,
                origin,
                kind: shooter.projectile,
                route,
            });
            shooter.remaining_burst_shots = shooter.shots_per_burst - 1;
            shooter.burst_timer.reset();
        }
    }
}

/// 底排弹丸从自身炮口向左发射，到边界后从 row 0 左端向右继续飞行。
fn shooter_projectile_route(
    layout: &LawnLayout,
    cell: GridCell,
    plant_position: Vec2,
    muzzle_offset: Vec2,
) -> (Vec2, ProjectileRoute) {
    if cell.is_peashooter_row() {
        (
            plant_position + muzzle_offset,
            ProjectileRoute::LeftEdgePortal {
                trigger_x: layout.origin.x,
                exit: Vec2::new(layout.origin.x, layout.path_y() + muzzle_offset.y),
            },
        )
    } else {
        (plant_position + muzzle_offset, ProjectileRoute::Direct)
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
    shots_per_burst: u8,
    remaining_burst_shots: u8,
    burst_timer: Timer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plant_rows_enforce_their_specializations() {
        let ground = GridCell { column: 4, row: 0 };
        let elevated = GridCell { column: 0, row: 2 };
        let peashooter_row = GridCell { column: 4, row: -2 };

        assert!(!plant_can_occupy(PlantKind::Sunflower, ground));
        assert!(plant_can_occupy(PlantKind::Sunflower, elevated));
        assert!(!plant_can_occupy(PlantKind::Peashooter, ground));
        assert!(!plant_can_occupy(PlantKind::Peashooter, elevated));
        assert!(plant_can_occupy(PlantKind::Peashooter, peashooter_row));
        assert!(plant_can_occupy(PlantKind::Repeater, peashooter_row));
        assert!(plant_can_occupy(PlantKind::GatlingPea, peashooter_row));
        assert!(!plant_can_occupy(PlantKind::WallNut, peashooter_row));
        assert!(plant_can_occupy(PlantKind::WallNut, ground));
        assert!(plant_can_occupy(PlantKind::Torchwood, ground));
        assert!(plant_can_occupy(PlantKind::Torchwood, peashooter_row));
        assert!(!plant_can_occupy(PlantKind::Torchwood, elevated));
    }

    #[test]
    fn lower_row_projectiles_start_locally_and_carry_a_portal_route() {
        let layout = LawnLayout::default();
        let muzzle = Vec2::new(36.0, 12.0);
        let lower = GridCell { column: 7, row: -2 };
        let plant_position = layout.cell_center(lower);
        let (origin, route) = shooter_projectile_route(&layout, lower, plant_position, muzzle);

        assert_eq!(origin, plant_position + muzzle);
        assert_eq!(
            route,
            ProjectileRoute::LeftEdgePortal {
                trigger_x: layout.origin.x,
                exit: Vec2::new(layout.origin.x, layout.path_y() + muzzle.y),
            }
        );
    }
}
