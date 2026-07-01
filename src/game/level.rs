//! 关卡系统
//!
//! 管理单关的完整生命周期，包括：
//!
//! - **波次生成**：按时间线自动生成僵尸（[`tick_wave_timeline`]）
//! - **太阳经济**：太阳存款（[`SunBank`]）与植物卡片冷却（[`PlantCards`]）
//! - **鼠标交互**：左键点击或按住扫过太阳进行收集（[`handle_world_clicks`]）
//! - **胜负判定**：僵尸突破房子左侧 → 失败；清空所有僵尸 → 胜利
//! - **太阳动画**：太阳拾取物上下浮动并旋转

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use bevy::sprite::Text2dShadow;
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::game::assets::GameAssets;
use crate::game::catalog::{ContentCatalog, ZombieKind};
use crate::game::combat::{EntityDied, Team};
use crate::game::config::GameplaySettings;
use crate::game::controls::ControlBindings;
use crate::game::lawn::{CellOccupancy, LawnLayout};
use crate::game::pause::game_not_paused;
use crate::game::plant::PlantKind;
use crate::game::projectile::PeaPathArrivalEffect;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::theme::UiTheme;
use crate::game::zombie::{SpawnZombie, Zombie};

/// 默认关卡的外部 RON 配置路径。
pub const DEFAULT_LEVEL_PATH: &str = "assets/levels/level_01.ron";
const SUN_PICKUP_LIFETIME_SECONDS: f32 = 15.0;

/// 关卡插件，注册资源、消息和所有关卡管理相关的系统。
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<LevelDefinition>() {
            let definition = LevelDefinition::load_from_file(DEFAULT_LEVEL_PATH)
                .unwrap_or_else(|error| panic!("failed to load {DEFAULT_LEVEL_PATH}: {error}"));
            app.insert_resource(definition);
        }
        let (starting_sun, initial_cards) = {
            let definition = app.world().resource::<LevelDefinition>();
            (
                definition.starting_sun,
                definition
                    .cards
                    .iter()
                    .map(|card| (card.plant, Duration::ZERO))
                    .collect(),
            )
        };
        app.insert_resource(SunBank {
            amount: starting_sun,
        })
        .insert_resource(PlantCards(initial_cards))
        .init_resource::<ShovelMode>()
        .init_resource::<SunSweepState>()
        .init_resource::<LevelRuntime>()
        .add_message::<SpawnSun>()
        .add_message::<LevelWon>()
        .add_message::<LevelLost>()
        .add_systems(Startup, validate_level_definition)
        .add_systems(
            OnEnter(GameState::Playing),
            reset_level_runtime.in_set(LevelSetupSet::Reset),
        )
        .add_systems(
            FixedUpdate,
            (
                tick_wave_timeline.before(crate::game::zombie::spawn_zombies),
                spawn_sun_pickups,
            )
                .in_set(GameSet::Spawn)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            tick_card_cooldowns
                .in_set(GameSet::LogicMovement)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            (check_defeat, check_victory)
                .chain()
                .in_set(GameSet::LevelOutcome)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            count_defeated_zombies
                .in_set(GameSet::LevelOutcome)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (handle_world_clicks, animate_sun_pickups)
                .run_if(in_state(GameState::Playing))
                .run_if(game_not_paused),
        );
    }
}

/// 关卡内其他初始化系统应排在资源重置之后。
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LevelSetupSet {
    Reset,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LevelId(pub String);

/// 固定卡片列表项，定义显示顺序与植物映射。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PlantCardDefinition {
    pub slot: u8,
    pub plant: PlantKind,
}

/// 僵尸生成点定义：指定在什么时间生成哪种僵尸。
#[derive(Debug, Clone, Copy)]
pub struct ZombieSpawnDefinition {
    /// 相对于关卡开始的生成时间（秒）。
    pub at_seconds: f32,
    /// 生成的僵尸种类。
    pub kind: ZombieKind,
}

/// 单个波次的全部僵尸生成点。RON 中每个 `waves` 数组项对应一个显式 `wave`。
#[derive(Debug, Clone)]
pub struct ZombieWaveDefinition {
    /// 本波开始时间（秒），包含上一波结束后的等待时间。
    pub start_seconds: f32,
    pub spawns: Vec<ZombieSpawnDefinition>,
}

