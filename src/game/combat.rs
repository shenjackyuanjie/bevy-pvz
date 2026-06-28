use bevy::prelude::*;

use super::schedule::GameSet;
use super::state::GameState;

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

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn damage(&mut self, amount: f32) -> bool {
        self.current = (self.current - amount.max(0.0)).max(0.0);
        debug_assert!(self.current <= self.max);
        self.current == 0.0
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    Plants,
    Zombies,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageKind {
    Projectile,
    Bite,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ApplyDamage {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub kind: DamageKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EntityDied {
    pub entity: Entity,
    pub killer: Entity,
    pub team: Team,
}

#[derive(Component, Debug)]
pub struct Dead {
    pub killer: Entity,
}

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
