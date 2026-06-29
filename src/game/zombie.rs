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
use crate::game::lawn::{GridCell, LawnLayout};
use crate::game::model::zombie_model_parts;
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
                Update,
                update_zombie_health_debug.run_if(in_state(GameState::Playing)),
            )
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

#[derive(Component)]
struct ZombieHealthText;

#[derive(Component)]
struct ZombieHealthBarFill;

#[derive(Component)]
struct ZombieHealthBarBackground;

type ZombieHealthTextFilter = (
    With<ZombieHealthText>,
    Without<ZombieHealthBarFill>,
    Without<ZombieHealthBarBackground>,
);

type ZombieHealthFillFilter = (
    With<ZombieHealthBarFill>,
    Without<ZombieHealthText>,
    Without<ZombieHealthBarBackground>,
);

type ZombieHealthBackgroundFilter = (
    With<ZombieHealthBarBackground>,
    Without<ZombieHealthText>,
    Without<ZombieHealthBarFill>,
);

/// 生成僵尸的请求消息。
#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnZombie {
    /// 要生成的僵尸种类。
    pub kind: ZombieKind,
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
        let position = Vec2::new(layout.right() + definition.spawn_offset_x, layout.path_y());
        let fallback_theme = UiTheme::default();
        let label = theme
            .as_ref()
            .map(|theme| &theme.zombie_label)
            .unwrap_or(&fallback_theme.zombie_label);
        let font = assets
            .as_ref()
            .map(|assets| assets.chinese_font.clone())
            .unwrap_or_default();
        let model_parts = zombie_model_parts(request.kind, 1.0);
        // 透明根节点承担碰撞与逻辑，子级色块、名牌和血条自动跟随。
        let mut entity = commands.spawn((
            (
                Sprite::from_color(
                    definition.visual.color.with_alpha(0.0),
                    definition.visual.size,
                ),
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
        ));
        entity.with_children(|parent| {
            for part in model_parts {
                parent.spawn((
                    Sprite::from_color(part.color, part.size),
                    Transform::from_xyz(part.offset.x, part.offset.y, part.z)
                        .with_rotation(Quat::from_rotation_z(part.rotation)),
                    Name::new(part.name),
                ));
            }
            parent.spawn((
                Text2d::new(definition.scene_label),
                TextFont {
                    font: font.clone(),
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
            ));
            parent.spawn((
                Sprite::from_color(Color::srgba(0.04, 0.04, 0.04, 0.9), Vec2::new(62.0, 10.0)),
                Transform::from_xyz(0.0, 49.0, 4.0),
                Visibility::Hidden,
                ZombieHealthBarBackground,
                Name::new("僵尸血条背景"),
            ));
            parent.spawn((
                Sprite::from_color(Color::srgb(0.2, 0.9, 0.18), Vec2::new(58.0, 6.0)),
                Transform::from_xyz(0.0, 49.0, 4.1),
                Visibility::Hidden,
                ZombieHealthBarFill,
                Name::new("僵尸血条"),
            ));
            parent.spawn((
                Text2d::new(format!(
                    "{:.0} / {:.0}",
                    definition.health, definition.health
                )),
                TextFont {
                    font,
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Text2dShadow {
                    offset: Vec2::new(1.0, -1.0),
                    color: Color::BLACK,
                },
                Transform::from_xyz(0.0, 63.0, 4.2),
                Visibility::Hidden,
                ZombieHealthText,
                Name::new("僵尸血量数值"),
            ));
        });
    }
}

/// 物理 debug 渲染开启时，同步显示僵尸的实时血量数值和血条。
fn update_zombie_health_debug(
    debug: Res<DebugRenderContext>,
    zombies: Query<(&Health, &Children), With<Zombie>>,
    mut texts: Query<(&mut Text2d, &mut Visibility), ZombieHealthTextFilter>,
    mut fills: Query<(&mut Sprite, &mut Transform, &mut Visibility), ZombieHealthFillFilter>,
    mut backgrounds: Query<&mut Visibility, ZombieHealthBackgroundFilter>,
) {
    let visibility = if debug.enabled {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for (health, children) in &zombies {
        let ratio = (health.current / health.max).clamp(0.0, 1.0);
        for child in children.iter() {
            if let Ok((mut text, mut child_visibility)) = texts.get_mut(child) {
                text.0 = format!("{:.0} / {:.0}", health.current, health.max);
                *child_visibility = visibility;
            }
            if let Ok((mut sprite, mut transform, mut child_visibility)) = fills.get_mut(child) {
                let width = 58.0 * ratio;
                sprite.custom_size = Some(Vec2::new(width, 6.0));
                sprite.color = if ratio > 0.5 {
                    Color::srgb(0.2, 0.9, 0.18)
                } else if ratio > 0.25 {
                    Color::srgb(0.95, 0.72, 0.12)
                } else {
                    Color::srgb(0.92, 0.18, 0.12)
                };
                transform.translation.x = (width - 58.0) * 0.5;
                *child_visibility = visibility;
            }
            if let Ok(mut child_visibility) = backgrounds.get_mut(child) {
                *child_visibility = visibility;
            }
        }
    }
}

/// 更新僵尸状态：检测道路前方的底层植物，切换 Walking / Eating。
///
/// 判断逻辑：检查僵尸前方距离 [-12, 62] 像素范围内是否存在植物，
/// 取最近的植物作为啃食目标；无植物则保持 Walking。
fn update_zombie_state(
    mut zombies: Query<(&Zombie, &Transform, &mut ZombieState)>,
    plants: Query<(Entity, &Transform, &GridCell), With<Plant>>,
) {
    for (zombie, zombie_transform, mut state) in &mut zombies {
        let zombie_x = zombie_transform.translation.x;
        let blocker = plants
            .iter()
            .filter(|(_, _, cell)| cell.is_ground())
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
