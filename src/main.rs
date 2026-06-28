//! Bevy PvZ 原型程序入口
//!
//! 基于 Bevy 引擎和 Rapier2D 物理引擎实现的 Plants vs. Zombies 单关原型。
//! 此处配置主窗口属性，加载 [`GamePlugin`] 并启动应用循环。

mod game;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use game::GamePlugin;

/// 程序入口点。
///
/// 创建 Bevy 应用，配置 1200×720 的可调整大小窗口（标题 "Bevy PvZ - Rapier 2D prototype"），
/// 添加默认插件集和自定义 [`GamePlugin`]，然后启动运行循环。
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
