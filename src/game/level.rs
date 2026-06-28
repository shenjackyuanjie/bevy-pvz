//! 关卡系统
//!
//! 管理单关的完整生命周期，包括：
//!
//! - **波次生成**：按时间线自动生成僵尸（[`tick_wave_timeline`]）
//! - **太阳经济**：太阳存款（[`SunBank`]）与植物卡片冷却（[`PlantCards`]）
//! - **植物选择**：按数字键 1/2/3 切换当前选中的植物（[`SelectedPlant`]）
//! - **鼠标交互**：左键点击收集太阳、在棋盘上放置植物（[`handle_world_clicks`]）
//! - **胜负判定**：僵尸突破房子左侧 → 失败；清空所有僵尸 → 胜利
//! - **太阳动画**：太阳拾取物上下浮动并旋转

use std::collections::HashMap;
use std::time::Duration;

use bevy::sprite::Text2dShadow;
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::game::combat::{EntityDied, Team};
use crate::game::lawn::{CellOccupancy, Lane, LawnLayout};
use crate::game::plant::{PlantKind, PlantRequest};
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::zombie::{SpawnZombie, Zombie};

/// 关卡插件，注册资源、消息和所有关卡管理相关的系统。
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelDefinition>()
            .init_resource::<LevelRuntime>()
            .init_resource::<SunBank>()
            .init_resource::<PlantCards>()
            .init_resource::<SelectedPlant>()
            .add_message::<SpawnSun>()
            .add_message::<LevelWon>()
            .add_message::<LevelLost>()
            .add_systems(OnEnter(GameState::Playing), reset_level_runtime)
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

/// 僵尸生成点定义：指定在什么时间、哪一行生成一个僵尸。
#[derive(Debug, Clone, Copy)]
pub struct ZombieSpawn {
    /// 相对于关卡开始的生成时间（秒）。
    pub at_seconds: f32,
    /// 僵尸出生的行。
    pub lane: Lane,
}

/// 关卡配置资源，定义所有僵尸波次。
#[derive(Resource, Debug, Clone)]
pub struct LevelDefinition {
    /// 按时间排序的僵尸生成队列。
    pub spawns: Vec<ZombieSpawn>,
}

impl Default for LevelDefinition {
    fn default() -> Self {
        Self {
            // 默认 11 个僵尸的关卡配置，分布在 5 行、约 53 秒内
            spawns: vec![
                ZombieSpawn {
                    at_seconds: 4.0,
                    lane: Lane(2),
                },
                ZombieSpawn {
                    at_seconds: 10.0,
                    lane: Lane(0),
                },
                ZombieSpawn {
                    at_seconds: 14.0,
                    lane: Lane(4),
                },
                ZombieSpawn {
                    at_seconds: 20.0,
                    lane: Lane(1),
                },
                ZombieSpawn {
                    at_seconds: 24.0,
                    lane: Lane(3),
                },
                ZombieSpawn {
                    at_seconds: 31.0,
                    lane: Lane(2),
                },
                ZombieSpawn {
                    at_seconds: 36.0,
                    lane: Lane(0),
                },
                ZombieSpawn {
                    at_seconds: 38.0,
                    lane: Lane(4),
                },
                ZombieSpawn {
                    at_seconds: 46.0,
                    lane: Lane(1),
                },
                ZombieSpawn {
                    at_seconds: 47.5,
                    lane: Lane(3),
                },
                ZombieSpawn {
                    at_seconds: 53.0,
                    lane: Lane(2),
                },
            ],
        }
    }
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
#[derive(Resource, Debug)]
pub struct SunBank {
    /// 太阳数量。
    pub amount: u32,
}

impl Default for SunBank {
    fn default() -> Self {
        Self { amount: 250 }
    }
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
#[derive(Resource, Debug)]
pub struct PlantCards(pub HashMap<PlantKind, Duration>);

impl Default for PlantCards {
    fn default() -> Self {
        Self(
            PlantKind::ALL
                .into_iter()
                .map(|kind| (kind, Duration::ZERO))
                .collect(),
        )
    }
}

impl PlantCards {
    /// 检查某植物卡片是否已冷却完毕（可用）。
    pub fn ready(&self, kind: PlantKind) -> bool {
        self.0.get(&kind).is_some_and(Duration::is_zero)
    }

    /// 触发冷却：使用后将冷却时间设为该植物的 `card_cooldown`。
    pub fn trigger(&mut self, kind: PlantKind) {
        self.0.insert(kind, kind.card_cooldown());
    }

    /// 查询某植物的剩余冷却时间。
    pub fn remaining(&self, kind: PlantKind) -> Duration {
        self.0.get(&kind).copied().unwrap_or(Duration::ZERO)
    }
}

/// 当前选中的植物资源，由数字键 1/2/3 切换。
#[derive(Resource, Debug)]
pub struct SelectedPlant(pub PlantKind);

impl Default for SelectedPlant {
    fn default() -> Self {
        Self(PlantKind::Peashooter)
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
    mut runtime: ResMut<LevelRuntime>,
    mut bank: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
    mut occupancy: ResMut<CellOccupancy>,
) {
    *runtime = LevelRuntime::default();
    *bank = SunBank::default();
    *cards = PlantCards::default();
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
        spawn.write(SpawnZombie { lane: next.lane });
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
    asset_server: Res<AssetServer>,
    mut requests: MessageReader<SpawnSun>,
) {
    let label_font = asset_server.load("fonts/NotoSansSC-VF.ttf");

    for request in requests.read() {
        // 太阳数值名牌与拾取物组成父子关系，浮动时保持同步且便于整体销毁。
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.86, 0.15), Vec2::splat(34.0)),
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
                    font: label_font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.22, 0.12, 0.01)),
                TextBackgroundColor(Color::srgba(1.0, 0.94, 0.55, 0.88)),
                TextLayout::new_with_justify(Justify::Center),
                Text2dShadow {
                    offset: Vec2::new(1.0, -1.0),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.65),
                },
                Transform::from_xyz(0.0, -28.0, 2.0),
                Name::new("太阳数值"),
            )],
        ));
    }
}

/// 数字键 1/2/3 切换当前选中的植物种类。
fn select_plant_card(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedPlant>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        selected.0 = PlantKind::Sunflower;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        selected.0 = PlantKind::Peashooter;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        selected.0 = PlantKind::WallNut;
    }
}

/// 自定义 SystemParam，封装鼠标点击所需的全部系统参数。
#[derive(SystemParam)]
struct WorldClickParams<'w, 's> {
    commands: Commands<'w, 's>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    window: Single<'w, 's, &'static Window>,
    camera: Single<'w, 's, (&'static Camera, &'static GlobalTransform)>,
    layout: Res<'w, LawnLayout>,
    selected: Res<'w, SelectedPlant>,
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
    if !params.mouse.just_pressed(MouseButton::Left) {
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
    if let Some((entity, _, pickup)) = params
        .pickups
        .iter()
        .find(|(_, transform, _)| transform.translation.truncate().distance(world) <= 28.0)
    {
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
    mut lost: MessageWriter<LevelLost>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if zombies
        .iter()
        .any(|transform| transform.translation.x <= layout.origin.x - 16.0)
    {
        lost.write(LevelLost);
        next_state.set(GameState::Defeat);
    }
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
        let mut cards = PlantCards::default();
        assert!(cards.ready(PlantKind::Peashooter));
        cards.trigger(PlantKind::Peashooter);
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
                spawns: vec![ZombieSpawn {
                    at_seconds: 0.0,
                    lane: Lane(2),
                }],
            })
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
}
