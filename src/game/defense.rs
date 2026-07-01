//! 房屋场景与单路一次性小推车。

use bevy::prelude::*;

use crate::game::combat::{ApplyDamage, DamageKind, Dead, Health};
use crate::game::lawn::LawnLayout;
use crate::game::level::LevelSetupSet;
use crate::game::schedule::GameSet;
use crate::game::state::{GameState, LevelEntity};
use crate::game::zombie::Zombie;

const MOWER_SPEED: f32 = 430.0;
const MOWER_TRIGGER_AHEAD: f32 = 74.0;
const MOWER_HIT_HALF_SIZE: Vec2 = Vec2::new(38.0, 32.0);
const MOWER_START_OFFSET_X: f32 = -46.0;
const MOWER_VISUAL_RIGHT_EXTENT: f32 = 47.0;

/// 待命小推车模型最右侧的世界 X 坐标。
pub(crate) fn lawn_mower_start_right(layout: &LawnLayout) -> f32 {
    layout.origin.x + MOWER_START_OFFSET_X + MOWER_VISUAL_RIGHT_EXTENT
}

pub struct HomeDefensePlugin;

impl Plugin for HomeDefensePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            setup_home_defenses.after(LevelSetupSet::Reset),
        )
        .add_systems(
            FixedUpdate,
            advance_lawn_mower
                .in_set(GameSet::LogicMovement)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            lawn_mower_hits_zombies
                .in_set(GameSet::Combat)
                .before(crate::game::combat::apply_damage)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
enum MowerState {
    Ready,
    Active,
}

#[derive(Component)]
struct LawnMower {
    state: MowerState,
}

type LivingZombieFilter = (With<Zombie>, Without<Dead>, Without<LawnMower>);
type MowerTarget<'a> = (Entity, &'a Transform, &'a Health);

fn setup_home_defenses(mut commands: Commands, layout: Res<LawnLayout>) {
    commands.spawn((
        Transform::from_xyz(layout.origin.x - 82.0, layout.path_y() + 132.0, -8.0),
        Visibility::Visible,
        LevelEntity,
        Name::new("房屋"),
        children![
            (
                Sprite::from_color(Color::srgb(0.82, 0.68, 0.48), Vec2::new(108.0, 166.0)),
                Transform::from_xyz(0.0, -18.0, 0.0),
                Name::new("房屋墙体"),
            ),
            (
                Sprite::from_color(Color::srgb(0.45, 0.12, 0.08), Vec2::new(78.0, 20.0)),
                Transform::from_xyz(-30.0, 72.0, 0.2).with_rotation(Quat::from_rotation_z(0.55)),
                Name::new("左屋顶"),
            ),
            (
                Sprite::from_color(Color::srgb(0.52, 0.15, 0.09), Vec2::new(78.0, 20.0)),
                Transform::from_xyz(30.0, 72.0, 0.2).with_rotation(Quat::from_rotation_z(-0.55)),
                Name::new("右屋顶"),
            ),
            (
                Sprite::from_color(Color::srgb(0.34, 0.16, 0.07), Vec2::new(34.0, 70.0)),
                Transform::from_xyz(-23.0, -64.0, 0.2),
                Name::new("房门"),
            ),
            (
                Sprite::from_color(Color::srgb(0.46, 0.78, 0.86), Vec2::splat(26.0)),
                Transform::from_xyz(25.0, 5.0, 0.2),
                Name::new("窗户"),
            ),
            (
                Sprite::from_color(Color::srgb(0.86, 0.91, 0.82), Vec2::new(3.0, 26.0)),
                Transform::from_xyz(25.0, 5.0, 0.3),
                Name::new("窗框竖条"),
            ),
            (
                Sprite::from_color(Color::srgb(0.86, 0.91, 0.82), Vec2::new(26.0, 3.0)),
                Transform::from_xyz(25.0, 5.0, 0.3),
                Name::new("窗框横条"),
            ),
            (
                Sprite::from_color(Color::srgb(0.37, 0.13, 0.08), Vec2::new(18.0, 52.0)),
                Transform::from_xyz(33.0, 92.0, -0.1),
                Name::new("烟囱"),
            ),
        ],
    ));

    commands.spawn((
        Transform::from_xyz(layout.origin.x + MOWER_START_OFFSET_X, layout.path_y(), 4.0),
        Visibility::Visible,
        LawnMower {
            state: MowerState::Ready,
        },
        LevelEntity,
        Name::new("草坪小推车"),
        children![
            (
                Sprite::from_color(Color::srgb(0.73, 0.08, 0.06), Vec2::new(50.0, 21.0)),
                Transform::from_xyz(0.0, 0.0, 0.2),
                Name::new("推车机身"),
            ),
            (
                Sprite::from_color(Color::srgb(0.92, 0.18, 0.08), Vec2::new(30.0, 13.0)),
                Transform::from_xyz(-6.0, 14.0, 0.3),
                Name::new("推车上盖"),
            ),
            (
                Sprite::from_color(Color::srgb(0.09, 0.09, 0.08), Vec2::splat(15.0)),
                Transform::from_xyz(-15.0, -15.0, 0.1),
                Name::new("推车左轮"),
            ),
            (
                Sprite::from_color(Color::srgb(0.09, 0.09, 0.08), Vec2::splat(15.0)),
                Transform::from_xyz(16.0, -15.0, 0.1),
                Name::new("推车右轮"),
            ),
            (
                Sprite::from_color(Color::srgb(0.68, 0.70, 0.70), Vec2::new(34.0, 7.0)),
                Transform::from_xyz(30.0, 0.0, 0.1),
                Name::new("推车前铲"),
            ),
            (
                Sprite::from_color(Color::srgb(0.27, 0.16, 0.09), Vec2::new(8.0, 58.0)),
                Transform::from_xyz(-24.0, 32.0, 0.0).with_rotation(Quat::from_rotation_z(-0.36)),
                Name::new("推车扶手"),
            ),
        ],
    ));
}

fn advance_lawn_mower(
    time: Res<Time<Fixed>>,
    layout: Res<LawnLayout>,
    zombies: Query<&Transform, (With<Zombie>, Without<Dead>)>,
    mut mowers: Query<(Entity, &mut LawnMower, &mut Transform), Without<Zombie>>,
    mut commands: Commands,
) {
    for (entity, mut mower, mut transform) in &mut mowers {
        if mower.state == MowerState::Ready
            && zombies
                .iter()
                .any(|zombie| zombie.translation.x <= transform.translation.x + MOWER_TRIGGER_AHEAD)
        {
            mower.state = MowerState::Active;
        }
        if mower.state == MowerState::Active {
            transform.translation.x += MOWER_SPEED * time.delta_secs();
            if transform.translation.x > layout.right() + MOWER_HIT_HALF_SIZE.x {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn lawn_mower_hits_zombies(
    mowers: Query<(Entity, &LawnMower, &Transform)>,
    zombies: Query<MowerTarget<'_>, LivingZombieFilter>,
    mut damage: MessageWriter<ApplyDamage>,
) {
    for (mower_entity, mower, mower_transform) in &mowers {
        if mower.state != MowerState::Active {
            continue;
        }
        for (zombie, zombie_transform, health) in &zombies {
            let delta =
                zombie_transform.translation.truncate() - mower_transform.translation.truncate();
            if delta.x.abs() <= MOWER_HIT_HALF_SIZE.x && delta.y.abs() <= MOWER_HIT_HALF_SIZE.y {
                damage.write(ApplyDamage {
                    source: mower_entity,
                    target: zombie,
                    amount: health.current,
                    kind: DamageKind::Mower,
                });
            }
        }
    }
}
