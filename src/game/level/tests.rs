
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
    let catalog = ContentCatalog::default();
    let mut cards = PlantCards::default();
    cards.0.insert(PlantKind::Peashooter, Duration::ZERO);
    assert!(cards.ready(PlantKind::Peashooter));
    cards.trigger(PlantKind::Peashooter, &catalog);
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
            waves: vec![ZombieWaveDefinition {
                spawns: vec![ZombieSpawnDefinition {
                    at_seconds: 0.0,
                    kind: ZombieKind::Basic,
                }],
            }],
            ..default()
        })
        .init_resource::<ContentCatalog>()
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

    assert_eq!(app.world().resource::<LevelRuntime>().next_wave, 1);
    let zombie_count = app
        .world_mut()
        .query_filtered::<Entity, With<Zombie>>()
        .iter(app.world())
        .count();
    assert_eq!(zombie_count, 1);
}

#[test]
fn ron_level_is_complete_and_valid() {
    LevelDefinition::load_from_file(DEFAULT_LEVEL_PATH)
        .unwrap()
        .validate(&ContentCatalog::default())
        .unwrap();
}

#[test]
fn validation_rejects_duplicate_cards_and_invalid_lawn() {
    let catalog = ContentCatalog::default();
    let mut level = LevelDefinition::default();
    level.cards[1].plant = level.cards[0].plant;
    assert!(
        level
            .validate(&catalog)
            .unwrap_err()
            .contains("duplicate plant card")
    );
    level = LevelDefinition::default();
    level.lawn.cell_size.y = 0.0;
    assert!(
        level
            .validate(&catalog)
            .unwrap_err()
            .contains("finite and positive")
    );
}

#[test]
fn reset_uses_the_current_level_definition() {
    let definition = LevelDefinition {
        starting_sun: 777,
        lawn: LawnLayout {
            columns: 7,
            ..default()
        },
        ..default()
    };
    let expected_card_count = definition.cards.len();

    let mut app = App::new();
    app.insert_resource(definition)
        .insert_resource(LevelRuntime {
            elapsed: Duration::from_secs(9),
            next_wave: 2,
            next_spawn_in_wave: 1,
            defeated_zombies: 1,
        })
        .insert_resource(SunBank { amount: 1 })
        .insert_resource(PlantCards::default())
        .insert_resource(ShovelMode {
            preview: Some(Entity::PLACEHOLDER),
        })
        .insert_resource(LawnLayout::default())
        .insert_resource(CellOccupancy::default())
        .add_systems(Update, reset_level_runtime);

    app.update();

    assert_eq!(app.world().resource::<SunBank>().amount, 777);
    assert!(!app.world().resource::<ShovelMode>().active());
    assert_eq!(app.world().resource::<LawnLayout>().columns, 7);
    assert_eq!(app.world().resource::<LevelRuntime>().next_wave, 0);
    assert_eq!(app.world().resource::<LevelRuntime>().next_spawn_in_wave, 0);
    assert_eq!(
        app.world().resource::<PlantCards>().0.len(),
        expected_card_count
    );
}
