//! 关卡系统
//!
//! 管理单关的完整生命周期，包括：
//!
//! - **波次生成**：按时间线自动生成僵尸（[`tick_wave_timeline`]）
//! - **太阳经济**：太阳存款（[`SunBank`]）与植物卡片冷却（[`PlantCards`]）
//! - **植物选择**：点击 UI 卡片或按数字键切换当前选中的植物（[`SelectedPlant`]）
//! - **鼠标交互**：左键点击收集太阳、在棋盘上放置植物（[`handle_world_clicks`]）
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
use crate::game::plant::{PlantKind, PlantRequest};
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::theme::UiTheme;
use crate::game::zombie::{SpawnZombie, Zombie};

/// 默认关卡的外部 RON 配置路径。
pub const DEFAULT_LEVEL_PATH: &str = "assets/levels/level_01.ron";

/// 关卡插件，注册资源、消息和所有关卡管理相关的系统。
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<LevelDefinition>() {
            let definition = LevelDefinition::load_from_file(DEFAULT_LEVEL_PATH)
                .unwrap_or_else(|error| panic!("failed to load {DEFAULT_LEVEL_PATH}: {error}"));
            app.insert_resource(definition);
        }
        let (starting_sun, default_plant, initial_cards) = {
            let definition = app.world().resource::<LevelDefinition>();
            (
                definition.starting_sun,
                definition.default_plant,
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
        .insert_resource(SelectedPlant(default_plant))
        .insert_resource(PlantCards(initial_cards))
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
            (select_plant_card, handle_world_clicks, animate_sun_pickups)
                .run_if(in_state(GameState::Playing)),
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

/// 卡片列表是顺序、快捷键与植物映射的唯一来源。
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PlantCardDefinition {
    pub slot: u8,
    pub key: KeyCode,
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

/// 关卡配置资源，定义所有僵尸波次。
#[derive(Resource, Debug, Clone)]
pub struct LevelDefinition {
    pub id: LevelId,
    pub display_name: String,
    pub starting_sun: u32,
    pub lawn: LawnLayout,
    pub cards: Vec<PlantCardDefinition>,
    pub default_plant: PlantKind,
    /// 按时间排序的僵尸生成队列。
    pub spawns: Vec<ZombieSpawnDefinition>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LevelConfig {
    id: String,
    display_name: String,
    starting_sun: u32,
    lawn: LawnConfig,
    cards: Vec<PlantCardConfig>,
    default_plant: PlantKind,
    waves: Vec<WaveConfig>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LawnConfig {
    columns: u8,
    cell_size: (f32, f32),
    path_y: f32,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlantCardConfig {
    slot: u8,
    plant: PlantKind,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WaveConfig {
    /// 距上一波最后一只僵尸的时间；第一波表示开局等待。
    delay: f32,
    kind: ZombieKind,
    count: u32,
    /// 同一波相邻僵尸之间的时间。
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
                -(f32::from(config.lawn.columns) * cell_size.x) * 0.5,
                config.lawn.path_y - cell_size.y * 0.5,
            ),
        };
        let cards = config
            .cards
            .into_iter()
            .map(|card| {
                Ok(PlantCardDefinition {
                    slot: card.slot,
                    key: card_key(card.slot).ok_or_else(|| {
                        format!("card slot {} has no numeric shortcut", card.slot)
                    })?,
                    plant: card.plant,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let spawns = expand_waves(config.waves)?;
        Ok(Self {
            id: LevelId(config.id),
            display_name: config.display_name,
            starting_sun: config.starting_sun,
            lawn,
            cards,
            default_plant: config.default_plant,
            spawns,
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
                .any(|other| other.key == card.key)
            {
                return Err(format!("duplicate card key {:?}", card.key));
            }
        }
        if !self
            .cards
            .iter()
            .any(|card| card.plant == self.default_plant)
        {
            return Err(format!(
                "default plant {:?} is not present in cards",
                self.default_plant
            ));
        }
        let mut previous = 0.0;
        for (index, spawn) in self.spawns.iter().enumerate() {
            if !spawn.at_seconds.is_finite() || spawn.at_seconds < 0.0 {
                return Err(format!(
                    "spawn {index} has invalid time {}",
                    spawn.at_seconds
                ));
            }
            if index > 0 && spawn.at_seconds < previous {
                return Err(format!("spawn timeline is not sorted at index {index}"));
            }
            if !catalog.contains_zombie(spawn.kind) {
                return Err(format!(
                    "spawn {index} references missing zombie {:?}",
                    spawn.kind
                ));
            }
            previous = spawn.at_seconds;
        }
        Ok(())
    }
}

fn expand_waves(waves: Vec<WaveConfig>) -> Result<Vec<ZombieSpawnDefinition>, String> {
    let mut spawns = Vec::new();
    let mut elapsed = 0.0;
    for (index, wave) in waves.into_iter().enumerate() {
        if !wave.delay.is_finite() || wave.delay < 0.0 {
            return Err(format!(
                "wave {index} delay must be finite and non-negative"
            ));
        }
        if wave.count == 0 {
            return Err(format!("wave {index} count must be positive"));
        }
        if !wave.interval.is_finite() || wave.interval < 0.0 {
            return Err(format!(
                "wave {index} interval must be finite and non-negative"
            ));
        }
        if wave.count > 1 && wave.interval == 0.0 {
            return Err(format!(
                "wave {index} interval must be positive when count is greater than one"
            ));
        }

        elapsed += wave.delay;
        for spawn_index in 0..wave.count {
            spawns.push(ZombieSpawnDefinition {
                at_seconds: elapsed,
                kind: wave.kind,
            });
            if spawn_index + 1 < wave.count {
                elapsed += wave.interval;
            }
        }
    }
    Ok(spawns)
}

fn card_key(slot: u8) -> Option<KeyCode> {
    Some(match slot {
        1 => KeyCode::Digit1,
        2 => KeyCode::Digit2,
        3 => KeyCode::Digit3,
        4 => KeyCode::Digit4,
        5 => KeyCode::Digit5,
        6 => KeyCode::Digit6,
        7 => KeyCode::Digit7,
        8 => KeyCode::Digit8,
        9 => KeyCode::Digit9,
        _ => return None,
    })
}

/// 关卡运行时数据资源，追踪游戏进行中的状态。
#[derive(Resource, Debug, Default)]
pub struct LevelRuntime {
    /// 关卡已流逝的时间。
    pub elapsed: Duration,
    /// 下一个要生成的波次索引（= 已生成的波次数）。
    pub next_spawn: usize,
    /// 已消灭的僵尸总数。
    pub defeated_zombies: usize,
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

/// 当前选中的植物资源，由 UI 卡片或数字键切换。
#[derive(Resource, Debug)]
pub struct SelectedPlant(pub PlantKind);

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
    mut selected: ResMut<SelectedPlant>,
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
    selected.0 = definition.default_plant;
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
    while let Some(next) = definition.spawns.get(runtime.next_spawn)
        && runtime.elapsed.as_secs_f32() >= next.at_seconds
    {
        spawn.write(SpawnZombie { kind: next.kind });
        runtime.next_spawn += 1;
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
        commands.spawn((
            Sprite::from_color(theme.sun_color, Vec2::splat(theme.sun_size)),
            Transform::from_translation(request.position.extend(8.0)),
            SunPickup {
                value: request.value,
                base_y: request.position.y,
                age: 0.0,
            },
            LevelEntity,
            Name::new("太阳拾取物"),
            children![(
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
            )],
        ));
    }
}

/// 根据当前关卡卡片定义切换选中的植物种类。
fn select_plant_card(
    keyboard: Res<ButtonInput<KeyCode>>,
    definition: Res<LevelDefinition>,
    mut selected: ResMut<SelectedPlant>,
) {
    for card in &definition.cards {
        if keyboard.just_pressed(card.key) {
            selected.0 = card.plant;
            break;
        }
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
    layout: Res<'w, LawnLayout>,
    selected: Res<'w, SelectedPlant>,
    ui_buttons: Query<'w, 's, &'static Interaction, With<Button>>,
    pickups: Query<'w, 's, (Entity, &'static Transform, &'static SunPickup)>,
    bank: ResMut<'w, SunBank>,
    plant: MessageWriter<'w, PlantRequest>,
}

/// 处理鼠标左键点击。
///
/// 逻辑顺序：
/// 1. 将屏幕坐标转换为世界坐标
/// 2. 先检测是否点到了太阳拾取物（28 像素范围内），是则收集并销毁
/// 3. 否则检测是否点到了棋盘格子，是则发出 [`PlantRequest`] 放置当前选中的植物
fn handle_world_clicks(mut params: WorldClickParams) {
    if !params.mouse.just_pressed(params.controls.place_or_collect) {
        return;
    }
    // Bevy UI 与世界共用鼠标输入；点击按钮时不能让同一次点击穿透到草坪。
    if params
        .ui_buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        return;
    }
    let Some(cursor) = params.window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = *params.camera;
    let Ok(world) = camera.viewport_to_world_2d(camera_transform, cursor) else {
        return;
    };

    // 先检测太阳拾取物点击
    if let Some((entity, _, pickup)) = params.pickups.iter().find(|(_, transform, _)| {
        transform.translation.truncate().distance(world) <= params.settings.sun_pickup_radius
    }) {
        params.bank.amount += pickup.value;
        params.commands.entity(entity).despawn();
        return;
    }

    // 再检测棋盘格子点击
    if let Some(cell) = params.layout.world_to_cell(world) {
        params.plant.write(PlantRequest {
            kind: params.selected.0,
            cell,
        });
    }
}

/// 太阳拾取物动画：上下浮动。名牌需要保持水平，因此不旋转整个实体。
fn animate_sun_pickups(time: Res<Time>, mut pickups: Query<(&mut Transform, &mut SunPickup)>) {
    for (mut transform, mut pickup) in &mut pickups {
        pickup.age += time.delta_secs();
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
    controls
        .validate(definition.cards.iter().map(|card| card.key))
        .expect("invalid control bindings");
}

/// 胜利判定：所有波次已生成完毕且场上无存活僵尸，则进入 Victory 状态。
fn check_victory(
    definition: Res<LevelDefinition>,
    runtime: Res<LevelRuntime>,
    zombies: Query<(), With<Zombie>>,
    mut won: MessageWriter<LevelWon>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if runtime.next_spawn == definition.spawns.len() && zombies.is_empty() {
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
mod tests {
    use super::*;

    #[test]
    fn sun_spending_is_atomic() {
        let mut bank = SunBank { amount: 75 };
        assert!(!bank.try_spend(100));
        assert_eq!(bank.amount, 75);
        assert!(bank.try_spend(50));
        assert_eq!(bank.amount, 25);
    }

    #[test]
    fn card_cooldown_tracks_ready_state() {
        let catalog = ContentCatalog::default();
        let mut cards = PlantCards::default();
        cards.0.insert(PlantKind::Peashooter, Duration::ZERO);
        assert!(cards.ready(PlantKind::Peashooter));
        cards.trigger(PlantKind::Peashooter, &catalog);
        assert!(!cards.ready(PlantKind::Peashooter));
        let remaining = cards.remaining(PlantKind::Peashooter);
        cards.0.insert(
            PlantKind::Peashooter,
            remaining.saturating_sub(Duration::from_secs(10)),
        );
        assert!(cards.ready(PlantKind::Peashooter));
    }

    #[test]
    fn final_wave_is_spawned_before_outcome_check() {
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin)
            .add_message::<SpawnZombie>()
            .add_message::<LevelWon>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(LawnLayout::default())
            .insert_resource(LevelRuntime::default())
            .insert_resource(LevelDefinition {
                spawns: vec![ZombieSpawnDefinition {
                    at_seconds: 0.0,
                    kind: ZombieKind::Basic,
                }],
                ..default()
            })
            .init_resource::<ContentCatalog>()
            .init_state::<GameState>()
            .add_systems(
                FixedUpdate,
                (
                    tick_wave_timeline,
                    super::super::zombie::spawn_zombies,
                    check_victory,
                )
                    .chain(),
            );

        app.world_mut().run_schedule(FixedUpdate);

        assert_eq!(app.world().resource::<LevelRuntime>().next_spawn, 1);
        let zombie_count = app
            .world_mut()
            .query_filtered::<Entity, With<Zombie>>()
            .iter(app.world())
            .count();
        assert_eq!(zombie_count, 1);
    }

    #[test]
    fn ron_level_is_complete_and_valid() {
        LevelDefinition::load_from_file(DEFAULT_LEVEL_PATH)
            .unwrap()
            .validate(&ContentCatalog::default())
            .unwrap();
    }

    #[test]
    fn validation_rejects_duplicate_keys_and_invalid_lawn() {
        let catalog = ContentCatalog::default();
        let mut level = LevelDefinition::default();
        level.cards[1].key = level.cards[0].key;
        assert!(
            level
                .validate(&catalog)
                .unwrap_err()
                .contains("duplicate card key")
        );
        level = LevelDefinition::default();
        level.lawn.cell_size.y = 0.0;
        assert!(
            level
                .validate(&catalog)
                .unwrap_err()
                .contains("finite and positive")
        );
    }

    #[test]
    fn reset_uses_the_current_level_definition() {
        let definition = LevelDefinition {
            starting_sun: 777,
            default_plant: PlantKind::WallNut,
            lawn: LawnLayout {
                columns: 7,
                ..default()
            },
            ..default()
        };

        let mut app = App::new();
        app.insert_resource(definition)
            .insert_resource(LevelRuntime {
                elapsed: Duration::from_secs(9),
                next_spawn: 2,
                defeated_zombies: 1,
            })
            .insert_resource(SunBank { amount: 1 })
            .insert_resource(PlantCards::default())
            .insert_resource(SelectedPlant(PlantKind::Sunflower))
            .insert_resource(LawnLayout::default())
            .insert_resource(CellOccupancy::default())
            .add_systems(Update, reset_level_runtime);

        app.update();

        assert_eq!(app.world().resource::<SunBank>().amount, 777);
        assert_eq!(
            app.world().resource::<SelectedPlant>().0,
            PlantKind::WallNut
        );
        assert_eq!(app.world().resource::<LawnLayout>().columns, 7);
        assert_eq!(app.world().resource::<LevelRuntime>().next_spawn, 0);
        assert_eq!(app.world().resource::<PlantCards>().0.len(), 3);
    }
}
