//! 物理引擎集成
//!
//! 基于 Bevy Rapier2D 实现物理世界，定义碰撞组（Collision Group）及冲突过滤规则。
//!
//! 碰撞组分配：
//! - `GROUP_1` — 植物（与僵尸、割草机碰撞）
//! - `GROUP_2` — 僵尸（与物理弹丸碰撞）
//! - `GROUP_3` — 普通弹丸（保留位，实际无碰撞体，逻辑命中检测）
//! - `GROUP_4` — 物理弹丸（与僵尸、其他物理弹丸、世界边界碰撞）
//! - `GROUP_5` — 世界边界（与物理弹丸碰撞）
//! - `GROUP_6` — 割草机（与植物相同过滤）

use std::time::Instant;

use bevy::diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::catalog::{ContentCatalog, ZombieKind};
use crate::game::config::GameplaySettings;
#[cfg(feature = "debug_tools")]
use crate::game::controls::ControlBindings;
use crate::game::defense::lawn_mower_start_left;
use crate::game::lawn::LawnLayout;
use crate::game::level::LevelSetupSet;
use crate::game::model::{model_bounds, zombie_model_parts};
use crate::game::projectile::Projectile;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};

/// 植物碰撞组。
pub const PLANT_GROUP: Group = Group::GROUP_1;
/// 僵尸碰撞组。
pub const ZOMBIE_GROUP: Group = Group::GROUP_2;
/// 普通弹丸碰撞组（仅保留，普通弹丸无物理碰撞体，使用扫掠检测）。
#[allow(dead_code)] // 逻辑弹丸没有碰撞体；此位仅保留给查询适配器使用。
pub const NORMAL_PROJECTILE_GROUP: Group = Group::GROUP_3;
/// 物理弹丸碰撞组（有实际碰撞体的弹丸）。
pub const PHYSICS_PROJECTILE_GROUP: Group = Group::GROUP_4;
/// 世界边界碰撞组（地板、侧墙）。
pub const WORLD_BOUNDARY_GROUP: Group = Group::GROUP_5;
/// 割草机碰撞组。
pub const MOWER_GROUP: Group = Group::GROUP_6;
/// 火炬树桩上半部点燃区碰撞组。
pub const TORCHWOOD_GROUP: Group = Group::GROUP_7;

/// Rapier 单次模拟步进的 CPU 耗时（毫秒）。
pub const PHYSICS_STEP_TIME: DiagnosticPath =
    DiagnosticPath::const_new("physics/step_simulation_time");

const LEFT_WALL_CLEARANCE_BEHIND_MOWER: f32 = 8.0;
const RIGHT_WALL_CENTER_OFFSET_FROM_ZOMBIE_RIGHT: f32 = 6.0;
const NORMAL_FIXED_UPDATE_HZ: f64 = 60.0;
const REDUCED_FIXED_UPDATE_HZ: f64 = 30.0;
const REDUCE_RATE_PROJECTILE_COUNT: usize = 2_500;
const RESTORE_RATE_PROJECTILE_COUNT: usize = 1_800;
const REDUCE_RATE_STEP_TIME_MS: f64 = 10.0;
const RESTORE_RATE_STEP_TIME_MS: f64 = 6.0;

/// 物理碰撞体调试渲染的启动配置。
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PhysicsDebugSettings {
    pub enabled: bool,
}

#[derive(Resource, Default)]
struct PhysicsStepTimer(Option<Instant>);

/// 最近一帧执行的物理步数及总耗时，用于识别 FixedUpdate 追赶开销。
#[derive(Resource, Debug, Default)]
pub struct PhysicsFrameStats {
    accumulating_time_ms: f64,
    accumulating_steps: u32,
    frame_time_ms: f64,
    frame_steps: u32,
    last_step_time_ms: Option<f64>,
}

impl PhysicsFrameStats {
    pub fn frame_time_ms(&self) -> f64 {
        self.frame_time_ms
    }

    pub fn frame_steps(&self) -> u32 {
        self.frame_steps
    }
}

/// 当前固定更新频率。大规模物理弹丸场景下自动降载。
#[derive(Resource, Debug)]
pub struct SimulationRate {
    hz: f64,
}

impl Default for SimulationRate {
    fn default() -> Self {
        Self {
            hz: NORMAL_FIXED_UPDATE_HZ,
        }
    }
}

impl SimulationRate {
    pub fn hz(&self) -> f64 {
        self.hz
    }

    pub fn is_reduced(&self) -> bool {
        self.hz == REDUCED_FIXED_UPDATE_HZ
    }
}

