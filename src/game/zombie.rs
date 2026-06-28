use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::combat::{ApplyDamage, DamageKind, Health, Team};
use super::lawn::{Lane, LawnLayout};
use super::physics::zombie_groups;
use super::plant::Plant;
use super::schedule::GameSet;
use super::state::{GameState, LevelEntity};

pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnZombie>()
            .add_systems(
                FixedUpdate,
                spawn_zombies
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_zombie_state,
                    advance_walking_zombies,
                    tick_zombie_attacks,
                )
                    .chain()
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Debug)]
pub struct Zombie {
    pub speed: f32,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum ZombieState {
    Walking,
    Eating { target: Entity },
}

#[derive(Component, Debug)]
struct AttackTimer(Timer);

#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnZombie {
    pub lane: Lane,
}

pub(crate) fn spawn_zombies(
    mut commands: Commands,
    mut requests: MessageReader<SpawnZombie>,
    layout: Res<LawnLayout>,
) {
    for request in requests.read() {
        let position = Vec2::new(layout.right() + 75.0, layout.lane_y(request.lane));
        commands.spawn((
            Sprite::from_color(Color::srgb(0.42, 0.48, 0.38), Vec2::new(58.0, 82.0)),
            Transform::from_translation(position.extend(2.0)),
            Zombie { speed: 17.0 },
            ZombieState::Walking,
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
            request.lane,
            Health::new(100.0),
            Team::Zombies,
            RigidBody::KinematicPositionBased,
            Collider::cuboid(29.0, 41.0),
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
            zombie_groups(),
            LevelEntity,
            Name::new("Basic zombie"),
        ));
    }
}

fn update_zombie_state(
    mut zombies: Query<(&Transform, &Lane, &mut ZombieState), With<Zombie>>,
    plants: Query<(Entity, &Transform, &Lane), With<Plant>>,
) {
    for (zombie_transform, zombie_lane, mut state) in &mut zombies {
        let zombie_x = zombie_transform.translation.x;
        let blocker = plants
            .iter()
            .filter(|(_, _, plant_lane)| *plant_lane == zombie_lane)
            .filter_map(|(entity, plant_transform, _)| {
                let distance = zombie_x - plant_transform.translation.x;
                (-12.0..=62.0)
                    .contains(&distance)
                    .then_some((distance.abs(), entity))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, entity)| entity);

        *state = blocker
            .map(|target| ZombieState::Eating { target })
            .unwrap_or(ZombieState::Walking);
    }
}

fn advance_walking_zombies(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(&Zombie, &ZombieState, &mut Transform)>,
) {
    for (zombie, state, mut transform) in &mut zombies {
        if *state == ZombieState::Walking {
            transform.translation.x -= zombie.speed * time.delta_secs();
        }
    }
}

fn tick_zombie_attacks(
    time: Res<Time<Fixed>>,
    mut zombies: Query<(Entity, &ZombieState, &mut AttackTimer)>,
    plants: Query<(), With<Plant>>,
    mut damage: MessageWriter<ApplyDamage>,
) {
    for (entity, state, mut timer) in &mut zombies {
        let ZombieState::Eating { target } = *state else {
            timer.0.reset();
            continue;
        };
        if !plants.contains(target) {
            continue;
        }
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            damage.write(ApplyDamage {
                source: entity,
                target,
                amount: 20.0,
                kind: DamageKind::Bite,
            });
        }
    }
}
