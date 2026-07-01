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
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

use crate::game::assets::GameAssets;
use crate::game::catalog::{ColliderCenterOffset, ColliderHalfSize, ContentCatalog};
use crate::game::combat::{ApplyDamage, DamageKind, EquipmentHealth, Health, Team};
use crate::game::lawn::{GridCell, LawnLayout};
use crate::game::model::{
    ZombieModelDetail, model_bounds, model_parts_mesh, zombie_model_parts_with_detail,
};
use crate::game::physics::zombie_groups;
use crate::game::plant::Plant;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::theme::{SceneLabelStyle, UiTheme};

pub use crate::game::catalog::ZombieKind;

const CHILL_DURATION: Duration = Duration::from_secs(10);
const CHILL_TIME_SCALE: f32 = 0.5;
const HORDE_SIMPLIFICATION_THRESHOLD: usize = 4_000;
const HORDE_FULL_DETAIL_RESTORE_THRESHOLD: usize = 3_200;
const ZOMBIE_RENDER_Z_BASE: f32 = 2.0;
const ZOMBIE_RENDER_Z_STEP: f32 = 0.000_001;
const ZOMBIE_RENDER_Z_SLOTS: u32 = 100_000;

/// 僵尸插件，注册生成、状态更新、行走和攻击系统。
pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZombieRenderAssets>()
            .init_resource::<ZombieRenderQuality>()
            .init_resource::<ZombieRenderSequence>()
            .add_message::<SpawnZombie>()
            .add_systems(
                Update,
                sync_zombie_render_quality.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                ensure_zombie_debug_children.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (update_zombie_health_debug, update_zombie_equipment_model)
                    .run_if(in_state(GameState::Playing)),
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
                    tick_chilled_zombies,
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

/// 冰豌豆施加的减速状态。
#[derive(Component, Debug)]
pub struct Chilled {
    timer: Timer,
}

impl Chilled {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(CHILL_DURATION, TimerMode::Once),
        }
    }
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

#[derive(Component)]
struct ZombieNameText;

#[derive(Clone)]
struct ZombieRenderMeshes {
    intact: Handle<Mesh>,
    broken_equipment: Handle<Mesh>,
}

impl ZombieRenderMeshes {
    fn mesh(&self, equipment_broken: bool) -> Handle<Mesh> {
        if equipment_broken {
            self.broken_equipment.clone()
        } else {
            self.intact.clone()
        }
    }
}

#[derive(Clone)]
struct ZombieRenderAssetSet {
    full: ZombieRenderMeshes,
    simplified: ZombieRenderMeshes,
    bounds: crate::game::model::ModelBounds,
}

