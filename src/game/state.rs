//! 游戏状态机与关卡实体标记
//!
//! 定义游戏的整体生命周期状态 [`GameState`](enum.GameState.html)：
//! `Loading` → `Playing` → `Victory` / `Defeat`。
//! 以及用于关卡实体追踪的标记组件 [`LevelEntity`]。

use bevy::prelude::*;

/// 游戏全局状态枚举，驱动关卡生命周期。
///
/// 状态转换：
/// - **默认进入 `Loading`** → 触发 OnEnter，由 `enter_playing` 立即切换到 `Playing`
/// - **`Playing`** → 游戏正常运行，所有核心系统在此状态下工作
/// - **`Victory`** → 所有僵尸已被消灭且波次全部生成
/// - **`Defeat`** → 有僵尸突破到房子左侧
#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum GameState {
    /// 加载状态（默认），短暂停留后自动进入 Playing。
    #[default]
    Loading,
    /// 游戏进行中，所有战斗/生成/UI 系统运行。
    Playing,
    /// 胜利状态，显示胜利画面，按 R 重开。
    Victory,
    /// 失败状态，显示失败画面，按 R 重开。
    Defeat,
}

/// 标记组件：标记当前关卡拥有的所有实体。
///
/// 退出 Playing 状态时，`cleanup_level` 会通过此组件一次性清除所有关卡实体，
/// 确保重启关卡时不会残留上一局的物体。
/// 所有在关卡中生成的实体（草坪格子、植物、僵尸、弹丸、太阳、UI 元素等）都应携带此组件。
#[derive(Component, Debug)]
pub struct LevelEntity;
