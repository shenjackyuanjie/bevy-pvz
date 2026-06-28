pub mod combat;
pub mod lawn;
pub mod level;
pub mod physics;
pub mod plant;
pub mod projectile;
pub mod schedule;
pub mod state;
pub mod ui;
pub mod zombie;

use bevy::prelude::*;

use self::combat::CombatPlugin;
use self::lawn::LawnPlugin;
use self::level::LevelPlugin;
use self::physics::GamePhysicsPlugin;
use self::plant::PlantPlugin;
use self::projectile::ProjectilePlugin;
use self::schedule::GameSet;
use self::state::{GameState, LevelEntity};
use self::ui::GameUiPlugin;
use self::zombie::ZombiePlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
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
            .add_systems(Update, restart_level);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Name::new("Main camera")));
}

fn enter_playing(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Playing);
}

fn cleanup_level(mut commands: Commands, level_entities: Query<Entity, With<LevelEntity>>) {
    for entity in &level_entities {
        commands.entity(entity).despawn();
    }
}

fn restart_level(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) && *state.get() != GameState::Loading {
        next_state.set(GameState::Loading);
    }
}
