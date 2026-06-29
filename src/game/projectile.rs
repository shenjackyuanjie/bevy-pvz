//! 弹丸系统
//!
//! 管理两种弹丸的完整生命周期：
//!
//! - **路径弹丸（Pea）**：无物理碰撞体，通过扫掠检测（Swept Circle）每帧检测与僵尸的碰撞，
//!   命中即销毁。速度为恒定线性。
//!
//! - **物理弹丸（PhysicsPea）**：拥有 Rapier2D 刚体和碰撞体，受重力影响，
//!   可在地板和墙面弹跳，命中僵尸后造成伤害并销毁。
//!
//! 调度阶段：
//! - `Spawn`：消费 [`SpawnProjectile`] 消息，根据种类创建弹丸实体
//! - `LogicMovement`：路径弹丸按速度前进
//! - `ContactRead`：路径弹丸扫掠检测 + 物理弹丸碰撞事件收集
//! - `Combat`：弹丸命中解析，发出 [`ApplyDamage`](crate::game::combat::ApplyDamage)
//! - `DeathAndCleanup`：普通路径弹丸完全飞出窗口后销毁

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::game::catalog::{ColliderHalfSize, ContentCatalog, ProjectileMotionDefinition};
use crate::game::combat::{ApplyDamage, DamageKind, Team};
#[cfg(feature = "debug_tools")]
use crate::game::controls::ControlBindings;
use crate::game::lawn::LawnLayout;
use crate::game::physics::physics_projectile_groups;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::zombie::Zombie;

pub use crate::game::catalog::ProjectileKind;

/// 弹丸插件，注册生成、运动、碰撞检测、伤害解析和生命周期管理的系统。
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectileRenderAssets>()
            .add_message::<SpawnProjectile>()
            .add_message::<ProjectileHit>()
            .add_systems(
                FixedUpdate,
                spawn_projectiles
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                advance_path_projectiles
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (query_path_projectile_hits, collect_rapier_collision_events)
                    .in_set(GameSet::ContactRead)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                resolve_projectile_hits
                    .in_set(GameSet::Combat)
                    .before(crate::game::combat::apply_damage)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                cleanup_path_projectiles_outside_window
                    .in_set(GameSet::DeathAndCleanup)
                    .run_if(in_state(GameState::Playing)),
            );
        #[cfg(feature = "debug_tools")]
        app.add_systems(Update, projectile_sandbox_keys);
    }
}

/// 弹丸核心组件，携带伤害数据和生命周期计时器。
#[derive(Component, Debug)]
pub struct Projectile {
    /// 发射此弹丸的实体（用于伤害归属）。
    pub owner: Entity,
    /// 弹丸所属阵营（用于过滤友军伤害）。
    pub team: Team,
    /// 命中伤害值。
    pub damage: f32,
}

/// 弹丸运动模式组件。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProjectileMotion {
    /// 路径运动模式：无物理、线性移动、扫掠碰撞检测。
    Path,
    /// 物理运动模式：Rapier 刚体驱动、真实碰撞事件。
    Physics,
}

/// 内部组件：路径弹丸的速度向量。
#[derive(Component, Debug)]
struct PathVelocity(Vec2);

/// 内部组件：路径弹丸上一帧的位置，用于扫掠碰撞检测。
#[derive(Component, Debug)]
struct PreviousPosition(Vec2);

/// 弹丸逻辑和物理碰撞共用的半径。
#[derive(Component, Debug, Clone, Copy)]
struct ProjectileRadius(f32);

#[derive(Clone)]
struct ProjectileRenderAssetSet {
    border_mesh: Handle<Mesh>,
    fill_mesh: Handle<Mesh>,
    border_material: Handle<ColorMaterial>,
    fill_material: Handle<ColorMaterial>,
}

/// 按弹丸种类复用圆形网格与纯色材质，避免每次发射都创建永久资产。
#[derive(Resource, Default)]
struct ProjectileRenderAssets(HashMap<ProjectileKind, ProjectileRenderAssetSet>);

/// 命中策略组件，控制弹丸命中后的行为。
#[derive(Component, Debug)]
pub struct HitPolicy {
    /// 命中时是否立即销毁。
    pub destroy_on_hit: bool,
    /// 剩余可穿透目标数，用 `u8::MAX` 表示无限。
    pub remaining_pierces: u8,
}