/// 物理引擎插件。
///
/// 配置 Rapier2D 物理管线（100 像素/米，固定在 FixedUpdate 调度中运行），
/// 设置碰撞组过滤规则与物理世界边界实体。
pub struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsStepTimer>()
            .init_resource::<PhysicsFrameStats>()
            .init_resource::<SimulationRate>()
            .register_diagnostic(Diagnostic::new(PHYSICS_STEP_TIME).with_suffix("ms"))
            .add_plugins((
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).in_fixed_schedule(),
                RapierDebugRenderPlugin::default().disabled(),
            ))
            .configure_sets(
                FixedUpdate,
                (
                    GameSet::Spawn.before(PhysicsSet::SyncBackend),
                    GameSet::LogicMovement
                        .after(GameSet::Spawn)
                        .before(PhysicsSet::SyncBackend),
                    GameSet::ContactRead
                        .after(PhysicsSet::Writeback)
                        .before(GameSet::Combat),
                ),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                setup_physics_world.after(LevelSetupSet::Reset),
            )
            .add_systems(
                FixedUpdate,
                (
                    begin_physics_step
                        .after(PhysicsSet::SyncBackend)
                        .before(PhysicsSet::StepSimulation),
                    end_physics_step
                        .after(PhysicsSet::StepSimulation)
                        .before(PhysicsSet::Writeback),
                ),
            )
            .add_systems(
                RunFixedMainLoop,
                finish_physics_frame.in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            )
            .add_systems(
                Update,
                adapt_fixed_update_rate.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Startup, apply_initial_physics_debug);
        #[cfg(feature = "debug_tools")]
        app.add_systems(Update, toggle_physics_debug);
    }
}

fn begin_physics_step(mut timer: ResMut<PhysicsStepTimer>) {
    timer.0 = Some(Instant::now());
}

fn end_physics_step(
    mut timer: ResMut<PhysicsStepTimer>,
    mut frame_stats: ResMut<PhysicsFrameStats>,
    mut diagnostics: Diagnostics,
) {
    let Some(started_at) = timer.0.take() else {
        return;
    };
    let elapsed_ms = started_at.elapsed().as_secs_f64() * 1_000.0;
    frame_stats.accumulating_time_ms += elapsed_ms;
    frame_stats.accumulating_steps += 1;
    diagnostics.add_measurement(&PHYSICS_STEP_TIME, || elapsed_ms);
}

fn finish_physics_frame(mut stats: ResMut<PhysicsFrameStats>) {
    stats.frame_time_ms = stats.accumulating_time_ms;
    stats.frame_steps = stats.accumulating_steps;
    if stats.accumulating_steps > 0 {
        stats.last_step_time_ms =
            Some(stats.accumulating_time_ms / f64::from(stats.accumulating_steps));
    }
    stats.accumulating_time_ms = 0.0;
    stats.accumulating_steps = 0;
}

fn adapt_fixed_update_rate(
    physics_projectiles: Query<(), (With<Projectile>, With<RigidBody>)>,
    frame_stats: Res<PhysicsFrameStats>,
    mut rate: ResMut<SimulationRate>,
    mut fixed_time: ResMut<Time<Fixed>>,
) {
    let projectile_count = physics_projectiles.iter().len();
    let step_time_ms = frame_stats.last_step_time_ms.unwrap_or(0.0);
    let target_hz = if rate.is_reduced() {
        if projectile_count <= RESTORE_RATE_PROJECTILE_COUNT
            && step_time_ms <= RESTORE_RATE_STEP_TIME_MS
        {
            NORMAL_FIXED_UPDATE_HZ
        } else {
            REDUCED_FIXED_UPDATE_HZ
        }
    } else if projectile_count >= REDUCE_RATE_PROJECTILE_COUNT
        || step_time_ms >= REDUCE_RATE_STEP_TIME_MS
    {
        REDUCED_FIXED_UPDATE_HZ
    } else {
        NORMAL_FIXED_UPDATE_HZ
    };

    if target_hz != rate.hz {
        rate.hz = target_hz;
        fixed_time.set_timestep_hz(target_hz);
    }
}

/// 将命令行启动配置应用到 Rapier 调试渲染上下文。
fn apply_initial_physics_debug(
    settings: Res<PhysicsDebugSettings>,
    mut debug: ResMut<DebugRenderContext>,
) {
    debug.enabled = settings.enabled;
}

/// 创建植物的碰撞组过滤配置。
///
/// 植物与僵尸、割草机发生碰撞。
pub fn plant_groups() -> CollisionGroups {
    CollisionGroups::new(PLANT_GROUP, ZOMBIE_GROUP | MOWER_GROUP)
}

