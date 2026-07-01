use super::*;
use crate::game::combat::Health;

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
fn swept_query_rejects_vertical_miss() {
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
fn left_route_moves_to_left_edge_then_up_then_right() {
    let mut route = LeftEdgePath {
        turn_x: -100.0,
        target_y: 40.0,
        phase: LeftEdgePathPhase::MoveLeft,
        after_arrival: PeaPathArrivalEffect::Straight,
    };
    let mut velocity = Vec2::new(-430.0, 0.0);
    let (position, previous, effect) = advance_path_step(
        Vec2::new(-95.0, -140.0),
        &mut velocity,
        0.1,
        Some(&mut route),
    );

    assert_eq!(position, Vec2::new(-100.0, -140.0));
    assert_eq!(previous, Vec2::new(-95.0, -140.0));
    assert_eq!(effect, None);
    assert_eq!(velocity, Vec2::new(0.0, 430.0));
    assert_eq!(route.phase, LeftEdgePathPhase::MoveUp);

    let (position, previous, effect) =
        advance_path_step(position, &mut velocity, 0.5, Some(&mut route));

    assert_eq!(position, Vec2::new(-100.0, 40.0));
    assert_eq!(previous, Vec2::new(-100.0, -140.0));
    assert_eq!(effect, None);
    assert_eq!(velocity, Vec2::new(430.0, 0.0));
    assert_eq!(route.phase, LeftEdgePathPhase::MoveRight);

    let (position, previous, effect) =
        advance_path_step(position, &mut velocity, 0.1, Some(&mut route));

    assert_eq!(position, Vec2::new(-57.0, 40.0));
    assert_eq!(previous, Vec2::new(-100.0, 40.0));
    assert_eq!(effect, None);
    assert_eq!(velocity, Vec2::new(430.0, 0.0));
}

#[test]
fn row_three_effect_turns_fire_pea_into_physics_fire_peas() {
    let mut app = App::new();
    app.insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(LawnLayout::default())
        .init_resource::<ContentCatalog>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<ProjectileRenderAssets>()
        .init_resource::<ProjectileRenderQuality>()
        .add_systems(FixedUpdate, advance_path_projectiles);
    let layout = app.world().resource::<LawnLayout>().clone();
    let source = app
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(layout.origin.x, layout.path_y()).extend(3.0)),
            Projectile {
                owner: Entity::PLACEHOLDER,
                team: Team::Plants,
                damage: 40.0,
            },
            ProjectileKind::FirePea,
            ProjectileRadius(9.0),
            HitRegistry::default(),
            TorchwoodRegistry::default(),
            PathVelocity(Vec2::new(0.0, 430.0)),
            PreviousPosition(Vec2::new(layout.origin.x, layout.path_y())),
            LeftEdgePath {
                turn_x: layout.origin.x,
                target_y: layout.path_y(),
                phase: LeftEdgePathPhase::MoveUp,
                after_arrival: PeaPathArrivalEffect::RowThreePhysicsLine,
            },
        ))
        .id();

    app.world_mut().run_schedule(FixedUpdate);

    assert!(app.world().get_entity(source).is_err());
    let world = app.world_mut();
    let mut query = world.query::<(
        &ProjectileKind,
        &ProjectileMotion,
        &Projectile,
        &ProjectileRadius,
        &Transform,
    )>();
    let mut spawned: Vec<_> = query
        .iter(world)
        .map(|(kind, motion, projectile, radius, transform)| {
            (
                *kind,
                *motion,
                projectile.damage,
                radius.0,
                transform.translation.truncate(),
            )
        })
        .collect();
    assert_eq!(spawned.len(), ROW_THREE_PHYSICS_LINE_COUNT);
    let expected_radius = physics_projectile_radius(
        world
            .resource::<ContentCatalog>()
            .projectile(ProjectileKind::FirePea)
            .radius,
    );
    assert!(
        spawned
            .iter()
            .all(|(kind, motion, damage, radius, _position)| {
                *kind == ProjectileKind::FirePea
                    && *motion == ProjectileMotion::Physics
                    && *damage == 40.0
                    && (*radius - expected_radius).abs() < 0.001
            })
    );
    spawned.sort_by(|left, right| left.4.x.total_cmp(&right.4.x));

    let left = layout.origin.x + expected_radius;
    let right = layout.right() - expected_radius;
    let mut offsets = Vec::with_capacity(spawned.len());
    for (index, (_kind, _motion, _damage, _radius, position)) in spawned.iter().enumerate() {
        let t = index as f32 / (ROW_THREE_PHYSICS_LINE_COUNT - 1) as f32;
        let base_x = left + (right - left) * t;
        let offset = position.x - base_x;
        assert!(
            offset.abs() <= ROW_THREE_PHYSICS_LINE_X_JITTER + 0.001,
            "physics pea {index} x offset should stay within jitter bounds"
        );
        offsets.push((offset * 1000.0).round() as i32);
    }
    offsets.sort_unstable();
    offsets.dedup();
    assert_eq!(
        offsets.len(),
        ROW_THREE_PHYSICS_LINE_COUNT,
        "row-three physics peas should each use a different x offset"
    );
}

