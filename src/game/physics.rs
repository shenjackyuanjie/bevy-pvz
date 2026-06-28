use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::lawn::LawnLayout;
use super::schedule::GameSet;
use super::state::{GameState, LevelEntity};

pub const PLANT_GROUP: Group = Group::GROUP_1;
pub const ZOMBIE_GROUP: Group = Group::GROUP_2;
#[allow(dead_code)] // Logic projectiles do not own colliders; the bit is reserved for query adapters.
pub const NORMAL_PROJECTILE_GROUP: Group = Group::GROUP_3;
pub const PHYSICS_PROJECTILE_GROUP: Group = Group::GROUP_4;
pub const WORLD_BOUNDARY_GROUP: Group = Group::GROUP_5;
pub const MOWER_GROUP: Group = Group::GROUP_6;

pub struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0).in_fixed_schedule(),
            RapierDebugRenderPlugin::default().disabled(),
        ))
        .configure_sets(
            FixedUpdate,
            (
                GameSet::Spawn.before(PhysicsSet::SyncBackend),
                GameSet::LogicMovement
                    .after(GameSet::Spawn)
                    .before(PhysicsSet::SyncBackend),
                GameSet::ContactRead
                    .after(PhysicsSet::Writeback)
                    .before(GameSet::Combat),
            ),
        )
        .add_systems(OnEnter(GameState::Playing), setup_physics_world)
        .add_systems(Update, toggle_physics_debug);
    }
}

pub fn plant_groups() -> CollisionGroups {
    CollisionGroups::new(PLANT_GROUP, ZOMBIE_GROUP | MOWER_GROUP)
}

pub fn zombie_groups() -> CollisionGroups {
    CollisionGroups::new(ZOMBIE_GROUP, PHYSICS_PROJECTILE_GROUP)
}

pub fn physics_projectile_groups() -> CollisionGroups {
    CollisionGroups::new(
        PHYSICS_PROJECTILE_GROUP,
        ZOMBIE_GROUP | PHYSICS_PROJECTILE_GROUP | WORLD_BOUNDARY_GROUP,
    )
}

fn world_groups() -> CollisionGroups {
    CollisionGroups::new(WORLD_BOUNDARY_GROUP, PHYSICS_PROJECTILE_GROUP)
}

fn setup_physics_world(mut commands: Commands, layout: Res<LawnLayout>) {
    let board_width = layout.cell_size.x * f32::from(layout.columns);
    let center_x = layout.origin.x + board_width * 0.5;

    // A floor below the five gameplay lanes gives thrown peas room to arc and bounce.
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(board_width * 0.75, 10.0),
        world_groups(),
        Transform::from_xyz(center_x, layout.origin.y - 55.0, 0.0),
        LevelEntity,
        Name::new("Physics floor"),
    ));

    // Side walls keep the physical sandbox bounded. Lifetime cleanup remains the final guard.
    for x in [layout.origin.x - 120.0, layout.right() + 220.0] {
        commands.spawn((
            RigidBody::Fixed,
            Collider::cuboid(10.0, 400.0),
            world_groups(),
            Transform::from_xyz(x, 0.0, 0.0),
            LevelEntity,
            Name::new("Physics side wall"),
        ));
    }
}

fn toggle_physics_debug(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut debug: ResMut<DebugRenderContext>,
) {
    if keyboard.just_pressed(KeyCode::KeyD) {
        debug.enabled = !debug.enabled;
    }
}
