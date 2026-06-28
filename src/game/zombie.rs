//! 僵尸系统
//!
//! 定义僵尸实体、状态机与行为逻辑。
//!
//! 僵尸有三种状态：
//! - **Walking**：正常向左行走，遇到植物时切换为 Eating
//! - **Eating**：停在原地对目标植物发动周期性啃食攻击
//!
//! 调度阶段：
//! - `Spawn`：消费 [`SpawnZombie`] 消息生成僵尸实体
//! - `LogicMovement`：状态更新（检测前方植物）→ 行走移动 → 攻击计时（链式执行）

use bevy::prelude::*;
use bevy::sprite::Text2dShadow;
use bevy_rapier2d::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::catalog::{ColliderHalfSize, ContentCatalog};
use crate::game::combat::{ApplyDamage, DamageKind, Health, Team};
use crate::game::lawn::{Lane, LawnLayout};
use crate::game::physics::zombie_groups;
use crate::game::plant::Plant;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::theme::UiTheme;

pub use crate::game::catalog::ZombieKind;

/// 僵尸插件，注册生成、状态更新、行走和攻击系统。
pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnZombie>()
            .add_systems(
                FixedUpdate,
                spawn_zombies
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_zombie_state,
                    advance_walking_zombies,
                    tick_zombie_attacks,
                )
                    .chain()
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 僵尸组件，携带移动速度属性。
#[derive(Component, Debug)]
pub struct Zombie {
    /// 行走速度（像素/秒）。
    pub speed: f32,
    pub attack_damage: f32,
    pub engage_min: f32,
    pub engage_max: f32,
}

/// 僵尸行为状态枚举。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZombieState {
    /// 行走状态：向左移动，寻找植物。
    Walking,
    /// 啃食状态：停在目标植物前，周期性造成伤害。
    Eating {
        /// 正在啃食的目标植物实体。
        target: Entity,
    },
}

/// 内部组件：僵尸攻击计时器，控制啃食频率（默认 1 秒间隔）。
#[derive(Component, Debug)]
struct AttackTimer(Timer);

/// 生成僵尸的请求消息。
#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnZombie {
    /// 要生成的僵尸种类。
    pub kind: ZombieKind,
    /// 僵尸出生的行。
    pub lane: Lane,
}

/// 处理 [`SpawnZombie`] 消息，在棋盘右侧生成基本僵尸实体。
pub(crate) fn spawn_zombies(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    theme: Option<Res<UiTheme>>,
    catalog: Res<ContentCatalog>,
    mut requests: MessageReader<SpawnZombie>,
    layout: Res<LawnLayout>,
) {
    for request in requests.read() {
        let definition = catalog.zombie(request.kind);
        let position = Vec2::new(
            layout.right() + definition.spawn_offset_x,
            layout.lane_y(request.lane),
        );
        let fallback_theme = UiTheme::default();
        let label = theme
            .as_ref()
            .map(|theme| &theme.zombie_label)
            .unwrap_or(&fallback_theme.zombie_label);
        let font = assets
            .as_ref()
            .map(|assets| assets.chinese_font.clone())
            .unwrap_or_default();
        // 名牌作为僵尸子实体，会自动跟随移动并在僵尸销毁时一并清理。
        commands.spawn((
            (
                Sprite::from_color(definition.visual.color, definition.visual.size),
                Transform::from_translation(position.extend(2.0)),
                Zombie {
                    speed: definition.speed,
                    attack_damage: definition.attack_damage,
                    engage_min: *definition.engage_range.start(),
                    engage_max: *definition.engage_range.end(),
                },
                request.kind,
                ZombieState::Walking,
                AttackTimer(Timer::new(definition.attack_interval, TimerMode::Repeating)),
                request.lane,
                Health::new(definition.health),
                Team::Zombies,
            ),
            (
                RigidBody::KinematicPositionBased,
                Collider::cuboid(
                    definition.collider_half_size.x,
                    definition.collider_half_size.y,
                ),
                ColliderHalfSize(definition.collider_half_size),
                LockedAxes::ROTATION_LOCKED,
                ActiveEvents::COLLISION_EVENTS,
                zombie_groups(),
                LevelEntity,
                Name::new(definition.display_name),
            ),
            children![(
                Text2d::new(definition.scene_label),
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
                Name::new("僵尸名称"),
            )],
        ));
    }
}

/// 更新僵尸状态：检测前方同行的植物，切换 Walking / Eating。
///
/// 判断逻辑：检查僵尸前方距离 [-12, 62] 像素范围内是否存在植物，
/// 取最近的植物作为啃食目标；无植物则保持 Walking。
fn update_zombie_state(
    mut zombies: Query<(&Zombie, &Transform, &Lane, &mut ZombieState)>,
    plants: Query<(Entity, &Transform, &Lane), With<Plant>>,
) {
    for (zombie, zombie_transform, zombie_lane, mut state) in &mut zombies {
        let zombie_x = zombie_transform.translation.x;
        let blocker = plants
            .iter()
            .filter(|(_, _, plant_lane)| *plant_lane == zombie_lane)
            .filter_map(|(entity, plant_transform, _)| {
                let distance = zombie_x - plant_transform.translation.x;
                (zombie.engage_min..=zombie.engage_max)
                    .contains(&distance)
                    .then_some((distance.abs(), entity))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, entity)| entity);

        *state = blocker
            .map(|target| ZombieState::Eating { target })
            .unwrap_or(ZombieState::Walking);
    }
}

/// 处于 Walking 状态的僵尸向左移动。
fn advance_walking_zombies(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(&Zombie, &ZombieState, &mut Transform)>,
) {
    for (zombie, state, mut transform) in &mut zombies {
        if *state == ZombieState::Walking {
            transform.translation.x -= zombie.speed * time.delta_secs();
        }
    }
}

/// 处于 Eating 状态的僵尸按攻击间隔对目标植物造成伤害。
///
/// 如果目标植物已被销毁，则重置计时器并等待下次状态更新。
fn tick_zombie_attacks(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(Entity, &Zombie, &ZombieState, &mut AttackTimer)>,
    plants: Query<(), With<Plant>>,
    mut damage: MessageWriter<ApplyDamage>,
) {
    for (entity, zombie, state, mut timer) in &mut zombies {
        let ZombieState::Eating { target } = *state else {
            timer.0.reset();
            continue;
        };
        if !plants.contains(target) {
            continue;
        }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            damage.write(ApplyDamage {
                source: entity,
                target,
                amount: zombie.attack_damage,
                kind: DamageKind::Bite,
            });
        }
    }
}
