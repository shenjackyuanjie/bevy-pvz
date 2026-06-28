use std::collections::HashMap;
use std::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};

use super::combat::{EntityDied, Team};
use super::lawn::{CellOccupancy, Lane, LawnLayout};
use super::plant::{PlantKind, PlantRequest};
use super::schedule::GameSet;
use super::state::{GameState, LevelEntity};
use super::zombie::{SpawnZombie, Zombie};

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
                    tick_wave_timeline.before(super::zombie::spawn_zombies),
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

#[derive(Debug, Clone, Copy)]
pub struct ZombieSpawn {
    pub at_seconds: f32,
    pub lane: Lane,
}

#[derive(Resource, Debug, Clone)]
pub struct LevelDefinition {
    pub spawns: Vec<ZombieSpawn>,
}

impl Default for LevelDefinition {
    fn default() -> Self {
        Self {
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

#[derive(Resource, Debug, Default)]
pub struct LevelRuntime {
    pub elapsed: Duration,
    pub next_spawn: usize,
    pub defeated_zombies: usize,
}

#[derive(Resource, Debug)]
pub struct SunBank {
    pub amount: u32,
}

impl Default for SunBank {
    fn default() -> Self {
        Self { amount: 250 }
    }
}

impl SunBank {
    pub fn try_spend(&mut self, amount: u32) -> bool {
        if self.amount < amount {
            return false;
        }
        self.amount -= amount;
        true
    }
}

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
    pub fn ready(&self, kind: PlantKind) -> bool {
        self.0.get(&kind).is_some_and(Duration::is_zero)
    }

    pub fn trigger(&mut self, kind: PlantKind) {
        self.0.insert(kind, kind.card_cooldown());
    }

    pub fn remaining(&self, kind: PlantKind) -> Duration {
        self.0.get(&kind).copied().unwrap_or(Duration::ZERO)
    }
}

#[derive(Resource, Debug)]
pub struct SelectedPlant(pub PlantKind);

impl Default for SelectedPlant {
    fn default() -> Self {
        Self(PlantKind::Peashooter)
    }
}

#[derive(Component, Debug)]
pub struct SunPickup {
    pub value: u32,
    base_y: f32,
    age: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct SpawnSun {
    pub position: Vec2,
    pub value: u32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct LevelWon;

#[derive(Message, Debug, Clone, Copy)]
pub struct LevelLost;

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

fn tick_card_cooldowns(time: Res<Time<Fixed>>, mut cards: ResMut<PlantCards>) {
    for remaining in cards.0.values_mut() {
        *remaining = remaining.saturating_sub(time.delta());
    }
}

fn spawn_sun_pickups(mut commands: Commands, mut requests: MessageReader<SpawnSun>) {
    for request in requests.read() {
        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.86, 0.15), Vec2::splat(34.0)),
            Transform::from_translation(request.position.extend(8.0)),
            SunPickup {
                value: request.value,
                base_y: request.position.y,
                age: 0.0,
            },
            LevelEntity,
            Name::new("Sun pickup"),
        ));
    }
}

fn select_plant_card(keyboard: Res<ButtonInput<KeyCode>>, mut selected: ResMut<SelectedPlant>) {
    if keyboard.just_pressed(KeyCode::Digit1) {
        selected.0 = PlantKind::Sunflower;
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        selected.0 = PlantKind::Peashooter;
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        selected.0 = PlantKind::WallNut;
    }
}

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

    if let Some((entity, _, pickup)) = params
        .pickups
        .iter()
        .find(|(_, transform, _)| transform.translation.truncate().distance(world) <= 28.0)
    {
        params.bank.amount += pickup.value;
        params.commands.entity(entity).despawn();
        return;
    }

    if let Some(cell) = params.layout.world_to_cell(world) {
        params.plant.write(PlantRequest {
            kind: params.selected.0,
            cell,
        });
    }
}

fn animate_sun_pickups(time: Res<Time>, mut pickups: Query<(&mut Transform, &mut SunPickup)>) {
    for (mut transform, mut pickup) in &mut pickups {
        pickup.age += time.delta_secs();
        transform.translation.y = pickup.base_y + (pickup.age * 2.2).sin() * 6.0;
        transform.rotate_z(time.delta_secs() * 0.7);
    }
}

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
