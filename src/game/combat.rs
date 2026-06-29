//! 战斗系统
//!
//! 实现核心战斗数据模型与流程：
//! - [`Health`]：生命值组件，支持伤害扣减与死亡判定
//! - [`Team`]：阵营标记（植物/僵尸）
//! - [`ApplyDamage`]：伤害请求消息，在 Combat 阶段处理
//! - [`Dead`] / [`EntityDied`]：死亡标记与死亡通知，在 DeathAndCleanup 阶段处理
//!
//! 流程：伤害消息 → 扣减生命值 → 生命归零时插入 Dead 组件 → 清理阶段发出 EntityDied 并销毁实体。

use bevy::prelude::*;

use crate::game::schedule::GameSet;
use crate::game::state::GameState;

/// 战斗插件，注册伤害应用与死亡清理系统。
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ApplyDamage>()
            .add_message::<EntityDied>()
            .add_systems(
                FixedUpdate,
                apply_damage
                    .in_set(GameSet::Combat)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                cleanup_dead_entities
                    .in_set(GameSet::DeathAndCleanup)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 生命值组件，记录当前生命值和最大生命值。
#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    /// 当前生命值，范围为 [0, max]。
    pub current: f32,
    /// 最大生命值，也是初始值。
    pub max: f32,
}

impl Health {
    /// 创建一个满生命值的 `Health` 组件。
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    /// 受到伤害，扣减生命值。
    ///
    /// 伤害值会被 clamp 到非负。生命值最低为 0。
    /// 返回 `true` 表示生命值降为 0（即死亡）。
    pub fn damage(&mut self, amount: f32) -> bool {
        self.current = (self.current - amount.max(0.0)).max(0.0);
        debug_assert!(self.current <= self.max);
        self.current == 0.0
    }
}

/// 阵营标记组件，区分敌我。
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    /// 植物阵营。
    Plants,
    /// 僵尸阵营。
    Zombies,
}

/// 伤害类型枚举，用于区分不同来源的伤害（可用于触发不同效果/动画）。
#[derive(Debug, Clone, Copy)]
pub enum DamageKind {
    /// 弹丸伤害（豌豆等）。
    Projectile,
    /// 啃食伤害（僵尸攻击）。
    Bite,
    /// 草坪小推车碾压。
    Mower,
}

/// 伤害请求消息。
///
/// 由攻击系统（弹丸命中、僵尸攻击）发送，[`apply_damage`] 系统消费并处理。
#[derive(Message, Debug, Clone, Copy)]
pub struct ApplyDamage {
    /// 伤害来源实体。
    pub source: Entity,
    /// 受伤害的目标实体。
    pub target: Entity,
    /// 伤害数值。
    pub amount: f32,
    /// 伤害类型。
    pub kind: DamageKind,
}

/// 实体死亡通知消息。
///
/// 由 [`cleanup_dead_entities`] 系统在销毁死亡实体前发送，
/// 用于关卡系统统计击杀数等。
#[derive(Message, Debug, Clone, Copy)]
pub struct EntityDied {
    /// 死亡的实体。
    pub entity: Entity,
    /// 击杀者实体（来自 `Dead.killer`）。
    pub killer: Entity,
    /// 死亡实体的阵营。
    pub team: Team,
}

/// 死亡标记组件，标记已被击杀的实体。
///
/// 实体携带此组件后会在下一轮的 `DeathAndCleanup` 阶段被清理。
#[derive(Component, Debug)]
pub struct Dead {
    /// 击杀此实体的来源实体。
    pub killer: Entity,
}

/// 处理所有 [`ApplyDamage`] 消息，将伤害应用到目标实体的 [`Health`] 组件上。
///
/// 如果目标生命值归零，则插入 [`Dead`] 组件标记其死亡。
/// 已经带有 `Dead` 组件的实体不再接受新伤害。
pub(crate) fn apply_damage(
    mut commands: Commands,
    mut damage: MessageReader<ApplyDamage>,
    mut targets: Query<&mut Health, Without<Dead>>,
) {
    for hit in damage.read() {
        let _kind = hit.kind;
        if let Ok(mut health) = targets.get_mut(hit.target)
            && health.damage(hit.amount)
        {
            commands
                .entity(hit.target)
                .insert(Dead { killer: hit.source });
        }
    }
}

/// 清理所有带有 [`Dead`] 组件的实体。
///
/// 对每个死亡实体先发出 [`EntityDied`] 通知消息，然后销毁实体。
pub(crate) fn cleanup_dead_entities(
    mut commands: Commands,
    dead: Query<(Entity, &Dead, &Team)>,
    mut died: MessageWriter<EntityDied>,
) {
    for (entity, dead, team) in &dead {
        died.write(EntityDied {
            entity,
            killer: dead.killer,
            team: *team,
        });
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_clamps_and_reports_death() {
        let mut health = Health::new(100.0);
        assert!(!health.damage(40.0));
        assert_eq!(health.current, 60.0);
        assert!(health.damage(80.0));
        assert_eq!(health.current, 0.0);
        assert_eq!(health.max, 100.0);
    }

    #[test]
    fn same_frame_damage_accumulates_before_death_cleanup() {
        let mut app = App::new();
        app.add_message::<ApplyDamage>()
            .add_systems(Update, apply_damage);
        let target = app
            .world_mut()
            .spawn((Health::new(100.0), Team::Zombies))
            .id();
        let source = Entity::PLACEHOLDER;
        for amount in [45.0, 60.0] {
            app.world_mut().write_message(ApplyDamage {
                source,
                target,
                amount,
                kind: DamageKind::Projectile,
            });
        }

        app.update();

        assert_eq!(app.world().get::<Health>(target).unwrap().current, 0.0);
        assert!(app.world().get::<Dead>(target).is_some());
    }
}
