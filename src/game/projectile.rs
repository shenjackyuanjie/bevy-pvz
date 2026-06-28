use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::combat::{ApplyDamage, DamageKind, Team};
use super::lawn::{Lane, LawnLayout};
use super::physics::physics_projectile_groups;
use super::schedule::GameSet;
use super::state::{GameState, LevelEntity};
use super::zombie::Zombie;

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnProjectile>()
            .add_message::<ProjectileHit>()
            .add_systems(
                FixedUpdate,
                spawn_projectiles
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                advance_path_projectiles
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (query_path_projectile_hits, collect_rapier_collision_events)
                    .in_set(GameSet::ContactRead)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                resolve_projectile_hits
                    .in_set(GameSet::Combat)
                    .before(super::combat::apply_damage)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                tick_projectile_lifetimes
                    .in_set(GameSet::DeathAndCleanup)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, projectile_sandbox_keys);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProjectileKind {
    Pea,
    PhysicsPea,
}

#[derive(Debug, Clone, Copy)]
pub struct ProjectileDefinition {
    pub damage: f32,
    pub lifetime: Duration,
    pub speed: f32,
}

impl ProjectileDefinition {
    pub fn for_kind(kind: ProjectileKind) -> Self {
        match kind {
            ProjectileKind::Pea => Self {
                damage: 20.0,
                lifetime: Duration::from_secs(5),
                speed: 430.0,
            },
            ProjectileKind::PhysicsPea => Self {
                damage: 35.0,
                lifetime: Duration::from_secs(8),
                speed: 330.0,
            },
        }
    }
}

#[derive(Component, Debug)]
pub struct Projectile {
    pub owner: Entity,
    pub team: Team,
    pub damage: f32,
    pub lifetime: Timer,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProjectileMotion {
    Path,
    Physics,
}

#[derive(Component, Debug)]
struct PathVelocity(Vec2);

#[derive(Component, Debug)]
struct PreviousPosition(Vec2);

#[derive(Component, Debug)]
pub struct HitPolicy {
    pub destroy_on_hit: bool,
    pub remaining_pierces: u8,
}

#[derive(Component, Debug, Default)]
struct HitRegistry(HashSet<Entity>);

#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnProjectile {
    pub owner: Entity,
    pub origin: Vec2,
    pub lane: Lane,
    pub kind: ProjectileKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ProjectileHit {
    pub projectile: Entity,
    pub target: Entity,
}

fn spawn_projectiles(mut commands: Commands, mut requests: MessageReader<SpawnProjectile>) {
    for request in requests.read() {
        let definition = ProjectileDefinition::for_kind(request.kind);
        let base = (
            Sprite::from_color(
                match request.kind {
                    ProjectileKind::Pea => Color::srgb(0.28, 0.92, 0.22),
                    ProjectileKind::PhysicsPea => Color::srgb(0.35, 0.85, 0.95),
                },
                Vec2::splat(18.0),
            ),
            Transform::from_translation(request.origin.extend(3.0)),
            Projectile {
                owner: request.owner,
                team: Team::Plants,
                damage: definition.damage,
                lifetime: Timer::new(definition.lifetime, TimerMode::Once),
            },
            request.lane,
            HitRegistry::default(),
            LevelEntity,
            Name::new(format!("{:?}", request.kind)),
        );

        match request.kind {
            ProjectileKind::Pea => {
                commands.spawn((
                    base,
                    ProjectileMotion::Path,
                    PathVelocity(Vec2::X * definition.speed),
                    PreviousPosition(request.origin),
                    HitPolicy {
                        destroy_on_hit: true,
                        remaining_pierces: 0,
                    },
                ));
            }
            ProjectileKind::PhysicsPea => {
                commands.spawn((
                    base,
                    ProjectileMotion::Physics,
                    HitPolicy {
                        destroy_on_hit: false,
                        remaining_pierces: u8::MAX,
                    },
                    RigidBody::Dynamic,
                    Collider::ball(9.0),
                    Velocity::linear(Vec2::new(definition.speed, 300.0)),
                    Restitution::coefficient(0.72),
                    Friction::coefficient(0.35),
                    GravityScale(1.0),
                    Ccd::enabled(),
                    ActiveEvents::COLLISION_EVENTS,
                    physics_projectile_groups(),
                ));
            }
        }
    }
}

fn advance_path_projectiles(
    time: Res<Time<Fixed>>,
    mut projectiles: Query<
        (&mut Transform, &PathVelocity, &mut PreviousPosition),
        With<Projectile>,
    >,
) {
    for (mut transform, velocity, mut previous) in &mut projectiles {
        previous.0 = transform.translation.truncate();
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.0);
    }
}