#[test]
fn spawn_request_builds_distinct_motion_pipelines() {
    let mut app = App::new();
    app.add_message::<SpawnProjectile>()
        .init_resource::<ContentCatalog>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<ProjectileRenderAssets>()
        .init_resource::<ProjectileRenderQuality>()
        .add_systems(Update, spawn_projectiles);
    for kind in [ProjectileKind::Pea, ProjectileKind::PhysicsPea] {
        app.world_mut().write_message(SpawnProjectile {
            owner: Entity::PLACEHOLDER,
            origin: Vec2::ZERO,
            kind,
            route: ProjectileRoute::Direct,
        });
    }

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(
        &ProjectileKind,
        &ProjectileMotion,
        &ProjectileRadius,
        Option<&RigidBody>,
        Option<&PathVelocity>,
        Option<&Collider>,
    )>();
    let spawned: Vec<_> = query
        .iter(world)
        .map(|(kind, motion, radius, body, path, collider)| {
            (
                *kind,
                *motion,
                radius.0,
                body.copied(),
                path.is_some(),
                collider.is_some(),
            )
        })
        .collect();
    assert_eq!(spawned.len(), 2);
    let catalog = ContentCatalog::default();
    assert!(spawned.contains(&(
        ProjectileKind::Pea,
        ProjectileMotion::Path,
        catalog.projectile(ProjectileKind::Pea).radius,
        None,
        true,
        false,
    )));
    assert!(spawned.contains(&(
        ProjectileKind::PhysicsPea,
        ProjectileMotion::Physics,
        physics_projectile_radius(catalog.projectile(ProjectileKind::PhysicsPea).radius),
        Some(RigidBody::Dynamic),
        false,
        true,
    )));
}

#[test]
fn physics_projectile_cleanup_uses_larger_bounds() {
    let half_window = Vec2::new(640.0, 360.0);
    let radius = 9.0;
    let path_outside = Vec2::new(half_window.x + radius + 1.0, 0.0);

    assert!(projectile_outside_cleanup_bounds(
        path_outside,
        radius,
        half_window,
        0.0
    ));
    assert!(!projectile_outside_cleanup_bounds(
        path_outside,
        radius,
        half_window,
        PHYSICS_PROJECTILE_CLEANUP_PADDING
    ));
    assert!(projectile_outside_cleanup_bounds(
        Vec2::new(
            half_window.x + PHYSICS_PROJECTILE_CLEANUP_PADDING + radius + 1.0,
            0.0,
        ),
        radius,
        half_window,
        PHYSICS_PROJECTILE_CLEANUP_PADDING
    ));
}

#[test]
fn ignition_changes_damage_kind_and_render_assets() {
    let mut app = App::new();
    app.add_message::<SpawnProjectile>()
        .add_message::<IgniteProjectile>()
        .init_resource::<ContentCatalog>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<ProjectileRenderAssets>()
        .init_resource::<ProjectileRenderQuality>()
        .add_systems(
            Update,
            (spawn_projectiles, apply_projectile_ignitions).chain(),
        );
    app.world_mut().write_message(SpawnProjectile {
        owner: Entity::PLACEHOLDER,
        origin: Vec2::ZERO,
        kind: ProjectileKind::Pea,
        route: ProjectileRoute::Direct,
    });
    app.update();

    let entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Projectile>>();
        query.single(world).unwrap()
    };
    app.world_mut().write_message(IgniteProjectile(entity));
    app.update();

    let world = app.world();
    assert_eq!(
        *world.get::<ProjectileKind>(entity).unwrap(),
        ProjectileKind::FirePea
    );
    assert_eq!(
        world.get::<Projectile>(entity).unwrap().damage,
        world
            .resource::<ContentCatalog>()
            .projectile(ProjectileKind::FirePea)
            .damage
    );
}

