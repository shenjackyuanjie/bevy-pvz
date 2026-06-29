//! 空格暂停、虚拟时间冻结与全屏灰色遮罩。

use bevy::prelude::*;

use crate::game::assets::GameAssets;
use crate::game::controls::ControlBindings;
use crate::game::state::{GameState, LevelEntity};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GamePause>()
            .add_systems(
                OnEnter(GameState::Playing),
                (reset_pause, setup_pause_overlay).chain(),
            )
            .add_systems(Update, toggle_pause.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), leave_playing_unpaused);
    }
}

#[derive(Resource, Debug, Default)]
pub struct GamePause {
    pub paused: bool,
}

#[derive(Component)]
struct PauseOverlay;

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
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.22, 0.22, 0.24, 0.68)),
        Visibility::Hidden,
        ZIndex(1000),
        PauseOverlay,
        LevelEntity,
        Name::new("暂停遮罩"),
        children![(
            Text::new("游戏已暂停\n按空格继续"),
            TextFont {
                font: assets.chinese_font.clone(),
                font_size: 34.0,
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
