//! 空格暂停、虚拟时间冻结与全屏灰色遮罩。

use bevy::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::catalog::{ContentCatalog, PlantKind, ZombieKind};
use crate::game::controls::ControlBindings;
use crate::game::model::{plant_model_parts, zombie_model_parts};
use crate::game::state::{GameState, LevelEntity};
use crate::game::ui::GameHudRoot;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamePause>()
            .add_systems(
                OnEnter(GameState::Playing),
                (reset_pause, setup_pause_overlay).chain(),
            )
            .add_systems(
                Update,
                (
                    toggle_pause,
                    sync_pause_hud_visibility,
                    sync_pause_debug_gallery,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), leave_playing_unpaused);
    }
}

#[derive(Resource, Debug, Default)]
pub struct GamePause {
    pub paused: bool,
}

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct PauseDebugGallery;

pub fn game_not_paused(pause: Res<GamePause>) -> bool {
    !pause.paused
}

fn reset_pause(mut pause: ResMut<GamePause>, mut time: ResMut<Time<Virtual>>) {
    pause.paused = false;
    time.unpause();
}

fn setup_pause_overlay(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            padding: UiRect::top(px(14)),
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.16, 0.16, 0.18, 0.48)),
        Visibility::Hidden,
        ZIndex(1000),
        PauseOverlay,
        LevelEntity,
        Name::new("暂停遮罩"),
        children![(
            Text::new("游戏已暂停  按空格继续"),
            TextFont {
                font: assets.chinese_font.clone(),
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.94, 0.94, 0.94)),
            TextLayout::new_with_justify(Justify::Center),
        )],
    ));
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    controls: Res<ControlBindings>,
    mut pause: ResMut<GamePause>,
    mut time: ResMut<Time<Virtual>>,
    mut overlay: Single<&mut Visibility, With<PauseOverlay>>,
) {
    if !keyboard.just_pressed(controls.pause) {
        return;
    }
    pause.paused = !pause.paused;
    if pause.paused {
        time.pause();
        **overlay = Visibility::Visible;
    } else {
        time.unpause();
        **overlay = Visibility::Hidden;
    }
}

fn sync_pause_debug_gallery(
    mut commands: Commands,
    pause: Res<GamePause>,
    assets: Res<GameAssets>,
    catalog: Res<ContentCatalog>,
    window: Single<&Window>,
    gallery: Query<Entity, With<PauseDebugGallery>>,
) {
    if pause.paused {
        if gallery.is_empty() {
            spawn_pause_debug_gallery(&mut commands, &assets, &catalog, &window);
        }
    } else {
        for entity in &gallery {
            commands.entity(entity).despawn();
        }
    }
}

