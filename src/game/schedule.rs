//! 游戏调度集合 [`GameSet`]
//!
//! 定义固定时间步长下各系统阶段的执行顺序。所有阶段在 `FixedUpdate` 调度中连锁执行：
//!
//! 1. **Spawn** — 生成新实体（植物放置、弹丸生成、僵尸生成、太阳掉落）
//! 2. **LogicMovement** — 逻辑移动（路径弹丸前进、僵尸行走/攻击、向日葵产太阳、冷却计时）
//! 3. **ContactRead** — 碰撞读取（路径弹丸的扫掠检测、Rapier 物理碰撞事件收集）
//! 4. **Combat** — 战斗结算（伤害应用、弹丸命中解析）
//! 5. **DeathAndCleanup** — 死亡清理（标记为 Dead 的实体清理、植物格子释放、弹丸超时销毁）
//! 6. **LevelOutcome** — 关卡结果判定（胜利/失败检查、击杀计数）

use bevy::prelude::*;

/// 游戏固定更新阶段的调度集合。
///
/// 所有变体按声明顺序在 `FixedUpdate` 中连锁执行。
/// 物理引擎的 `SyncBackend` 穿插在 Spawn→LogicMovement 之间，
/// `Writeback` 在 ContactRead 之前，确保物理状态与游戏逻辑正确同步。
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameSet {
    /// 生成新实体：植物、弹丸、僵尸、太阳。
    Spawn,
    /// 逻辑层移动与行为：弹丸前进、僵尸行走/啃食、向日葵产太阳、冷却计时。
    LogicMovement,
    /// 读取碰撞事件：路径弹丸的扫掠命中检测、Rapier 物理碰撞事件。
    ContactRead,
    /// 战斗伤害结算与命中解析。
    Combat,
    /// 清理死亡实体、释放占用格子、销毁超时弹丸。
    DeathAndCleanup,
    /// 关卡胜负判定与击杀统计。
    LevelOutcome,
}