/// 关卡配置资源，定义所有僵尸波次。
#[derive(Resource, Debug, Clone)]
pub struct LevelDefinition {
    pub id: LevelId,
    pub display_name: String,
    pub starting_sun: u32,
    pub always_shoot: bool,
    pub pea_path_arrival_effect: PeaPathArrivalEffect,
    pub gatling_pea_upgrade_only: bool,
    pub lawn: LawnLayout,
    pub cards: Vec<PlantCardDefinition>,
    /// 保留波次边界的僵尸生成计划。
    pub waves: Vec<ZombieWaveDefinition>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelConfig {
    id: String,
    display_name: String,
    starting_sun: u32,
    #[serde(default)]
    always_shoot: bool,
    #[serde(default)]
    pea_path_arrival_effect: PeaPathArrivalEffect,
    #[serde(default)]
    gatling_pea_upgrade_only: bool,
    lawn: LawnConfig,
    waves: Vec<WaveConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LawnConfig {
    columns: u8,
    cell_size: (f32, f32),
    center_x: f32,
    path_y: f32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WaveConfig {
    /// 距上一波最后一只僵尸的时间；第一波表示开局等待。
    delay: f32,
    /// 本波内的刷怪条目；条目 delay 均相对于本波开始。
    wave: Vec<WaveSpawnConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WaveSpawnConfig {
    /// 距本波开始的时间。
    delay: f32,
    kind: ZombieKind,
    count: u32,
    /// 同一条刷怪条目内相邻僵尸之间的时间。
    interval: f32,
}

impl Default for LevelDefinition {
    fn default() -> Self {
        Self::from_ron_str(include_str!("../../assets/levels/level_01.ron"))
            .expect("invalid bundled level_01.ron")
    }
}

impl LevelDefinition {
    /// 从外部 RON 文件读取关卡；修改文件后重新启动游戏即可生效。
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        Self::from_ron_str(&source)
            .map_err(|error| format!("cannot parse {}: {error}", path.display()))
    }

    fn from_ron_str(source: &str) -> Result<Self, String> {
        let config: LevelConfig = ron::from_str(source).map_err(|error| error.to_string())?;
        let cell_size = Vec2::new(config.lawn.cell_size.0, config.lawn.cell_size.1);
        let lawn = LawnLayout {
            columns: config.lawn.columns,
            cell_size,
            origin: Vec2::new(
                config.lawn.center_x - (f32::from(config.lawn.columns) * cell_size.x) * 0.5,
                config.lawn.path_y - cell_size.y * 0.5,
            ),
        };
        let cards = default_plant_cards();
        let waves = expand_waves(config.waves)?;
        Ok(Self {
            id: LevelId(config.id),
            display_name: config.display_name,
            starting_sun: config.starting_sun,
            always_shoot: config.always_shoot,
            pea_path_arrival_effect: config.pea_path_arrival_effect,
            gatling_pea_upgrade_only: config.gatling_pea_upgrade_only,
            lawn,
            cards,
            waves,
        })
    }

    pub fn validate(&self, catalog: &ContentCatalog) -> Result<(), String> {
        if self.id.0.trim().is_empty() {
            return Err("level id must not be empty".into());
        }
        if self.display_name.trim().is_empty() {
            return Err("level display name must not be empty".into());
        }
        if self.lawn.columns == 0
            || !self.lawn.cell_size.is_finite()
            || !self.lawn.origin.is_finite()
            || self.lawn.cell_size.min_element() <= 0.0
        {
            return Err("lawn dimensions must be finite and positive".into());
        }
        if self.cards.is_empty() {
            return Err("level must contain at least one plant card".into());
        }
        for (index, card) in self.cards.iter().enumerate() {
            if !catalog.contains_plant(card.plant) {
                return Err(format!(
                    "card {} references missing plant {:?}",
                    card.slot, card.plant
                ));
            }
            if self.cards[..index]
                .iter()
                .any(|other| other.slot == card.slot)
            {
                return Err(format!("duplicate card slot {}", card.slot));
            }
            if self.cards[..index]
                .iter()
                .any(|other| other.plant == card.plant)
            {
                return Err(format!("duplicate plant card {:?}", card.plant));
            }
        }
        if self.waves.is_empty() {
            return Err("level must contain at least one zombie wave".into());
        }
        let mut previous = 0.0;
        for (wave_index, wave) in self.waves.iter().enumerate() {
            if wave.spawns.is_empty() {
                return Err(format!(
                    "wave {wave_index} must contain at least one zombie"
                ));
            }
            for (spawn_index, spawn) in wave.spawns.iter().enumerate() {
                if !spawn.at_seconds.is_finite() || spawn.at_seconds < 0.0 {
                    return Err(format!(
                        "wave {wave_index} spawn {spawn_index} has invalid time {}",
                        spawn.at_seconds
                    ));
                }
                if spawn.at_seconds < previous {
                    return Err(format!(
                        "spawn timeline is not sorted at wave {wave_index} spawn {spawn_index}"
                    ));
                }
                if !catalog.contains_zombie(spawn.kind) {
                    return Err(format!(
                        "wave {wave_index} spawn {spawn_index} references missing zombie {:?}",
                        spawn.kind
                    ));
                }
                previous = spawn.at_seconds;
            }
        }
        Ok(())
    }
}

fn default_plant_cards() -> Vec<PlantCardDefinition> {
    [
        PlantKind::Sunflower,
        PlantKind::TwinSunflower,
        PlantKind::Peashooter,
        PlantKind::Repeater,
        PlantKind::GatlingPea,
        PlantKind::SnowPea,
        PlantKind::WallNut,
        PlantKind::Torchwood,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, plant)| PlantCardDefinition {
        slot: index as u8 + 1,
        plant,
    })
    .collect()
}

fn expand_waves(waves: Vec<WaveConfig>) -> Result<Vec<ZombieWaveDefinition>, String> {
    let mut definitions = Vec::with_capacity(waves.len());
    let mut elapsed = 0.0;
    for (index, wave) in waves.into_iter().enumerate() {
        if !wave.delay.is_finite() || wave.delay < 0.0 {
            return Err(format!(
                "wave {index} delay must be finite and non-negative"
            ));
        }
        if wave.wave.is_empty() {
            return Err(format!(
                "wave {index} must contain at least one spawn entry"
            ));
        }

        elapsed += wave.delay;
        let wave_start = elapsed;
        let mut spawns = Vec::new();
        for (entry_index, entry) in wave.wave.into_iter().enumerate() {
            if !entry.delay.is_finite() || entry.delay < 0.0 {
                return Err(format!(
                    "wave {index} entry {entry_index} delay must be finite and non-negative"
                ));
            }
            if entry.count == 0 {
                return Err(format!(
                    "wave {index} entry {entry_index} count must be positive"
                ));
            }
            if !entry.interval.is_finite() || entry.interval < 0.0 {
                return Err(format!(
                    "wave {index} entry {entry_index} interval must be finite and non-negative"
                ));
            }
            if entry.count > 1 && entry.interval == 0.0 {
                return Err(format!(
                    "wave {index} entry {entry_index} interval must be positive when count is greater than one"
                ));
            }

            for spawn_index in 0..entry.count {
                spawns.push(ZombieSpawnDefinition {
                    at_seconds: wave_start + entry.delay + entry.interval * spawn_index as f32,
                    kind: entry.kind,
                });
            }
        }
        spawns.sort_by(|a, b| a.at_seconds.total_cmp(&b.at_seconds));
        elapsed = spawns
            .last()
            .map(|spawn| spawn.at_seconds)
            .unwrap_or(wave_start);
        definitions.push(ZombieWaveDefinition {
            start_seconds: wave_start,
            spawns,
        });
    }
    Ok(definitions)
}

/// 关卡运行时数据资源，追踪游戏进行中的状态。
#[derive(Resource, Debug, Default)]
pub struct LevelRuntime {
    /// 关卡已流逝的时间。
    pub elapsed: Duration,
    /// 当前等待生成的波次索引。
    pub next_wave: usize,
    /// 当前波次中下一个等待生成的僵尸索引。
    pub next_spawn_in_wave: usize,
    /// 已消灭的僵尸总数。
    pub defeated_zombies: usize,
}

impl LevelRuntime {
    pub fn waves_started(&self) -> usize {
        self.next_wave + usize::from(self.next_spawn_in_wave > 0)
    }
}

/// 太阳银行资源，存储玩家当前拥有的太阳数量。
#[derive(Resource, Debug, Default)]
pub struct SunBank {
    /// 太阳数量。
    pub amount: u32,
}

impl SunBank {
    /// 尝试花费太阳：如果余额充足则扣除并返回 `true`，否则返回 `false`。
    pub fn try_spend(&mut self, amount: u32) -> bool {
        if self.amount < amount {
            return false;
        }
        self.amount -= amount;
        true
    }
}

/// 植物卡片冷却资源，记录每种植物距离下次可用的剩余时间。
#[derive(Resource, Debug, Default)]
pub struct PlantCards(pub HashMap<PlantKind, Duration>);

impl PlantCards {
    /// 检查某植物卡片是否已冷却完毕（可用）。
    pub fn ready(&self, kind: PlantKind) -> bool {
        self.0.get(&kind).is_some_and(Duration::is_zero)
    }

    /// 触发冷却：使用后将冷却时间设为该植物的 `card_cooldown`。
    pub fn trigger(&mut self, kind: PlantKind, catalog: &ContentCatalog) {
        self.0.insert(kind, catalog.plant(kind).card_cooldown);
    }

    /// 查询某植物的剩余冷却时间。
    pub fn remaining(&self, kind: PlantKind) -> Duration {
        self.0.get(&kind).copied().unwrap_or(Duration::ZERO)
    }
}

/// 当前正在拖拽的铲子预览实体。
#[derive(Resource, Debug, Default)]
pub struct ShovelMode {
    pub preview: Option<Entity>,
}

impl ShovelMode {
    pub fn active(&self) -> bool {
        self.preview.is_some()
    }
}

/// 太阳拾取物组件，标记掉落的太阳实体。
#[derive(Component, Debug)]
pub struct SunPickup {
    /// 拾取后增加的太阳数量。
    pub value: u32,
    /// 初始 Y 坐标（用于浮动动画基准）。
    base_y: f32,
    /// 已存在时间（秒，用于驱动浮动和旋转动画）。
    age: f32,
}

#[derive(Resource, Debug, Default)]
struct SunSweepState {
    previous_world: Option<Vec2>,
    blocked_by_ui: bool,
}

/// 生成太阳的请求消息，由向日葵系统发出。
#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnSun {
    /// 太阳生成的位置。
    pub position: Vec2,
    /// 太阳的价值。
    pub value: u32,
}

/// 关卡胜利消息（目前未消费，保留以备扩展）。
#[derive(Message, Debug, Clone, Copy)]
pub struct LevelWon;

/// 关卡失败消息（目前未消费，保留以备扩展）。
#[derive(Message, Debug, Clone, Copy)]
pub struct LevelLost;

/// 进入 Playing 状态时重置所有关卡运行时数据。
fn reset_level_runtime(
    definition: Res<LevelDefinition>,
    mut runtime: ResMut<LevelRuntime>,
    mut bank: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut shovel: ResMut<ShovelMode>,
    mut sun_sweep: ResMut<SunSweepState>,
    mut layout: ResMut<LawnLayout>,
    mut occupancy: ResMut<CellOccupancy>,
) {
    *runtime = LevelRuntime::default();
    bank.amount = definition.starting_sun;
    cards.0 = definition
        .cards
        .iter()
        .map(|card| (card.plant, Duration::ZERO))
        .collect();
    shovel.preview = None;
    *sun_sweep = SunSweepState::default();
    *layout = definition.lawn.clone();
    occupancy.0.clear();
}

/// 波次时间线驱动：每帧累加已用时间，当达到某个生成点时发出 [`SpawnZombie`] 消息。
fn tick_wave_timeline(
    time: Res<Time<Fixed>>,
    definition: Res<LevelDefinition>,
    mut runtime: ResMut<LevelRuntime>,
    mut spawn: MessageWriter<SpawnZombie>,
) {
    runtime.elapsed += time.delta();
    while let Some(wave) = definition.waves.get(runtime.next_wave) {
        while let Some(next) = wave.spawns.get(runtime.next_spawn_in_wave)
            && runtime.elapsed.as_secs_f32() >= next.at_seconds
        {
            spawn.write(SpawnZombie { kind: next.kind });
            runtime.next_spawn_in_wave += 1;
        }
        if runtime.next_spawn_in_wave < wave.spawns.len() {
            break;
        }
        runtime.next_wave += 1;
        runtime.next_spawn_in_wave = 0;
    }
}

/// 每帧减少所有植物卡片的冷却时间。
fn tick_card_cooldowns(time: Res<Time<Fixed>>, mut cards: ResMut<PlantCards>) {
    for remaining in cards.0.values_mut() {
        *remaining = remaining.saturating_sub(time.delta());
    }
}

/// 消费 [`SpawnSun`] 消息，生成太阳拾取物实体。
fn spawn_sun_pickups(
    mut commands: Commands,
    assets: Res<GameAssets>,
    theme: Res<UiTheme>,
    mut requests: MessageReader<SpawnSun>,
) {
    for request in requests.read() {
        let label = &theme.sun_label;
        // 太阳数值名牌与拾取物组成父子关系，浮动时保持同步且便于整体销毁。
        let mut sun = commands.spawn((
            Sprite::from_color(theme.sun_color.with_alpha(0.0), Vec2::splat(theme.sun_size)),
            Transform::from_translation(request.position.extend(8.0)),
            SunPickup {
                value: request.value,
                base_y: request.position.y,
                age: 0.0,
            },
            LevelEntity,
            Name::new("太阳拾取物"),
        ));
        sun.with_children(|parent| {
            for index in 0..8 {
                let angle = index as f32 * std::f32::consts::FRAC_PI_4;
                let offset = Vec2::from_angle(angle) * 23.0;
                parent.spawn((
                    Sprite::from_color(Color::srgba(1.0, 0.72, 0.05, 0.88), Vec2::new(15.0, 5.0)),
                    Transform::from_xyz(offset.x, offset.y, 0.0)
                        .with_rotation(Quat::from_rotation_z(angle)),
                    Name::new("太阳光芒"),
                ));
            }
            parent.spawn((
                Sprite::from_color(Color::srgb(0.94, 0.55, 0.02), Vec2::splat(35.0)),
                Transform::from_xyz(0.0, 0.0, 0.1),
                Name::new("太阳外圈"),
            ));
            parent.spawn((
                Sprite::from_color(theme.sun_color, Vec2::splat(27.0)),
                Transform::from_xyz(0.0, 0.0, 0.2),
                Name::new("太阳内核"),
            ));
            parent.spawn((
                Sprite::from_color(Color::srgba(1.0, 1.0, 0.78, 0.82), Vec2::splat(8.0)),
                Transform::from_xyz(-7.0, 7.0, 0.3),
                Name::new("太阳高光"),
            ));
            parent.spawn((
                Text2d::new(format!("太阳 +{}", request.value)),
                TextFont {
                    font: assets.chinese_font.clone(),
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
                Transform::from_xyz(label.offset.x, label.offset.y, 2.0),
                Name::new("太阳数值"),
            ));
        });
    }
}

/// 自定义 SystemParam，封装鼠标点击所需的全部系统参数。
#[derive(SystemParam)]
struct WorldClickParams<'w, 's> {
    commands: Commands<'w, 's>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    controls: Res<'w, ControlBindings>,
    settings: Res<'w, GameplaySettings>,
    window: Single<'w, 's, &'static Window>,
    camera: Single<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    ui_buttons: Query<'w, 's, &'static Interaction, With<Button>>,
    pickups: Query<'w, 's, (Entity, &'static Transform, &'static SunPickup)>,
    bank: ResMut<'w, SunBank>,
    sweep: ResMut<'w, SunSweepState>,
}

/// 处理鼠标点击和按住后的连续扫掠收集。
///
/// 从世界区域开始按住时，将鼠标相邻两帧位置连成线段并收集扫过的太阳。
/// 从 UI 按钮开始的拖拽会保持屏蔽，避免植物和铲子操作误收集。
fn handle_world_clicks(mut params: WorldClickParams) {
    let button = params.controls.place_or_collect;
    if !params.mouse.pressed(button) {
        *params.sweep = SunSweepState::default();
        return;
    }
    if params.mouse.just_pressed(button) {
        // 从 UI 开始的整段拖拽都不收集，避免植物卡片和铲子拖拽误触草坪。
        params.sweep.blocked_by_ui = params
            .ui_buttons
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed);
        params.sweep.previous_world = None;
    }
    if params.sweep.blocked_by_ui {
        return;
    }
    let Some(cursor) = params.window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = *params.camera;
    let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        params.sweep.previous_world = None;
        return;
    };