/// 创建僵尸的碰撞组过滤配置。
///
/// 僵尸与物理弹丸发生碰撞。
pub fn zombie_groups() -> CollisionGroups {
    CollisionGroups::new(ZOMBIE_GROUP, PHYSICS_PROJECTILE_GROUP)
}

/// 创建物理弹丸的碰撞组过滤配置。
///
/// 物理弹丸与僵尸、其他物理弹丸、世界边界以及火炬点燃区发生碰撞。
///
/// 弹丸自身不请求碰撞事件，豌豆互撞只参与物理解算；僵尸和火炬碰撞体
/// 负责请求命中与点燃所需的事件，避免产生大量无用的豌豆互撞事件。
pub fn physics_projectile_groups() -> CollisionGroups {
    CollisionGroups::new(
        PHYSICS_PROJECTILE_GROUP,
        ZOMBIE_GROUP | PHYSICS_PROJECTILE_GROUP | WORLD_BOUNDARY_GROUP | TORCHWOOD_GROUP,
    )
}

/// 火炬树桩点燃区只与物理豌豆发生事件。
pub fn torchwood_groups() -> CollisionGroups {
    CollisionGroups::new(TORCHWOOD_GROUP, PHYSICS_PROJECTILE_GROUP)
}

/// 创建世界边界的碰撞组过滤配置。
fn world_groups() -> CollisionGroups {
    CollisionGroups::new(WORLD_BOUNDARY_GROUP, PHYSICS_PROJECTILE_GROUP)
}

/// 在 Playing 状态进入时创建物理世界边界实体。
///
/// 包括一个顶面与草坪底边齐平的地板，以及左右两侧的侧墙。
fn setup_physics_world(
    mut commands: Commands,
    layout: Res<LawnLayout>,
    settings: Res<GameplaySettings>,
    catalog: Res<ContentCatalog>,
) {
    let boundary = physics_boundary_layout(&layout, &settings, &catalog);

    // Collider::cuboid 使用半高，因此中心下移一个 thickness 后顶面正好对齐 row 0 底边。
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(boundary.floor_half_size.x, boundary.floor_half_size.y),
        Friction {
            coefficient: settings.physics_floor_friction,
            combine_rule: CoefficientCombineRule::Min,
        },
        world_groups(),
        Transform::from_xyz(boundary.floor_center.x, boundary.floor_center.y, 0.0),
        LevelEntity,
        Name::new("Physics floor"),
    ));

    // 左右侧墙：将物理沙箱限定在边界内。
    for x in boundary.wall_x {
        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(
                settings.physics_boundary_thickness,
                settings.physics_wall_half_height,
            ),
            world_groups(),
            Transform::from_xyz(x, 0.0, 0.0),
            LevelEntity,
            Name::new("Physics side wall"),
        ));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PhysicsBoundaryLayout {
    floor_center: Vec2,
    floor_half_size: Vec2,
    wall_x: [f32; 2],
}

fn physics_boundary_layout(
    layout: &LawnLayout,
    settings: &GameplaySettings,
    catalog: &ContentCatalog,
) -> PhysicsBoundaryLayout {
    let zombie_spawn_right = ZombieKind::ALL
        .into_iter()
        .map(|kind| {
            let definition = catalog.zombie(kind);
            let bounds = model_bounds(&zombie_model_parts(kind, 1.0));
            layout.right() + definition.spawn_offset_x + bounds.center.x + bounds.half_size.x
        })
        .max_by(f32::total_cmp)
        .expect("zombie catalog must not be empty");
    let wall_x = [
        lawn_mower_start_left(layout)
            - LEFT_WALL_CLEARANCE_BEHIND_MOWER
            - settings.physics_boundary_thickness,
        zombie_spawn_right + RIGHT_WALL_CENTER_OFFSET_FROM_ZOMBIE_RIGHT,
    ];
    let floor_top_y = layout.origin.y;
    let floor_half_width = (wall_x[1] - wall_x[0]) * 0.5 + settings.physics_boundary_thickness;
    PhysicsBoundaryLayout {
        floor_center: Vec2::new(
            (wall_x[0] + wall_x[1]) * 0.5,
            floor_top_y - settings.physics_boundary_thickness,
        ),
        floor_half_size: Vec2::new(floor_half_width, settings.physics_boundary_thickness),
        wall_x,
    }
}

/// 按 D 键切换物理碰撞体的调试渲染显示/隐藏。
#[cfg(feature = "debug_tools")]
fn toggle_physics_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    mut debug: ResMut<DebugRenderContext>,
) {
    if keyboard.just_pressed(controls.toggle_physics) {
        debug.enabled = !debug.enabled;
    }
}
