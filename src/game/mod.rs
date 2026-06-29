//! 游戏根模块
//!
//! 本模块是游戏逻辑的入口，负责：
//! - 公开所有子模块（战斗、草坪、关卡、物理、植物、弹丸、调度、状态、UI、僵尸）
//! - 组合所有子插件为统一的 [`GamePlugin`]
//! - 配置固定时间步长更新（60 Hz）下各 [`GameSet`] 阶段的连锁执行顺序
//!
//! 调度顺序：
//! `Spawn` → `LogicMovement` → `ContactRead` → `Combat` → `DeathAndCleanup` → `LevelOutcome`

pub mod assets;
pub mod catalog;
pub mod combat;
pub mod config;
pub mod controls;
pub mod lawn;
pub mod level;
pub mod model;
pub mod physics;
pub mod plant;
pub mod projectile;
pub mod schedule;
pub mod state;
pub mod theme;
pub mod ui;
pub mod zombie;

use bevy::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::catalog::ContentCatalog;
use crate::game::config::GameplaySettings;
use crate::game::controls::ControlBindings;
use crate::game::theme::UiTheme;

use crate::game::combat::CombatPlugin;
use crate::game::lawn::LawnPlugin;
use crate::game::level::LevelPlugin;
use crate::game::physics::{GamePhysicsPlugin, PhysicsDebugSettings};
use crate::game::plant::PlantPlugin;
use crate::game::projectile::ProjectilePlugin;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::ui::GameUiPlugin;
use crate::game::zombie::ZombiePlugin;

/// 游戏主插件，聚合所有子插件并配置全局系统调度。
///
/// 添加到 `App` 后会自动初始化状态机、注册固定时间步长、
/// 配置系统集连锁顺序、附加所有子插件以及注册关卡生命周期相关的 Startup/OnEnter/OnExit 系统。
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_resource::<ContentCatalog>()
            .init_resource::<GameplaySettings>()
            .init_resource::<ControlBindings>()
            .init_resource::<PhysicsDebugSettings>()
            .init_resource::<UiTheme>()
            .init_resource::<GameAssets>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .configure_sets(
                FixedUpdate,
                (
                    GameSet::Spawn,
                    GameSet::LogicMovement,
                    GameSet::ContactRead,
                    GameSet::Combat,
                    GameSet::DeathAndCleanup,
                    GameSet::LevelOutcome,
                )
                    .chain(),
            )
            .add_plugins((
                GamePhysicsPlugin,
                LawnPlugin,
                CombatPlugin,
                ProjectilePlugin,
                PlantPlugin,
                ZombiePlugin,
                LevelPlugin,
                GameUiPlugin,
            ))
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Loading), enter_playing)
            .add_systems(OnExit(GameState::Playing), cleanup_level)
            .add_systems(Startup, validate_runtime_config)
            .add_systems(Update, restart_level);
    }
}

/// 生成主摄像机实体（2D 正交相机）。
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Name::new("Main camera")));
}

/// 进入 Loading 状态后立即切换到 Playing，触发关卡初始化。
fn enter_playing(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

/// 退出 Playing 状态时清除所有带有 [`LevelEntity`] 标记的实体，确保关卡完全重置。
fn cleanup_level(mut commands: Commands, level_entities: Query<Entity, With<LevelEntity>>) {
    for entity in &level_entities {
        commands.entity(entity).despawn();
    }
}

/// 按 R 键重新开始关卡（Loading 状态下忽略，防止重复触发）。
fn restart_level(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(controls.restart) && *state.get() != GameState::Loading {
        next_state.set(GameState::Loading);
    }
}

fn validate_runtime_config(catalog: Res<ContentCatalog>, settings: Res<GameplaySettings>) {
    catalog
        .validate()
        .expect("invalid built-in content catalog");
    settings.validate().expect("invalid gameplay settings");
}