    let sweep_start = params.sweep.previous_world.unwrap_or(world);
    params.sweep.previous_world = Some(world);
    let pickup_radius_squared = params.settings.sun_pickup_radius.powi(2);
    let mut collected = 0;
    for (entity, transform, pickup) in &params.pickups {
        if point_segment_distance_squared(transform.translation.truncate(), sweep_start, world)
            <= pickup_radius_squared
        {
            collected += pickup.value;
            params.commands.entity(entity).despawn();
        }
    }
    params.bank.amount += collected;
}

fn point_segment_distance_squared(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_squared();
    if length_squared <= f32::EPSILON {
        return point.distance_squared(start);
    }
    let t = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance_squared(start + segment * t)
}

/// 太阳拾取物动画：上下浮动。名牌需要保持水平，因此不旋转整个实体。
fn animate_sun_pickups(
    mut commands: Commands,
    time: Res<Time>,
    mut pickups: Query<(Entity, &mut Transform, &mut SunPickup)>,
) {
    for (entity, mut transform, mut pickup) in &mut pickups {
        pickup.age += time.delta_secs();
        if pickup.age >= SUN_PICKUP_LIFETIME_SECONDS {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation.y = pickup.base_y + (pickup.age * 2.2).sin() * 6.0;
    }
}

/// 失败判定：如果有僵尸的 X 坐标 ≤ 棋盘原点左 16 像素，则进入 Defeat 状态。
fn check_defeat(
    zombies: Query<&Transform, With<Zombie>>,
    layout: Res<LawnLayout>,
    settings: Res<GameplaySettings>,
    mut lost: MessageWriter<LevelLost>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if zombies
        .iter()
        .any(|transform| transform.translation.x <= layout.origin.x - settings.defeat_offset_x)
    {
        lost.write(LevelLost);
        next_state.set(GameState::Defeat);
    }
}

fn validate_level_definition(
    definition: Res<LevelDefinition>,
    catalog: Res<ContentCatalog>,
    controls: Res<ControlBindings>,
) {
    definition
        .validate(&catalog)
        .expect("invalid loaded level definition");
    controls.validate().expect("invalid control bindings");
}

/// 胜利判定：所有波次已生成完毕且场上无存活僵尸，则进入 Victory 状态。
fn check_victory(
    definition: Res<LevelDefinition>,
    runtime: Res<LevelRuntime>,
    zombies: Query<(), With<Zombie>>,
    mut won: MessageWriter<LevelWon>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if runtime.next_wave == definition.waves.len() && zombies.is_empty() {
        won.write(LevelWon);
        next_state.set(GameState::Victory);
    }
}

/// 统计已消灭的僵尸数量，并输出调试日志。
fn count_defeated_zombies(
    mut deaths: MessageReader<EntityDied>,
    mut runtime: ResMut<LevelRuntime>,
) {
    for death in deaths.read() {
        if death.team == Team::Zombies {
            runtime.defeated_zombies += 1;
        }
        debug!(
            "entity {:?} ({:?}) was killed by {:?}",
            death.entity, death.team, death.killer
        );
    }
}

#[cfg(test)]
mod tests;