/// 内部组件：已命中实体注册表，防止对同一目标的重复命中。
#[derive(Component, Debug, Default)]
struct HitRegistry(HashSet<Entity>);

/// 生成弹丸的请求消息。
#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnProjectile {
    /// 发射者实体。
    pub owner: Entity,
    /// 弹丸生成位置。
    pub origin: Vec2,
    /// 弹丸种类。
    pub kind: ProjectileKind,
}

/// 弹丸命中事件消息，由碰撞检测系统发出，[`resolve_projectile_hits`] 消费。
#[derive(Message, Debug, Clone, Copy)]
pub struct ProjectileHit {
    /// 命中的弹丸实体。
    pub projectile: Entity,
    /// 被命中的目标实体。
    pub target: Entity,
}

/// 消费 [`SpawnProjectile`] 消息，创建对应的弹丸实体。
///
/// 根据种类不同，普通豌豆附加路径运动组件，物理豌豆附加 Rapier 物理组件。
fn spawn_projectiles(
    mut commands: Commands,
    catalog: Res<ContentCatalog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut render_assets: ResMut<ProjectileRenderAssets>,
    mut requests: MessageReader<SpawnProjectile>,
) {
    for request in requests.read() {
        let definition = catalog.projectile(request.kind);
        let render = render_assets
            .0
            .entry(request.kind)
            .or_insert_with(|| ProjectileRenderAssetSet {
                border_mesh: meshes.add(Circle::new(definition.radius)),
                fill_mesh: meshes.add(Circle::new(
                    definition.radius - definition.visual.border_width,
                )),
                border_material: materials.add(definition.visual.border_color),
                fill_material: materials.add(definition.visual.fill_color),
            })
            .clone();

        // 共享基础组件：圆形描边、变换、Projectile 核心组件和命中注册表等
        let base = (
            Mesh2d(render.border_mesh),
            MeshMaterial2d(render.border_material),
            Transform::from_translation(request.origin.extend(3.0)),
            Projectile {
                owner: request.owner,
                team: Team::Plants,
                damage: definition.damage,
            },
            request.kind,
            ProjectileRadius(definition.radius),
            HitRegistry::default(),
            LevelEntity,
            Name::new(format!("{:?}", request.kind)),
        );

        let mut projectile = match definition.motion {
            ProjectileMotionDefinition::Path { velocity } => commands.spawn((
                base,
                ProjectileMotion::Path,
                PathVelocity(velocity),
                PreviousPosition(request.origin),
                HitPolicy {
                    destroy_on_hit: definition.hit_policy.destroy_on_hit,
                    remaining_pierces: definition.hit_policy.max_pierces,
                },
            )),
            ProjectileMotionDefinition::Physics {
                initial_velocity,
                gravity_scale,
                restitution,
                friction,
                ccd,
            } => commands.spawn((
                base,
                ProjectileMotion::Physics,
                HitPolicy {
                    destroy_on_hit: definition.hit_policy.destroy_on_hit,
                    remaining_pierces: definition.hit_policy.max_pierces,
                },
                RigidBody::Dynamic,
                Collider::ball(definition.radius),
                Velocity::linear(initial_velocity),
                Restitution::coefficient(restitution),
                Friction::coefficient(friction),
                GravityScale(gravity_scale),
                Ccd { enabled: ccd },
                ActiveEvents::COLLISION_EVENTS,
                physics_projectile_groups(),
            )),
        };
        projectile.with_child((
            Mesh2d(render.fill_mesh),
            MeshMaterial2d(render.fill_material),
            Transform::from_xyz(0.0, 0.0, 0.1),
            Name::new("Projectile fill"),
        ));
    }
}

/// 路径弹丸运动：每帧记录上一帧位置并按速度向量更新当前位置。
fn advance_path_projectiles(
    time: Res<Time<Fixed>>,
    mut projectiles: Query<
        (&mut Transform, &PathVelocity, &mut PreviousPosition),
        With<Projectile>,
    >,
) {
    for (mut transform, velocity, mut previous) in &mut projectiles {
        previous.0 = transform.translation.truncate();
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
    }
}

/// 路径弹丸的 Query 数据类型别名。
type PathProjectileData<'a> = (
    Entity,
    &'a Transform,
    &'a PreviousPosition,
    &'a HitRegistry,
    &'a ProjectileRadius,
);

/// 路径弹丸 Query 过滤条件。
type PathProjectileFilter = (With<Projectile>, With<PathVelocity>);