type PathProjectileData<'a> = (
    Entity,
    &'a Transform,
    &'a PreviousPosition,
    &'a Lane,
    &'a HitRegistry,
);
type PathProjectileFilter = (With<Projectile>, With<PathVelocity>);

fn query_path_projectile_hits(
    projectiles: Query<PathProjectileData<'_>, PathProjectileFilter>,
    zombies: Query<(Entity, &Transform, &Lane), With<Zombie>>,
    mut hits: MessageWriter<ProjectileHit>,
) {
    for (projectile, transform, previous, projectile_lane, registry) in &projectiles {
        let end = transform.translation.truncate();
        let mut nearest: Option<(f32, Entity)> = None;

        for (zombie, zombie_transform, zombie_lane) in &zombies {
            if projectile_lane != zombie_lane || registry.0.contains(&zombie) {
                continue;
            }
            let center = zombie_transform.translation.truncate();
            if let Some(t) = swept_circle_hit_t(previous.0, end, center, Vec2::new(34.0, 42.0), 9.0)
                && nearest.is_none_or(|(best_t, _)| t < best_t)
            {
                nearest = Some((t, zombie));
            }
        }

        if let Some((_, target)) = nearest {
            hits.write(ProjectileHit { projectile, target });
        }
    }
}

fn collect_rapier_collision_events(
    mut collisions: MessageReader<CollisionEvent>,
    projectiles: Query<&ProjectileMotion, With<Projectile>>,
    zombies: Query<(), With<Zombie>>,
    mut hits: MessageWriter<ProjectileHit>,
) {
    for collision in collisions.read() {
        let CollisionEvent::Started(a, b, _) = *collision else {
            continue;
        };
        let pair = if projectiles.get(a) == Ok(&ProjectileMotion::Physics) && zombies.contains(b) {
            Some((a, b))
        } else if projectiles.get(b) == Ok(&ProjectileMotion::Physics) && zombies.contains(a) {
            Some((b, a))
        } else {
            None
        };
        if let Some((projectile, target)) = pair {
            hits.write(ProjectileHit { projectile, target });
        }
    }
}

fn resolve_projectile_hits(
    mut commands: Commands,
    mut hits: MessageReader<ProjectileHit>,
    mut projectiles: Query<(&Projectile, &mut HitPolicy, &mut HitRegistry)>,
    mut damage: MessageWriter<ApplyDamage>,
) {
    for hit in hits.read() {
        let Ok((projectile, mut policy, mut registry)) = projectiles.get_mut(hit.projectile) else {
            continue;
        };
        if !registry.0.insert(hit.target) {
            continue;
        }
        if projectile.team != Team::Plants {
            continue;
        }
        damage.write(ApplyDamage {
            source: projectile.owner,
            target: hit.target,
            amount: projectile.damage,
            kind: DamageKind::Projectile,
        });

        if policy.destroy_on_hit || policy.remaining_pierces == 0 {
            commands.entity(hit.projectile).despawn();
        } else {
            policy.remaining_pierces = policy.remaining_pierces.saturating_sub(1);
        }
    }
}

