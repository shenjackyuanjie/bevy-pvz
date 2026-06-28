mod game;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy PvZ - Rapier 2D prototype".into(),
                resolution: WindowResolution::new(1200, 720),
                present_mode: PresentMode::AutoVsync,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}