/// 路径弹丸碰撞检测：对每个弹丸，扫掠其上一帧到当前位置的线段，
/// 检测是否与道路上的僵尸发生碰撞（使用 swept_circle_hit_t 算法），取最近的命中。
fn query_path_projectile_hits(
    projectiles: Query<PathProjectileData<'_>, PathProjectileFilter>,
    zombies: Query<(Entity, &Transform, &ColliderHalfSize), With<Zombie>>,
    mut hits: MessageWriter<ProjectileHit>,
) {
    for (projectile, transform, previous, registry, radius) in &projectiles {
        let end = transform.translation.truncate();
        let mut nearest: Option<(f32, Entity)> = None;

        for (zombie, zombie_transform, collider) in &zombies {
            if registry.0.contains(&zombie) {
                continue;
            }
            let center = zombie_transform.translation.truncate();
            if let Some(t) = swept_circle_hit_t(previous.0, end, center, collider.0, radius.0)
                && nearest.is_none_or(|(best_t, _)| t < best_t)
            {
                nearest = Some((t, zombie));
            }
        }

        if let Some((_, target)) = nearest {
            hits.write(ProjectileHit { projectile, target });
        }
    }
}

/// 收集 Rapier2D 碰撞事件，筛选出物理弹丸与僵尸之间的碰撞，转化为 [`ProjectileHit`] 消息。
fn collect_rapier_collision_events(
    mut collisions: MessageReader<CollisionEvent>,
    projectiles: Query<&ProjectileMotion, With<Projectile>>,
    zombies: Query<(), With<Zombie>>,
    mut hits: MessageWriter<ProjectileHit>,
) {
    for collision in collisions.read() {
        let CollisionEvent::Started(a, b, _) = *collision else {
            continue;
        };
        let pair = if projectiles.get(a) == Ok(&ProjectileMotion::Physics) && zombies.contains(b) {
            Some((a, b))
        } else if projectiles.get(b) == Ok(&ProjectileMotion::Physics) && zombies.contains(a) {
            Some((b, a))
        } else {
            None
        };
        if let Some((projectile, target)) = pair {
            hits.write(ProjectileHit { projectile, target });
        }
    }
}

/// 处理所有 [`ProjectileHit`] 消息。
///
/// 对每个命中：
/// 1. 检查是否已注册过（防止重复命中同一目标）
/// 2. 检查弹丸阵营（仅植物弹丸造成伤害）
/// 3. 发出 [`ApplyDamage`] 伤害消息
/// 4. 按命中策略决定销毁弹丸或减少穿透计数
fn resolve_projectile_hits(
    mut commands: Commands,
    mut hits: MessageReader<ProjectileHit>,
    mut projectiles: Query<(&Projectile, &mut HitPolicy, &mut HitRegistry)>,
    mut damage: MessageWriter<ApplyDamage>,
    mut consumed: Local<HashSet<Entity>>,
) {
    consumed.clear();
    for hit in hits.read() {
        if consumed.contains(&hit.projectile) {
            continue;
        }
        let Ok((projectile, mut policy, mut registry)) = projectiles.get_mut(hit.projectile) else {
            continue;
        };
        if !registry.0.insert(hit.target) {
            continue;
        }
        if projectile.team != Team::Plants {
            continue;
        }
        damage.write(ApplyDamage {
            source: projectile.owner,
            target: hit.target,
            amount: projectile.damage,
            kind: DamageKind::Projectile,
        });

        if policy.destroy_on_hit || policy.remaining_pierces == 0 {
            consumed.insert(hit.projectile);
            commands.entity(hit.projectile).despawn();
        } else if policy.remaining_pierces != u8::MAX {
            policy.remaining_pierces = policy.remaining_pierces.saturating_sub(1);
        }
    }
}

/// 普通路径豌豆完全飞出当前窗口后销毁；物理豌豆不参与此清理。
fn cleanup_path_projectiles_outside_window(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    projectiles: Query<(Entity, &Transform, &ProjectileRadius), With<PathVelocity>>,
) {
    let half_window = Vec2::new(window.resolution.width(), window.resolution.height()) * 0.5;
    for (entity, transform, radius) in &projectiles {
        let position = transform.translation.truncate();
        if position.x + radius.0 < -half_window.x
            || position.x - radius.0 > half_window.x
            || position.y + radius.0 < -half_window.y
            || position.y - radius.0 > half_window.y
        {
            commands.entity(entity).despawn();
        }
    }
}