fn tick_projectile_lifetimes(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut projectiles: Query<(Entity, &Transform, &mut Projectile)>,
) {
    for (entity, transform, mut projectile) in &mut projectiles {
        projectile.lifetime.tick(time.delta());
        let position = transform.translation;
        if projectile.lifetime.is_finished() || position.x.abs() > 900.0 || position.y.abs() > 650.0
        {
            commands.entity(entity).despawn();
        }
    }
}

fn projectile_sandbox_keys(
    keyboard: Res<ButtonInput<KeyCode>>,
    layout: Res<LawnLayout>,
    state: Res<State<GameState>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    let lane = Lane(2);
    let origin = Vec2::new(layout.origin.x + 30.0, layout.lane_y(lane));
    let kind = if keyboard.just_pressed(KeyCode::KeyN) {
        Some(ProjectileKind::Pea)
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        Some(ProjectileKind::PhysicsPea)
    } else {
        None
    };
    if let Some(kind) = kind {
        spawn.write(SpawnProjectile {
            owner: Entity::PLACEHOLDER,
            origin,
            lane,
            kind,
        });
    }
}

/// Sweeps a circle along a segment against an axis-aligned box and returns first contact time.
/// Expanding the box by the circle radius reduces this to a segment/AABB slab test.
fn swept_circle_hit_t(
    start: Vec2,
    end: Vec2,
    center: Vec2,
    half: Vec2,
    radius: f32,
) -> Option<f32> {
    let expanded = half + Vec2::splat(radius);
    let min = center - expanded;
    let max = center + expanded;
    let direction = end - start;
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 1.0;

    for axis in 0..2 {
        let origin = start[axis];
        let delta = direction[axis];
        if delta.abs() < f32::EPSILON {
            if origin < min[axis] || origin > max[axis] {
                return None;
            }
            continue;
        }
        let inverse = 1.0 / delta;
        let mut near = (min[axis] - origin) * inverse;
        let mut far = (max[axis] - origin) * inverse;
        if near > far {
            std::mem::swap(&mut near, &mut far);
        }
        enter = enter.max(near);
        exit = exit.min(far);
        if enter > exit {
            return None;
        }
    }
    (exit >= 0.0 && enter <= 1.0).then_some(enter.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swept_query_hits_even_when_endpoints_miss() {
        let hit = swept_circle_hit_t(
            Vec2::new(-100.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::ZERO,
            Vec2::splat(20.0),
            5.0,
        );
        assert!(hit.is_some());
        assert!(hit.unwrap() > 0.0 && hit.unwrap() < 1.0);
    }

    #[test]
    fn swept_query_rejects_other_lane() {
        assert_eq!(
            swept_circle_hit_t(
                Vec2::new(-100.0, 100.0),
                Vec2::new(100.0, 100.0),
                Vec2::ZERO,
                Vec2::splat(20.0),
                5.0,
            ),
            None
        );
    }

    #[test]
    fn spawn_request_builds_distinct_motion_pipelines() {
        let mut app = App::new();
        app.add_message::<SpawnProjectile>()
            .add_systems(Update, spawn_projectiles);
        for kind in [ProjectileKind::Pea, ProjectileKind::PhysicsPea] {
            app.world_mut().write_message(SpawnProjectile {
                owner: Entity::PLACEHOLDER,
                origin: Vec2::ZERO,
                lane: Lane(2),
                kind,
            });
        }

        app.update();

        let world = app.world_mut();
        let mut query =
            world.query::<(&ProjectileMotion, Option<&RigidBody>, Option<&PathVelocity>)>();
        let spawned: Vec<_> = query
            .iter(world)
            .map(|(motion, body, path)| (*motion, body.copied(), path.is_some()))
            .collect();
        assert_eq!(spawned.len(), 2);
        assert!(spawned.contains(&(ProjectileMotion::Path, None, true)));
        assert!(spawned.contains(&(ProjectileMotion::Physics, Some(RigidBody::Dynamic), false)));
    }
}