fn sync_pause_hud_visibility(
    pause: Res<GamePause>,
    mut roots: Query<&mut Visibility, With<GameHudRoot>>,
) {
    let visibility = if pause.paused {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    for mut root in &mut roots {
        *root = visibility;
    }
}

fn spawn_pause_debug_gallery(
    commands: &mut Commands,
    assets: &GameAssets,
    catalog: &ContentCatalog,
    window: &Window,
) {
    let half_width = window.resolution.width() * 0.5;
    let half_height = window.resolution.height() * 0.5;
    let font = assets.chinese_font.clone();

    spawn_gallery_title(
        commands,
        Vec2::new(-half_width + 80.0, half_height - 54.0),
        "植物",
        font.clone(),
    );
    let plant_cell = Vec2::new(118.0, 104.0);
    let plant_start_x = -(PlantKind::ALL.len() as f32 - 1.0) * plant_cell.x * 0.5;
    for (index, kind) in PlantKind::ALL.iter().copied().enumerate() {
        let definition = catalog.plant(kind);
        spawn_gallery_item(
            commands,
            Vec2::new(
                plant_start_x + index as f32 * plant_cell.x,
                half_height - 124.0,
            ),
            definition.display_name,
            font.clone(),
            plant_model_parts(kind, 1.0),
            0.72,
            plant_cell,
        );
    }

    spawn_gallery_title(
        commands,
        Vec2::new(-half_width + 80.0, half_height - 216.0),
        "僵尸",
        font.clone(),
    );
    let zombie_cell = Vec2::new(124.0, 96.0);
    let zombie_columns = ((window.resolution.width() / zombie_cell.x).floor() as usize)
        .clamp(6, 9)
        .min(ZombieKind::ALL.len());
    let zombie_start_x = -(zombie_columns as f32 - 1.0) * zombie_cell.x * 0.5;
    let zombie_start_y = half_height - 286.0;
    for (index, kind) in ZombieKind::ALL.iter().copied().enumerate() {
        let column = index % zombie_columns;
        let row = index / zombie_columns;
        let definition = catalog.zombie(kind);
        spawn_gallery_item(
            commands,
            Vec2::new(
                zombie_start_x + column as f32 * zombie_cell.x,
                zombie_start_y - row as f32 * zombie_cell.y,
            ),
            definition.display_name,
            font.clone(),
            zombie_model_parts(kind, 1.0),
            0.43,
            zombie_cell,
        );
    }
}

fn spawn_gallery_title(commands: &mut Commands, position: Vec2, text: &str, font: Handle<Font>) {
    commands.spawn((
        Text2d::new(text),
        TextFont {
            font,
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.92, 0.62)),
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_translation(position.extend(88.0)),
        PauseDebugGallery,
        LevelEntity,
        Name::new(format!("暂停图鉴标题 {text}")),
    ));
}

fn spawn_gallery_item(
    commands: &mut Commands,
    position: Vec2,
    label: &str,
    font: Handle<Font>,
    parts: Vec<crate::game::model::ModelPart>,
    scale: f32,
    panel_size: Vec2,
) {
    let mut item = commands.spawn((
        Transform::from_translation(position.extend(86.0)),
        Visibility::Visible,
        PauseDebugGallery,
        LevelEntity,
        Name::new(format!("暂停图鉴 {label}")),
    ));
    item.with_children(|parent| {
        parent.spawn((
            Sprite::from_color(
                Color::srgba(0.02, 0.025, 0.02, 0.58),
                panel_size - Vec2::splat(6.0),
            ),
            Transform::from_xyz(0.0, 0.0, -0.2),
            Name::new("图鉴背景"),
        ));
        for part in parts {
            parent.spawn((
                Sprite::from_color(part.color, part.size * scale),
                Transform::from_xyz(part.offset.x * scale, part.offset.y * scale + 7.0, part.z)
                    .with_rotation(Quat::from_rotation_z(part.rotation)),
                Name::new(part.name),
            ));
        }
        parent.spawn((
            Text2d::new(label),
            TextFont {
                font,
                font_size: 9.5,
                ..default()
            },
            TextColor(Color::srgb(0.96, 0.96, 0.88)),
            TextBackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.42)),
            TextLayout::new_with_justify(Justify::Center),
            Transform::from_xyz(0.0, -panel_size.y * 0.39, 4.0),
            Name::new("图鉴名称"),
        ));
    });
}

fn leave_playing_unpaused(mut pause: ResMut<GamePause>, mut time: ResMut<Time<Virtual>>) {
    pause.paused = false;
    time.unpause();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_pauses_virtual_time_and_shows_overlay() {
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(KeyCode::Space);
        let mut app = App::new();
        app.insert_resource(keyboard)
            .init_resource::<ControlBindings>()
            .init_resource::<GamePause>()
            .init_resource::<Time<Virtual>>()
            .add_systems(Update, toggle_pause);
        let overlay = app
            .world_mut()
            .spawn((PauseOverlay, Visibility::Hidden))
            .id();

        app.update();

        assert!(app.world().resource::<GamePause>().paused);
        assert!(app.world().resource::<Time<Virtual>>().is_paused());
        assert_eq!(
            *app.world().get::<Visibility>(overlay).unwrap(),
            Visibility::Visible
        );
    }
}