#[test]
fn fire_pea_deals_direct_and_small_same_lane_splash_damage() {
    let mut app = App::new();
    app.add_message::<ProjectileHit>()
        .add_message::<ApplyDamage>()
        .add_systems(
            Update,
            (resolve_projectile_hits, crate::game::combat::apply_damage).chain(),
        );
    let projectile = app
        .world_mut()
        .spawn((
            Projectile {
                owner: Entity::PLACEHOLDER,
                team: Team::Plants,
                damage: 40.0,
            },
            ProjectileKind::FirePea,
            HitPolicy {
                destroy_on_hit: true,
                remaining_pierces: 0,
            },
            HitRegistry::default(),
        ))
        .id();
    let spawn_zombie = |world: &mut World, kind, position: Vec2| {
        world
            .spawn((
                Zombie {
                    speed: 1.0,
                    attack_damage: 1.0,
                    engage_min: 0.0,
                    engage_max: 1.0,
                },
                kind,
                Transform::from_translation(position.extend(0.0)),
                Health::new(100.0),
                Team::Zombies,
            ))
            .id()
    };
    let target = spawn_zombie(app.world_mut(), ZombieKind::Basic, Vec2::ZERO);
    let nearby = spawn_zombie(app.world_mut(), ZombieKind::Conehead, Vec2::new(30.0, 0.0));
    let newspaper = spawn_zombie(app.world_mut(), ZombieKind::Newspaper, Vec2::new(35.0, 0.0));
    let far = spawn_zombie(
        app.world_mut(),
        ZombieKind::Basic,
        Vec2::new(FIRE_SPLASH_HALF_SIZE.x + 1.0, 0.0),
    );
    app.world_mut()
        .write_message(ProjectileHit { projectile, target });

    app.update();

    assert_eq!(app.world().get::<Health>(target).unwrap().current, 60.0);
    assert_eq!(app.world().get::<Health>(nearby).unwrap().current, 86.0);
    assert_eq!(app.world().get::<Health>(newspaper).unwrap().current, 86.0);
    assert_eq!(app.world().get::<Health>(far).unwrap().current, 100.0);
    assert!(!fire_splash_triggers(ZombieKind::ScreenDoor));
    assert!(!fire_splash_affects(ZombieKind::Ladder));
    assert!(fire_splash_affects(ZombieKind::Newspaper));
    assert!(fire_splash_affects(ZombieKind::Zomboni));
}

#[test]
fn ice_pea_chills_direct_target_unless_barrier_blocks_it() {
    let mut app = App::new();
    app.add_message::<ProjectileHit>()
        .add_message::<ApplyDamage>()
        .add_systems(Update, resolve_projectile_hits);
    let projectile = app
        .world_mut()
        .spawn((
            Projectile {
                owner: Entity::PLACEHOLDER,
                team: Team::Plants,
                damage: 20.0,
            },
            ProjectileKind::IcePea,
            HitPolicy {
                destroy_on_hit: true,
                remaining_pierces: 0,
            },
            HitRegistry::default(),
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Zombie {
                speed: 1.0,
                attack_damage: 1.0,
                engage_min: 0.0,
                engage_max: 1.0,
            },
            ZombieKind::Basic,
            Transform::default(),
            Health::new(100.0),
            Team::Zombies,
        ))
        .id();

    app.world_mut()
        .write_message(ProjectileHit { projectile, target });
    app.update();

    assert!(app.world().get::<Chilled>(target).is_some());
    assert!(ice_pea_chills(ZombieKind::Basic, None));
    assert!(!ice_pea_chills(ZombieKind::ScreenDoor, None));

    let intact_barrier = EquipmentHealth::new(100.0);
    let mut broken_barrier = EquipmentHealth::new(100.0);
    broken_barrier.current = 0.0;
    assert!(!ice_pea_chills(
        ZombieKind::ScreenDoor,
        Some(&intact_barrier)
    ));
    assert!(ice_pea_chills(
        ZombieKind::ScreenDoor,
        Some(&broken_barrier)
    ));
}

#[test]
fn fire_pea_direct_hit_clears_chill() {
    let mut app = App::new();
    app.add_message::<ProjectileHit>()
        .add_message::<ApplyDamage>()
        .add_systems(Update, resolve_projectile_hits);
    let projectile = app
        .world_mut()
        .spawn((
            Projectile {
                owner: Entity::PLACEHOLDER,
                team: Team::Plants,
                damage: 40.0,
            },
            ProjectileKind::FirePea,
            HitPolicy {
                destroy_on_hit: true,
                remaining_pierces: 0,
            },
            HitRegistry::default(),
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Zombie {
                speed: 1.0,
                attack_damage: 1.0,
                engage_min: 0.0,
                engage_max: 1.0,
            },
            ZombieKind::Basic,
            Transform::default(),
            Health::new(100.0),
            Team::Zombies,
            Chilled::new(),
        ))
        .id();

    app.world_mut()
        .write_message(ProjectileHit { projectile, target });
    app.update();

    assert!(app.world().get::<Chilled>(target).is_none());
}
