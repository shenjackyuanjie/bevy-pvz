use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::combat::{Dead, Health, Team};
use super::lawn::{CellOccupancy, GridCell, Lane, LawnLayout};
use super::level::{PlantCards, SpawnSun, SunBank};
use super::physics::plant_groups;
use super::projectile::{ProjectileKind, SpawnProjectile};
use super::schedule::GameSet;
use super::state::GameState;
use super::zombie::Zombie;

pub struct PlantPlugin;

impl Plugin for PlantPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlantRequest>()
            .add_systems(
                FixedUpdate,
                place_plants
                    .in_set(GameSet::Spawn)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (fire_ready_shooters, produce_sun)
                    .in_set(GameSet::LogicMovement)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                release_dead_plant_cells
                    .in_set(GameSet::DeathAndCleanup)
                    .before(super::combat::cleanup_dead_entities)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PlantKind {
    Sunflower,
    Peashooter,
    WallNut,
}

impl PlantKind {
    pub const ALL: [Self; 3] = [Self::Sunflower, Self::Peashooter, Self::WallNut];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Sunflower => "Sunflower",
            Self::Peashooter => "Peashooter",
            Self::WallNut => "Wall-nut",
        }
    }

    pub fn price(self) -> u32 {
        match self {
            Self::Sunflower | Self::WallNut => 50,
            Self::Peashooter => 100,
        }
    }

    pub fn card_cooldown(self) -> Duration {
        match self {
            Self::Sunflower => Duration::from_secs(5),
            Self::Peashooter => Duration::from_secs(4),
            Self::WallNut => Duration::from_secs(8),
        }
    }

    fn health(self) -> f32 {
        match self {
            Self::Sunflower => 100.0,
            Self::Peashooter => 120.0,
            Self::WallNut => 600.0,
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Sunflower => Color::srgb(0.98, 0.72, 0.12),
            Self::Peashooter => Color::srgb(0.12, 0.72, 0.20),
            Self::WallNut => Color::srgb(0.55, 0.30, 0.12),
        }
    }
}

#[derive(Component, Debug)]
pub struct Plant;

#[derive(Component, Debug)]
struct ActionTimer(Timer);

#[derive(Message, Debug, Clone, Copy)]
pub struct PlantRequest {
    pub kind: PlantKind,
    pub cell: GridCell,
}

fn place_plants(
    mut commands: Commands,
    mut requests: MessageReader<PlantRequest>,
    layout: Res<LawnLayout>,
    mut occupancy: ResMut<CellOccupancy>,
    mut sun: ResMut<SunBank>,
    mut cards: ResMut<PlantCards>,
) {
    for request in requests.read() {
        if !occupancy.is_available(request.cell, &layout)
            || !cards.ready(request.kind)
            || !sun.try_spend(request.kind.price())
        {
            continue;
        }

        cards.trigger(request.kind);
        let position = layout.cell_center(request.cell);
        let mut entity = commands.spawn((
            Sprite::from_color(request.kind.color(), Vec2::new(58.0, 68.0)),
            Transform::from_translation(position.extend(1.0)),
            Plant,
            request.kind,
            request.cell,
            Lane(request.cell.row),
            Health::new(request.kind.health()),
            Team::Plants,
            RigidBody::Fixed,
            Collider::cuboid(29.0, 34.0),
            plant_groups(),
            super::state::LevelEntity,
            Name::new(request.kind.display_name()),
        ));

        if request.kind != PlantKind::WallNut {
            let seconds = match request.kind {
                PlantKind::Sunflower => 7.0,
                PlantKind::Peashooter => 1.35,
                PlantKind::WallNut => unreachable!(),
            };
            entity.insert(ActionTimer(Timer::from_seconds(
                seconds,
                TimerMode::Repeating,
            )));
        }
        match request.kind {
            PlantKind::Sunflower => {
                entity.insert(Sunflower);
            }
            PlantKind::Peashooter => {
                entity.insert(Peashooter);
            }
            PlantKind::WallNut => {}
        }
        let id = entity.id();
        occupancy.0.insert(request.cell, id);
    }
}

fn fire_ready_shooters(
    time: Res<Time<Fixed>>,
    mut shooters: Query<(Entity, &Transform, &Lane, &mut ActionTimer), With<Peashooter>>,
    zombies: Query<(&Transform, &Lane), With<Zombie>>,
    mut spawn: MessageWriter<SpawnProjectile>,
) {
    for (entity, transform, lane, mut timer) in &mut shooters {
        timer.0.tick(time.delta());
        let has_target = zombies.iter().any(|(zombie_transform, zombie_lane)| {
            zombie_lane == lane && zombie_transform.translation.x > transform.translation.x
        });
        if has_target && timer.0.just_finished() {
            spawn.write(SpawnProjectile {
                owner: entity,
                origin: transform.translation.truncate() + Vec2::new(36.0, 12.0),
                lane: *lane,
                kind: ProjectileKind::Pea,
            });
        }
    }
}

fn produce_sun(
    time: Res<Time<Fixed>>,
    mut sunflowers: Query<(&Transform, &mut ActionTimer), With<Sunflower>>,
    mut spawn: MessageWriter<SpawnSun>,
) {
    for (transform, mut timer) in &mut sunflowers {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            spawn.write(SpawnSun {
                position: transform.translation.truncate() + Vec2::new(18.0, 24.0),
                value: 25,
            });
        }
    }
}

fn release_dead_plant_cells(
    dead_plants: Query<&GridCell, (With<Plant>, With<Dead>)>,
    mut occupancy: ResMut<CellOccupancy>,
) {
    for cell in &dead_plants {
        occupancy.0.remove(cell);
    }
}

// Marker components make behavior queries explicit and cheap.
#[derive(Component)]
struct Sunflower;
#[derive(Component)]
struct Peashooter;
