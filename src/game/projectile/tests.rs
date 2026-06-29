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
fn left_route_teleports_without_sweeping_between_rows_and_reverses() {
    let portal = LeftEdgePortal {
        trigger_x: -100.0,
        exit: Vec2::new(-100.0, 40.0),
    };
    let mut velocity = Vec2::new(-430.0, 0.0);
    let (position, previous) =
        advance_path_step(Vec2::new(-95.0, -140.0), &mut velocity, 0.1, Some(portal));

    assert_eq!(position, portal.exit);
    assert_eq!(previous, portal.exit);
    assert_eq!(velocity, Vec2::new(430.0, 0.0));
}

#[test]
fn spawn_request_builds_distinct_motion_pipelines() {
    let mut app = App::new();
    app.add_message::<SpawnProjectile>()
        .init_resource::<ContentCatalog>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .init_resource::<ProjectileRenderAssets>()
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
        catalog.projectile(ProjectileKind::PhysicsPea).radius,
        Some(RigidBody::Dynamic),
        false,
        true,
    )));
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
fn torchwood_turns_ice_peas_back_to_normal_peas() {
    assert_eq!(
        torchwood_output_kind(ProjectileKind::IcePea),
        ProjectileKind::Pea
    );
    assert_eq!(
        torchwood_output_kind(ProjectileKind::PhysicsPea),
        ProjectileKind::FirePea
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
    assert_eq!(app.world().get::<Health>(newspaper).unwrap().current, 100.0);
    assert_eq!(app.world().get::<Health>(far).unwrap().current, 100.0);
    assert!(!fire_splash_triggers(ZombieKind::ScreenDoor));
    assert!(!fire_splash_affects(ZombieKind::Zomboni));
    assert_eq!(
        fire_direct_damage(40.0, ProjectileKind::FirePea, Some(ZombieKind::Newspaper)),
        80.0
    );
}