impl ZombieRenderAssetSet {
    fn meshes(&self, detail: ZombieModelDetail) -> &ZombieRenderMeshes {
        match detail {
            ZombieModelDetail::Full => &self.full,
            ZombieModelDetail::Simplified => &self.simplified,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ZombieRenderAssets {
    material: Option<Handle<ColorMaterial>>,
    models: HashMap<ZombieKind, ZombieRenderAssetSet>,
}

#[derive(Resource)]
pub(crate) struct ZombieRenderQuality {
    detail: ZombieModelDetail,
}

impl Default for ZombieRenderQuality {
    fn default() -> Self {
        Self {
            detail: ZombieModelDetail::Full,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct ZombieRenderSequence(u32);

#[derive(Component)]
struct ZombieDebugOverlaySpawned;

/// 用于避免同一系统中多组查询冲突的排除过滤。
type ZombieHealthTextFilter = (
    With<ZombieHealthText>,
    Without<ZombieHealthBarFill>,
    Without<ZombieHealthBarBackground>,
    Without<ZombieNameText>,
);

type ZombieHealthFillFilter = (
    With<ZombieHealthBarFill>,
    Without<ZombieHealthText>,
    Without<ZombieHealthBarBackground>,
    Without<ZombieNameText>,
);

type ZombieHealthBackgroundFilter = (
    With<ZombieHealthBarBackground>,
    Without<ZombieHealthText>,
    Without<ZombieHealthBarFill>,
    Without<ZombieNameText>,
);

type ZombieNameTextFilter = (
    With<ZombieNameText>,
    Without<ZombieHealthText>,
    Without<ZombieHealthBarFill>,
    Without<ZombieHealthBarBackground>,
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
    debug: Option<Res<DebugRenderContext>>,
    theme: Option<Res<UiTheme>>,
    catalog: Res<ContentCatalog>,
    mut requests: MessageReader<SpawnZombie>,
    layout: Res<LawnLayout>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut render_assets: ResMut<ZombieRenderAssets>,
    render_quality: Res<ZombieRenderQuality>,
    mut render_sequence: Option<ResMut<ZombieRenderSequence>>,
) {
    let debug_enabled = debug.as_ref().is_some_and(|context| context.enabled);

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
        let render = zombie_render_assets(
            request.kind,
            &mut meshes,
            &mut materials,
            &mut render_assets,
        );
        let render_meshes = render.meshes(render_quality.detail);
        let material = render_assets.material.as_ref().unwrap().clone();
        let render_z = render_sequence
            .as_deref_mut()
            .map(next_zombie_render_z)
            .unwrap_or(ZOMBIE_RENDER_Z_BASE);
        // 根节点承担逻辑和完整模型渲染，装备破坏时直接切换共享网格。
        let mut entity = commands.spawn((
            (
                Mesh2d(render_meshes.mesh(false)),
                MeshMaterial2d(material),
                Transform::from_translation(position.extend(render_z)),
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
                Collider::compound(vec![(
                    render.bounds.center,
                    0.0,
                    Collider::cuboid(render.bounds.half_size.x, render.bounds.half_size.y),
                )]),
                ColliderHalfSize(render.bounds.half_size),
                ColliderCenterOffset(render.bounds.center),
                LockedAxes::ROTATION_LOCKED,
                ActiveEvents::COLLISION_EVENTS,
                zombie_groups(),
                LevelEntity,
                Name::new(definition.display_name),
            ),
        ));
        if let Some(equipment_health) = definition.equipment_health {
            entity.insert(EquipmentHealth::new(equipment_health));
        }
        if debug_enabled {
            entity.insert(ZombieDebugOverlaySpawned);
        }
        if debug_enabled {
            entity.with_children(|parent| {
                let full_health = definition.health + definition.equipment_health.unwrap_or(0.0);
                spawn_zombie_debug_overlay(
                    parent,
                    definition.scene_label,
                    full_health,
                    font,
                    label,
                );
            });
        }
    }
}

fn zombie_render_assets(
    kind: ZombieKind,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    render_assets: &mut ZombieRenderAssets,
) -> ZombieRenderAssetSet {
    if let Some(render) = render_assets.models.get(&kind) {
        return render.clone();
    }
    render_assets
        .material
        .get_or_insert_with(|| materials.add(ColorMaterial::default()));

    let full_parts = zombie_model_parts_with_detail(kind, 1.0, ZombieModelDetail::Full);
    let simplified_parts = zombie_model_parts_with_detail(kind, 1.0, ZombieModelDetail::Simplified);
    let render = ZombieRenderAssetSet {
        bounds: model_bounds(&full_parts),
        full: zombie_render_meshes(&full_parts, meshes),
        simplified: zombie_render_meshes(&simplified_parts, meshes),
    };
    render_assets.models.insert(kind, render.clone());
    render
}

fn zombie_render_meshes(
    parts: &[crate::game::model::ModelPart],
    meshes: &mut Assets<Mesh>,
) -> ZombieRenderMeshes {
    let intact = meshes.add(model_parts_mesh(parts));
    let body: Vec<_> = parts
        .iter()
        .copied()
        .filter(|part| !part.is_equipment)
        .collect();
    let has_equipment = body.len() != parts.len();
    ZombieRenderMeshes {
        broken_equipment: if has_equipment {
            meshes.add(model_parts_mesh(&body))
        } else {
            intact.clone()
        },
        intact,
    }
}

fn next_zombie_render_z(sequence: &mut ZombieRenderSequence) -> f32 {
    let slot = sequence.0 % ZOMBIE_RENDER_Z_SLOTS;
    sequence.0 = sequence.0.wrapping_add(1);
    ZOMBIE_RENDER_Z_BASE + slot as f32 * ZOMBIE_RENDER_Z_STEP
}

fn sync_zombie_render_quality(
    mut quality: ResMut<ZombieRenderQuality>,
    render_assets: Res<ZombieRenderAssets>,
    mut zombies: Query<(&ZombieKind, Option<&EquipmentHealth>, &mut Mesh2d), With<Zombie>>,
) {
    let zombie_count = zombies.iter().len();
    let next_detail = render_detail_with_hysteresis(
        zombie_count,
        quality.detail,
        HORDE_SIMPLIFICATION_THRESHOLD,
        HORDE_FULL_DETAIL_RESTORE_THRESHOLD,
    );
    if next_detail == quality.detail {
        return;
    }
    quality.detail = next_detail;

    for (kind, equipment, mut mesh) in &mut zombies {
        let Some(render) = render_assets.models.get(kind) else {
            continue;
        };
        let desired = render.meshes(next_detail);
        mesh.0 = desired.mesh(equipment.is_some_and(EquipmentHealth::is_broken));
    }
}

fn render_detail_with_hysteresis(
    count: usize,
    current: ZombieModelDetail,
    simplify_at: usize,
    restore_at: usize,
) -> ZombieModelDetail {
    match current {
        ZombieModelDetail::Full if count >= simplify_at => ZombieModelDetail::Simplified,
        ZombieModelDetail::Simplified if count <= restore_at => ZombieModelDetail::Full,
        _ => current,
    }
}

fn ensure_zombie_debug_children(
    mut commands: Commands,
    debug: Res<DebugRenderContext>,
    assets: Option<Res<GameAssets>>,
    theme: Option<Res<UiTheme>>,
    catalog: Res<ContentCatalog>,
    zombies: Query<
        (Entity, &ZombieKind, &Health, Option<&EquipmentHealth>),
        (With<Zombie>, Without<ZombieDebugOverlaySpawned>),
    >,
) {
    if !debug.enabled {
        return;
    }

    let fallback_theme = UiTheme::default();
    let label = theme
        .as_ref()
        .map(|theme| &theme.zombie_label)
        .unwrap_or(&fallback_theme.zombie_label);
    let font = assets
        .as_ref()
        .map(|assets| assets.chinese_font.clone())
        .unwrap_or_default();

    for (entity, kind, health, equipment) in &zombies {
        let definition = catalog.zombie(*kind);
        let full_health = health.max + equipment.map_or(0.0, |item| item.max);
        commands
            .entity(entity)
            .insert(ZombieDebugOverlaySpawned)
            .with_children(|parent| {
                spawn_zombie_debug_overlay(
                    parent,
                    definition.scene_label,
                    full_health,
                    font.clone(),
                    label,
                );
            });
    }
}

fn spawn_zombie_debug_overlay(
    parent: &mut ChildSpawnerCommands,
    scene_label: &str,
    full_health: f32,
    font: Handle<Font>,
    label: &SceneLabelStyle,
) {
    parent.spawn((
        Text2d::new(scene_label),
        TextFont {
            font: font.clone(),
            font_size: label.font_size,
            ..default()
        },
        TextColor(label.text),
        TextBackgroundColor(label.background),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(label.offset.x, label.offset.y, 3.0),
        Visibility::Hidden,
        ZombieNameText,
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
        Text2d::new(format!("{full_health:.0} / {full_health:.0}")),
        TextFont {
            font,
            font_size: 9.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 63.0, 4.2),
        Visibility::Hidden,
        ZombieHealthText,
        Name::new("僵尸血量数值"),
    ));
}

/// 物理 debug 渲染开启时，同步显示僵尸的实时血量数值和血条。
fn update_zombie_health_debug(
    debug: Res<DebugRenderContext>,
    zombies: Query<
        (Ref<Health>, Option<Ref<EquipmentHealth>>, &Children),
        (With<Zombie>, With<ZombieDebugOverlaySpawned>),
    >,
    mut texts: Query<(&mut Text2d, &mut Visibility), ZombieHealthTextFilter>,
    mut fills: Query<(&mut Sprite, &mut Transform, &mut Visibility), ZombieHealthFillFilter>,
    mut backgrounds: Query<&mut Visibility, ZombieHealthBackgroundFilter>,
    mut name_texts: Query<&mut Visibility, ZombieNameTextFilter>,
) {
    if !debug.enabled && !debug.is_changed() {
        return;
    }

    let visibility = if debug.enabled {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for (health, equipment, children) in &zombies {
        let should_refresh = debug.is_changed()
            || health.is_changed()
            || equipment.as_ref().is_some_and(|item| item.is_changed());
        if !should_refresh {
            continue;
        }

        let equipment_current = equipment.as_ref().map(|item| item.current).unwrap_or(0.0);
        let equipment_max = equipment.as_ref().map(|item| item.max).unwrap_or(0.0);
        let current = health.current + equipment_current;
        let max = health.max + equipment_max;
        let ratio = (current / max).clamp(0.0, 1.0);
        for child in children.iter() {
            if let Ok((mut text, mut child_visibility)) = texts.get_mut(child) {
                if debug.enabled {
                    text.0 = if equipment_max > 0.0 {
                        format!("{current:.0} / {max:.0}  装备 {equipment_current:.0}")
                    } else {
                        format!("{current:.0} / {max:.0}")
                    };
                }
                *child_visibility = visibility;
            }
            if let Ok((mut sprite, mut transform, mut child_visibility)) = fills.get_mut(child) {
                if debug.enabled {
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
                }
                *child_visibility = visibility;
            }
            if let Ok(mut child_visibility) = backgrounds.get_mut(child) {
                *child_visibility = visibility;
            }
            if let Ok(mut child_visibility) = name_texts.get_mut(child) {
                *child_visibility = visibility;
            }
        }
    }
}

/// 装备血量耗尽后切换为不含装备的合并模型。
fn update_zombie_equipment_model(
    render_assets: Res<ZombieRenderAssets>,
    render_quality: Res<ZombieRenderQuality>,
    mut zombies: Query<
        (&ZombieKind, &EquipmentHealth, &mut Mesh2d),
        (
            With<Zombie>,
            Or<(Added<EquipmentHealth>, Changed<EquipmentHealth>)>,
        ),
    >,
) {
    for (kind, equipment, mut mesh) in &mut zombies {
        let Some(render) = render_assets.models.get(kind) else {
            continue;
        };
        mesh.0 = render
            .meshes(render_quality.detail)
            .mesh(equipment.is_broken());
    }
}

/// 更新僵尸状态：检测道路前方的底层植物，切换 Walking / Eating。
///
/// 判断逻辑：检查僵尸前方距离 [-12, 62] 像素范围内是否存在植物，
/// 取最近的植物作为啃食目标；无植物则保持 Walking。
fn update_zombie_state(
    mut zombies: Query<(&Zombie, &Transform, &mut ZombieState)>,
    plants: Query<(Entity, &Transform, &GridCell), With<Plant>>,
    mut plant_x_index: Local<Vec<(f32, Entity)>>,
) {
    plant_x_index.clear();
    plant_x_index.extend(
        plants
            .iter()
            .filter(|(_, _, cell)| cell.is_ground())
            .map(|(entity, transform, _)| (transform.translation.x, entity)),
    );
    plant_x_index.sort_unstable_by(|left, right| left.0.total_cmp(&right.0));

    for (zombie, zombie_transform, mut state) in &mut zombies {
        let zombie_x = zombie_transform.translation.x;
        let first =
            plant_x_index.partition_point(|(plant_x, _)| *plant_x < zombie_x - zombie.engage_max);
        let last =
            plant_x_index.partition_point(|(plant_x, _)| *plant_x <= zombie_x - zombie.engage_min);
        let blocker = plant_x_index[first..last]
            .iter()
            .filter_map(|(plant_x, entity)| {
                let distance = zombie_x - *plant_x;
                (zombie.engage_min..=zombie.engage_max)
                    .contains(&distance)
                    .then_some((distance.abs(), *entity))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, entity)| entity);

        let next_state = blocker
            .map(|target| ZombieState::Eating { target })
            .unwrap_or(ZombieState::Walking);
        if *state != next_state {
            *state = next_state;
        }
    }
}

/// 处于 Walking 状态的僵尸向左移动。
fn tick_chilled_zombies(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut zombies: Query<(Entity, &mut Chilled)>,
) {
    for (entity, mut chilled) in &mut zombies {
        chilled.timer.tick(time.delta());
        if chilled.timer.is_finished() {
            commands.entity(entity).remove::<Chilled>();
        }
    }
}

fn advance_walking_zombies(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(&Zombie, &ZombieState, &mut Transform, Option<&Chilled>)>,
) {
    for (zombie, state, mut transform, chilled) in &mut zombies {
        if *state == ZombieState::Walking {
            transform.translation.x -=
                effective_zombie_speed(zombie.speed, chilled.is_some()) * time.delta_secs();
        }
    }
}

fn effective_zombie_speed(base_speed: f32, chilled: bool) -> f32 {
    if chilled {
        base_speed * CHILL_TIME_SCALE
    } else {
        base_speed
    }
}

/// 处于 Eating 状态的僵尸按攻击间隔对目标植物造成伤害。
///
/// 如果目标植物已被销毁，则重置计时器并等待下次状态更新。
fn tick_zombie_attacks(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(
        Entity,
        &Zombie,
        &ZombieState,
        &mut AttackTimer,
        Option<&Chilled>,
    )>,
    plants: Query<(), With<Plant>>,
    mut damage: MessageWriter<ApplyDamage>,
) {
    for (entity, zombie, state, mut timer, chilled) in &mut zombies {
        let ZombieState::Eating { target } = *state else {
            timer.0.reset();
            continue;
        };
        if !plants.contains(target) {
            continue;
        }
        let delta = if chilled.is_some() {
            time.delta().mul_f32(CHILL_TIME_SCALE)
        } else {
            time.delta()
        };
        timer.0.tick(delta);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zombie_render_detail_uses_hysteresis() {
        assert_eq!(
            render_detail_with_hysteresis(10, ZombieModelDetail::Full, 10, 8),
            ZombieModelDetail::Simplified
        );
        assert_eq!(
            render_detail_with_hysteresis(9, ZombieModelDetail::Simplified, 10, 8),
            ZombieModelDetail::Simplified
        );
        assert_eq!(
            render_detail_with_hysteresis(8, ZombieModelDetail::Simplified, 10, 8),
            ZombieModelDetail::Full
        );
    }
}