/// 调试用：N 键发射普通豌豆，P 键发射物理豌豆（沙盒模式）。
#[cfg(feature = "debug_tools")]
fn projectile_sandbox_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    layout: Res<LawnLayout>,
    state: Res<State<GameState>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    let origin = Vec2::new(layout.origin.x + 30.0, layout.path_y());
    let kind = if keyboard.just_pressed(controls.spawn_path_projectile) {
        Some(ProjectileKind::Pea)
    } else if keyboard.just_pressed(controls.spawn_physics_projectile) {
        Some(ProjectileKind::PhysicsPea)
    } else {
        None
    };
    if let Some(kind) = kind {
        spawn.write(SpawnProjectile {
            owner: Entity::PLACEHOLDER,
            origin,
            kind,
        });
    }
}

/// 将圆沿线段扫掠，检测与轴对齐盒体（AABB）的碰撞，返回首次接触时间参数 t ∈ [0, 1]。
///
/// 将盒体按圆半径扩展后，问题简化为线段与扩展 AABB 的板条（slab）测试。
/// `start` / `end`：线段的起点和终点；`center`：AABB 中心；
/// `half`：AABB 的半边长；`radius`：圆的半径。
/// 返回 `None` 表示无碰撞，`Some(t)` 表示在参数 t 处首次接触。
fn swept_circle_hit_t(
    start: Vec2,
    end: Vec2,
    center: Vec2,
    half: Vec2,
    radius: f32,
) -> Option<f32> {
    let expanded = half + Vec2::splat(radius);
    let min = center - expanded;
    let max = center + expanded;
    let direction = end - start;
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 1.0;

    for axis in 0..2 {
        let origin = start[axis];
        let delta = direction[axis];
        if delta.abs() < f32::EPSILON {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
            continue;
        }
        let inverse = 1.0 / delta;
        let mut near = (min[axis] - origin) * inverse;
        let mut far = (max[axis] - origin) * inverse;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        enter = enter.max(near);
        exit = exit.min(far);
        if enter > exit {
            return None;
        }
    }
    (exit >= 0.0 && enter <= 1.0).then_some(enter.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swept_query_hits_even_when_endpoints_miss() {
        let hit = swept_circle_hit_t(
            Vec2::new(-100.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::ZERO,
            Vec2::splat(20.0),
            5.0,
        );
        assert!(hit.is_some());
        assert!(hit.unwrap() > 0.0 && hit.unwrap() < 1.0);
    }

    #[test]
    fn swept_query_rejects_vertical_miss() {
        assert_eq!(
            swept_circle_hit_t(
                Vec2::new(-100.0, 100.0),
                Vec2::new(100.0, 100.0),
                Vec2::ZERO,
                Vec2::splat(20.0),
                5.0,
            ),
            None
        );
    }

    #[test]
    fn spawn_request_builds_distinct_motion_pipelines() {
        let mut app = App::new();
        app.add_message::<SpawnProjectile>()
            .init_resource::<ContentCatalog>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .init_resource::<ProjectileRenderAssets>()
            .add_systems(Update, spawn_projectiles);
        for kind in [ProjectileKind::Pea, ProjectileKind::PhysicsPea] {
            app.world_mut().write_message(SpawnProjectile {
                owner: Entity::PLACEHOLDER,
                origin: Vec2::ZERO,
                kind,
            });
        }

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<(
            &ProjectileKind,
            &ProjectileMotion,
            &ProjectileRadius,
            Option<&RigidBody>,
            Option<&PathVelocity>,
            Option<&Collider>,
        )>();
        let spawned: Vec<_> = query
            .iter(world)
            .map(|(kind, motion, radius, body, path, collider)| {
                (
                    *kind,
                    *motion,
                    radius.0,
                    body.copied(),
                    path.is_some(),
                    collider.is_some(),
                )
            })
            .collect();
        assert_eq!(spawned.len(), 2);
        let catalog = ContentCatalog::default();
        assert!(spawned.contains(&(
            ProjectileKind::Pea,
            ProjectileMotion::Path,
            catalog.projectile(ProjectileKind::Pea).radius,
            None,
            true,
            false,
        )));
        assert!(spawned.contains(&(
            ProjectileKind::PhysicsPea,
            ProjectileMotion::Physics,
            catalog.projectile(ProjectileKind::PhysicsPea).radius,
            Some(RigidBody::Dynamic),
            false,
            true,
        )));
    }
}
