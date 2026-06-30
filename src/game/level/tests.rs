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
                start_seconds: 0.0,
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
    for path in [
        DEFAULT_LEVEL_PATH,
        "assets/levels/level_row_three_physics_line.ron",
    ] {
        LevelDefinition::load_from_file(path)
            .unwrap()
            .validate(&ContentCatalog::default())
            .unwrap();
    }
}

#[test]
fn row_three_physics_line_uses_six_waves_and_ends_at_six_minutes_five_seconds() {
    let level =
        LevelDefinition::load_from_file("assets/levels/level_row_three_physics_line.ron").unwrap();

    let last_spawn = level
        .waves
        .iter()
        .filter_map(|wave| wave.spawns.last())
        .map(|spawn| spawn.at_seconds)
        .max_by(f32::total_cmp)
        .unwrap();
    assert!(
        (last_spawn - 365.0).abs() < 0.01,
        "last spawn should be at 365s, got {last_spawn}"
    );
}

#[test]
fn ron_wave_entries_are_relative_to_their_wave_start() {
    let level = LevelDefinition::from_ron_str(
        r#"
        (
            id: "test_wave_offsets",
            display_name: "测试波次",
            starting_sun: 50,
            lawn: (
                columns: 9,
                cell_size: (90.0, 90.0),
                center_x: -50.0,
                path_y: -215.0,
            ),
            waves: [
                (
                    delay: 5.0,
                    wave: [
                        (delay: 1.0, kind: Basic, count: 2, interval: 3.0),
                        (delay: 1.0, kind: Conehead, count: 1, interval: 1.0),
                        (delay: 0.0, kind: Imp, count: 1, interval: 1.0),
                    ],
                ),
                (
                    delay: 4.0,
                    wave: [
                        (delay: 2.0, kind: Buckethead, count: 1, interval: 1.0),
                    ],
                ),
            ],
        )
        "#,
    )
    .unwrap();

    assert!(!level.always_shoot);
    assert_eq!(level.cards, default_plant_cards());
    let first_wave_times: Vec<f32> = level.waves[0]
        .spawns
        .iter()
        .map(|spawn| spawn.at_seconds)
        .collect();
    assert_eq!(first_wave_times, vec![5.0, 6.0, 6.0, 9.0]);
    assert!(
        level.waves[0]
            .spawns
            .iter()
            .any(|spawn| spawn.at_seconds == 6.0 && spawn.kind == ZombieKind::Basic)
    );
    assert!(
        level.waves[0]
            .spawns
            .iter()
            .any(|spawn| spawn.at_seconds == 6.0 && spawn.kind == ZombieKind::Conehead)
    );
    assert_eq!(level.waves[0].start_seconds, 5.0);
    assert_eq!(level.waves[1].start_seconds, 13.0);
    assert_eq!(level.waves[1].spawns[0].at_seconds, 15.0);
}

#[test]
fn ron_level_reads_optional_rules() {
    let level = LevelDefinition::from_ron_str(
        r#"
        (
            id: "test_always_shoot",
            display_name: "持续射击",
            starting_sun: 50,
            always_shoot: true,
            pea_path_arrival_effect: RowThreePhysicsLine,
            gatling_pea_upgrade_only: true,
            lawn: (
                columns: 9,
                cell_size: (90.0, 90.0),
                center_x: -50.0,
                path_y: -215.0,
            ),
            waves: [
                (
                    delay: 1.0,
                    wave: [
                        (delay: 0.0, kind: Basic, count: 1, interval: 1.0),
                    ],
                ),
            ],
        )
        "#,
    )
    .unwrap();

    assert!(level.always_shoot);
    assert_eq!(
        level.pea_path_arrival_effect,
        PeaPathArrivalEffect::RowThreePhysicsLine
    );
    assert!(level.gatling_pea_upgrade_only);
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
